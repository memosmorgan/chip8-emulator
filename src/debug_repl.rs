//! Interactive debugger / REPL.
//!
//! An interactive, headless debugger REPL that drives the existing CHIP-8
//! core (`Cpu`, `Memory`, `Display`, `Input`) without opening a window. It
//! reuses [`crate::disassembler::disassemble_range`] for opcode listings and
//! [`crate::debugger::format_cpu_snapshot`] for register dumps, and provides
//! a breakpoint manager plus a watchpoint manager (V registers and single
//! memory bytes). Command parsing is split into [`parse_debug_command`] and
//! [`parse_u16_value`] so it can be unit-tested without touching stdin.
//!
//! See `docs/debugger-repl.md` for the full command reference.

use std::fmt::Write as _;
use std::io::{stdin, stdout, Write};

use crate::cpu::Cpu;
use crate::debugger::format_cpu_snapshot;
use crate::disassembler::{disassemble_opcode, disassemble_range};
use crate::display::Display;
use crate::input::Input;
use crate::memory::{Memory, MEMORY_SIZE};

/// Default number of instructions printed by `disasm` when no count is given.
pub const DEFAULT_DISASM_COUNT: usize = 10;
/// Maximum number of bytes the `mem` command will dump in one go.
pub const MAX_MEM_DUMP_LEN: usize = 256;

/// A breakpoint on a CHIP-8 program address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Breakpoint {
    pub id: u32,
    pub address: u16,
}

/// A watchpoint tracking either a V register or a single memory byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchTarget {
    /// A V register, indexed `0..=15`.
    Reg { index: u8 },
    /// A single byte of memory at `address`.
    Mem { address: u16 },
}

/// A watchpoint with an assigned id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Watchpoint {
    pub id: u32,
    pub target: WatchTarget,
}

/// Manages a collection of address breakpoints.
#[derive(Debug, Default)]
pub struct BreakpointManager {
    breakpoints: Vec<Breakpoint>,
    next_id: u32,
}

impl BreakpointManager {
    /// Create an empty manager. The first assigned id is `1`.
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a breakpoint at `address`, assigning the next id. Does not dedupe.
    pub fn add(&mut self, address: u16) -> Breakpoint {
        let bp = Breakpoint {
            id: self.next_id,
            address,
        };
        self.next_id += 1;
        self.breakpoints.push(bp);
        bp
    }

    /// Remove and return the breakpoint with the given id.
    pub fn remove(&mut self, id: u32) -> Result<Breakpoint, String> {
        if let Some(pos) = self.breakpoints.iter().position(|b| b.id == id) {
            Ok(self.breakpoints.remove(pos))
        } else {
            Err(format!("No breakpoint with id {id}"))
        }
    }

    /// Remove all breakpoints. `next_id` is left unchanged.
    pub fn clear(&mut self) {
        self.breakpoints.clear();
    }

    /// Borrow the current breakpoints as a slice.
    pub fn list(&self) -> &[Breakpoint] {
        &self.breakpoints
    }

    /// Whether any breakpoint is set at `address`.
    pub fn contains_address(&self, address: u16) -> bool {
        self.breakpoints.iter().any(|b| b.address == address)
    }

    /// Whether no breakpoints are set.
    pub fn is_empty(&self) -> bool {
        self.breakpoints.is_empty()
    }
}

/// Manages a collection of watchpoints.
#[derive(Debug, Default)]
pub struct WatchManager {
    watchpoints: Vec<Watchpoint>,
    next_id: u32,
}

