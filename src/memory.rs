//! CHIP-8 memory.
//!
//! CHIP-8 has a flat 4096-byte address space. Programs are normally loaded
//! starting at `0x200`. This module provides safe, bounds-checked access to
//! that memory and a helper to read a 2-byte opcode (big-endian).

/// Total size of the CHIP-8 address space in bytes.
pub const MEMORY_SIZE: usize = 4096;

/// Address where the hex fontset is loaded in memory.
pub const FONT_START: u16 = 0x050;

/// Number of bytes per font sprite (4x5 glyphs, one byte per row).
pub const FONT_BYTES_PER_CHAR: usize = 5;

/// Standard CHIP-8 hex fontset: 16 sprites (0-F), 5 bytes each.
const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// CHIP-8 memory: flat 4096-byte array.
#[derive(Debug, Clone)]
pub struct Memory {
    bytes: [u8; MEMORY_SIZE],
}

impl Memory {
    /// Create a new memory filled with zeros, with the hex fontset loaded at
    /// [`FONT_START`].
    pub fn new() -> Self {
        let mut bytes = [0u8; MEMORY_SIZE];
        let start = FONT_START as usize;
        bytes[start..start + FONTSET.len()].copy_from_slice(&FONTSET);
        Memory { bytes }
    }

    /// Read a single byte at `address`.
    ///
    /// Returns an error string if the address is out of bounds.
    pub fn read_byte(&self, address: u16) -> Result<u8, String> {
        let addr = address as usize;
        if addr >= MEMORY_SIZE {
            return Err(format!("read_byte: address {address:#06x} out of bounds"));
        }
        Ok(self.bytes[addr])
    }

    /// Write a single byte `value` at `address`.
    ///
    /// Returns an error string if the address is out of bounds.
    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<(), String> {
        let addr = address as usize;
        if addr >= MEMORY_SIZE {
            return Err(format!("write_byte: address {address:#06x} out of bounds"));
        }
        self.bytes[addr] = value;
        Ok(())
    }

    /// Read a 2-byte opcode at `address` (big-endian: high byte first).
    ///
    /// Both `address` and `address + 1` must be within bounds.
    /// Example: mem[0x200]=0xAB, mem[0x201]=0xCD -> read_opcode(0x200) = 0xABCD.
    pub fn read_opcode(&self, address: u16) -> Result<u16, String> {
        let lo_addr = address as usize;
        let hi_addr = lo_addr + 1;
        if lo_addr >= MEMORY_SIZE {
            return Err(format!("read_opcode: address {address:#06x} out of bounds"));
        }
        if hi_addr >= MEMORY_SIZE {
            return Err(format!(
                "read_opcode: address+1 {:#06x} out of bounds",
                address + 1
            ));
        }
        let hi = self.bytes[lo_addr] as u16;
        let lo = self.bytes[hi_addr] as u16;
        Ok((hi << 8) | lo)
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
