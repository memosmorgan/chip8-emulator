//! CHIP-8 CPU.
//!
//! Implements the CPU state plus a fetch/decode/execute cycle supporting the
//! core CHIP-8 opcodes: jumps, calls/returns, skips, register loads and
//! arithmetic/logic operations, keypad input (SKP/SKNP/LD Vx,K), delay/sound
//! timer opcodes, BNNN (JP V0, NNN), CXNN (RND Vx, NN via an internal
//! xorshift32 PRNG), and FX33 (LD B, Vx — binary-coded decimal store). `step`
//! mutates a [`Display`] and supports the CLS (00E0) and DRW (DXYN) opcodes.

use crate::debugger::CpuSnapshot;
use crate::display::Display;
use crate::input::Input;
use crate::memory::{Memory, FONT_BYTES_PER_CHAR, FONT_START};
use crate::opcode;

/// Number of general-purpose V registers (V0..VF).
pub const NUM_REGISTERS: usize = 16;
/// Stack depth for CHIP-8 subroutine calls.
pub const STACK_SIZE: usize = 16;
/// Address where CHIP-8 programs are loaded and execution starts.
pub const PROGRAM_START: u16 = 0x200;

/// CHIP-8 CPU state.
#[derive(Debug, Clone)]
pub struct Cpu {
    /// General-purpose 8-bit registers V0..VF.
    pub v: [u8; NUM_REGISTERS],
    /// 16-bit index register I.
    pub i: u16,
    /// Program counter.
    pub pc: u16,
    /// Stack pointer.
    pub sp: u8,
    /// Call stack.
    pub stack: [u16; STACK_SIZE],
    /// Delay timer (decrements at 60 Hz when non-zero).
    pub delay_timer: u8,
    /// Sound timer (decrements at 60 Hz; beeps while non-zero).
    pub sound_timer: u8,
    /// Internal PRNG state for the CXNN (RND) opcode; not cryptographic.
    pub rng_state: u32,
}

