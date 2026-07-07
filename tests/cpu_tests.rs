use chip8_emulator::cpu::{Cpu, PROGRAM_START};
use chip8_emulator::display::Display;
use chip8_emulator::input::Input;
use chip8_emulator::memory::Memory;

#[test]
fn cpu_initial_state() {
    let cpu = Cpu::new();
    assert_eq!(cpu.pc, 0x200);
    assert_eq!(cpu.pc, PROGRAM_START);
    assert_eq!(cpu.i, 0);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.delay_timer, 0);
    assert_eq!(cpu.sound_timer, 0);
    for v in cpu.v.iter() {
        assert_eq!(*v, 0);
    }
    for s in cpu.stack.iter() {
        assert_eq!(*s, 0);
    }
}

#[test]
fn fetch_opcode_returns_correct_value_and_advances_pc() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    mem.write_byte(0x200, 0xAB).unwrap();
    mem.write_byte(0x201, 0xCD).unwrap();
    let opcode = cpu.fetch_opcode(&mem).unwrap();
    assert_eq!(opcode, 0xABCD);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn fetch_opcode_does_not_change_pc_on_error() {
    let mut cpu = Cpu::new();
    let mem = Memory::new();
    // Put pc at the very last byte so read_opcode needs 0x0FFF and 0x1000
    // (the latter is out of bounds).
    cpu.pc = 0x0FFF;
    let result = cpu.fetch_opcode(&mem);
    assert!(result.is_err());
    assert_eq!(cpu.pc, 0x0FFF, "pc must not change on fetch error");
}

#[test]
fn step_executes_jump_1nnn() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    // Write opcode 0x1208 at 0x200 (big-endian: 0x12 then 0x08)
    mem.write_byte(0x200, 0x12).unwrap();
    mem.write_byte(0x201, 0x08).unwrap();
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x1208);
    assert_eq!(cpu.pc, 0x208);
}

#[test]
fn step_executes_set_register_6xnn() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    // Write opcode 0x6A0F at 0x200
    mem.write_byte(0x200, 0x6A).unwrap();
    mem.write_byte(0x201, 0x0F).unwrap();
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x6A0F);
    assert_eq!(cpu.v[0xA], 0x0F);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_returns_error_for_unsupported_opcode() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    // Write an unsupported opcode 0x0123 at 0x200 (0NNN SYS addr is ignored/unsupported here).
    mem.write_byte(0x200, 0x01).unwrap();
    mem.write_byte(0x201, 0x23).unwrap();
    let result = cpu.step(&mut mem, &mut display, &input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err, "Unsupported opcode: 0x0123");
}

fn write_opcode(mem: &mut Memory, addr: u16, opcode: u16) {
    mem.write_byte(addr, ((opcode >> 8) & 0xFF) as u8).unwrap();
    mem.write_byte(addr + 1, (opcode & 0xFF) as u8).unwrap();
}

#[test]
fn step_executes_call_2nnn() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    write_opcode(&mut mem, 0x200, 0x2300);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x2300);
    assert_eq!(cpu.pc, 0x300);
    assert_eq!(cpu.stack[0], 0x202);
    assert_eq!(cpu.sp, 1);
}

#[test]
fn step_executes_return_00ee() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.stack[0] = 0x250;
    cpu.sp = 1;
    cpu.pc = 0x200;
    write_opcode(&mut mem, 0x200, 0x00EE);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x00EE);
    assert_eq!(cpu.pc, 0x250);
    assert_eq!(cpu.sp, 0);
}

#[test]
fn call_errors_on_stack_overflow() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.sp = 16;
    write_opcode(&mut mem, 0x200, 0x2ABC);
    let result = cpu.step(&mut mem, &mut display, &input);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Stack overflow"));
    assert_eq!(cpu.sp, 16);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn return_errors_on_stack_underflow() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    write_opcode(&mut mem, 0x200, 0x00EE);
    let result = cpu.step(&mut mem, &mut display, &input);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Stack underflow"));
    assert_eq!(cpu.sp, 0);
}

