# Code Layout

This document maps the main modules in `chip8_emulator` and notes how the
headless core connects to the optional window runtime.

## Module map

```txt
src/
├─ lib.rs          # crate root; declares all modules as `pub mod`
├─ main.rs         # binary entry point: CLI dispatch (headless / window / debugger / disassemble)
├─ cpu.rs          # CPU state, fetch/decode/execute, timer decrement (tick_timers), xorshift32 PRNG
├─ memory.rs       # 4096-byte RAM with bounds-checked byte/opcode reads
├─ rom.rs          # loads a ROM file into memory at 0x200
├─ opcode.rs       # opcode decoding helpers (nibbles/operands)
├─ disassembler.rs # formats CHIP-8 opcodes as assembly-like text (read-only)
├─ display.rs      # 64x32 monochrome display buffer; to_ascii() and save_ppm() helpers
├─ input.rs        # 16-key keypad input state (press/release/query, no window)
├─ timer.rs        # timer constants (TIMER_HZ=60)
├─ debugger.rs     # CpuSnapshot and TraceEntry plain-data records + formatters
├─ debug_repl.rs   # interactive headless debugger REPL (step/continue/disasm/mem/break/watch)
└─ runtime.rs      # minifb window renderer (feature-gated behind `window`), framebuffer,
                   # keyboard mapping, wall-clock CPU/timer loop, CLI parsing
```

## Fetch/decode/execute cycle

CHIP-8 instructions are 2 bytes, stored big-endian in memory. The CPU:

1. **Fetch**: reads 2 bytes at `pc` via `Memory::read_opcode`.
2. **Decode**: splits the opcode into nibbles and operands via `opcode::decode`.
3. **Execute**: mutates registers, memory, display, stack, timers.
4. Advance `pc` (handled in `fetch_opcode` for the non-branching case).
5. **Timers**: the host loop calls `Cpu::tick_timers()` (~60 Hz) to decrement
   delay/sound timers.

## Module responsibilities

The crate is split into a **headless core** and a thin **host layer**.

Headless core:

- `cpu.rs`: `Cpu` state and `step(memory, display, input)`. Reads keypad state
  via an immutable `&Input`. Carries an `rng_state` xorshift32 PRNG with
  `set_rng_seed` for deterministic tests. `tick_timers()` decrements
  `delay_timer`/`sound_timer` by 1 if non-zero (no underflow).
- `memory.rs`: 4096-byte RAM; all access is bounds-checked and returns
  `Result<_, String>`.
- `rom.rs`: `load_rom_to_memory` with a size check.
- `opcode.rs`: `decode` splitting opcodes into nibbles/operands.
- `disassembler.rs`: `disassemble_opcode` / `disassemble_range`. Reuses
  `opcode::decode` and `Memory::read_opcode`. It never calls `Cpu::step`,
  mutates memory, changes registers, draws pixels, ticks timers, or opens a
  window. Unsupported opcodes are formatted as `UNKNOWN 0xNNNN` without adding
  execution support.
- `display.rs`: 64x32 monochrome buffer. `to_ascii()` dumps it as `█`/space
  (debug only). `save_ppm(path, scale)` writes an ASCII PPM (P3) image of
  `(64*scale) x (32*scale)`, white-on-black, via `std::fs` (no image crate).
  `--scale 0` is rejected by the parser, so the default export is `640 x 320`.
- `input.rs`: `Input` with `press_key` / `release_key` / `is_pressed` /
  `first_pressed_key` / `clear`. No real keyboard event loop here.
- `timer.rs`: `pub const TIMER_HZ: u32 = 60;` only; no real-time measurement.
- `debugger.rs`: `CpuSnapshot` (point-in-time copy of `pc`, `i`, `sp`, `v`,
  `stack`, `delay_timer`, `sound_timer`) and `TraceEntry` (`pc_before` /
  `opcode` / `pc_after` / `i_after` / `v_after` + cycle index). Includes
  `format_cpu_snapshot` and `format_trace_entry`.
- `debug_repl.rs`: `run_debugger` REPL driving `Cpu`/`Memory`/`Display`/`Input`
  without a window or fixed cycle budget. Reuses
  `disassembler::disassemble_range`/`disassemble_opcode` and
  `debugger::format_cpu_snapshot`. Command parsing
  (`parse_debug_command`, `parse_u16_value`, `parse_reg_index`) and the
  breakpoint/watchpoint managers (`BreakpointManager`, `WatchManager`) are
  tested separately; only `run_debugger` itself touches stdin/stdout. Supports
  address breakpoints and V-register / memory-byte watchpoints. See
  `docs/debugger-repl.md`.

