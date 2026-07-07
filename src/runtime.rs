//! Runtime: real-time window renderer + input + wall-clock timer scheduling.
//!
//! Wires the headless core (`Cpu`, `Memory`, `Display`, `Input`) into a real
//! window using `minifb`:
//!
//! - a 64x32 framebuffer upscaled by `scale` and shown via `minifb`,
//! - host-keyboard-to-CHIP-8-keypad mapping (QWERTY layout),
//! - wall-clock driven CPU stepping at a configurable cycles-per-second,
//! - 60 Hz delay/sound timer ticking driven by elapsed wall-clock time.
//!
//! The `minifb`-backed `run_window` implementation (plus
//! [`minifb_key_to_host_key`]) is feature-gated behind the `window` cargo
//! feature; it depends on `minifb` and is binary-only. When the feature is
//! disabled, [`run_window`] is replaced by a stub that prints an error and
//! returns a non-zero exit code.
//!
//! The pure helpers ([`HostKey`], [`map_key_to_chip8`], [`parse_cli_args`],
//! [`build_framebuffer`], [`RuntimeConfig`], [`CliOptions`]) and the module
//! constants are featureless and compile (and are unit-tested) without the
//! `window` feature.

use crate::cpu::Cpu;
#[cfg(feature = "window")]
use crate::debugger::format_cpu_snapshot;
use crate::display::{Display, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::input::Input;
use crate::memory::Memory;
#[cfg(feature = "window")]
use crate::timer::TIMER_HZ;
#[cfg(feature = "window")]
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
#[cfg(feature = "window")]
use std::time::Instant;

/// Framebuffer color for a lit (on) pixel. White-ish.
pub const PIXEL_ON: u32 = 0x00FFFFFF; // white
/// Framebuffer color for an unlit (off) pixel. Black.
pub const PIXEL_OFF: u32 = 0x00000000; // black

/// Default cycles for headless mode when no max-cycles is given.
pub const DEFAULT_HEADLESS_MAX_CYCLES: u32 = 1000;
/// Default pixel scale (window pixels per CHIP-8 pixel).
pub const DEFAULT_SCALE: usize = 10;
/// Default CPU cycles per second for window mode.
pub const DEFAULT_HZ: u32 = 700;

/// Default pixel scale used when exporting a screenshot via `--screenshot`
/// when no other scale is available (headless mode does not use the window
/// `--scale`). Used by `main.rs`.
pub const DEFAULT_SCREENSHOT_SCALE: usize = 10;
/// Default number of instructions printed by `--disassemble`.
pub const DEFAULT_DISASSEMBLE_COUNT: usize = 32;

/// Logical host key used for keyboard mapping. This is a small abstraction
/// over the physical key so the mapping can be unit-tested without depending
/// on `minifb::Key` (which is awkward in tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKey {
    Key1,
    Key2,
    Key3,
    Key4,
    Q,
    W,
    E,
    R,
    A,
    S,
    D,
    F,
    Z,
    X,
    C,
    V,
}

/// Map a logical host key to a CHIP-8 hex keypad code (0x0..=0xF).
///
/// Layout (host -> CHIP-8):
///
/// ```text
/// 1 2 3 4     1 2 3 C
/// Q W E R  -> 4 5 6 D
/// A S D F  -> 7 8 9 E
/// Z X C V  -> A 0 B F
/// ```
pub fn map_key_to_chip8(key: HostKey) -> Option<u8> {
    match key {
        HostKey::Key1 => Some(0x1),
        HostKey::Key2 => Some(0x2),
        HostKey::Key3 => Some(0x3),
        HostKey::Key4 => Some(0xC),
        HostKey::Q => Some(0x4),
        HostKey::W => Some(0x5),
        HostKey::E => Some(0x6),
        HostKey::R => Some(0xD),
        HostKey::A => Some(0x7),
        HostKey::S => Some(0x8),
        HostKey::D => Some(0x9),
        HostKey::F => Some(0xE),
        HostKey::Z => Some(0xA),
        HostKey::X => Some(0x0),
        HostKey::C => Some(0xB),
        HostKey::V => Some(0xF),
    }
}