#[test]
fn step_executes_skip_equal_3xnn_when_equal() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x42;
    write_opcode(&mut mem, 0x200, 0x3142);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x3142);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn step_does_not_skip_3xnn_when_not_equal() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x00;
    write_opcode(&mut mem, 0x200, 0x3142);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x3142);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_skip_not_equal_4xnn_when_not_equal() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x00;
    write_opcode(&mut mem, 0x200, 0x4142);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x4142);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn step_executes_skip_equal_register_5xy0() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x77;
    cpu.v[2] = 0x77;
    write_opcode(&mut mem, 0x200, 0x5120);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x5120);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn step_executes_skip_not_equal_register_9xy0() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x10;
    cpu.v[2] = 0x20;
    write_opcode(&mut mem, 0x200, 0x9120);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x9120);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn step_rejects_invalid_5xy_nonzero_last_nibble() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    write_opcode(&mut mem, 0x200, 0x5121);
    let result = cpu.step(&mut mem, &mut display, &input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unsupported opcode: 0x5121");
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_rejects_invalid_9xy_nonzero_last_nibble() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    write_opcode(&mut mem, 0x200, 0x9121);
    let result = cpu.step(&mut mem, &mut display, &input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unsupported opcode: 0x9121");
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_add_byte_7xnn_wrapping() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0xFF;
    write_opcode(&mut mem, 0x200, 0x7102);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x7102);
    assert_eq!(cpu.v[1], 0x01);
    assert_eq!(cpu.v[0xF], 0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_ld_vx_vy_8xy0() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[2] = 0xAB;
    write_opcode(&mut mem, 0x200, 0x8120);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8120);
    assert_eq!(cpu.v[1], 0xAB);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_or_8xy1() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0xF0;
    cpu.v[2] = 0x0F;
    write_opcode(&mut mem, 0x200, 0x8121);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8121);
    assert_eq!(cpu.v[1], 0xFF);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_and_8xy2() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0xF0;
    cpu.v[2] = 0x3C;
    write_opcode(&mut mem, 0x200, 0x8122);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8122);
    assert_eq!(cpu.v[1], 0x30);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_xor_8xy3() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0xFF;
    cpu.v[2] = 0x0F;
    write_opcode(&mut mem, 0x200, 0x8123);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8123);
    assert_eq!(cpu.v[1], 0xF0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_add_register_8xy4_without_carry() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x10;
    cpu.v[2] = 0x20;
    write_opcode(&mut mem, 0x200, 0x8124);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8124);
    assert_eq!(cpu.v[1], 0x30);
    assert_eq!(cpu.v[0xF], 0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_add_register_8xy4_with_carry() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0xFF;
    cpu.v[2] = 0x01;
    write_opcode(&mut mem, 0x200, 0x8124);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8124);
    assert_eq!(cpu.v[1], 0x00);
    assert_eq!(cpu.v[0xF], 1);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_sub_8xy5_without_borrow() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x50;
    cpu.v[2] = 0x20;
    write_opcode(&mut mem, 0x200, 0x8125);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8125);
    assert_eq!(cpu.v[1], 0x30);
    assert_eq!(cpu.v[0xF], 1);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_sub_8xy5_with_borrow() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x10;
    cpu.v[2] = 0x20;
    write_opcode(&mut mem, 0x200, 0x8125);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8125);
    assert_eq!(cpu.v[1], 0xF0);
    assert_eq!(cpu.v[0xF], 0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_subn_8xy7_without_borrow() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x20;
    cpu.v[2] = 0x50;
    write_opcode(&mut mem, 0x200, 0x8127);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8127);
    assert_eq!(cpu.v[1], 0x30);
    assert_eq!(cpu.v[0xF], 1);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_subn_8xy7_with_borrow() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x50;
    cpu.v[2] = 0x20;
    write_opcode(&mut mem, 0x200, 0x8127);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8127);
    assert_eq!(cpu.v[1], 0xD0);
    assert_eq!(cpu.v[0xF], 0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_shr_8xy6_sets_vf_to_old_lsb() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x03;
    write_opcode(&mut mem, 0x200, 0x8106);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x8106);
    assert_eq!(cpu.v[1], 0x01);
    assert_eq!(cpu.v[0xF], 1);
    assert_eq!(cpu.pc, 0x202);

    // Additional case: LSB == 0
    let mut cpu2 = Cpu::new();
    let mut mem2 = Memory::new();
    let mut display2 = Display::new();
    let input2 = Input::new();
    cpu2.v[1] = 0x04;
    write_opcode(&mut mem2, 0x200, 0x8106);
    let executed2 = cpu2.step(&mut mem2, &mut display2, &input2).unwrap();
    assert_eq!(executed2, 0x8106);
    assert_eq!(cpu2.v[1], 0x02);
    assert_eq!(cpu2.v[0xF], 0);
    assert_eq!(cpu2.pc, 0x202);
}