impl WatchManager {
    /// Create an empty manager. The first assigned id is `1`.
    pub fn new() -> Self {
        Self {
            watchpoints: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a watchpoint on `target`, assigning the next id. Does not dedupe.
    pub fn add(&mut self, target: WatchTarget) -> Watchpoint {
        let wp = Watchpoint {
            id: self.next_id,
            target,
        };
        self.next_id += 1;
        self.watchpoints.push(wp);
        wp
    }

    /// Remove and return the watchpoint with the given id.
    pub fn remove(&mut self, id: u32) -> Result<Watchpoint, String> {
        if let Some(pos) = self.watchpoints.iter().position(|w| w.id == id) {
            Ok(self.watchpoints.remove(pos))
        } else {
            Err(format!("No watchpoint with id {id}"))
        }
    }

    /// Remove all watchpoints. `next_id` is left unchanged.
    pub fn clear(&mut self) {
        self.watchpoints.clear();
    }

    /// Borrow the current watchpoints as a slice.
    pub fn list(&self) -> &[Watchpoint] {
        &self.watchpoints
    }

    /// Whether no watchpoints are set.
    pub fn is_empty(&self) -> bool {
        self.watchpoints.is_empty()
    }
}

/// A parsed debugger command. Produced by [`parse_debug_command`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugCommand {
    /// Show the help text.
    Help,
    /// Print the CPU register/state snapshot.
    Regs,
    /// Execute exactly one CPU instruction.
    Step,
    /// Execute `n` CPU instructions, stopping on the first error or breakpoint hit.
    Continue(usize),
    /// Disassemble `count` instructions starting at `address` (defaults to
    /// the current PC and [`DEFAULT_DISASM_COUNT`]).
    Disasm { address: Option<u16>, count: usize },
    /// Dump `len` bytes of memory starting at `address`.
    Mem { address: u16, len: usize },
    /// Print the call stack (SP + used slots).
    Stack,
    /// Print the display buffer as ASCII.
    Display,
    /// Print the delay and sound timer values.
    Timers,
    /// Tick the delay/sound timers once and print the new values.
    Tick,
    /// Exit the debugger.
    Quit,
    /// Set a breakpoint at `address` (`break <addr>`).
    Break(u16),
    /// List all breakpoints (`breaks`).
    Breaks,
    /// Delete the breakpoint with the given id (`delete <id>`).
    DeleteBreak(u32),
    /// Remove all breakpoints (`clear-breaks`).
    ClearBreaks,
    /// Set a watchpoint on V register `index` (`watch reg <VX>`).
    WatchReg(u8),
    /// Set a watchpoint on memory byte `address` (`watch mem <addr>`).
    WatchMem(u16),
    /// List all watchpoints (`watches`).
    Watches,
    /// Delete the watchpoint with the given id (`delete-watch <id>`).
    DeleteWatch(u32),
    /// Remove all watchpoints (`clear-watches`).
    ClearWatches,
}

/// Parse a single numeric value as a `u16`.
///
/// Accepts decimal (`"512"`) or hexadecimal with a case-insensitive `0x`
/// prefix (`"0x200"`, `"0xABCD"`, `"0xab"`). Hex digits may be upper or
/// lower case. Empty input returns `Err("empty value")`.
pub fn parse_u16_value(input: &str) -> Result<u16, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty value".to_string());
    }
    let lower = s.to_ascii_lowercase();
    if let Some(hex) = lower.strip_prefix("0x") {
        if hex.is_empty() {
            return Err(format!("invalid value: '{s}'"));
        }
        match u32::from_str_radix(hex, 16) {
            Ok(n) if n <= u16::MAX as u32 => Ok(n as u16),
            Ok(_) => Err(format!("value out of range for u16: '{s}'")),
            Err(_) => Err(format!("invalid value: '{s}'")),
        }
    } else {
        match s.parse::<u32>() {
            Ok(n) if n <= u16::MAX as u32 => Ok(n as u16),
            Ok(_) => Err(format!("value out of range for u16: '{s}'")),
            Err(_) => Err(format!("invalid value: '{s}'")),
        }
    }
}

/// Parse a V-register index token for `watch reg`.
///
/// Accepts:
/// - `VX` / `vx` (single hex digit after the `v`, case-insensitive),
/// - `0x0`..`0xF` (hex with `0x` prefix, value must be `<= 15`),
/// - `0`..`15` (decimal).
///
/// Error messages mention the offending token.
pub fn parse_reg_index(input: &str) -> Result<u8, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty register index".to_string());
    }
    let lower = s.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix('v') {
        // Single hex digit after the V prefix.
        if rest.len() != 1 {
            return Err(format!(
                "invalid register index: '{s}' (expected single hex digit after V)"
            ));
        }
        match u8::from_str_radix(rest, 16) {
            Ok(n) if n <= 0xF => Ok(n),
            Ok(_) => Err(format!("register index out of range (0..=15): '{s}'")),
            Err(_) => Err(format!("invalid register index: '{s}'")),
        }
    } else if let Some(hex) = lower.strip_prefix("0x") {
        match u8::from_str_radix(hex, 16) {
            Ok(n) if n <= 0xF => Ok(n),
            Ok(_) => Err(format!("register index out of range (0..=15): '{s}'")),
            Err(_) => Err(format!("invalid register index: '{s}'")),
        }
    } else {
        match s.parse::<u8>() {
            Ok(n) if n <= 0xF => Ok(n),
            Ok(_) => Err(format!("register index out of range (0..=15): '{s}'")),
            Err(_) => Err(format!("invalid register index: '{s}'")),
        }
    }
}

