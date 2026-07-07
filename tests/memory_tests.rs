use chip8_emulator::memory::{Memory, FONT_START, MEMORY_SIZE};

#[test]
fn memory_starts_zeroed() {
    let mem = Memory::new();
    let font_end = FONT_START + 80;
    for addr in 0..MEMORY_SIZE as u16 {
        if addr >= FONT_START && addr < font_end {
            continue;
        }
        assert_eq!(
            mem.read_byte(addr).unwrap(),
            0,
            "addr {:#06x} not zero",
            addr
        );
    }
}

#[test]
fn write_and_read_byte() {
    let mut mem = Memory::new();
    mem.write_byte(0x200, 0xAB).unwrap();
    mem.write_byte(0x201, 0xCD).unwrap();
    assert_eq!(mem.read_byte(0x200).unwrap(), 0xAB);
    assert_eq!(mem.read_byte(0x201).unwrap(), 0xCD);
}

#[test]
fn read_opcode_combines_two_bytes_big_endian() {
    let mut mem = Memory::new();
    mem.write_byte(0x200, 0xAB).unwrap();
    mem.write_byte(0x201, 0xCD).unwrap();
    assert_eq!(mem.read_opcode(0x200).unwrap(), 0xABCD);
}

#[test]
fn read_byte_out_of_bounds_errors() {
    let mem = Memory::new();
    assert!(mem.read_byte(0x1000).is_err());
    // 0x0FFF is the last valid byte; 0x0FFF + 1 is out of bounds.
    assert!(mem.read_byte(0x1000).is_err());
}

#[test]
fn write_byte_out_of_bounds_errors() {
    let mut mem = Memory::new();
    assert!(mem.write_byte(0x1000, 0x01).is_err());
    assert!(mem.write_byte(0x0FFF, 0x01).is_ok()); // last valid byte
}

#[test]
fn read_opcode_out_of_bounds_errors() {
    let mem = Memory::new();
    // 0x0FFF + 1 = 0x1000 out of bounds: cannot read 2 bytes.
    assert!(mem.read_opcode(0x0FFF).is_err());
    // 0x1000 fully out of bounds.
    assert!(mem.read_opcode(0x1000).is_err());
}

#[test]
fn memory_new_loads_fontset() {
    let mem = Memory::new();
    // Digit "0" sprite (5 bytes) at FONT_START.
    let expected_zero = [0xF0, 0x90, 0x90, 0x90, 0xF0];
    for (i, b) in expected_zero.iter().enumerate() {
        assert_eq!(
            mem.read_byte(FONT_START + i as u16).unwrap(),
            *b,
            "font byte {} for digit 0 mismatch",
            i
        );
    }
    // Digit "F" sprite (5 bytes) at FONT_START + 15*5.
    let expected_f = [0xF0, 0x80, 0xF0, 0x80, 0x80];
    let f_start = FONT_START + 15 * 5;
    for (i, b) in expected_f.iter().enumerate() {
        assert_eq!(
            mem.read_byte(f_start + i as u16).unwrap(),
            *b,
            "font byte {} for digit F mismatch",
            i
        );
    }
    // Bytes immediately before FONT_START should still be zero.
    assert_eq!(mem.read_byte(FONT_START - 1).unwrap(), 0);
}
