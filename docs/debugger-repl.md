# Interactive Debugger

The interactive debugger is a headless REPL that drives the existing CHIP-8
core (`Cpu`, `Memory`, `Display`, `Input`) one instruction at a time. It is
useful for stepping through a ROM, inspecting registers, memory, the stack,
and the display buffer, without opening a window or running a fixed cycle
budget. The debugger also supports address breakpoints and V-register /
memory-byte watchpoints, all driven from the REPL without touching the CPU
execution core.

## Starting the debugger

```bash
cargo run -- roms/example.ch8 --debugger
```

`--debugger` is headless and cannot be combined with `--window` or
`--disassemble`. `--trace` is optional and has no effect inside the REPL
(each `step`/`continue` already prints executed instructions). The program
loads the ROM into memory at `0x200`, prints a banner, then prompts with
`debug> ` and reads commands from stdin until `quit` (or EOF).

## Commands

| Command                  | Alias | Description                                                       |
| ----------------------- | ----- | ----------------------------------------------------------------- |
| `help`                  |       | List all commands.                                                |
| `regs`                  |       | Print the CPU register/state snapshot (PC, I, SP, V0..VF, stack). |
| `step`                  | `s`   | Execute exactly one CPU instruction and show `PC: before -> after`. |
| `continue <n>`          | `c`   | Execute `n` instructions, stopping on the first error or on a breakpoint hit. |
| `disasm [<addr> <count>]` |     | Disassemble `count` instructions starting at `addr`. Defaults: current PC, 10. |
| `mem <addr> <len>`      |       | Dump `len` bytes of memory (max 256) as hex, 16 per line.         |
| `stack`                 |       | Print `SP` and the used stack slots.                              |
| `display`               |       | Print the 64x32 display buffer as ASCII (`█`/space) framed with `|...|`. |
| `timers`                |       | Print `DT` and `ST`.                                              |
| `tick`                  |       | Tick the delay/sound timers once and print the new values.        |
| `break <addr>`          |       | Set a breakpoint at address `addr` (decimal or `0x`-hex). Returns a new breakpoint id. |
| `breaks`                |       | List all breakpoints as `<id>  0xADDR`.                           |
| `delete <id>`           |       | Delete the breakpoint with the given id.                          |
| `clear-breaks`          |       | Remove all breakpoints.                                           |
| `watch reg <VX>`        |       | Watch V register `VX` (`V0`..`VF`, decimal `0`..`15`, or `0x0`..`0xF`). |
| `watch mem <addr>`      |       | Watch a single memory byte at `addr`.                             |
| `watches`               |       | List all watchpoints as `<id>  <desc>`.                           |
| `delete-watch <id>`     |       | Delete the watchpoint with the given id.                          |
| `clear-watches`         |       | Remove all watchpoints.                                           |
| `quit`                  | `q`   | Exit the debugger (also exits on EOF).                            |

The command word is case-insensitive (`HELP`, `Regs`, `S` all work). Address
arguments accept either decimal (`512`) or hex with a case-insensitive `0x`
prefix (`0x200`, `0xABCD`). Counts/lengths are decimal. The `VX` argument to
`watch reg` accepts a single case-insensitive hex digit (`V0`..`VF`), a
decimal value (`0`..`15`), or a `0x`-prefixed hex value (`0x0`..`0xF`).

On a CPU error during `step` or `continue`, the REPL prints `CPU error: ...`
(or `CPU error at instruction <i>: ...` for `continue`), then a CPU state
dump and the display buffer, and **continues the REPL**; it does not exit.
Use `quit` (or close stdin) to leave.

## Examples