Host layer:

- `runtime.rs`: owns everything that touches the host:
  - `minifb` window lifecycle and the per-frame CPU step loop.
  - Framebuffer: scaling the 64x32 buffer to `(64*scale) x (32*scale)` for
    `window.update_with_buffer`.
  - Keyboard mapping between host keys and the CHIP-8 keypad, expressed via a
    logical `HostKey` enum so the mapping can be tested without
    `minifb::Key`.
  - Wall-clock scheduling of CPU cycles and timer ticks (`std::time::Instant`,
    `dt` clamped to 0.1s max).
  - CLI parsing for `--window`, `--scale`, `--hz`, `--trace`, `--dump-state`,
    `--screenshot`, `--debugger`, `--disassemble`/`--disassemble-count`, and the
    positional `[max-cycles]`, producing `CliOptions` + `RuntimeConfig`.

## Headless core vs window runtime

The headless core (`cpu.rs`, `memory.rs`, `display.rs`, `input.rs`,
`timer.rs`, `debugger.rs`, `disassembler.rs`, `debug_repl.rs`) has no
external dependencies or I/O. `runtime.rs` is a thin host layer that drives
that core with `std::time::Instant`-based wall-clock timing.

`minifb` is an **optional** dependency gated behind the `window` cargo feature.
`Cargo.toml` declares `default = []` and `window = ["dep:minifb"]`, so a plain
build does not pull in `minifb`. `runtime.rs` feature-gates all `minifb`
usage (`use minifb::...`, the `minifb_key_to_host_key` helper, and the real
`run_window`); without the feature, `run_window` is a stub that prints a
rebuild hint to stderr and returns exit code 1.

This split keeps CI and portable headless builds free of the system libraries
`minifb` needs on Linux (X11/Wayland development packages). A headless machine
can build and test the emulator without installing any windowing dev
packages. `cargo test` runs without `minifb`; `cargo test --features window`
additionally compiles/tests the window-gated code paths.

The pure helpers in `runtime.rs` (`map_key_to_chip8`,
`minifb_key_to_host_key`, `parse_cli_args`, `build_framebuffer`,
`RuntimeConfig`/`CliOptions` defaults) are covered by `tests/runtime_tests.rs`.
The window loop itself is the only piece not unit-tested (it requires an X
server).

## Constants and types

```text
pub const PIXEL_ON: u32 = 0x00FFFFFF;            // white, lit pixel
pub const PIXEL_OFF: u32 = 0x00000000;           // black, unlit pixel
pub const DEFAULT_HEADLESS_MAX_CYCLES: u32 = 1000;
pub const DEFAULT_SCALE: usize = 10;
pub const DEFAULT_HZ: u32 = 700;
pub const TIMER_HZ: u32 = 60;
```

- `HostKey`: a logical host key abstraction (`Key1..Key4`, `Q`, `W`, `E`,
  `R`, `A`, `S`, `D`, `F`, `Z`, `X`, `C`, `V`) decoupled from `minifb::Key`
  so the mapping can be tested directly.
- `map_key_to_chip8(HostKey) -> Option<u8>`: host key to CHIP-8 key.
- `minifb_key_to_host_key(minifb::Key) -> Option<HostKey>`: `minifb::Key` to
  `HostKey` (feature-gated).
- `RuntimeConfig { scale, cycles_per_second, max_cycles, trace, dump_state }`
  with `Default` (scale=10, cycles_per_second=700, max_cycles=None,
  trace=false, dump_state=false) and `new()`.
- `CliOptions { rom_path, max_cycles, trace, dump_state, window, scale, hz,
  screenshot, debugger, disassemble, disassemble_count }` and
  `parse_cli_args(&[String]) -> Result<CliOptions, String>`.

## Framebuffer

`build_framebuffer(display, scale, buffer)` writes a
`(64*scale) x (32*scale)` framebuffer. Each CHIP-8 display pixel becomes a
`scale x scale` block of `PIXEL_ON` (white, lit) or `PIXEL_OFF` (black,
unlit). `run_window` creates a `minifb` window titled `"CHIP-8 emulator"`
with `ScaleMode::Stretch` and `Scale::X1`, and calls
`window.update_with_buffer` after each CPU step batch.