#[test]
fn step_executes_shl_8xye_sets_vf_to_old_msb() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0x80;
    write_opcode(&mut mem, 0x200, 0x810E);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x810E);
    assert_eq!(cpu.v[1], 0x00);
    assert_eq!(cpu.v[0xF], 1);
    assert_eq!(cpu.pc, 0x202);
}
#[test]
fn step_executes_ld_i_annn() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    write_opcode(&mut mem, 0x200, 0xA300);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0xA300);
    assert_eq!(cpu.i, 0x300);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_add_i_fx1e() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.i = 0x300;
    cpu.v[1] = 0x10;
    write_opcode(&mut mem, 0x200, 0xF11E);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0xF11E);
    assert_eq!(cpu.i, 0x310);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_fx55_store_registers() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.i = 0x300;
    cpu.v[0] = 1;
    cpu.v[1] = 2;
    cpu.v[2] = 3;
    write_opcode(&mut mem, 0x200, 0xF255);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0xF255);
    assert_eq!(mem.read_byte(0x300).unwrap(), 1);
    assert_eq!(mem.read_byte(0x301).unwrap(), 2);
    assert_eq!(mem.read_byte(0x302).unwrap(), 3);
    // I must be unchanged.
    assert_eq!(cpu.i, 0x300);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_fx65_load_registers() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.i = 0x300;
    mem.write_byte(0x300, 1).unwrap();
    mem.write_byte(0x301, 2).unwrap();
    mem.write_byte(0x302, 3).unwrap();
    write_opcode(&mut mem, 0x200, 0xF265);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0xF265);
    assert_eq!(cpu.v[0], 1);
    assert_eq!(cpu.v[1], 2);
    assert_eq!(cpu.v[2], 3);
    // I must be unchanged.
    assert_eq!(cpu.i, 0x300);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_fx29_sets_i_to_font_address() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0xA;
    write_opcode(&mut mem, 0x200, 0xF129);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0xF129);
    let expected = chip8_emulator::memory::FONT_START + 0xA * 5;
    assert_eq!(cpu.i, expected);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_cls_00e0() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    // Turn on a pixel first.
    display.set_pixel(5, 5, true).unwrap();
    assert!(display.get_pixel(5, 5).unwrap());
    write_opcode(&mut mem, 0x200, 0x00E0);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0x00E0);
    assert!(!display.get_pixel(5, 5).unwrap());
    // VF must not change.
    assert_eq!(cpu.v[0xF], 0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_draw_dxyn() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.i = 0x300;
    mem.write_byte(0x300, 0b1111_0000).unwrap();
    cpu.v[1] = 0;
    cpu.v[2] = 0;
    write_opcode(&mut mem, 0x200, 0xD121);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0xD121);
    assert!(display.get_pixel(0, 0).unwrap());
    assert!(display.get_pixel(1, 0).unwrap());
    assert!(display.get_pixel(2, 0).unwrap());
    assert!(display.get_pixel(3, 0).unwrap());
    assert!(!display.get_pixel(4, 0).unwrap());
    assert_eq!(cpu.v[0xF], 0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_draw_sets_vf_on_collision() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.i = 0x300;
    mem.write_byte(0x300, 0b1111_0000).unwrap();
    cpu.v[1] = 0;
    cpu.v[2] = 0;
    write_opcode(&mut mem, 0x200, 0xD121);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(cpu.v[0xF], 0);
    // Draw same sprite again at same location -> XOR turns pixels off -> collision.
    cpu.pc = 0x200;
    let collided = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(collided, 0xD121);
    assert_eq!(cpu.v[0xF], 1);
}

