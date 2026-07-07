//! Integration tests for the headless CHIP-8 keypad input state (`src/input.rs`).

use chip8_emulator::input::{Input, KEY_COUNT};

#[test]
fn input_starts_with_no_keys_pressed() {
    let input = Input::new();
    for key in 0..KEY_COUNT as u8 {
        assert!(!input.is_pressed(key).unwrap());
    }
    assert_eq!(input.first_pressed_key(), None);
}

#[test]
fn input_press_and_release_key() {
    let mut input = Input::new();
    input.press_key(0x5).unwrap();
    assert!(input.is_pressed(0x5).unwrap());
    assert_eq!(input.first_pressed_key(), Some(0x5));
    input.release_key(0x5).unwrap();
    assert!(!input.is_pressed(0x5).unwrap());
    assert_eq!(input.first_pressed_key(), None);
}

#[test]
fn input_rejects_invalid_key() {
    let mut input = Input::new();
    // 0x10 is the first invalid key (valid range is 0x0..=0x0F).
    assert!(input.press_key(0x10).is_err());
    assert!(input.release_key(0x10).is_err());
    assert!(input.is_pressed(0x10).is_err());
    // A high invalid value must also error.
    assert!(input.press_key(0xFF).is_err());
    // No key should have been pressed by the failed calls.
    assert_eq!(input.first_pressed_key(), None);
}

#[test]
fn input_first_pressed_key_returns_lowest_pressed_key() {
    let mut input = Input::new();
    input.press_key(0x9).unwrap();
    input.press_key(0x3).unwrap();
    input.press_key(0xC).unwrap();
    // The lowest pressed index must be returned (deterministic).
    assert_eq!(input.first_pressed_key(), Some(0x3));
}

#[test]
fn input_clear_releases_all_keys() {
    let mut input = Input::new();
    input.press_key(0x1).unwrap();
    input.press_key(0x7).unwrap();
    input.press_key(0xF).unwrap();
    assert_eq!(input.first_pressed_key(), Some(0x1));
    input.clear();
    for key in 0..KEY_COUNT as u8 {
        assert!(!input.is_pressed(key).unwrap());
    }
    assert_eq!(input.first_pressed_key(), None);
}
