# CHIP-8 Spec Notes

Quick reference for the CHIP-8 details this emulator depends on.

## Memory and machine state

- 4096 bytes of RAM, addresses `0x000` to `0xFFF`.
- Programs are loaded at `0x200`.
- `0x000` to `0x1FF` is kept for interpreter/font data.
- 16 general-purpose 8-bit registers: `V0` to `VF`.
- `VF` is also used as a flag by several instructions.
- 16-bit index register `I`.
- Program counter starts at `0x200`.
- 16-level call stack.
- 16-key hex keypad.
- 64x32 monochrome display.
- Delay and sound timers decrement at 60 Hz while non-zero.

## Instruction shape

All CHIP-8 instructions are 2 bytes, stored big-endian.

Common fields:

- `NNN`: 12-bit address
- `NN`: 8-bit immediate
- `N`: 4-bit nibble
- `X`, `Y`: register indexes

Examples:

- `1NNN`: jump
- `2NNN`: call
- `6XNN`: set `Vx`
- `7XNN`: add byte to `Vx`
- `ANNN`: set `I`
- `DXYN`: draw sprite

## Fonts

The built-in 4x5 hex font for digits `0` to `F` is stored at `0x050`
(`FONT_START` in `src/memory.rs`). Each digit is 5 bytes.

## Behavior choices

Some CHIP-8 behavior differs between interpreters. This emulator currently
uses the common modern choices below.

- `8XY6` / `8XYE`: shift `Vx` in place and ignore `Vy`; `VF` gets the bit
  shifted out.
- `FX55` / `FX65`: leave `I` unchanged after storing/loading registers.
- `BNNN`: jump to `NNN + V0`.

Other notes:

- `2NNN` pushes the post-fetch return address, so `00EE` returns to the
  instruction after the call.
- `DXYN` draws with XOR and wraps at the 64x32 display edges.
- `DXY0` is treated as unsupported because 16x16 sprites are SUPER-CHIP.
- `FX0A` waits for a key by rewinding `pc` so the same instruction runs again.
- `CXNN` uses a small deterministic xorshift32 PRNG. It is only for the CHIP-8
  random opcode, not for cryptography.
- `FX33` stores hundreds, tens, and ones at `I`, `I+1`, and `I+2`, with bounds
  checks before writing.

## Debug output

`Display::to_ascii()` is only a terminal dump of the 64x32 display buffer. It
does not open a window. Window rendering lives in `src/runtime.rs` behind the
optional `window` feature.
