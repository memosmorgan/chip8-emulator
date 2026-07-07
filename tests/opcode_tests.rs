//! Integration tests for the opcode decoding helpers.

use chip8_emulator::opcode::{decode, DecodedOpcode};

#[test]
fn decode_6a0f_correctly() {
    let d: DecodedOpcode = decode(0x6A0F);
    assert_eq!(d.raw, 0x6A0F);
    assert_eq!(d.n1, 0x6);
    assert_eq!(d.n2, 0xA);
    assert_eq!(d.n3, 0x0);
    assert_eq!(d.n4, 0xF);
    assert_eq!(d.x, 0xA);
    assert_eq!(d.y, 0x0);
    assert_eq!(d.n, 0xF);
    assert_eq!(d.nn, 0x0F);
    assert_eq!(d.nnn, 0xA0F);
}

#[test]
fn decode_1abc_nnn_is_abc() {
    let d = decode(0x1ABC);
    assert_eq!(d.nnn, 0xABC);
    assert_eq!(d.n1, 0x1);
}

#[test]
fn decode_8124_xy_n() {
    let d = decode(0x8124);
    assert_eq!(d.x, 1);
    assert_eq!(d.y, 2);
    assert_eq!(d.n, 4);
    assert_eq!(d.n1, 0x8);
    assert_eq!(d.n2, 0x1);
    assert_eq!(d.n3, 0x2);
    assert_eq!(d.n4, 0x4);
    assert_eq!(d.nn, 0x24);
}

#[test]
fn decode_preserves_raw() {
    let d = decode(0x7F3B);
    assert_eq!(d.raw, 0x7F3B);
    assert_eq!(d.nn, 0x3B);
    assert_eq!(d.n1, 0x7);
    assert_eq!(d.n4, 0xB);
}