/// Convert a `minifb::Key` into the logical `HostKey` used for mapping.
///
/// Returns `None` for keys that are not part of the CHIP-8 keypad mapping.
///
/// Only available when the `window` cargo feature is enabled, since it
/// depends on `minifb::Key`.
#[cfg(feature = "window")]
pub fn minifb_key_to_host_key(key: Key) -> Option<HostKey> {
    match key {
        Key::Key1 => Some(HostKey::Key1),
        Key::Key2 => Some(HostKey::Key2),
        Key::Key3 => Some(HostKey::Key3),
        Key::Key4 => Some(HostKey::Key4),
        Key::Q => Some(HostKey::Q),
        Key::W => Some(HostKey::W),
        Key::E => Some(HostKey::E),
        Key::R => Some(HostKey::R),
        Key::A => Some(HostKey::A),
        Key::S => Some(HostKey::S),
        Key::D => Some(HostKey::D),
        Key::F => Some(HostKey::F),
        Key::Z => Some(HostKey::Z),
        Key::X => Some(HostKey::X),
        Key::C => Some(HostKey::C),
        Key::V => Some(HostKey::V),
        _ => None,
    }
}

/// Configuration for the runtime window loop.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Pixel scale (window pixels per CHIP-8 pixel).
    pub scale: usize,
    /// Target CPU cycles per second (wall-clock driven).
    pub cycles_per_second: u32,
    /// Optional cap on the total number of CPU cycles to execute.
    pub max_cycles: Option<usize>,
    /// Print one trace line per executed opcode.
    pub trace: bool,
    /// Print a CPU state snapshot when the loop ends.
    pub dump_state: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig {
            scale: DEFAULT_SCALE,
            cycles_per_second: DEFAULT_HZ,
            max_cycles: None,
            trace: false,
            dump_state: false,
        }
    }
}

