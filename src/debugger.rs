//! Debugging and trace tools.
//!
//! Provides non-interactive debugging primitives:
//! - [`CpuSnapshot`]: a point-in-time copy of the CPU state (registers,
//!   index, program counter, stack pointer, stack, delay/sound timers)
//!   used for state dumps.
//! - [`TraceEntry`]: one fetch/decode/execute cycle recorded as
//!   `pc_before` / `opcode` / `pc_after` / `i_after` / `v_after`, plus the
//!   cycle index, used for opcode traces.
//! - [`format_cpu_snapshot`] and [`format_trace_entry`]: human-readable
//!   formatters for the two records above.
//!
//! `Cpu::snapshot()` constructs a [`CpuSnapshot`] by reading the CPU fields
//! through an immutable borrow; this module never mutates CPU state.

use std::fmt::Write as _;

/// Number of general-purpose V registers and stack slots (matches the CPU).
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;

/// A point-in-time copy of the CPU state, used for state dumps.
///
/// Built by [`Cpu::snapshot`](crate::cpu::Cpu::snapshot). This is a plain
/// data snapshot — it does not borrow the CPU and does not mutate it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuSnapshot {
    /// Program counter.
    pub pc: u16,
    /// Index register.
    pub i: u16,
    /// Stack pointer.
    pub sp: u8,
    /// General-purpose 8-bit registers V0..VF.
    pub v: [u8; NUM_REGISTERS],
    /// Call stack.
    pub stack: [u16; STACK_SIZE],
    /// Delay timer.
    pub delay_timer: u8,
    /// Sound timer.
    pub sound_timer: u8,
}

impl CpuSnapshot {
    /// Create an all-zero snapshot (pc = 0, all registers/timers cleared).
    pub fn new() -> Self {
        CpuSnapshot {
            pc: 0,
            i: 0,
            sp: 0,
            v: [0u8; NUM_REGISTERS],
            stack: [0u16; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

impl Default for CpuSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// One fetch/decode/execute cycle recorded for tracing.
///
/// `pc_before` is the program counter at the start of the cycle, `opcode`
/// is the raw 16-bit instruction that was executed, and the `_after` fields
/// capture the CPU state right after execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEntry {
    /// Index of the cycle this entry records (0-based).
    pub cycle: usize,
    /// Program counter at the start of the cycle (before fetch).
    pub pc_before: u16,
    /// Raw 16-bit opcode that was executed.
    pub opcode: u16,
    /// Program counter after execution.
    pub pc_after: u16,
    /// Index register value after execution.
    pub i_after: u16,
    /// V register values after execution.
    pub v_after: [u8; NUM_REGISTERS],
}

impl Default for TraceEntry {
    fn default() -> Self {
        TraceEntry {
            cycle: 0,
            pc_before: 0,
            opcode: 0,
            pc_after: 0,
            i_after: 0,
            v_after: [0u8; NUM_REGISTERS],
        }
    }
}

/// Format a [`CpuSnapshot`] as a readable multi-line string.
///
/// Example output:
/// ```text
/// PC=0x0200 I=0x0300 SP=0 DT=0 ST=0
/// V0=00 V1=0A V2=00 V3=00 V4=00 V5=00 V6=00 V7=00 V8=00 V9=00 VA=00 VB=00 VC=00 VD=00 VE=00 VF=01
/// STACK[0]=0x0202 STACK[1]=0x0000 ... STACK[15]=0x0000
/// ```
///
/// `PC` and `I` are printed as `0x{:04X}`; `SP`, `DT`, `ST` as plain decimal;
/// V registers as `{:02X}` (no `0x` prefix); stack entries as `0x{:04X}`.
pub fn format_cpu_snapshot(snapshot: &CpuSnapshot) -> String {
    let mut out = String::new();

    writeln!(
        out,
        "PC=0x{:04X} I=0x{:04X} SP={} DT={} ST={}",
        snapshot.pc, snapshot.i, snapshot.sp, snapshot.delay_timer, snapshot.sound_timer
    )
    .expect("writing to String never fails");

    let reg_labels = [
        "V0", "V1", "V2", "V3", "V4", "V5", "V6", "V7", "V8", "V9", "VA", "VB", "VC", "VD", "VE",
        "VF",
    ];
    let mut line = String::from("");
    for (idx, label) in reg_labels.iter().enumerate() {
        write!(line, "{}={:02X}", label, snapshot.v[idx]).expect("writing to String never fails");
        if idx < NUM_REGISTERS - 1 {
            line.push(' ');
        }
    }
    writeln!(out, "{}", line).expect("writing to String never fails");

    let mut stack_line = String::from("");
    for idx in 0..STACK_SIZE {
        write!(stack_line, "STACK[{}]=0x{:04X}", idx, snapshot.stack[idx])
            .expect("writing to String never fails");
        if idx < STACK_SIZE - 1 {
            stack_line.push(' ');
        }
    }
    writeln!(out, "{}", stack_line).expect("writing to String never fails");

    out
}

/// Format a [`TraceEntry`] as a single-line trace string.
///
/// Example output:
/// ```text
/// #000012 PC=0x0200 OP=0x6A0F -> PC=0x0202 I=0x0000
/// ```
///
/// The cycle index is zero-padded to 6 digits.
pub fn format_trace_entry(entry: &TraceEntry) -> String {
    format!(
        "#{:06} PC=0x{:04X} OP=0x{:04X} -> PC=0x{:04X} I=0x{:04X}",
        entry.cycle, entry.pc_before, entry.opcode, entry.pc_after, entry.i_after
    )
}
