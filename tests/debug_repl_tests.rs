use chip8_emulator::debug_repl::{
    format_memory_dump, parse_debug_command, parse_reg_index, parse_u16_value, Breakpoint,
    BreakpointManager, DebugCommand, WatchManager, WatchTarget, Watchpoint, DEFAULT_DISASM_COUNT,
    MAX_MEM_DUMP_LEN,
};
use chip8_emulator::memory::Memory;

#[test]
fn parse_debug_command_help() {
    assert_eq!(parse_debug_command("help"), Ok(DebugCommand::Help));
}

#[test]
fn parse_debug_command_step_alias() {
    assert_eq!(parse_debug_command("step"), Ok(DebugCommand::Step));
    assert_eq!(parse_debug_command("s"), Ok(DebugCommand::Step));
}

#[test]
fn parse_debug_command_continue() {
    assert_eq!(
        parse_debug_command("continue 100"),
        Ok(DebugCommand::Continue(100))
    );
    assert_eq!(parse_debug_command("c 5"), Ok(DebugCommand::Continue(5)));
}

#[test]
fn parse_debug_command_rejects_continue_zero() {
    assert!(parse_debug_command("continue 0").is_err());
}

#[test]
fn parse_debug_command_disasm_default() {
    assert_eq!(
        parse_debug_command("disasm"),
        Ok(DebugCommand::Disasm {
            address: None,
            count: DEFAULT_DISASM_COUNT,
        })
    );
}

#[test]
fn parse_debug_command_disasm_with_hex_address_and_count() {
    assert_eq!(
        parse_debug_command("disasm 0x200 16"),
        Ok(DebugCommand::Disasm {
            address: Some(0x200),
            count: 16,
        })
    );
}

#[test]
fn parse_debug_command_mem_with_hex_address() {
    assert_eq!(
        parse_debug_command("mem 0x300 32"),
        Ok(DebugCommand::Mem {
            address: 0x300,
            len: 32,
        })
    );
}

#[test]
fn parse_debug_command_rejects_large_mem_len() {
    let result = parse_debug_command("mem 0x300 999999");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains(&MAX_MEM_DUMP_LEN.to_string()),
        "error should mention max len: {err}"
    );
}

#[test]
fn parse_debug_command_stack() {
    assert_eq!(parse_debug_command("stack"), Ok(DebugCommand::Stack));
}

#[test]
fn parse_debug_command_display() {
    assert_eq!(parse_debug_command("display"), Ok(DebugCommand::Display));
}

#[test]
fn parse_debug_command_timers() {
    assert_eq!(parse_debug_command("timers"), Ok(DebugCommand::Timers));
}

#[test]
fn parse_debug_command_tick() {
    assert_eq!(parse_debug_command("tick"), Ok(DebugCommand::Tick));
}

#[test]
fn parse_debug_command_quit_alias() {
    assert_eq!(parse_debug_command("quit"), Ok(DebugCommand::Quit));
    assert_eq!(parse_debug_command("q"), Ok(DebugCommand::Quit));
}

#[test]
fn parse_u16_value_accepts_hex() {
    assert_eq!(parse_u16_value("0x200"), Ok(0x200));
    assert_eq!(parse_u16_value("0xABCD"), Ok(0xABCD));
    assert_eq!(parse_u16_value("0xab"), Ok(0xAB));
}

#[test]
fn parse_u16_value_accepts_decimal() {
    assert_eq!(parse_u16_value("512"), Ok(512));
    assert_eq!(parse_u16_value("0"), Ok(0));
}

#[test]
fn parse_u16_value_rejects_invalid() {
    assert!(parse_u16_value("0xGG").is_err());
    assert!(parse_u16_value("foo").is_err());
    assert!(parse_u16_value("").is_err());
}

#[test]
fn parse_debug_command_case_insensitive() {
    assert_eq!(parse_debug_command("HELP"), Ok(DebugCommand::Help));
    assert_eq!(parse_debug_command("Regs"), Ok(DebugCommand::Regs));
}

#[test]
fn parse_debug_command_rejects_mem_zero_len() {
    assert!(parse_debug_command("mem 0x200 0").is_err());
}