/// Parse a single line of debugger input into a [`DebugCommand`].
///
/// The command word is matched case-insensitively. Arguments are
/// whitespace-separated. Returns a descriptive `Err` on bad input.
pub fn parse_debug_command(input: &str) -> Result<DebugCommand, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("empty command".to_string());
    }
    let mut parts = trimmed.split_whitespace();
    let first = parts.next().expect("non-empty after trim");
    let cmd = first.to_ascii_lowercase();

    match cmd.as_str() {
        "help" => Ok(DebugCommand::Help),
        "regs" => Ok(DebugCommand::Regs),
        "step" | "s" => Ok(DebugCommand::Step),
        "continue" | "c" => {
            let arg = parts
                .next()
                .ok_or_else(|| "continue requires a count: 'continue <n>'".to_string())?;
            let n = arg
                .parse::<usize>()
                .map_err(|_| format!("invalid continue count: '{arg}'"))?;
            if n == 0 {
                return Err("continue count must be greater than 0".to_string());
            }
            Ok(DebugCommand::Continue(n))
        }
        "disasm" => {
            let collected: Vec<&str> = parts.collect();
            match collected.len() {
                0 => Ok(DebugCommand::Disasm {
                    address: None,
                    count: DEFAULT_DISASM_COUNT,
                }),
                1 => {
                    let n = collected[0]
                        .parse::<usize>()
                        .map_err(|_| format!("invalid disasm count: '{}'", collected[0]))?;
                    if n == 0 {
                        return Err("disasm count must be greater than 0".to_string());
                    }
                    Ok(DebugCommand::Disasm {
                        address: None,
                        count: n,
                    })
                }
                2 => {
                    let address = parse_u16_value(collected[0])?;
                    let n = collected[1]
                        .parse::<usize>()
                        .map_err(|_| format!("invalid disasm count: '{}'", collected[1]))?;
                    if n == 0 {
                        return Err("disasm count must be greater than 0".to_string());
                    }
                    Ok(DebugCommand::Disasm {
                        address: Some(address),
                        count: n,
                    })
                }
                _ => Err("disasm takes at most 2 args".to_string()),
            }
        }
        "mem" => {
            let collected: Vec<&str> = parts.collect();
            if collected.len() != 2 {
                return Err("mem requires <addr> <len>".to_string());
            }
            let address = parse_u16_value(collected[0])?;
            let len = collected[1]
                .parse::<usize>()
                .map_err(|_| format!("invalid mem length: '{}'", collected[1]))?;
            if len == 0 {
                return Err("mem length must be greater than 0".to_string());
            }
            if len > MAX_MEM_DUMP_LEN {
                return Err(format!(
                    "mem length exceeds maximum of {}",
                    MAX_MEM_DUMP_LEN
                ));
            }
            Ok(DebugCommand::Mem { address, len })
        }
        "stack" => Ok(DebugCommand::Stack),
        "display" => Ok(DebugCommand::Display),
        "timers" => Ok(DebugCommand::Timers),
        "tick" => Ok(DebugCommand::Tick),
        "quit" | "q" => Ok(DebugCommand::Quit),
        "break" => {
            let collected: Vec<&str> = parts.collect();
            if collected.len() != 1 {
                return Err("break requires an address: 'break <addr>'".to_string());
            }
            let address = parse_u16_value(collected[0])?;
            Ok(DebugCommand::Break(address))
        }
        "breaks" => {
            if parts.next().is_some() {
                return Err("breaks takes no arguments".to_string());
            }
            Ok(DebugCommand::Breaks)
        }
        "delete" => {
            let arg = parts
                .next()
                .ok_or_else(|| "delete requires an id: 'delete <id>'".to_string())?;
            if parts.next().is_some() {
                return Err("delete takes exactly one argument: 'delete <id>'".to_string());
            }
            let id = arg
                .parse::<u32>()
                .map_err(|_| format!("invalid breakpoint id: '{arg}'"))?;
            Ok(DebugCommand::DeleteBreak(id))
        }
        "clear-breaks" => {
            if parts.next().is_some() {
                return Err("clear-breaks takes no arguments".to_string());
            }
            Ok(DebugCommand::ClearBreaks)
        }
        "watch" => {
            let sub = parts.next().ok_or_else(|| {
                "watch requires a subcommand: 'watch reg <VX>' or 'watch mem <addr>'".to_string()
            })?;
            match sub.to_ascii_lowercase().as_str() {
                "reg" => {
                    let arg = parts.next().ok_or_else(|| {
                        "watch reg requires a register: 'watch reg <VX>'".to_string()
                    })?;
                    if parts.next().is_some() {
                        return Err(
                            "watch reg takes exactly one argument: 'watch reg <VX>'".to_string()
                        );
                    }
                    let index = parse_reg_index(arg)?;
                    Ok(DebugCommand::WatchReg(index))
                }
                "mem" => {
                    let arg = parts.next().ok_or_else(|| {
                        "watch mem requires an address: 'watch mem <addr>'".to_string()
                    })?;
                    if parts.next().is_some() {
                        return Err(
                            "watch mem takes exactly one argument: 'watch mem <addr>'".to_string()
                        );
                    }
                    let address = parse_u16_value(arg)?;
                    Ok(DebugCommand::WatchMem(address))
                }
                other => Err(format!(
                    "unknown watch subcommand: '{other}' (expected 'reg' or 'mem')"
                )),
            }
        }
        "watches" => {
            if parts.next().is_some() {
                return Err("watches takes no arguments".to_string());
            }
            Ok(DebugCommand::Watches)
        }
        "delete-watch" => {
            let arg = parts
                .next()
                .ok_or_else(|| "delete-watch requires an id: 'delete-watch <id>'".to_string())?;
            if parts.next().is_some() {
                return Err(
                    "delete-watch takes exactly one argument: 'delete-watch <id>'".to_string(),
                );
            }
            let id = arg
                .parse::<u32>()
                .map_err(|_| format!("invalid watchpoint id: '{arg}'"))?;
            Ok(DebugCommand::DeleteWatch(id))
        }
        "clear-watches" => {
            if parts.next().is_some() {
                return Err("clear-watches takes no arguments".to_string());
            }
            Ok(DebugCommand::ClearWatches)
        }
        other => Err(format!(
            "Unknown command: {}\nType 'help' for available commands.",
            other
        )),
    }
}

