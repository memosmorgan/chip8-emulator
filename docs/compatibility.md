# Compatibility Report

This document tracks which CHIP-8 test ROMs the emulator has been run
against, the expected behavior, the observed result, and (where
available) a screenshot.

**ROM binaries are not committed to this repository.** Users must download
public/open test ROMs themselves and place them under `roms/external/`
(which is git-ignored for `*.ch8` / `*.rom` files; see `.gitignore` and
`roms/README.md`). Never commit commercial or copyrighted ROMs.

## How to reproduce a row

```bash
# Headless run with screenshot export:
cargo run -- roms/external/<rom>.ch8 <cycles> --screenshot screenshots/generated/<rom>.ppm

# With full trace + state dump for debugging:
cargo run -- roms/external/<rom>.ch8 <cycles> --trace --dump-state --screenshot screenshots/generated/<rom>.ppm

# Window mode (requires --features window):
cargo run --features window -- roms/external/<rom>.ch8 --window --hz 700 --scale 10
```

See `roms/README.md` for the ROM copyright policy and download links.

## Results

| ROM | Source | Mode | Cycles/Hz | Expected | Result | Screenshot | Notes |
|-----|--------|------|-----------|----------|--------|------------|-------|
| `roms/example.ch8` | self-written | headless | 100 | clears screen, sets V1=1, loops | Pass | screenshots/example.ppm | Self-written minimal smoke test; committed to repo. |
| `1-chip8-logo.ch8` | Timendus chip8-test-suite (GPL-3.0) | headless | 100 | "CHIP 8" splash renders, then idle self-jump | Pass | screenshots/generated/1-chip8-logo.ppm | "CHIP 8" splash rendered correctly; enters idle self-jump at 0x024E. Tests CLS, 6XNN, ANNN, DXYN (aligned), 1NNN. |
| `2-ibm-logo.ch8` | Timendus chip8-test-suite (GPL-3.0) | headless | 100 | IBM logo renders, then idle self-jump | Pass | screenshots/generated/2-ibm-logo.ppm | IBM logo rendered correctly; idle self-jump at 0x0228. Tests CLS, 6XNN, ANNN, 7XNN, DXYN (unaligned), 1NNN. |
| `3-corax_plus.ch8` | Timendus chip8-test-suite (GPL-3.0) | headless | 5000 | all opcode checks show checkmarks, pass-halt | Pass | screenshots/generated/3-corax_plus.ppm | All opcode checks show checkmarks (no X). Reaches pass-halt self-jump at 0x049C. Covers 3XNN,4XNN,5XY0,7XNN,9XY0,1NNN,2NNN,00EE,8XY0-7,E,FX55,FX65,FX33,FX1E, register-overflow. |
| `4-flags.ch8` | Timendus chip8-test-suite (GPL-3.0) | headless | 5000 | all HAPPY/CARRY/OTHER flag rows show checkmarks | Pass | screenshots/generated/4-flags.ppm | All HAPPY/CARRY/OTHER flag rows show checkmarks. Pass-halt at 0x0542. Covers 8XY1/2/3/4/5/6/7/E flag behavior including VF-as-operand cases. |
| `5-quirks.ch8` | Timendus chip8-test-suite (GPL-3.0) | headless | 5000 | interactive quirk-selection menu, then keypad input | Partial | screenshots/generated/5-quirks.ppm | Interactive quirk-selection menu opens and renders correctly, then waits for keypad input (FX0A / EX9E / EXA1). Headless cannot progress; needs `--features window --window` with manual keypad input in window mode to drive to completion. The menu rendering confirms CLS/DRAW/JP/skip opcodes work. |
| `6-keypad.ch8` | Timendus chip8-test-suite (GPL-3.0) | headless | 3000 | keypad-test menu, then keypad input | Partial | screenshots/generated/6-keypad.ppm | Keypad-test menu opens and renders, then waits on FX0A. Headless cannot progress; needs `--features window --window` with manual keypad input in window mode to drive to completion. |
| `7-beep.ch8` | Timendus chip8-test-suite (GPL-3.0) | headless | 200 | speaker icon blinks on each beep pulse | Pass | screenshots/generated/7-beep_on.ppm | Speaker icon renders (cycle 200 capture; the icon blinks on each beep pulse, so a 5000-cycle capture may land while the icon is off and show a blank screen). Tests ST/DT timers, DRW, EXA1 skip, FX07, FX15, FX18. Auto-SOS path runs without keypress. |
| `8-scrolling.ch8` | Timendus chip8-test-suite (GPL-3.0) | headless | 3000 | SUPER-CHIP scroll/high-res test | Fail (expected) | screenshots/generated/8-scrolling_error.ppm | CPU error at cycle 8: "Unsupported opcode: 0x00FE". 0x00FE is the SUPER-CHIP high-res-enable opcode; this emulator implements base CHIP-8 only (64x32), so rejecting SUPER-CHIP opcodes is the documented behavior. The scrolling test requires SUPER-CHIP scroll/high-res opcodes (00FE/00FF/00CN/00FB/00FC, DXY0 16x16 sprites). Not in scope for base CHIP-8. |
| `test_opcode.ch8` (Corax89) | https://github.com/corax89/chip8-test-rom | headless | 5000 | all "OK" rows rendered, pass-halt | Pass | screenshots/generated/test_opcode.ppm | Classic corax89 opcode test; all "OK" rows rendered. Pass-halt self-jump at 0x03DC. |

