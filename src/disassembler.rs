//! CHIP-8 disassembler helpers.
//!
//! This module formats raw CHIP-8 opcodes as assembly-like text without
//! executing them or mutating emulator state.

use crate::memory::Memory;
use crate::opcode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisassembledInstruction {
    pub address: u16,
    pub opcode: u16,
    pub mnemonic: String,
}

pub fn disassemble_opcode(opcode: u16) -> String {
    let decoded = opcode::decode(opcode);
    let vx = reg(decoded.x);
    let vy = reg(decoded.y);

    match decoded.n1 {
        0x0 => match decoded.raw {
            0x00E0 => "CLS".to_string(),
            0x00EE => "RET".to_string(),
            _ => format!("SYS {}", addr(decoded.nnn)),
        },
        0x1 => format!("JP {}", addr(decoded.nnn)),
        0x2 => format!("CALL {}", addr(decoded.nnn)),
        0x3 => format!("SE {vx}, {}", byte(decoded.nn)),
        0x4 => format!("SNE {vx}, {}", byte(decoded.nn)),
        0x5 => {
            if decoded.n4 == 0 {
                format!("SE {vx}, {vy}")
            } else {
                unknown(opcode)
            }
        }
        0x6 => format!("LD {vx}, {}", byte(decoded.nn)),
        0x7 => format!("ADD {vx}, {}", byte(decoded.nn)),
        0x8 => match decoded.n4 {
            0x0 => format!("LD {vx}, {vy}"),
            0x1 => format!("OR {vx}, {vy}"),
            0x2 => format!("AND {vx}, {vy}"),
            0x3 => format!("XOR {vx}, {vy}"),
            0x4 => format!("ADD {vx}, {vy}"),
            0x5 => format!("SUB {vx}, {vy}"),
            0x6 => format!("SHR {vx}"),
            0x7 => format!("SUBN {vx}, {vy}"),
            0xE => format!("SHL {vx}"),
            _ => unknown(opcode),
        },
        0x9 => {
            if decoded.n4 == 0 {
                format!("SNE {vx}, {vy}")
            } else {
                unknown(opcode)
            }
        }
        0xA => format!("LD I, {}", addr(decoded.nnn)),
        0xB => format!("JP V0, {}", addr(decoded.nnn)),
        0xC => format!("RND {vx}, {}", byte(decoded.nn)),
        0xD => format!("DRW {vx}, {vy}, {}", nibble(decoded.n)),
        0xE => match decoded.nn {
            0x9E => format!("SKP {vx}"),
            0xA1 => format!("SKNP {vx}"),
            _ => unknown(opcode),
        },
        0xF => match decoded.nn {
            0x07 => format!("LD {vx}, DT"),
            0x0A => format!("LD {vx}, K"),
            0x15 => format!("LD DT, {vx}"),
            0x18 => format!("LD ST, {vx}"),
            0x1E => format!("ADD I, {vx}"),
            0x29 => format!("LD F, {vx}"),
            0x33 => format!("LD B, {vx}"),
            0x55 => format!("LD [I], V0..{vx}"),
            0x65 => format!("LD V0..{vx}, [I]"),
            _ => unknown(opcode),
        },
        _ => unknown(opcode),
    }
}

pub fn disassemble_at(memory: &Memory, address: u16) -> Result<DisassembledInstruction, String> {
    let opcode = memory.read_opcode(address)?;
    Ok(DisassembledInstruction {
        address,
        opcode,
        mnemonic: disassemble_opcode(opcode),
    })
}

pub fn disassemble_range(
    memory: &Memory,
    start: u16,
    count: usize,
) -> Result<Vec<DisassembledInstruction>, String> {
    let mut instructions = Vec::with_capacity(count);
    let mut address = start;

    for _ in 0..count {
        instructions.push(disassemble_at(memory, address)?);
        address = address
            .checked_add(2)
            .ok_or_else(|| "disassemble_range: address overflow".to_string())?;
    }

    Ok(instructions)
}

fn reg(index: usize) -> String {
    format!("V{index:X}")
}

fn addr(value: u16) -> String {
    format!("0x{value:04X}")
}

fn byte(value: u8) -> String {
    format!("0x{value:02X}")
}

fn nibble(value: u8) -> String {
    format!("0x{value:X}")
}

fn unknown(opcode: u16) -> String {
    format!("UNKNOWN 0x{opcode:04X}")
}
