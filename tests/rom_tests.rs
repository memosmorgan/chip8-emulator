use chip8_emulator::cpu::PROGRAM_START;
use chip8_emulator::memory::Memory;
use chip8_emulator::rom::load_rom_to_memory;
use std::env;
use std::fs;
use std::path::PathBuf;

fn temp_path(name: &str) -> PathBuf {
    let mut p = env::temp_dir();
    p.push(name);
    p
}

#[test]
fn fake_rom_is_loaded_at_0x200() {
    let path = temp_path("chip8_fake_rom.ch8");
    let payload: Vec<u8> = (0..16).collect();
    fs::write(&path, &payload).unwrap();

    let mut mem = Memory::new();
    let n = load_rom_to_memory(&path, &mut mem).unwrap();
    assert_eq!(n, payload.len());

    for (i, byte) in payload.iter().enumerate() {
        let addr = PROGRAM_START as usize + i;
        assert_eq!(
            mem.read_byte(addr as u16).unwrap(),
            *byte,
            "mismatch at idx {}",
            i
        );
    }

    let _ = fs::remove_file(&path);
}

#[test]
fn rom_too_big_errors() {
    let path = temp_path("chip8_big_rom.ch8");
    // 4096 - 0x200 = 3584; 3585 bytes does not fit.
    let payload = vec![0xAAu8; 3585];
    fs::write(&path, &payload).unwrap();

    let mut mem = Memory::new();
    let result = load_rom_to_memory(&path, &mut mem);
    assert!(result.is_err(), "expected error for oversized ROM");

    let _ = fs::remove_file(&path);
}

#[test]
fn rom_loaded_byte_count_matches() {
    let path = temp_path("chip8_count_rom.ch8");
    let payload = vec![0x01u8; 100];
    fs::write(&path, &payload).unwrap();

    let mut mem = Memory::new();
    let n = load_rom_to_memory(&path, &mut mem).unwrap();
    assert_eq!(n, 100);

    let _ = fs::remove_file(&path);
}