```text
debug> disasm
0200: 00E0  CLS
0202: 6101  LD V1, 0x01
0204: 1202  JP 0x0202
...

debug> disasm 0x200 3
0200: 00E0  CLS
0202: 6101  LD V1, 0x01
0204: 1202  JP 0x0202

debug> step
0200: 00E0  CLS
PC: 0x0200 -> 0x0202

debug> regs
--- CPU State ---
PC=0x0202 I=0x0000 SP=0 DT=0 ST=0
V0=00 V1=00 ... VF=00
STACK[0]=0x0000 ... STACK[15]=0x0000

debug> mem 0x200 16
0200: 00 E0 61 01 12 02 00 00 00 00 00 00 00 00 00 00

debug> continue 5
0202: 6101  LD V1, 0x01
0204: 1202  JP 0x0202
0202: 6101  LD V1, 0x01
0204: 1202  JP 0x0202
0202: 6101  LD V1, 0x01
Executed 5 instructions.
PC=0x0204

debug> quit
```

## Breakpoints and Watchpoints

Breakpoints and watchpoints are kept in the in-memory REPL session only; they
are not persisted and do not affect the CPU core, the opcode set, or the
window runtime.

### Breakpoints

A breakpoint triggers on the PC value **before** an instruction is fetched
and executed. During `continue <n>`, before each instruction the REPL checks
whether the current PC matches a breakpoint address. If it does, the REPL
stops **without executing that instruction** and prints:

```text
Breakpoint hit at 0xADDR
Stopped at breakpoint. Executed N instructions.
PC=0xADDR
```

The breakpoint remains set after it is hit. Because the check happens before
the fetch, simply running `continue <n>` again from a breakpointed PC will
immediately re-hit the same breakpoint (zero instructions executed). To
advance past it, either `step` past the instruction at that PC (which
executes it once), or `delete <id>` / `clear-breaks` to remove the
breakpoint first.

`step` does not stop on a breakpoint (the user explicitly asked for one
instruction), but if the current PC matches a breakpoint it prints a hint
line:

```text
Note: breakpoint at 0xADDR
```

### Watchpoints

Watchpoints fire when a watched V register or memory byte changes between
the before/after snapshots taken around each executed instruction. This
applies to both `step` and `continue`. When a watched value changes, the
REPL prints one line per change:

```text
Watch <id> changed: <desc> 0x<old> -> 0x<new>
```

`<desc>` is `VX` (e.g. `VA`) for register watches and `MEM[0xADDR]` (e.g.
`MEM[0x0300]`) for memory watches. Values are printed as two-digit
lowercase/uppercase-hex bytes. A watchpoint that does not change during an
instruction produces no output.

### IDs

Breakpoint and watchpoint ids both start at `1` and increment for each new
item. `clear-breaks` and `clear-watches` remove all current items but keep
the id counter as-is, so the next `break`/`watch` added continues the
sequence rather than resetting to 1. `delete <id>` / `delete-watch <id>`
remove a single item and likewise do not affect the counter.

### Example: breakpoint

```text
debug> disasm 0x200 4
0200: 00E0  CLS
0202: 6101  LD V1, 0x01
0204: 1202  JP 0x0202
0206: 00EE  RET

debug> break 0x204
Added breakpoint 1 at 0x0204

debug> breaks
1  0x0204

debug> step
0200: 00E0  CLS
PC: 0x0200 -> 0x0202

debug> continue 10
0202: 6101  LD V1, 0x01
Breakpoint hit at 0x0204
Stopped at breakpoint. Executed 1 instructions.
PC=0x0204

debug> step
Note: breakpoint at 0x0204
0204: 1202  JP 0x0202
PC: 0x0204 -> 0x0202
```

### Example: watchpoint

```text
debug> watch reg V1
Added watchpoint 1 on V1

debug> watches
1  V1

debug> step
0202: 6101  LD V1, 0x01
PC: 0x0202 -> 0x0204
Watch 1 changed: V1 0x00 -> 0x01
```

## Notes

- Breakpoints and watchpoints are session-only and not persisted; there is no
  save-state support yet.
- Command parsing and the breakpoint/watchpoint managers live in
  `src/debug_repl.rs` (`parse_debug_command`, `parse_u16_value`,
  `parse_reg_index`, `format_memory_dump`, `BreakpointManager`,
  `WatchManager`, `Breakpoint`, `Watchpoint`, `WatchTarget`).