impl RuntimeConfig {
    /// Create a `RuntimeConfig` with the default values.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Parsed CLI options shared between headless and window modes.
#[derive(Debug, Clone)]
pub struct CliOptions {
    /// Path to the ROM file to load.
    pub rom_path: String,
    /// Optional cap on the number of CPU cycles to execute (headless default:
    /// [`DEFAULT_HEADLESS_MAX_CYCLES`] when `None`).
    pub max_cycles: Option<u32>,
    /// Print one trace line per executed opcode.
    pub trace: bool,
    /// Print a CPU state snapshot when execution ends.
    pub dump_state: bool,
    /// Run in a real window instead of headless.
    pub window: bool,
    /// Pixel scale for window mode.
    pub scale: usize,
    /// CPU cycles per second for window mode.
    pub hz: u32,
    /// Optional path to write a PPM screenshot of the display at the end of
    /// execution (headless) or when the window closes (window mode). `None`
    /// means no screenshot is written.
    pub screenshot: Option<String>,
    /// Print a disassembly listing and exit without executing the ROM.
    pub disassemble: bool,
    /// Number of instructions to print when `disassemble` is enabled.
    pub disassemble_count: usize,
    /// Run the interactive headless debugger REPL instead of executing.
    pub debugger: bool,
}

/// Parse CLI args into [`CliOptions`].
///
/// Accepted forms:
///
/// ```text
/// <rom-path> [max-cycles] [--trace] [--dump-state] [--window] [--scale <n>] [--hz <n>] [--screenshot <path>] [--disassemble] [--disassemble-count <n>] [--debugger]
/// ```
///
/// `args[0]` is the program name. Flags may appear in any order after the rom
/// path. `max-cycles` is an optional positional `u32` that comes right after
/// the rom path (only if it is not a `--` flag). Returns `Err` on: missing rom
/// path, unknown flag, invalid numeric value, `scale == 0`, `hz == 0`,
/// `disassemble_count == 0`, or incompatible mode flags (`--debugger` cannot
/// be combined with `--window` or `--disassemble`).
pub fn parse_cli_args(args: &[String]) -> Result<CliOptions, String> {
    if args.len() < 2 {
        return Err(
            "Usage: chip8-emulator <rom-path> [max-cycles] [--trace] [--dump-state] [--window] [--scale <n>] [--hz <n>] [--screenshot <path>] [--disassemble] [--disassemble-count <n>] [--debugger]"
                .to_string(),
        );
    }
    let rom_path = args[1].clone();

    let mut max_cycles: Option<u32> = None;
    let mut trace = false;
    let mut dump_state = false;
    let mut window = false;
    let mut scale: usize = DEFAULT_SCALE;
    let mut hz: u32 = DEFAULT_HZ;
    let mut screenshot: Option<String> = None;
    let mut disassemble = false;
    let mut disassemble_count = DEFAULT_DISASSEMBLE_COUNT;
    let mut debugger = false;

    let mut idx = 2;
    // The optional max-cycles positional, if present and not a flag, comes
    // immediately after the rom path.
    if let Some(s) = args.get(idx) {
        if !s.starts_with("--") {
            match s.parse::<u32>() {
                Ok(n) => max_cycles = Some(n),
                Err(_) => return Err(format!("invalid max-cycles value: '{s}'")),
            }
            idx += 1;
        }
    }

    while let Some(arg) = args.get(idx) {
        match arg.as_str() {
            "--trace" => trace = true,
            "--dump-state" => dump_state = true,
            "--window" => window = true,
            "--disassemble" => disassemble = true,
            "--debugger" => debugger = true,
            "--scale" => {
                let v = args
                    .get(idx + 1)
                    .ok_or_else(|| "missing value for --scale".to_string())?;
                let n = v
                    .parse::<usize>()
                    .map_err(|_| format!("invalid scale value: '{v}'"))?;
                if n == 0 {
                    return Err("scale must be greater than 0".to_string());
                }
                scale = n;
                idx += 1;
            }
            "--hz" => {
                let v = args
                    .get(idx + 1)
                    .ok_or_else(|| "missing value for --hz".to_string())?;
                let n = v
                    .parse::<u32>()
                    .map_err(|_| format!("invalid hz value: '{v}'"))?;
                if n == 0 {
                    return Err("hz must be greater than 0".to_string());
                }
                hz = n;
                idx += 1;
            }
            "--screenshot" => {
                let v = args
                    .get(idx + 1)
                    .ok_or_else(|| "missing value for --screenshot".to_string())?;
                screenshot = Some(v.clone());
                idx += 1;
            }
            "--disassemble-count" => {
                let v = args
                    .get(idx + 1)
                    .ok_or_else(|| "missing value for --disassemble-count".to_string())?;
                let n = v
                    .parse::<usize>()
                    .map_err(|_| format!("invalid disassemble-count value: '{v}'"))?;
                if n == 0 {
                    return Err("disassemble-count must be greater than 0".to_string());
                }
                disassemble_count = n;
                idx += 1;
            }
            s if s.starts_with("--") => return Err(format!("Unknown flag: {arg}")),
            _ => return Err(format!("Unexpected argument: {arg}")),
        }
        idx += 1;
    }

    if window && disassemble {
        return Err("--disassemble cannot be used with --window".to_string());
    }
    if debugger && window {
        return Err("--debugger cannot be used with --window".to_string());
    }
    if debugger && disassemble {
        return Err("--debugger cannot be used with --disassemble".to_string());
    }

    Ok(CliOptions {
        rom_path,
        max_cycles,
        trace,
        dump_state,
        window,
        scale,
        hz,
        screenshot,
        disassemble,
        disassemble_count,
        debugger,
    })
}

/// Build a window-sized framebuffer from the display buffer.
///
/// `buffer` must be exactly `DISPLAY_WIDTH * scale * DISPLAY_HEIGHT * scale`
/// long. Each display pixel is expanded to a `scale` x `scale` block of
/// [`PIXEL_ON`] or [`PIXEL_OFF`].
pub fn build_framebuffer(
    display: &Display,
    scale: usize,
    buffer: &mut [u32],
) -> Result<(), String> {
    if scale == 0 {
        return Err("build_framebuffer: scale must be greater than 0".to_string());
    }
    let width = DISPLAY_WIDTH * scale;
    let height = DISPLAY_HEIGHT * scale;
    if buffer.len() != width * height {
        return Err(format!(
            "build_framebuffer: buffer length {} does not match expected {} ({}x{})",
            buffer.len(),
            width * height,
            width,
            height
        ));
    }
    for y in 0..DISPLAY_HEIGHT {
        for x in 0..DISPLAY_WIDTH {
            let on = display.get_pixel(x, y)?;
            let color = if on { PIXEL_ON } else { PIXEL_OFF };
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x * scale + dx;
                    let py = y * scale + dy;
                    buffer[py * width + px] = color;
                }
            }
        }
    }
    Ok(())
}

