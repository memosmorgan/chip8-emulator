# Screenshots

The `screenshots/` directory holds exported PPM (P3 ASCII) images produced
by the emulator's `--screenshot` flag, plus selected PNG images.

## Layout

```text
screenshots/
├─ README.md      # this file (committed)
├─ example.ppm    # committed screenshot of the self-written roms/example.ch8 smoke test
├─ ibm-logo.png   # README image converted from a generated PPM
└─ generated/     # screenshots from external test ROMs - git-ignored (see .gitignore)
   └─ *.ppm
```

- `example.ppm` is a committed screenshot of the self-written
  `roms/example.ch8` smoke test, so the repo always has at least one
  reviewable artifact without depending on any third-party ROM.
- `generated/` holds screenshots produced from external test ROMs that the
  user downloads into `roms/external/`. It is git-ignored (see `.gitignore`);
  do not commit those `.ppm` files.
- `ibm-logo.png` is the selected README image. It is converted from
  `screenshots/generated/2-ibm-logo.ppm` and may be committed as a PNG.

## Producing a screenshot

```bash
cargo run -- roms/example.ch8 100 --screenshot screenshots/example.ppm
```

To add a new README screenshot, first export a PPM, then convert the chosen
frame to PNG before committing it:

```bash
magick screenshots/generated/2-ibm-logo.ppm screenshots/ibm-logo.png
```

`Display::save_ppm(path, scale)` writes a `(64*scale) x (32*scale)` ASCII
PPM (P3), white-on-black, via `std::fs`; no image dependency is added. The
scale is the `--scale` value (default 10, i.e. `640 x 320`).

## Notes

- PPM is intentionally dependency-free but relatively large; convert to PNG
  for long-term storage.
- On a fatal CPU error in headless mode the screenshot is still attempted
  before the runner exits (see `docs/debugging-notes.md` and
  `docs/architecture.md`).
- See `docs/compatibility.md` for the per-ROM screenshot rows associated with
  each public test ROM.