/// Format `len` bytes of memory starting at `address` as a hex dump.
///
/// 16 bytes per line, each line prefixed with `XXXX: `. Returns `Err` if the
/// requested range falls outside the 4096-byte address space.
pub fn format_memory_dump(memory: &Memory, address: u16, len: usize) -> Result<String, String> {
    let start = address as usize;
    if start.checked_add(len).is_none_or(|end| end > MEMORY_SIZE) {
        return Err(format!(
            "memory dump range 0x{:04X}..0x{:04X} out of bounds (memory size = {})",
            start,
            start + len,
            MEMORY_SIZE
        ));
    }

    let mut out = String::new();
    let mut offset = 0;
    while offset < len {
        let line_addr = start + offset;
        write!(out, "{:04X}:", line_addr).expect("write to String");
        let chunk_end = (offset + 16).min(len);
        for i in offset..chunk_end {
            let byte = memory.read_byte((start + i) as u16)?;
            write!(out, " {:02X}", byte).expect("write to String");
        }
        if chunk_end < len {
            out.push('\n');
        }
        offset = chunk_end;
    }
    Ok(out)
}

/// Print the help text listing all debugger commands.
fn print_help() {
    println!("Available commands:");
    println!("  help              Show this help text");
    println!("  regs              Print CPU register/state snapshot");
    println!("  step (s)          Execute one CPU instruction");
    println!("  continue <n> (c)  Execute n instructions, stopping on error/breakpoint");
    println!("  disasm [<addr> <count>]  Disassemble instructions (default: PC, 10)");
    println!("  mem <addr> <len>  Dump memory bytes (len <= 256)");
    println!("  stack             Print the call stack (SP + used slots)");
    println!("  display           Print the display buffer as ASCII");
    println!("  timers            Print delay/sound timer values");
    println!("  tick              Tick delay/sound timers once");
    println!("  break <addr>      Set a breakpoint at address");
    println!("  breaks            List all breakpoints");
    println!("  delete <id>       Delete breakpoint by id");
    println!("  clear-breaks      Remove all breakpoints");
    println!("  watch reg <VX>    Watch V register (VX, decimal 0..15, or 0x0..0xF)");
    println!("  watch mem <addr>  Watch a single memory byte");
    println!("  watches           List all watchpoints");
    println!("  delete-watch <id> Delete watchpoint by id");
    println!("  clear-watches     Remove all watchpoints");
    println!("  quit (q)          Exit the debugger");
}

