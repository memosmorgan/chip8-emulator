# Disassembler

The disassembler converts CHIP-8 opcodes into readable assembly-like text.
It reads bytes from memory and formats instructions, but it never executes
the ROM, changes CPU registers, draws pixels, ticks timers, or opens a
window.

This is useful while debugging because a ROM can be inspected before running
it. The listing makes control flow, register setup, drawing instructions, and
unsupported-opcode boundaries visible without needing an interactive
debugger.

## Usage

```bash
cargo run -- roms/example.ch8 --disassemble
cargo run -- roms/example.ch8 --disassemble --disassemble-count 10
```

The default listing starts at `0x0200` and prints 32 instructions. The
`--disassemble-count <n>` form prints the requested number of 2-byte
instructions.

`--disassemble` is headless-only and conflicts with `--window` (one mode
inspects bytes while the other executes the emulator). The process exits
before any execution loop starts.

## Example output

```text
Disassembly for roms/example.ch8
0200: 00E0  CLS
0202: 6101  LD V1, 0x01
0204: 1202  JP 0x0202
```

## Unsupported opcodes

Opcodes that are not part of the currently supported CHIP-8 instruction set
are formatted as `UNKNOWN 0xNNNN`. The disassembler does not add execution
support for those opcodes; it only reports that the bytes do not map to a
known mnemonic.
