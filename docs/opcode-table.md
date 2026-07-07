# Opcode Table

Reference table for the opcode behavior handled by `Cpu::step()`. `0NNN` is
left unsupported because it was a legacy machine-code call on original
systems.

| Opcode | Name | Notes |
|---|---|---|
| `0NNN` | SYS addr | Unsupported; returns an error. |
| `00E0` | CLS | Clears the display buffer. |
| `00EE` | RET | Returns from a subroutine; stack underflow is checked. |
| `1NNN` | JP addr | Jumps to `NNN`. |
| `2NNN` | CALL addr | Calls `NNN`; pushes the post-fetch `pc`. |
| `3XNN` | SE Vx, NN | Skips if `Vx == NN`. |
| `4XNN` | SNE Vx, NN | Skips if `Vx != NN`. |
| `5XY0` | SE Vx, Vy | Skips if `Vx == Vy`; other `5XYN` forms error. |
| `6XNN` | LD Vx, NN | Sets `Vx = NN`. |
| `7XNN` | ADD Vx, NN | Wrapping add; `VF` unchanged. |
| `8XY0` | LD Vx, Vy | Sets `Vx = Vy`. |
| `8XY1` | OR Vx, Vy | Sets `Vx = Vx OR Vy`. |
| `8XY2` | AND Vx, Vy | Sets `Vx = Vx AND Vy`. |
| `8XY3` | XOR Vx, Vy | Sets `Vx = Vx XOR Vy`. |
| `8XY4` | ADD Vx, Vy | Adds with carry in `VF`. |
| `8XY5` | SUB Vx, Vy | Subtracts; `VF = 1` when there is no borrow. |
| `8XY6` | SHR Vx | Modern shift on `Vx`; `VF` gets the old LSB. |
| `8XY7` | SUBN Vx, Vy | Sets `Vx = Vy - Vx`; `VF = 1` when there is no borrow. |
| `8XYE` | SHL Vx | Modern shift on `Vx`; `VF` gets the old MSB. |
| `9XY0` | SNE Vx, Vy | Skips if `Vx != Vy`; other `9XYN` forms error. |
| `ANNN` | LD I, NNN | Sets `I = NNN`. |
| `BNNN` | JP V0, NNN | Jumps to `NNN + V0` with wrapping arithmetic. |
| `CXNN` | RND Vx, NN | Sets `Vx = random_byte & NN`. |
| `DXYN` | DRW Vx, Vy, N | XOR sprite draw with wrapping; `VF` is the collision flag. |
| `EX9E` | SKP Vx | Skips if the key in `Vx` is pressed. |
| `EXA1` | SKNP Vx | Skips if the key in `Vx` is not pressed. |
| `FX07` | LD Vx, DT | Reads the delay timer into `Vx`. |
| `FX0A` | LD Vx, K | Waits for a key press by re-running the instruction. |
| `FX15` | LD DT, Vx | Sets the delay timer. |
| `FX18` | LD ST, Vx | Sets the sound timer. |
| `FX1E` | ADD I, Vx | Adds `Vx` to `I` with wrapping arithmetic. |
| `FX29` | LD F, Vx | Points `I` to the font sprite for digit `Vx`. |
| `FX33` | LD B, Vx | Stores BCD digits at `I`, `I+1`, and `I+2`. |
| `FX55` | LD [I], Vx | Stores `V0..Vx`; `I` is unchanged. |
| `FX65` | LD Vx, [I] | Loads `V0..Vx`; `I` is unchanged. |

## Deferred / Quirk-sensitive behavior

Some CHIP-8 opcodes have well-known "classic vs. modern" behavioral
differences between original COSMAC VIP interpreters and newer
implementations. This emulator currently uses the modern/simple behavior for
the cases below.

### `8XY6` / `8XYE`: shift quirk

- **Classic CHIP-8**: shifts `VY` and stores the result in `VX` (i.e.
  `VX = VY >> 1` / `VX = VY << 1`), and `VF` holds the shifted-out bit.
- **Modern/simple (this project)**: shifts `VX` in place (`VX >>= 1` /
  `VX <<= 1`), ignores `VY`, and `VF` holds the old LSB/MSB of `VX`.

### `FX55` / `FX65`: load/store I increment quirk

- **Classic CHIP-8**: increments `I` by `x + 1` after the load/store.
- **Modern/simple (this project)**: leaves `I` unchanged.

### `BNNN`: jump with register offset quirk

- **Classic CHIP-8**: `pc = NNN + V0`.
- **Some interpreters (e.g. CHIP-48 / SCHIP on HP48)**: `pc = NNN + VX`
  where `X` is the high nibble of the address.
- **This project**: uses `V0` only (`pc = NNN + V0`, wrapping).