/// Run the emulator in a real minifb window with wall-clock timing.
///
/// Returns the process exit code: `0` on clean close (window closed by the
/// user or max-cycles reached), `1` on a CPU error or window-creation error.
///
/// This function is not unit-tested; it is driven by the binary when invoked
/// with the `--window` flag. When the `window` cargo feature is disabled this
/// symbol is replaced by a stub that prints an error and returns `1`.
#[cfg(feature = "window")]
pub fn run_window(
    cpu: &mut Cpu,
    memory: &mut Memory,
    display: &mut Display,
    input: &mut Input,
    config: &RuntimeConfig,
) -> i32 {
    let width = DISPLAY_WIDTH * config.scale;
    let height = DISPLAY_HEIGHT * config.scale;

    let mut window = match Window::new(
        "CHIP-8 emulator",
        width,
        height,
        WindowOptions {
            resize: false,
            scale: Scale::X1,
            scale_mode: ScaleMode::Stretch,
            ..WindowOptions::default()
        },
    ) {
        Ok(w) => w,
        Err(e) => {
            println!("Error: failed to create window: {e}");
            return 1;
        }
    };

    let mut buffer: Vec<u32> = vec![PIXEL_OFF; width * height];

    let timer_step = 1.0 / TIMER_HZ as f64;
    let mut last = Instant::now();
    let mut cycle_accumulator: f64 = 0.0;
    let mut timer_accumulator: f64 = 0.0;
    let mut executed: usize = 0;

    while window.is_open() {
        // --- Input: clear and re-sample current pressed keys each frame. ---
        input.clear();
        let mut escape_pressed = false;
        for key in window.get_keys() {
            if key == Key::Escape {
                escape_pressed = true;
                break;
            }
            if let Some(host) = minifb_key_to_host_key(key) {
                if let Some(chip8) = map_key_to_chip8(host) {
                    let _ = input.press_key(chip8);
                }
            }
        }
        if escape_pressed {
            break;
        }

        // --- Timing. ---
        let now = Instant::now();
        let mut dt = now.duration_since(last).as_secs_f64();
        last = now;
        // Clamp dt to avoid the "spiral of death" after a stall.
        if dt > 0.1 {
            dt = 0.1;
        }

        // --- CPU stepping at cycles_per_second. ---
        cycle_accumulator += dt * config.cycles_per_second as f64;
        let mut to_run = cycle_accumulator.floor() as usize;
        if let Some(cap) = config.max_cycles {
            let remaining = cap.saturating_sub(executed);
            if to_run > remaining {
                to_run = remaining;
            }
        }
        for _ in 0..to_run {
            let pc_before = cpu.pc;
            match cpu.step(memory, display, input) {
                Ok(opcode) => {
                    if config.trace {
                        println!(
                            "#{:06} PC=0x{:04X} OP=0x{:04X} -> PC=0x{:04X} I=0x{:04X}",
                            executed, pc_before, opcode, cpu.pc, cpu.i
                        );
                    }
                    executed += 1;
                }
                Err(e) => {
                    println!("CPU error at cycle {}: {}", executed, e);
                    println!("Executed cycles: {}", executed);
                    println!("--- CPU State ---");
                    println!("{}", format_cpu_snapshot(&cpu.snapshot()));
                    return 1;
                }
            }
            if let Some(cap) = config.max_cycles {
                if executed >= cap {
                    break;
                }
            }
        }
        cycle_accumulator -= to_run as f64;

        // --- 60 Hz timer ticking. ---
        timer_accumulator += dt;
        while timer_accumulator >= timer_step {
            cpu.tick_timers();
            timer_accumulator -= timer_step;
        }

        // --- Render. ---
        if let Err(e) = build_framebuffer(display, config.scale, &mut buffer) {
            println!("Error: {e}");
            return 1;
        }
        if let Err(e) = window.update_with_buffer(&buffer, width, height) {
            println!("Error: failed to update window: {e}");
            return 1;
        }

        if let Some(cap) = config.max_cycles {
            if executed >= cap {
                break;
            }
        }
    }

    if config.dump_state {
        println!("--- CPU State ---");
        println!("{}", format_cpu_snapshot(&cpu.snapshot()));
    }
    println!("Executed cycles: {}", executed);
    0
}

#[cfg(not(feature = "window"))]
pub fn run_window(
    _cpu: &mut Cpu,
    _memory: &mut Memory,
    _display: &mut Display,
    _input: &mut Input,
    _config: &RuntimeConfig,
) -> i32 {
    eprintln!("Window runtime is not enabled.");
    eprintln!("Rebuild with:");
    eprintln!("cargo run --features window -- <rom-path> --window");
    1
}
