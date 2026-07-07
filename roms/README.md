# ROMs

This repository does **not** include copyrighted commercial ROMs. Only
public-domain, freely redistributable test ROMs, self-written ROMs, or
ROMs whose license explicitly permits redistribution should be placed
here.

## Layout

```text
roms/
├─ README.md        # this file (committed)
├─ example.ch8      # self-written tiny test ROM (committed)
└─ external/        # for third-party / public test ROMs you download locally
   └─ *.ch8 / *.rom # git-ignored; do not commit these
```

* `roms/example.ch8` is a small self-written test ROM that is committed to
  the repo. It clears the screen, sets `V1 = 1`, and loops. It is safe to
  redistribute.
* `roms/external/` is a place for you to drop public/open test ROMs you
  download yourself. Files matching `roms/external/*.ch8` and
  `roms/external/*.rom` are **git-ignored** (see `.gitignore`). Do **not**
  commit third-party ROM binaries unless their license explicitly allows
  redistribution.

## Allowed ROMs

Place only ROMs that are:

* public domain,
* freely redistributable test ROMs,
* ROMs you wrote yourself,
* or ROMs whose license explicitly permits redistribution.

Do **not** commit commercial or copyrighted game ROMs.

## Suggested public test ROMs (download separately)

These are commonly used open test ROMs. Download them yourself and place
them under `roms/external/`. Do not commit the binaries unless their
license permits it. See `docs/compatibility.md` for the full per-ROM
results and `screenshots/README.md` for the screenshot workflow.

* **chip8-test-suite** (Timendus): a comprehensive modern instruction and
  behavior test suite. Source: https://github.com/Timendus/chip8-test-suite
* **Corax89 test ROM** (`test_opcode.ch8`): a classic opcode-coverage test
  ROM. The ROM file lives at the repo root:
  https://github.com/corax89/chip8-test-rom/raw/master/test_opcode.ch8
  (note: not under `rom/`, despite some mirrors linking it that way).
* **IBM logo ROM**: the classic first program, good for basic draw
  testing.

## Running a ROM

Headless (no window feature needed):

```bash
cargo run -- roms/example.ch8 500
cargo run -- roms/example.ch8 500 --trace --dump-state
```

With a screenshot export:

```bash
cargo run -- roms/example.ch8 100 --screenshot screenshots/example.ppm
```

Running a downloaded public test ROM (place it under `roms/external/`
yourself):

```bash
cargo run -- roms/external/test_opcode.ch8 5000 --screenshot screenshots/generated/test_opcode.ppm
```

Window mode (requires the `window` cargo feature):

```bash
cargo run --features window -- roms/external/test_opcode.ch8 --window --hz 700 --scale 10
```

See `docs/compatibility.md` for the per-ROM compatibility report and the
project `README.md` for the overall feature set.