/// Print the display buffer as ASCII, framed with `|...|` per row.
fn print_display(display: &Display) {
    println!("Display:");
    for line in display.to_ascii().lines() {
        println!("|{}|", line);
    }
}

/// Print the CPU state snapshot, prefixed with a header line.
fn print_cpu_state(cpu: &Cpu) {
    println!("--- CPU State ---");
    print!("{}", format_cpu_snapshot(&cpu.snapshot()));
}

/// Describe a [`WatchTarget`] for display (`V0`..`VF` or `MEM[0xADDR]`).
fn describe_watch_target(target: WatchTarget) -> String {
    match target {
        WatchTarget::Reg { index } => format!("V{:X}", index),
        WatchTarget::Mem { address } => format!("MEM[0x{:04X}]", address),
    }
}

/// Read the current value tracked by a watchpoint.
fn read_watch_value(cpu: &Cpu, memory: &Memory, target: WatchTarget) -> Result<u8, String> {
    match target {
        WatchTarget::Reg { index } => Ok(cpu.v[index as usize]),
        WatchTarget::Mem { address } => memory.read_byte(address),
    }
}

/// Snapshot every watchpoint's current value, in [`WatchManager::list`] order.
fn snapshot_watches(cpu: &Cpu, memory: &Memory, watches: &WatchManager) -> Result<Vec<u8>, String> {
    watches
        .list()
        .iter()
        .map(|wp| read_watch_value(cpu, memory, wp.target))
        .collect()
}

/// Print any watchpoint changes given before/after value snapshots.
fn report_watch_changes(watches: &WatchManager, before: &[u8], after: &[u8]) {
    for (wp, (a, b)) in watches.list().iter().zip(before.iter().zip(after.iter())) {
        if a != b {
            println!(
                "Watch {} changed: {} 0x{:02X} -> 0x{:02X}",
                wp.id,
                describe_watch_target(wp.target),
                a,
                b
            );
        }
    }
}