#[test]
fn parse_debug_command_rejects_continue_missing_arg() {
    assert!(parse_debug_command("continue").is_err());
}

#[test]
fn parse_debug_command_empty_is_error() {
    assert!(parse_debug_command("").is_err());
    assert!(parse_debug_command("   ").is_err());
}

#[test]
fn parse_debug_command_unknown() {
    let result = parse_debug_command("xyz");
    assert!(result.is_err());
    assert!(
        result.unwrap_err().contains("Unknown command"),
        "should mention Unknown command"
    );
}

#[test]
fn format_memory_dump_basic() {
    let mut mem = Memory::new();
    mem.write_byte(0x300, 0xF0).unwrap();
    mem.write_byte(0x301, 0x90).unwrap();
    mem.write_byte(0x302, 0x90).unwrap();
    mem.write_byte(0x303, 0x90).unwrap();

    let dump = format_memory_dump(&mem, 0x300, 4).expect("dump ok");
    assert!(
        dump.starts_with("0300: F0"),
        "dump should start with 0300: F0, got: {dump}"
    );
    assert!(
        dump.contains("F0 90 90 90"),
        "dump should contain the bytes: {dump}"
    );
}

#[test]
fn format_memory_dump_out_of_bounds() {
    let result = format_memory_dump(&Memory::new(), 0x0FF0, 32);
    assert!(result.is_err(), "out-of-bounds dump should error");
}

#[test]
fn format_memory_dump_max_len_ok() {
    let result = format_memory_dump(&Memory::new(), 0, MAX_MEM_DUMP_LEN);
    assert!(
        result.is_ok(),
        "dump of exactly MAX_MEM_DUMP_LEN at 0 should be ok"
    );
}

// Breakpoint / watchpoint parser tests.

#[test]
fn parse_debug_command_break_hex_and_decimal() {
    assert_eq!(
        parse_debug_command("break 0x200"),
        Ok(DebugCommand::Break(0x200))
    );
    assert_eq!(
        parse_debug_command("break 512"),
        Ok(DebugCommand::Break(512))
    );
}

#[test]
fn parse_debug_command_break_errors() {
    assert!(parse_debug_command("break").is_err());
    assert!(parse_debug_command("break 0x200 extra").is_err());
    assert!(parse_debug_command("break foo").is_err());
}

#[test]
fn parse_debug_command_breaks() {
    assert_eq!(parse_debug_command("breaks"), Ok(DebugCommand::Breaks));
    assert!(parse_debug_command("breaks now").is_err());
}

#[test]
fn parse_debug_command_delete() {
    assert_eq!(
        parse_debug_command("delete 3"),
        Ok(DebugCommand::DeleteBreak(3))
    );
    assert!(parse_debug_command("delete abc").is_err());
    assert!(parse_debug_command("delete").is_err());
}

#[test]
fn parse_debug_command_clear_breaks() {
    assert_eq!(
        parse_debug_command("clear-breaks"),
        Ok(DebugCommand::ClearBreaks)
    );
    assert!(parse_debug_command("clear-breaks now").is_err());
}

#[test]
fn parse_debug_command_watch_reg() {
    assert_eq!(
        parse_debug_command("watch reg V0"),
        Ok(DebugCommand::WatchReg(0))
    );
    assert_eq!(
        parse_debug_command("watch reg VA"),
        Ok(DebugCommand::WatchReg(10))
    );
    assert_eq!(
        parse_debug_command("watch reg vf"),
        Ok(DebugCommand::WatchReg(15))
    );
    assert_eq!(
        parse_debug_command("watch reg 5"),
        Ok(DebugCommand::WatchReg(5))
    );
    assert_eq!(
        parse_debug_command("watch reg 0xA"),
        Ok(DebugCommand::WatchReg(10))
    );
}

#[test]
fn parse_debug_command_watch_reg_errors() {
    assert!(parse_debug_command("watch reg V10").is_err());
    assert!(parse_debug_command("watch reg 16").is_err());
    assert!(parse_debug_command("watch reg").is_err());
}

#[test]
fn parse_debug_command_watch_mem() {
    assert_eq!(
        parse_debug_command("watch mem 0x300"),
        Ok(DebugCommand::WatchMem(0x300))
    );
    assert_eq!(
        parse_debug_command("watch mem 768"),
        Ok(DebugCommand::WatchMem(768))
    );
}