impl Cpu {
    /// Create a CPU in its initial state: registers cleared, pc = 0x200.
    pub fn new() -> Self {
        Cpu {
            v: [0u8; NUM_REGISTERS],
            i: 0,
            pc: PROGRAM_START,
            sp: 0,
            stack: [0u16; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            // Default xorshift32 seed. Fixed so two fresh CPUs start with the
            // same stream of random bytes (deterministic by default).
            rng_state: 0xA5A5_A5A5,
        }
    }

    /// Set the PRNG seed. Useful for deterministic tests.
    pub fn set_rng_seed(&mut self, seed: u32) {
        self.rng_state = seed;
    }

    /// Generate one pseudo-random byte using xorshift32.
    ///
    /// This is only for the RND opcode, not cryptographic use.
    fn next_random_byte(&mut self) -> u8 {
        // xorshift32: must never be zero or it stays zero.
        let mut x = self.rng_state;
        if x == 0 {
            x = 0xA5A5_A5A5;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state = x;
        (x & 0xFF) as u8
    }

    /// Fetch the next opcode from memory at the current program counter.
    ///
    /// On success the program counter is advanced by 2 (one instruction) and
    /// the 16-bit opcode is returned. On error the program counter is left
    /// unchanged and the error is returned.
    pub fn fetch_opcode(&mut self, memory: &Memory) -> Result<u16, String> {
        match memory.read_opcode(self.pc) {
            Ok(opcode) => {
                self.pc += 2;
                Ok(opcode)
            }
            Err(e) => Err(e),
        }
    }

    /// Execute a single fetch/decode/execute cycle.
    ///
    /// On success returns the raw opcode that was executed. On error the
    /// program counter may have already advanced by 2 (when fetch succeeded
    /// but the opcode was unsupported).
    pub fn step(
        &mut self,
        memory: &mut Memory,
        display: &mut Display,
        input: &Input,
    ) -> Result<u16, String> {
        let opcode = self.fetch_opcode(memory)?;
        let decoded = opcode::decode(opcode);
        match decoded.n1 {
            0x0 => match decoded.raw {
                0x00EE => {
                    // 00EE: RET from subroutine.
                    if self.sp == 0 {
                        return Err("Stack underflow on RET: sp == 0".to_string());
                    }
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                0x00E0 => {
                    // 00E0: CLS — clear the display.
                    display.clear();
                }
                _ => {
                    return Err(format!("Unsupported opcode: {:#06X}", opcode));
                }
            },
            0x1 => {
                // 1NNN: jump to address NNN.
                self.pc = decoded.nnn;
            }
            0x2 => {
                // 2NNN: call subroutine at NNN.
                if self.sp >= STACK_SIZE as u8 {
                    return Err(format!("Stack overflow on CALL: sp == {}", self.sp));
                }
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = decoded.nnn;
            }
            0x3 => {
                // 3XNN: skip if VX == NN.
                if self.v[decoded.x] == decoded.nn {
                    self.pc += 2;
                }
            }
            0x4 => {
                // 4XNN: skip if VX != NN.
                if self.v[decoded.x] != decoded.nn {
                    self.pc += 2;
                }
            }
            0x5 => {
                // 5XY0: skip if VX == VY.
                if decoded.n4 != 0 {
                    return Err(format!("Unsupported opcode: {:#06X}", opcode));
                }
                if self.v[decoded.x] == self.v[decoded.y] {
                    self.pc += 2;
                }
            }
            0x6 => {
                // 6XNN: set register VX to NN.
                self.v[decoded.x] = decoded.nn;
            }
            0x7 => {
                // 7XNN: add NN to VX (wrapping, no carry flag).
                self.v[decoded.x] = self.v[decoded.x].wrapping_add(decoded.nn);
            }
            0x8 => {
                match decoded.n4 {
                    0x0 => {
                        // 8XY0: LD Vx, Vy.
                        self.v[decoded.x] = self.v[decoded.y];
                    }
                    0x1 => {
                        // 8XY1: OR Vx, Vy.
                        self.v[decoded.x] |= self.v[decoded.y];
                    }
                    0x2 => {
                        // 8XY2: AND Vx, Vy.
                        self.v[decoded.x] &= self.v[decoded.y];
                    }
                    0x3 => {
                        // 8XY3: XOR Vx, Vy.
                        self.v[decoded.x] ^= self.v[decoded.y];
                    }
                    0x4 => {
                        // 8XY4: ADD Vx, Vy with carry flag.
                        let (sum, carry) = self.v[decoded.x].overflowing_add(self.v[decoded.y]);
                        self.v[decoded.x] = sum;
                        self.v[0xF] = if carry { 1 } else { 0 };
                    }
                    0x5 => {
                        // 8XY5: SUB Vx, Vy. VF = 1 if no borrow.
                        let (diff, borrow) = self.v[decoded.x].overflowing_sub(self.v[decoded.y]);
                        self.v[decoded.x] = diff;
                        self.v[0xF] = if borrow { 0 } else { 1 };
                    }
                    0x6 => {
                        // 8XY6: SHR Vx. VF = old LSB.
                        self.v[0xF] = self.v[decoded.x] & 1;
                        self.v[decoded.x] >>= 1;
                    }
                    0x7 => {
                        // 8XY7: SUBN Vx, Vy. VF = 1 if no borrow.
                        let (diff, borrow) = self.v[decoded.y].overflowing_sub(self.v[decoded.x]);
                        self.v[decoded.x] = diff;
                        self.v[0xF] = if borrow { 0 } else { 1 };
                    }
                    0xE => {
                        // 8XYE: SHL Vx. VF = old MSB.
                        self.v[0xF] = (self.v[decoded.x] >> 7) & 1;
                        self.v[decoded.x] <<= 1;
                    }
                    _ => {
                        return Err(format!("Unsupported opcode: {:#06X}", opcode));
                    }
                }
            }
            0x9 => {
                // 9XY0: skip if VX != VY.
                if decoded.n4 != 0 {
                    return Err(format!("Unsupported opcode: {:#06X}", opcode));
                }
                if self.v[decoded.x] != self.v[decoded.y] {
                    self.pc += 2;
                }
            }
            0xA => {
                // ANNN: LD I, NNN.
                self.i = decoded.nnn;
            }
            0xB => {
                // BNNN: JP V0, NNN — pc = NNN + V0 (wrapping, no panic on overflow).
                self.pc = decoded.nnn.wrapping_add(self.v[0] as u16);
            }
            0xC => {
                // CXNN: RND Vx, NN — Vx = random_byte & NN.
                let rnd = self.next_random_byte();
                self.v[decoded.x] = rnd & decoded.nn;
            }
            0xD => {
                // DXYN: DRW Vx, Vy, N — draw sprite at (Vx, Vy), height N, XOR.
                let height = decoded.n as usize;
                if height == 0 {
                    // DXY0 (Super-CHIP 16x16 sprite) is not supported in base CHIP-8.
                    return Err(format!("Unsupported opcode: {:#06X}", opcode));
                }
                let start = self.i as usize;
                if start + height > crate::memory::MEMORY_SIZE {
                    return Err(format!(
                        "DRW: sprite read out of bounds: I={:#06x}, height={}",
                        self.i, height
                    ));
                }
                let mut sprite = [0u8; 16]; // max height for base CHIP-8 is 15; 16 is safe
                for (row, byte) in sprite.iter_mut().enumerate().take(height) {
                    *byte = memory.read_byte(self.i + row as u16)?;
                }
                let collided =
                    display.draw_sprite(self.v[decoded.x], self.v[decoded.y], &sprite[..height]);
                self.v[0xF] = if collided { 1 } else { 0 };
            }
            0xE => {
                match decoded.nn {
                    0x9E => {
                        // EX9E: SKP Vx — skip if key in Vx is pressed.
                        let key = self.v[decoded.x];
                        if key > 0x0F {
                            return Err(format!("SKP: invalid key {:#04X} in V{}", key, decoded.x));
                        }
                        let pressed = input.is_pressed(key)?;
                        if pressed {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        // EXA1: SKNP Vx — skip if key in Vx is not pressed.
                        let key = self.v[decoded.x];
                        if key > 0x0F {
                            return Err(format!(
                                "SKNP: invalid key {:#04X} in V{}",
                                key, decoded.x
                            ));
                        }
                        let pressed = input.is_pressed(key)?;
                        if !pressed {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        return Err(format!("Unsupported opcode: {:#06X}", opcode));
                    }
                }
            }
            0xF => {
                match decoded.nn {
                    0x07 => {
                        // FX07: LD Vx, DT — read delay timer into Vx.
                        self.v[decoded.x] = self.delay_timer;
                    }
                    0x0A => {
                        // FX0A: LD Vx, K — wait for a key press.
                        match input.first_pressed_key() {
                            Some(key) => {
                                self.v[decoded.x] = key;
                            }
                            None => {
                                // No key pressed: wait. Undo the pc advance done by fetch_opcode
                                // so the same instruction is re-executed next cycle.
                                self.pc -= 2;
                            }
                        }
                    }
                    0x15 => {
                        // FX15: LD DT, Vx — set delay timer to Vx.
                        self.delay_timer = self.v[decoded.x];
                    }
                    0x18 => {
                        // FX18: LD ST, Vx — set sound timer to Vx.
                        self.sound_timer = self.v[decoded.x];
                    }
                    0x1E => {
                        // FX1E: ADD I, Vx (wrapping, no VF).
                        self.i = self.i.wrapping_add(self.v[decoded.x] as u16);
                    }
                    0x29 => {
                        // FX29: LD F, Vx. Point I to font sprite for digit VX.
                        self.i =
                            FONT_START + (self.v[decoded.x] as u16 * FONT_BYTES_PER_CHAR as u16);
                    }
                    0x55 => {
                        // FX55: LD [I], V0..Vx. I unchanged.
                        for reg in 0..=decoded.x {
                            memory.write_byte(self.i + reg as u16, self.v[reg])?;
                        }
                    }
                    0x65 => {
                        // FX65: LD V0..Vx, [I]. I unchanged.
                        for reg in 0..=decoded.x {
                            self.v[reg] = memory.read_byte(self.i + reg as u16)?;
                        }
                    }
                    0x33 => {
                        // FX33: LD B, Vx — store BCD of Vx at I, I+1, I+2.
                        // Bounds check: I+2 must be within memory.
                        let end = self.i.wrapping_add(2);
                        if end as usize >= crate::memory::MEMORY_SIZE {
                            return Err(format!(
                                "FX33: BCD store out of bounds: I={:#06x} (I+2 would reach {:#06x})",
                                self.i, end
                            ));
                        }
                        let val = self.v[decoded.x];
                        let hundreds = val / 100;
                        let tens = (val / 10) % 10;
                        let ones = val % 10;
                        memory.write_byte(self.i, hundreds)?;
                        memory.write_byte(self.i + 1, tens)?;
                        memory.write_byte(self.i + 2, ones)?;
                    }
                    _ => {
                        return Err(format!("Unsupported opcode: {:#06X}", opcode));
                    }
                }
            }
            _ => {
                return Err(format!("Unsupported opcode: {:#06X}", opcode));
            }
        }
        Ok(opcode)
    }

    /// Decrement the delay and sound timers by one tick. The host loop is
    /// expected to call this at ~60 Hz (see `runtime.rs` / `main.rs`).
    /// Timers never underflow: they stop at 0.
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    /// Capture a snapshot of the current CPU state for debugging/tracing.
    pub fn snapshot(&self) -> CpuSnapshot {
        CpuSnapshot {
            pc: self.pc,
            i: self.i,
            sp: self.sp,
            v: self.v,
            stack: self.stack,
            delay_timer: self.delay_timer,
            sound_timer: self.sound_timer,
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
