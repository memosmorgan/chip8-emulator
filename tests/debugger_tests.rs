//! Integration tests for debugger snapshots and trace formatting.
//!
//! Covers [`Cpu::snapshot`], [`format_cpu_snapshot`], and [`format_trace_entry`].

use chip8_emulator::cpu::Cpu;
use chip8_emulator::debugger::{format_cpu_snapshot, format_trace_entry, CpuSnapshot, TraceEntry};

#[test]
fn cpu_snapshot_captures_register_state() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0202;
    cpu.i = 0x0300;
    cpu.sp = 2;
    cpu.v[0] = 0x0A;
    cpu.v[0xF] = 0x01;
    cpu.stack[0] = 0x0202;
    cpu.delay_timer = 5;
    cpu.sound_timer = 7;

    let snap = cpu.snapshot();

    assert_eq!(snap.pc, 0x0202);
    assert_eq!(snap.i, 0x0300);
    assert_eq!(snap.sp, 2);
    assert_eq!(snap.v[0], 0x0A);
    assert_eq!(snap.v[0xF], 0x01);
    assert_eq!(snap.stack[0], 0x0202);
    assert_eq!(snap.delay_timer, 5);
    assert_eq!(snap.sound_timer, 7);

    // Whole V array must match the CPU's V array.
    assert_eq!(snap.v, cpu.v);
}

#[test]
fn format_cpu_snapshot_contains_key_registers() {
    let mut cpu = Cpu::new();
    cpu.pc = 0x0200;
    cpu.i = 0x0300;
    cpu.v[0] = 0x0A;
    cpu.v[0xF] = 0x01;
    cpu.stack[0] = 0x0202;

    let snap: CpuSnapshot = cpu.snapshot();
    let out = format_cpu_snapshot(&snap);

    assert!(out.contains("PC="), "missing PC=: {out}");
    assert!(out.contains("I="), "missing I=: {out}");
    assert!(out.contains("V0="), "missing V0=: {out}");
    assert!(out.contains("VF="), "missing VF=: {out}");
    assert!(out.contains("SP="), "missing SP=: {out}");
    assert!(out.contains("DT="), "missing DT=: {out}");
    assert!(out.contains("ST="), "missing ST=: {out}");
    assert!(out.contains("STACK[0]="), "missing STACK[0]=: {out}");
}

#[test]
fn format_trace_entry_contains_cycle_pc_and_opcode() {
    let entry = TraceEntry {
        cycle: 18,
        pc_before: 0x0200,
        opcode: 0x6A0F,
        pc_after: 0x0202,
        i_after: 0x0000,
        v_after: [0u8; 16],
    };
    let out = format_trace_entry(&entry);

    assert!(out.contains("#000018"), "missing cycle field: {out}");
    assert!(out.contains("PC=0x0200"), "missing pc_before: {out}");
    assert!(out.contains("OP=0x6A0F"), "missing opcode: {out}");
    assert!(out.contains("-> PC=0x0202"), "missing pc_after: {out}");
    assert!(out.contains("I=0x0000"), "missing i_after: {out}");
}
