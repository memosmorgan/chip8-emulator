# CHIP-8 Emulator in Rust

CHIP-8 emulator written in Rust.

![IBM logo running in the emulator](screenshots/ibm-logo.png)

> IBM Logo test ROM running through the emulator's CHIP-8 display pipeline.

Most standard CHIP-8 ROMs run correctly. The current scope is classic CHIP-8,
without SUPER-CHIP or audio output.

## What Works

- Classic CHIP-8 instruction set
- 4096-byte memory and ROM loading at `0x200`
- 64x32 monochrome display buffer with XOR sprite drawing
- Delay and sound timer state
- 16-key keypad state and window keyboard mapping
- Headless runs with optional trace and state dumps
- Window mode through the optional `window` feature
- PPM screenshot export
- Static disassembler
- Interactive debugger with breakpoints and watchpoints

## Quick Start

Run the small included ROM headlessly:

```bash
cargo run -- roms/example.ch8 100
```

Run with a window:

```bash
cargo run --features window -- roms/example.ch8 --window
```

Adjust window speed or scale:

```bash
cargo run --features window -- roms/example.ch8 --window --hz 700 --scale 10
```

Write a PPM screenshot:

```bash
cargo run -- roms/example.ch8 100 --screenshot screenshots/example.ppm
```

Generated `.ppm` files are ignored by default. Selected README screenshots
may be converted and committed as `.png`, such as `screenshots/ibm-logo.png`.

Disassemble a ROM without running it:

```bash
cargo run -- roms/example.ch8 --disassemble --disassemble-count 10
```

Start the debugger:

```bash
cargo run -- roms/example.ch8 --debugger
```

## Keyboard Mapping

```text
Keyboard: 1 2 3 4   CHIP-8: 1 2 3 C
Keyboard: Q W E R   CHIP-8: 4 5 6 D
Keyboard: A S D F   CHIP-8: 7 8 9 E
Keyboard: Z X C V   CHIP-8: A 0 B F
```

`ESC` closes the window.

## Current Limitations

- SUPER-CHIP opcodes are not supported.
- The sound timer is tracked, but there is no audio backend.
- Some test ROMs need manual keyboard input to finish.
- Screenshot export writes dependency-free PPM files; selected images can be
  converted to PNG for the README.
- Window mode requires `--features window`.

## Debugger Example

```txt
debug> disasm
0200: 00E0  CLS
0202: 6101  LD V1, 0x01
0204: 1202  JP 0x0202

debug> break 0x204
Added breakpoint 1 at 0x0204

debug> watch reg V1
Added watchpoint 1 on V1

debug> continue 10
0202: 6101  LD V1, 0x01
Watch 1 changed: V1 0x00 -> 0x01
```

More debugger commands are in `docs/debugger-repl.md`.

## Tests

```bash
cargo test
cargo test --features window
```

## ROM Policy

- Commercial or copyrighted ROMs are not included in this repository.
- External test ROMs go in `roms/external/`.
- `*.ch8` and `*.rom` files under `roms/external/` are gitignored.
- Use only public-domain, self-written, or properly licensed ROMs.

## Notes

Useful technical notes live in `docs/`:

- `docs/compatibility.md` - local test ROM results
- `docs/debugger-repl.md` - debugger commands
- `docs/disassembler.md` - static opcode listing mode
- `docs/opcode-table.md` - opcode behavior notes
- `docs/chip8-spec-notes.md` - CHIP-8 reference notes
- `docs/architecture.md` - module layout and runtime notes
- `docs/debugging-notes.md` - trace, state dump, and screenshot behavior
