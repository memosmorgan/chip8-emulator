//! ROM loading.
//!
//! Reads a ROM file from disk and copies its bytes into CHIP-8 memory
//! starting at `0x200`. No ROM execution happens here.

use crate::cpu::PROGRAM_START;
use crate::memory::{Memory, MEMORY_SIZE};
use std::fs;
use std::path::Path;

/// Load a ROM file at `path` into `memory` starting at address `0x200`.
///
/// Returns the number of bytes loaded on success, or an error string if the
/// file cannot be read or does not fit in memory.
pub fn load_rom_to_memory<P: AsRef<Path>>(path: P, memory: &mut Memory) -> Result<usize, String> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|e| format!("load_rom: failed to read {:?}: {e}", path))?;

    let capacity = MEMORY_SIZE - PROGRAM_START as usize;
    if bytes.len() > capacity {
        return Err(format!(
            "load_rom: ROM size {} bytes exceeds available capacity {} bytes (4096 - 0x200)",
            bytes.len(),
            capacity
        ));
    }

    let base = PROGRAM_START as usize;
    for (i, byte) in bytes.iter().enumerate() {
        memory.write_byte((base + i) as u16, *byte)?;
    }

    Ok(bytes.len())
}