## Keyboard mapping (host -> CHIP-8)

```text
1 2 3 4     1 2 3 C
Q W E R  -> 4 5 6 D
A S D F  -> 7 8 9 E
Z X C V  -> A 0 B F
```

`ESC` closes the window cleanly (exit 0).

## Runtime loop (run_window)

Per frame:

1. Clear `Input`; sample `window.get_keys()`. If `ESC` is pressed, break.
   Other mapped keys are pressed for the current frame (current-pressed-state
   semantics per frame).
2. Wall-clock timing via `std::time::Instant`; `dt` is clamped to 0.1s max.
3. CPU cycle accumulator: `cycle_accumulator += dt * cycles_per_second`;
   runs `floor(acc)` cycles (bounded by remaining `max_cycles` if set),
   subtracts the consumed count.
4. Timer accumulator: `timer_accumulator += dt`; while `>= 1/60` calls
   `cpu.tick_timers()`. Timers tick at ~60Hz wall-clock (not the headless
   "every 8 cycles" rough cadence).
5. After stepping: build the framebuffer and call `window.update_with_buffer`.

`max_cycles` reached -> clean break (window closes, exit 0). On `Cpu::step`
error, `run_window` prints `CPU error at cycle {n}: {e}`,
`Executed cycles: {n}`, a `--- CPU State ---` header plus
`format_cpu_snapshot`, and returns 1 (the window is dropped when the function
returns). On clean exit, if `dump_state` is set it prints the snapshot, then
prints `Executed cycles: {n}` and returns 0.

## Trace `pc_before`

`Cpu::step` only returns the raw opcode, so the trace `pc_before` field is
captured by the runner. In both headless and window mode, `pc_before` is
captured immediately before `cpu.step(...)` is called, so it is the true
pre-fetch PC of this step. A trace line looks like:

```text
#000012 PC=0x0200 OP=0x6101 -> PC=0x0202 I=0x0000
```

Headless `--trace` and window-mode `--trace` are consistent in this respect.

## Headless mode

Headless mode (the default) runs a bounded `cpu.step()` loop (default 1000
cycles), with timer ticks on a rough cadence (`TIMER_TICK_INTERVAL = 8`
cycles). `--trace` and `--dump-state` work as described in
`docs/debugging-notes.md`. On a CPU error it prints the cycle/error/snapshot
and the `--- Display ---` ASCII dump, then exits 1. `--screenshot` writes a
PPM at the end of the run; on the error path the runner still attempts the
screenshot before returning 1 (covered by the
`headless_screenshot_is_saved_on_cpu_error` regression test in
`tests/runtime_tests.rs`). The positional `[max-cycles]` works in both modes;
in window mode it makes the window close after that many cycles.

`--debugger` (see `docs/debugger-repl.md`) and `--disassemble` (see
`docs/disassembler.md`) are headless-only modes that exit before the
execution loop starts; both conflict with `--window`, and `--disassemble`
conflicts with `--debugger`.

## CLI

```text
<rom-path> [max-cycles] [--trace] [--dump-state] [--screenshot <path>]
           [--window] [--scale <n>] [--hz <n>]
           [--debugger] [--disassemble] [--disassemble-count <n>]
```

`max-cycles` is an optional positional `u32` immediately after the rom path
(only if it is not a `--` flag). Flags may appear in any order. `--scale 0`
and `--hz 0` are rejected. Defaults: `scale=10`, `hz=700`.

## Library vs binary

`lib.rs` declares all modules as `pub mod`, so integration tests under
`tests/` can import them with `use chip8_emulator::...`. `main.rs` is a thin
binary that depends on the library crate.

## Design rules

- Memory access is always bounds-checked and returns `Result<_, String>`.
- The CPU never advances `pc` when a fetch fails.
- CPU `step` produces side effects on both `Memory` and `Display`: it can
  read sprite bytes from memory and write pixels into the display buffer
  (e.g. `00E0` CLS, `DXYN` DRW).
- The headless core has zero external dependencies; everything uses `std`.
- CPU `step` reads keypad input via an immutable `&Input` reference (the CPU
  only queries keypad state; it does not mutate `Input`).
- No audio, save state, or SUPER-CHIP support for now; base CHIP-8 only
  (64x32).