#[test]
fn step_draw_reads_multiple_sprite_rows() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.i = 0x300;
    mem.write_byte(0x300, 0b1111_0000).unwrap();
    mem.write_byte(0x301, 0b0000_1111).unwrap();
    cpu.v[1] = 0;
    cpu.v[2] = 0;
    // D122: draw sprite of height 2 at (0,0)
    write_opcode(&mut mem, 0x200, 0xD122);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0xD122);
    // Row 0: first 4 pixels on
    assert!(display.get_pixel(0, 0).unwrap());
    assert!(display.get_pixel(3, 0).unwrap());
    assert!(!display.get_pixel(4, 0).unwrap());
    // Row 1: last 4 pixels on
    assert!(display.get_pixel(4, 1).unwrap());
    assert!(display.get_pixel(7, 1).unwrap());
    assert!(!display.get_pixel(0, 1).unwrap());
    assert_eq!(cpu.v[0xF], 0);
}

#[test]
fn step_rejects_dxyn_with_zero_height() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    // D120: height 0 (DXY0 Super-CHIP) unsupported in base CHIP-8.
    write_opcode(&mut mem, 0x200, 0xD120);
    let result = cpu.step(&mut mem, &mut display, &input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unsupported opcode: 0xD120");
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_draw_errors_when_sprite_memory_out_of_bounds() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    // Set I near the end of memory so a tall sprite overflows.
    cpu.i = 0x0FFA;
    cpu.v[1] = 0;
    cpu.v[2] = 0;
    // D12F: height 15 -> I + 15 = 0x1009 > 0x0FFF (MEMORY_SIZE)
    write_opcode(&mut mem, 0x200, 0xD12F);
    let result = cpu.step(&mut mem, &mut display, &input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("out of bounds"), "unexpected error: {}", err);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_skip_if_key_pressed_ex9e() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let mut input = Input::new();
    cpu.v[1] = 0xA;
    input.press_key(0xA).unwrap();
    write_opcode(&mut mem, 0x200, 0xE19E);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0xE19E);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn step_does_not_skip_ex9e_when_key_not_pressed() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0xA;
    write_opcode(&mut mem, 0x200, 0xE19E);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_executes_skip_if_key_not_pressed_exa1() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0xA;
    write_opcode(&mut mem, 0x200, 0xE1A1);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn step_does_not_skip_exa1_when_key_pressed() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let mut input = Input::new();
    cpu.v[1] = 0xA;
    input.press_key(0xA).unwrap();
    write_opcode(&mut mem, 0x200, 0xE1A1);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_errors_when_key_register_contains_invalid_key() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 0xFF;
    write_opcode(&mut mem, 0x200, 0xE19E);
    let result = cpu.step(&mut mem, &mut display, &input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("invalid key"), "unexpected error: {}", err);
}

#[test]
fn step_fx0a_stores_pressed_key() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let mut input = Input::new();
    input.press_key(0xB).unwrap();
    write_opcode(&mut mem, 0x200, 0xF10A);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(cpu.v[1], 0xB);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_fx0a_waits_when_no_key_pressed() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    write_opcode(&mut mem, 0x200, 0xF10A);
    let executed = cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(executed, 0xF10A);
    // pc must be rewound back to 0x200 so the same instruction re-runs.
    assert_eq!(cpu.pc, 0x200);
}