#[test]
fn parse_debug_command_watch_errors() {
    assert!(parse_debug_command("watch foo 0x300").is_err());
    assert!(parse_debug_command("watch").is_err());
    assert!(parse_debug_command("watch reg V0 extra").is_err());
    assert!(parse_debug_command("watch mem 0x300 extra").is_err());
}

#[test]
fn parse_debug_command_watches() {
    assert_eq!(parse_debug_command("watches"), Ok(DebugCommand::Watches));
    assert!(parse_debug_command("watches now").is_err());
}

#[test]
fn parse_debug_command_delete_watch() {
    assert_eq!(
        parse_debug_command("delete-watch 2"),
        Ok(DebugCommand::DeleteWatch(2))
    );
    assert!(parse_debug_command("delete-watch x").is_err());
    assert!(parse_debug_command("delete-watch").is_err());
}

#[test]
fn parse_debug_command_clear_watches() {
    assert_eq!(
        parse_debug_command("clear-watches"),
        Ok(DebugCommand::ClearWatches)
    );
    assert!(parse_debug_command("clear-watches now").is_err());
}

// parse_reg_index tests.

#[test]
fn parse_reg_index_valid() {
    assert_eq!(parse_reg_index("V0"), Ok(0));
    assert_eq!(parse_reg_index("va"), Ok(10));
    assert_eq!(parse_reg_index("0xA"), Ok(10));
    assert_eq!(parse_reg_index("5"), Ok(5));
}

#[test]
fn parse_reg_index_errors() {
    assert!(parse_reg_index("0x10").is_err());
    assert!(parse_reg_index("V10").is_err());
    assert!(parse_reg_index("16").is_err());
    assert!(parse_reg_index("G").is_err());
    assert!(parse_reg_index("").is_err());
}

// BreakpointManager tests.

#[test]
fn breakpoint_manager_add_list_contains() {
    let mut mgr = BreakpointManager::new();
    let bp1 = mgr.add(0x200);
    let bp2 = mgr.add(0x300);
    assert_eq!(bp1.id, 1);
    assert_eq!(bp1.address, 0x200);
    assert_eq!(bp2.id, 2);
    assert_eq!(bp2.address, 0x300);
    assert_eq!(mgr.list().len(), 2);
    assert!(mgr.contains_address(0x200));
    assert!(!mgr.contains_address(0x204));
}

#[test]
fn breakpoint_manager_remove() {
    let mut mgr = BreakpointManager::new();
    mgr.add(0x200);
    mgr.add(0x300);
    let removed = mgr.remove(1).expect("id 1 exists");
    assert_eq!(
        removed,
        Breakpoint {
            id: 1,
            address: 0x200
        }
    );
    assert_eq!(mgr.list().len(), 1);
    assert!(mgr.remove(999).is_err());
}

#[test]
fn breakpoint_manager_clear_is_empty() {
    let mut mgr = BreakpointManager::new();
    mgr.add(0x200);
    mgr.add(0x300);
    mgr.clear();
    assert!(mgr.is_empty());
    assert_eq!(mgr.list().len(), 0);
}

// WatchManager tests.

#[test]
fn watch_manager_add_list_remove() {
    let mut mgr = WatchManager::new();
    let wp1 = mgr.add(WatchTarget::Reg { index: 0 });
    let wp2 = mgr.add(WatchTarget::Mem { address: 0x300 });
    assert_eq!(wp1.id, 1);
    assert_eq!(wp2.id, 2);
    assert_eq!(mgr.list().len(), 2);
    let removed = mgr.remove(2).expect("id 2 exists");
    assert_eq!(
        removed,
        Watchpoint {
            id: 2,
            target: WatchTarget::Mem { address: 0x300 }
        }
    );
}

#[test]
fn watch_manager_clear_is_empty() {
    let mut mgr = WatchManager::new();
    mgr.add(WatchTarget::Reg { index: 0 });
    mgr.add(WatchTarget::Mem { address: 0x300 });
    mgr.clear();
    assert!(mgr.is_empty());
}