## Detailed findings

The base CHIP-8 opcode and flag ROMs pass. Corax89 `test_opcode.ch8`,
Timendus `3-corax_plus.ch8`, and `4-flags.ch8` reach their pass-halt
self-jumps with all expected checkmarks and no `X` marks. These cover the
core arithmetic/logic, skip, jump, subroutine, flag, and
FX33/FX55/FX65/FX1E paths, including register-overflow and VF-as-operand
cases.

Both logo ROMs render correctly: `1-chip8-logo.ch8` draws the "CHIP 8"
splash and `2-ibm-logo.ch8` draws the IBM logo, then both enter idle
self-jumps. They cover CLS, 6XNN, ANNN, 7XNN, DXYN (aligned and unaligned
sprite draws), and 1NNN.

The `7-beep.ch8` ROM passes. The speaker icon blinks, so high-cycle
captures can land while it is off; a cycle-200 capture shows it clearly.
This covers ST/DT timers, DRW, EXA1 skip, and FX07/FX15/FX18. Audio output
is not implemented, so this checks timer/draw behavior only.

The two interactive menu ROMs, `5-quirks.ch8` and `6-keypad.ch8`,
render their menus correctly (confirming CLS/DRAW/JP/skip opcodes work)
but need keypad input and are not good headless-only checks
(FX0A / EX9E / EXA1). They are marked `Partial` and need
`--features window --window` with real keypresses to drive to
completion.

The `8-scrolling.ch8` ROM FAILS at cycle 8 with "Unsupported opcode:
0x00FE". This emulator implements base CHIP-8 only (64x32), while this
ROM requires SUPER-CHIP opcodes: `00FE`/`00FF` (hi/lo-res enable), `00CN`
(scroll down N lines), `00FB`/`00FC` (scroll right/left), and `DXY0`
(16x16 sprites). It stays marked `Fail (expected)`.

## Screenshot-on-CPU-error behavior

The headless CPU-error path in `src/main.rs` calls `maybe_save_screenshot`
before returning exit code 1, so `--screenshot` actually produces a PPM even
when the CPU errors out (this makes debugging failing ROMs such as the
SUPER-CHIP scrolling test much easier). A regression test
`headless_screenshot_is_saved_on_cpu_error` in `tests/runtime_tests.rs`
spawns the binary with a 2-byte `0x00FE` ROM plus `--screenshot` and asserts
the PPM is written.

## Reproducing these results

```bash
cargo run -- roms/external/1-chip8-logo.ch8 100 --screenshot screenshots/generated/1-chip8-logo.ppm
cargo run -- roms/external/2-ibm-logo.ch8 100 --screenshot screenshots/generated/2-ibm-logo.ppm
cargo run -- roms/external/3-corax_plus.ch8 5000 --screenshot screenshots/generated/3-corax_plus.ppm
cargo run -- roms/external/4-flags.ch8 5000 --screenshot screenshots/generated/4-flags.ppm
cargo run -- roms/external/7-beep.ch8 200 --screenshot screenshots/generated/7-beep_on.ppm
cargo run -- roms/external/test_opcode.ch8 5000 --screenshot screenshots/generated/test_opcode.ppm
# Interactive (need window + keypresses):
cargo run --features window -- roms/external/5-quirks.ch8 --window --hz 700 --scale 10
cargo run --features window -- roms/external/6-keypad.ch8 --window --hz 700 --scale 10
# Expected SUPER-CHIP failure (base CHIP-8 rejects 0x00FE):
cargo run -- roms/external/8-scrolling.ch8 3000 --trace --dump-state --screenshot screenshots/generated/8-scrolling_error.ppm
```

## Legend

* **Result**: `Pass` if the ROM behaves as expected, `Partial` if it
  runs but some checks fail, `Not Tested` if it has not been run
  headless, `Fail` if it crashes or produces clearly wrong output, and
  `Fail (expected)` for deliberate scope-boundary failures such as
  SUPER-CHIP opcodes rejected by this base-CHIP-8-only emulator.
* **Screenshot**: path to a generated `.ppm` under `screenshots/`, or
  `none` if absent. Generated `.ppm` files are git-ignored; commit a converted
  PNG manually if you want to keep it in the repo.
* **Mode**: `headless` (default, no window feature) or `window`
  (requires `--features window`).

## Updating this file

When you run a new test ROM:

1. Download the ROM into `roms/external/` (do not commit it).
2. Run it headless with `--screenshot` and `--trace`/`--dump-state` as
   needed.
3. Inspect the trace, the state dump, and the exported `.ppm` screenshot.
4. Update the table row: set `Result`, `Cycles/Hz`, `Screenshot`, and
   `Notes`.
5. If you want to keep a screenshot in the repo, convert the `.ppm` to PNG
   and commit only the PNG (not the ROM).
