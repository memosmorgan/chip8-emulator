# Debugging Notes

This document describes the non-interactive debugging tools the emulator
provides (`--trace`, `--dump-state`, `--screenshot`) and how they behave in
headless and window mode. For the interactive debugger REPL see
`docs/debugger-repl.md`; for the static opcode listing see
`docs/disassembler.md`.

## CPU state dump (`--dump-state`)

The `--dump-state` flag causes the runner to print the CPU state at the end
of execution, via `format_cpu_snapshot(&cpu.snapshot())`. A snapshot is
**always** printed when `Cpu::step` returns an error (regardless of the
flag) so the post-error state is visible.

Example output:

```text
--- CPU State ---
PC=0x0204 I=0x0000 SP=0 DT=0 ST=0
V0=00 V1=01 V2=00 V3=00 V4=00 V5=00 V6=00 V7=00 V8=00 V9=00 VA=00 VB=00 VC=00 VD=00 VE=00 VF=00
STACK[0]=0x0000 STACK[1]=0x0000 ... STACK[15]=0x0000
```

The first line has `PC` and `I` in `0x{:04X}` form and `SP`, `DT`, `ST` as
plain decimal. The second line lists `V0..VF` as `{:02X}` (no `0x` prefix).
The third line lists all 16 stack slots as `0x{:04X}`.

## Opcode trace (`--trace`)

The `--trace` flag prints one line per executed cycle, via
`format_trace_entry`. Each line records the cycle index, the program
counter before fetch, the raw opcode, the program counter after execution,
and the index register after execution.

Example line:

```text
#000012 PC=0x0200 OP=0x6A0F -> PC=0x0202 I=0x0000
```

The cycle index is zero-padded to 6 digits. The `v_after` array is captured
into the `TraceEntry` but is not printed by the default formatter (kept in
the struct for programmatic use / future expansion).

## Combining flags

Both flags can be combined; they are independent:

```bash
cargo run -- roms/example.ch8 500 --trace --dump-state
```

The trace lines are printed during the loop, and the CPU state snapshot is
printed at the end (and on error).

## Error output

When `Cpu::step` returns an `Err`, the headless runner in `main.rs`:

1. Prints `CPU error at cycle {cycle}: {error}`.
2. Prints `Executed cycles: {executed}`.
3. Prints a `--- CPU State ---` header followed by
   `format_cpu_snapshot(&cpu.snapshot())` (the post-error state; note
   that `step` may have already advanced `pc` or mutated registers before
   the error was returned).
4. Prints a `--- Display ---` header followed by the ASCII display dump.
5. Exits with code 1.

To read an error dump: look at `PC` (where execution stopped), the opcode
in the error message, the `V` registers, and `I`/`SP` to understand what
the program was doing when it failed.

## Unsupported opcode debug flow

Unsupported opcodes return `Err(format!("Unsupported opcode: {:#06X}", opcode))`
from `Cpu::step`. The runner treats this as a fatal error and dumps the
CPU state + display as described above. `0NNN` (the legacy `SYS addr`
machine-code call) is treated as unsupported/ignored: it falls into the
same `_ => return Err(...)` branch as any other unknown opcode, so running
a ROM that contains `0NNN` will halt with an "Unsupported opcode" error
and a full state dump.

## Integration points

`debugger.rs` exposes CPU snapshots, trace records, and formatters. The
runners create those records around `Cpu::step`; formatting does not mutate
CPU state.

## Window mode (`--window`)

Window mode lives in `runtime.rs` (see `docs/architecture.md`). The
headless debugging behavior above is unchanged. The notes below cover only
what is different in window mode.

### `--trace` in window mode

`--window` and `--trace` can be combined. In window mode `run_window`
prints one trace line per executed cycle. `pc_before` is captured
immediately before `cpu.step(...)` is called, matching headless `--trace`.
A window-mode trace
line looks like:

```text
#000012 PC=0x0200 OP=0x6101 -> PC=0x0202 I=0x0000
```

where the first `PC` is the address fetched for that step.

### Runtime error behavior in window mode

When `Cpu::step` returns an `Err` inside `run_window`:

1. Prints `CPU error at cycle {n}: {e}`.
2. Prints `Executed cycles: {n}`.
3. Prints a `--- CPU State ---` header followed by
   `format_cpu_snapshot(&cpu.snapshot())`.
4. Returns exit code 1. The window is dropped automatically when
   `run_window` returns.

Note that window mode does **not** print the ASCII display dump on error
(the window itself is the display). Headless mode still prints the
`--- Display ---` ASCII dump on error.

### `--dump-state` in window mode

`--dump-state` in window mode prints the CPU snapshot (via
`format_cpu_snapshot`) when the loop ends cleanly (ESC closed, window
closed, or `max_cycles` reached), then prints `Executed cycles: {n}`.

### `max_cycles` in window mode

Passing a positional `max_cycles` in window mode causes a clean close
after that many cycles (exit 0). Example:

```bash
cargo run -- roms/example.ch8 500 --window
```

The window closes automatically once the cycle budget is consumed.

### ESC

`ESC` closes the window cleanly (exit 0). No error dump is produced on a
clean ESC exit; `--dump-state` still applies if it was passed.

## Feature-gated window runtime

The `minifb` window backend is gated behind the `window` cargo feature.

- Build/run window mode with `--features window`.
- Running `--window` without the `window` feature prints a rebuild hint
  (telling you to rebuild with `--features window`) to stderr and exits
  with code 1.
- The headless debugging tools (`--trace`, `--dump-state`) work
  featurelessly; `cargo test` runs without `minifb`. Use
  `cargo test --features window` to also compile/test the window-gated
  code paths.
- Window-mode trace `pc_before` is captured immediately before
  `cpu.step(...)`, so headless and window `--trace` output are consistent.

## Screenshot-assisted debugging

When a test ROM misbehaves, combine the existing tools with the
`--screenshot` flag to capture a reviewable artifact:

```bash
cargo run -- roms/external/test_opcode.ch8 5000 \
  --trace --dump-state --screenshot screenshots/generated/test_opcode.ppm
```

This produces:

* one trace line per executed opcode (`--trace`),
* a final CPU state snapshot (`--dump-state`),
* a PPM image of the final display buffer (`--screenshot`).

If the ROM errors out, the runner still prints the `--- CPU State ---` and
`--- Display ---` dumps; the screenshot is written from the display buffer at
the end of the run (success path) or after the window closes (window mode).
On a fatal CPU error in headless mode the screenshot is still attempted at
the end of the headless run (this is covered by the
`headless_screenshot_is_saved_on_cpu_error` regression test in
`tests/runtime_tests.rs`); it reflects whatever the display buffer holds at
that point.

### Known limitations

* In headless mode, the screenshot reflects the display buffer at the end
  of the run. The `--- Display ---` ASCII dump printed on a CPU error is the
  most reliable artifact for the exact failure point.
* In window mode, the screenshot is taken from the last rendered frame
  after the window closes (clean exit or `max_cycles` reached). On a CPU
  error the window returns `1` and the screenshot reflects the last
  successful frame.
* PPM (P3) is ASCII and relatively large; convert to PNG for long-term
  storage. Generated `.ppm` files are git-ignored (see `.gitignore`); see
  `screenshots/README.md` for the screenshots directory layout.