#[test]
fn step_fx07_loads_delay_timer() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.delay_timer = 42;
    write_opcode(&mut mem, 0x200, 0xF107);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(cpu.v[1], 42);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_fx15_sets_delay_timer() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 30;
    write_opcode(&mut mem, 0x200, 0xF115);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(cpu.delay_timer, 30);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn step_fx18_sets_sound_timer() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[1] = 20;
    write_opcode(&mut mem, 0x200, 0xF118);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(cpu.sound_timer, 20);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn tick_timers_decrements_nonzero_timers() {
    let mut cpu = Cpu::new();
    cpu.delay_timer = 5;
    cpu.sound_timer = 3;
    cpu.tick_timers();
    assert_eq!(cpu.delay_timer, 4);
    assert_eq!(cpu.sound_timer, 2);
    // A second tick should decrement again.
    cpu.tick_timers();
    assert_eq!(cpu.delay_timer, 3);
    assert_eq!(cpu.sound_timer, 1);
}

#[test]
fn tick_timers_does_not_underflow() {
    let mut cpu = Cpu::new();
    cpu.delay_timer = 0;
    cpu.sound_timer = 0;
    cpu.tick_timers();
    assert_eq!(cpu.delay_timer, 0);
    assert_eq!(cpu.sound_timer, 0);
    // Also verify that a timer at 1 ticks down to 0 and stays there.
    cpu.delay_timer = 1;
    cpu.sound_timer = 1;
    cpu.tick_timers();
    assert_eq!(cpu.delay_timer, 0);
    assert_eq!(cpu.sound_timer, 0);
    cpu.tick_timers();
    assert_eq!(cpu.delay_timer, 0);
    assert_eq!(cpu.sound_timer, 0);
}

#[test]
fn step_executes_bnnn_jump_with_v0_offset() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.v[0] = 0x10;
    write_opcode(&mut mem, 0x200, 0xB300);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(cpu.pc, 0x310);
}

#[test]
fn step_executes_cxnn_random_and_mask() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.set_rng_seed(0x1234_5678);
    write_opcode(&mut mem, 0x200, 0xC10F);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    // V1 must have no bits outside the mask 0x0F.
    assert_eq!(cpu.v[1] & !0x0F, 0, "V1 & !0x0F must be zero");
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn cxnn_is_deterministic_with_same_seed() {
    let mut cpu_a = Cpu::new();
    let mut cpu_b = Cpu::new();
    cpu_a.set_rng_seed(0xDEAD_BEEF);
    cpu_b.set_rng_seed(0xDEAD_BEEF);
    let mut mem_a = Memory::new();
    let mut mem_b = Memory::new();
    let mut display_a = Display::new();
    let mut display_b = Display::new();
    let input = Input::new();
    write_opcode(&mut mem_a, 0x200, 0xC2FF);
    write_opcode(&mut mem_b, 0x200, 0xC2FF);
    cpu_a.step(&mut mem_a, &mut display_a, &input).unwrap();
    cpu_b.step(&mut mem_b, &mut display_b, &input).unwrap();
    assert_eq!(cpu_a.v[2], cpu_b.v[2], "same seed must yield same Vx");
}

#[test]
fn step_executes_fx33_bcd_store() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    cpu.i = 0x300;
    cpu.v[1] = 123;
    write_opcode(&mut mem, 0x200, 0xF133);
    cpu.step(&mut mem, &mut display, &input).unwrap();
    assert_eq!(mem.read_byte(0x300).unwrap(), 1);
    assert_eq!(mem.read_byte(0x301).unwrap(), 2);
    assert_eq!(mem.read_byte(0x302).unwrap(), 3);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn fx33_errors_when_memory_out_of_bounds() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();
    let mut display = Display::new();
    let input = Input::new();
    // 0x0FFF is last byte; I+1 and I+2 are out of bounds.
    cpu.i = 0x0FFF;
    cpu.v[1] = 99;
    write_opcode(&mut mem, 0x200, 0xF133);
    let result = cpu.step(&mut mem, &mut display, &input);
    assert!(result.is_err(), "FX33 with I near end of memory must error");
}