/// Run the interactive debugger REPL.
///
/// Reads commands from stdin, executes them against the provided CPU/memory/
/// display/input, and writes output to stdout. Returns the process exit code
/// (0 on clean quit or EOF, 1 on stdin read error).
pub fn run_debugger(
    cpu: &mut Cpu,
    memory: &mut Memory,
    display: &mut Display,
    input: &Input,
) -> i32 {
    let stdin = stdin();
    let mut stdout = stdout();

    let mut breaks = BreakpointManager::new();
    let mut watches = WatchManager::new();

    println!("CHIP-8 interactive debugger. Type 'help' for commands.");

    loop {
        print!("debug> ");
        if stdout.flush().is_err() {
            return 1;
        }

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => return 0, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("stdin read error: {e}");
                return 1;
            }
        }

        if line.trim().is_empty() {
            continue;
        }

        let cmd = match parse_debug_command(&line) {
            Ok(c) => c,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };

        match cmd {
            DebugCommand::Help => print_help(),
            DebugCommand::Regs => print_cpu_state(cpu),
            DebugCommand::Step => {
                let pc_before = cpu.pc;
                if breaks.contains_address(pc_before) {
                    println!("Note: breakpoint at 0x{:04X}", pc_before);
                }
                let before = snapshot_watches(cpu, memory, &watches).unwrap_or_default();
                match cpu.step(memory, display, input) {
                    Ok(opcode) => {
                        println!(
                            "{:04X}: {:04X}  {}",
                            pc_before,
                            opcode,
                            disassemble_opcode(opcode)
                        );
                        println!("PC: 0x{:04X} -> 0x{:04X}", pc_before, cpu.pc);
                        if let Ok(after) = snapshot_watches(cpu, memory, &watches) {
                            report_watch_changes(&watches, &before, &after);
                        }
                    }
                    Err(e) => {
                        println!("CPU error: {e}");
                        print_cpu_state(cpu);
                        println!("--- Display ---");
                        print_display(display);
                    }
                }
            }
            DebugCommand::Continue(n) => {
                let mut executed = 0usize;
                let mut hit_breakpoint = false;
                for _ in 0..n {
                    let pc_before = cpu.pc;
                    if breaks.contains_address(pc_before) {
                        println!("Breakpoint hit at 0x{:04X}", pc_before);
                        hit_breakpoint = true;
                        break;
                    }
                    let before = snapshot_watches(cpu, memory, &watches).unwrap_or_default();
                    match cpu.step(memory, display, input) {
                        Ok(opcode) => {
                            println!(
                                "{:04X}: {:04X}  {}",
                                pc_before,
                                opcode,
                                disassemble_opcode(opcode)
                            );
                            if let Ok(after) = snapshot_watches(cpu, memory, &watches) {
                                report_watch_changes(&watches, &before, &after);
                            }
                            executed += 1;
                        }
                        Err(e) => {
                            println!("CPU error at instruction {}: {e}", executed);
                            print_cpu_state(cpu);
                            println!("--- Display ---");
                            print_display(display);
                            break;
                        }
                    }
                }
                if hit_breakpoint {
                    println!("Stopped at breakpoint. Executed {} instructions.", executed);
                } else {
                    println!("Executed {} instructions.", executed);
                }
                println!("PC=0x{:04X}", cpu.pc);
            }
            DebugCommand::Disasm { address, count } => {
                let start = address.unwrap_or(cpu.pc);
                match disassemble_range(memory, start, count) {
                    Ok(instructions) => {
                        for ins in instructions {
                            println!("{:04X}: {:04X}  {}", ins.address, ins.opcode, ins.mnemonic);
                        }
                    }
                    Err(e) => println!("Error: {e}"),
                }
            }
            DebugCommand::Mem { address, len } => match format_memory_dump(memory, address, len) {
                Ok(dump) => println!("{}", dump),
                Err(e) => println!("Error: {e}"),
            },
            DebugCommand::Stack => {
                println!("SP={}", cpu.sp);
                for i in 0..cpu.sp as usize {
                    println!("[{}] 0x{:04X}", i, cpu.stack[i]);
                }
            }
            DebugCommand::Display => print_display(display),
            DebugCommand::Timers => {
                println!("DT={} ST={}", cpu.delay_timer, cpu.sound_timer);
            }
            DebugCommand::Tick => {
                cpu.tick_timers();
                println!("DT={} ST={}", cpu.delay_timer, cpu.sound_timer);
            }
            DebugCommand::Break(addr) => {
                let bp = breaks.add(addr);
                println!("Added breakpoint {} at 0x{:04X}", bp.id, bp.address);
            }
            DebugCommand::Breaks => {
                if breaks.is_empty() {
                    println!("No breakpoints.");
                } else {
                    for bp in breaks.list() {
                        println!("{:<2}  0x{:04X}", bp.id, bp.address);
                    }
                }
            }
            DebugCommand::DeleteBreak(id) => match breaks.remove(id) {
                Ok(bp) => println!("Deleted breakpoint {} at 0x{:04X}", bp.id, bp.address),
                Err(e) => println!("Error: {e}"),
            },
            DebugCommand::ClearBreaks => {
                breaks.clear();
                println!("Cleared all breakpoints.");
            }
            DebugCommand::WatchReg(index) => {
                let wp = watches.add(WatchTarget::Reg { index });
                println!("Added watchpoint {} on V{:X}", wp.id, index);
            }
            DebugCommand::WatchMem(addr) => {
                let wp = watches.add(WatchTarget::Mem { address: addr });
                println!("Added watchpoint {} on MEM[0x{:04X}]", wp.id, addr);
            }
            DebugCommand::Watches => {
                if watches.is_empty() {
                    println!("No watchpoints.");
                } else {
                    for wp in watches.list() {
                        println!("{:<2}  {}", wp.id, describe_watch_target(wp.target));
                    }
                }
            }
            DebugCommand::DeleteWatch(id) => match watches.remove(id) {
                Ok(wp) => println!(
                    "Deleted watchpoint {} ({})",
                    wp.id,
                    describe_watch_target(wp.target)
                ),
                Err(e) => println!("Error: {e}"),
            },
            DebugCommand::ClearWatches => {
                watches.clear();
                println!("Cleared all watchpoints.");
            }
            DebugCommand::Quit => return 0,
        }
    }
}
