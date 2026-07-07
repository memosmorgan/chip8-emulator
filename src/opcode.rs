//! Opcode decoding helpers for CHIP-8.
//!
//! CHIP-8 instructions are 16 bits wide and are typically described as a
//! sequence of four nibbles. [`decode`] extracts all the commonly used
//! sub-fields (individual nibbles, register indices, low byte, and 12-bit
//! address) from a raw 16-bit opcode so callers can pattern-match on them.

/// A CHIP-8 opcode split into its commonly used sub-fields.
///
/// All fields are derived from a single 16-bit `raw` value via bit operations.
/// No validation is performed; any 16-bit value can be decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedOpcode {
    /// The original 16-bit opcode, unchanged.
    pub raw: u16,
    /// First nibble (bits 15..12).
    pub n1: u8,
    /// Second nibble (bits 11..8).
    pub n2: u8,
    /// Third nibble (bits 7..4).
    pub n3: u8,
    /// Fourth nibble (bits 3..0).
    pub n4: u8,
    /// Second nibble as a register index (same value as `n2`).
    pub x: usize,
    /// Third nibble as a register index (same value as `n3`).
    pub y: usize,
    /// Fourth nibble (same value as `n4`).
    pub n: u8,
    /// Low byte (bits 7..0).
    pub nn: u8,
    /// Low 12 bits (bits 11..0), typically used as an address.
    pub nnn: u16,
}

/// Decode a raw 16-bit CHIP-8 opcode into its sub-fields.
///
/// Example for opcode `0x6A0F`:
/// - `raw`  = `0x6A0F`
/// - `n1`   = `0x6`, `n2` = `0xA`, `n3` = `0x0`, `n4` = `0xF`
/// - `x`    = `0xA`, `y` = `0x0`
/// - `n`    = `0xF`
/// - `nn`   = `0x0F`
/// - `nnn`  = `0xA0F`
pub fn decode(opcode: u16) -> DecodedOpcode {
    let n1 = ((opcode >> 12) & 0xF) as u8;
    let n2 = ((opcode >> 8) & 0xF) as u8;
    let n3 = ((opcode >> 4) & 0xF) as u8;
    let n4 = (opcode & 0xF) as u8;

    DecodedOpcode {
        raw: opcode,
        n1,
        n2,
        n3,
        n4,
        x: n2 as usize,
        y: n3 as usize,
        n: n4,
        nn: (opcode & 0xFF) as u8,
        nnn: opcode & 0x0FFF,
    }
}
