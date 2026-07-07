//! Program entry point.
//!
//! Usage:
//!   chip8-emulator <rom-path> [max-cycles] [--trace] [--dump-state]
//!                  [--window] [--scale <n>] [--hz <n>] [--disassemble]
//!
//! Without `--window` the program runs headless.

use chip8_emulator::cpu::Cpu;
use chip8_emulator::debug_repl::run_debugger;
use chip8_emulator::debugger::{format_cpu_snapshot, format_trace_entry, TraceEntry};
use chip8_emulator::disassembler::disassemble_range;
use chip8_emulator::display::Display;
use chip8_emulator::input::Input;
use chip8_emulator::memory::Memory;
use chip8_emulator::rom::load_rom_to_memory;
use chip8_emulator::runtime::{
    parse_cli_args, CliOptions, RuntimeConfig, DEFAULT_HEADLESS_MAX_CYCLES,
};

/// Tick timers every N CPU steps. Not real 60Hz; just a rough cadence.
const TIMER_TICK_INTERVAL: u32 = 8;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let exit_code = run(&args);
    std::process::exit(exit_code);
}

fn run(args: &[String]) -> i32 {
    let opts = match parse_cli_args(args) {
        Ok(o) => o,
        Err(msg) => {
            println!("Error: {msg}");
            return 1;
        }
    };

    let mut memory = Memory::new();
    let loaded = match load_rom_to_memory(&opts.rom_path, &mut memory) {
        Ok(n) => n,
        Err(e) => {
            println!("Error: {e}");
            return 1;
        }
    };

    if opts.disassemble {
        return run_disassemble(&memory, &opts);
    }

    let mut cpu = Cpu::new();
    let mut display = Display::new();
    let mut input = Input::new();

    println!("Loaded ROM: {}", opts.rom_path);
    println!("Loaded bytes: {}", loaded);

    if opts.debugger {
        return run_debugger(&mut cpu, &mut memory, &mut display, &input);
    }

    if opts.window {
        let config = build_runtime_config(&opts);
        let code = chip8_emulator::runtime::run_window(
            &mut cpu,
            &mut memory,
            &mut display,
            &mut input,
            &config,
        );
        maybe_save_screenshot(&display, &opts);
        return code;
    }

    run_headless(&mut cpu, &mut memory, &mut display, &input, &opts)
}

fn run_disassemble(memory: &Memory, opts: &CliOptions) -> i32 {
    let instructions = match disassemble_range(memory, Cpu::new().pc, opts.disassemble_count) {
        Ok(instructions) => instructions,
        Err(e) => {
            println!("Error: {e}");
            return 1;
        }
    };

    println!("Disassembly for {}", opts.rom_path);
    for instruction in instructions {
        println!(
            "{:04X}: {:04X}  {}",
            instruction.address, instruction.opcode, instruction.mnemonic
        );
    }
    0
}

/// If `opts.screenshot` is set, write the current display buffer to that
/// path as a PPM image. Creates the parent directory if missing. Prints
/// feedback but does not change the exit code.
fn maybe_save_screenshot(display: &Display, opts: &CliOptions) {
    if let Some(path) = &opts.screenshot {
        // We auto-create the parent directory so
        // `--screenshot screenshots/example.ppm` works without a manual `mkdir`.
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    println!(
                        "Error: failed to create screenshot directory '{}': {}",
                        parent.display(),
                        e
                    );
                    return;
                }
            }
        }
        // `opts.scale` always defaults to DEFAULT_SCALE (=10) from the parser,
        // so it is always > 0; use it directly as the screenshot scale.
        match display.save_ppm(path, opts.scale) {
            Ok(()) => println!("Screenshot saved: {}", path),
            Err(e) => println!("Error: failed to write screenshot to '{}': {}", path, e),
        }
    }
}

/// Build a [`RuntimeConfig`] from parsed [`CliOptions`] for window mode.
fn build_runtime_config(opts: &CliOptions) -> RuntimeConfig {
    RuntimeConfig {
        scale: opts.scale,
        cycles_per_second: opts.hz,
        max_cycles: opts.max_cycles.map(|n| n as usize),
        trace: opts.trace,
        dump_state: opts.dump_state,
    }
}

/// Run the emulator headlessly.
fn run_headless(
    cpu: &mut Cpu,
    memory: &mut Memory,
    display: &mut Display,
    input: &Input,
    opts: &CliOptions,
) -> i32 {
    let max_cycles = opts.max_cycles.unwrap_or(DEFAULT_HEADLESS_MAX_CYCLES);

    let mut executed: u32 = 0;
    for cycle in 0..max_cycles {
        let pc_before = cpu.pc;
        match cpu.step(memory, display, input) {
            Ok(opcode) => {
                if opts.trace {
                    let entry = TraceEntry {
                        cycle: cycle as usize,
                        pc_before,
                        opcode,
                        pc_after: cpu.pc,
                        i_after: cpu.i,
                        v_after: cpu.v,
                    };
                    println!("{}", format_trace_entry(&entry));
                }
                executed += 1;
                if (cycle + 1) % TIMER_TICK_INTERVAL == 0 {
                    cpu.tick_timers();
                }
            }
            Err(e) => {
                println!("CPU error at cycle {}: {}", cycle, e);
                println!("Executed cycles: {}", executed);
                println!("--- CPU State ---");
                println!("{}", format_cpu_snapshot(&cpu.snapshot()));
                println!("--- Display ---");
                print_display(display);
                maybe_save_screenshot(display, opts);
                return 1;
            }
        }
    }

    if opts.dump_state {
        println!("--- CPU State ---");
        println!("{}", format_cpu_snapshot(&cpu.snapshot()));
    }
    println!("Executed cycles: {}", executed);
    print_display(display);
    maybe_save_screenshot(display, opts);
    0
}

fn print_display(display: &Display) {
    println!("Display:");
    let ascii = display.to_ascii();
    // Print each line with a small indent for readability, but keep it simple.
    // to_ascii already ends each row with '\n'.
    for line in ascii.lines() {
        println!("|{}|", line);
    }
}
