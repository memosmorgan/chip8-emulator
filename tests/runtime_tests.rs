use chip8_emulator::display::{Display, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use chip8_emulator::runtime::{
    build_framebuffer, map_key_to_chip8, parse_cli_args, HostKey, RuntimeConfig,
    DEFAULT_DISASSEMBLE_COUNT, PIXEL_OFF, PIXEL_ON,
};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn keyboard_mapping_matches_chip8_layout() {
    let expected = [
        (HostKey::Key1, 0x1),
        (HostKey::Key2, 0x2),
        (HostKey::Key3, 0x3),
        (HostKey::Key4, 0xC),
        (HostKey::Q, 0x4),
        (HostKey::W, 0x5),
        (HostKey::E, 0x6),
        (HostKey::R, 0xD),
        (HostKey::A, 0x7),
        (HostKey::S, 0x8),
        (HostKey::D, 0x9),
        (HostKey::F, 0xE),
        (HostKey::Z, 0xA),
        (HostKey::X, 0x0),
        (HostKey::C, 0xB),
        (HostKey::V, 0xF),
    ];
    for (host, chip8) in expected.iter() {
        assert_eq!(map_key_to_chip8(*host), Some(*chip8));
    }
}

#[test]
fn runtime_config_default_values_are_reasonable() {
    let cfg = RuntimeConfig::default();
    assert!(cfg.scale > 0);
    assert!(cfg.cycles_per_second > 0);
    assert_eq!(cfg.max_cycles, None);
    assert!(!cfg.trace);
    assert!(!cfg.dump_state);
}

#[test]
fn runtime_config_defaults() {
    let cfg = RuntimeConfig::new();
    assert_eq!(cfg.scale, RuntimeConfig::default().scale);
    assert_eq!(
        cfg.cycles_per_second,
        RuntimeConfig::default().cycles_per_second
    );
}

#[test]
fn invalid_scale_or_hz_is_rejected() {
    assert!(parse_cli_args(&args(&["prog", "rom.ch8", "--window", "--scale", "0"])).is_err());
    assert!(parse_cli_args(&args(&["prog", "rom.ch8", "--window", "--hz", "0"])).is_err());
}

#[test]
fn parse_cli_args_positive_window() {
    let opts = parse_cli_args(&args(&[
        "prog", "rom.ch8", "--window", "--hz", "700", "--scale", "12",
    ]))
    .expect("valid args");
    assert_eq!(opts.rom_path, "rom.ch8");
    assert!(opts.window);
    assert_eq!(opts.hz, 700);
    assert_eq!(opts.scale, 12);
    assert_eq!(opts.max_cycles, None);
    assert!(!opts.trace);
    assert!(!opts.dump_state);
}

#[test]
fn parse_cli_args_headless_with_max_cycles_and_flags() {
    let opts = parse_cli_args(&args(&[
        "prog",
        "rom.ch8",
        "500",
        "--trace",
        "--dump-state",
    ]))
    .expect("valid");
    assert!(!opts.window);
    assert_eq!(opts.max_cycles, Some(500));
    assert!(opts.trace);
    assert!(opts.dump_state);
}

#[test]
fn build_framebuffer_single_pixel_at_origin() {
    let mut display = Display::new();
    display.set_pixel(0, 0, true).expect("set pixel at origin");

    let scale = 2;
    let width = DISPLAY_WIDTH * scale;
    let height = DISPLAY_HEIGHT * scale;
    let mut buffer = vec![PIXEL_OFF; width * height];
    build_framebuffer(&display, scale, &mut buffer).expect("build framebuffer");

    // Top-left 2x2 block should be on.
    assert_eq!(buffer[0], PIXEL_ON, "buffer[0] (row0 col0) should be ON");
    assert_eq!(buffer[1], PIXEL_ON, "buffer[1] (row0 col1) should be ON");
    assert_eq!(
        buffer[width], PIXEL_ON,
        "buffer[width] (row1 col0) should be ON"
    );
    assert_eq!(
        buffer[width + 1],
        PIXEL_ON,
        "buffer[width+1] (row1 col1) should be ON"
    );
    // The cell right after the 2x2 block in the first row should be OFF.
    assert_eq!(buffer[2], PIXEL_OFF, "buffer[2] (row0 col2) should be OFF");
    // Sanity: total length matches.
    assert_eq!(buffer.len(), width * height);
    assert_eq!(width * height, 64 * 2 * 32 * 2);
}

#[cfg(not(feature = "window"))]
#[test]
fn window_feature_disabled_returns_error_code() {
    use chip8_emulator::cpu::Cpu;
    use chip8_emulator::input::Input;
    use chip8_emulator::memory::Memory;
    use chip8_emulator::runtime::RuntimeConfig;

    let mut cpu = Cpu::new();
    let mut memory = Memory::new();
    let mut display = Display::new();
    let mut input = Input::new();
    let config = RuntimeConfig::default();
    let code = chip8_emulator::runtime::run_window(
        &mut cpu,
        &mut memory,
        &mut display,
        &mut input,
        &config,
    );
    assert_eq!(code, 1);
}

#[cfg(feature = "window")]
#[test]
fn minifb_key_mapping_compiles_with_window_feature() {
    use chip8_emulator::runtime::{map_key_to_chip8, minifb_key_to_host_key, HostKey};
    use minifb::Key;

    // Round-trip a few keys through minifb::Key -> HostKey -> CHIP-8 code.
    let cases = [
        (Key::Key1, HostKey::Key1, 0x1),
        (Key::W, HostKey::W, 0x5),
        (Key::V, HostKey::V, 0xF),
    ];
    for (mk, host, chip8) in cases.iter() {
        let mapped_host = minifb_key_to_host_key(*mk).expect("mapped host key");
        assert_eq!(mapped_host, *host);
        assert_eq!(map_key_to_chip8(mapped_host), Some(*chip8));
    }

    // Unmapped key returns None.
    assert!(minifb_key_to_host_key(Key::F1).is_none());
}

#[test]
fn parse_cli_args_accepts_screenshot_path() {
    let opts = parse_cli_args(&args(&[
        "prog",
        "rom.ch8",
        "100",
        "--screenshot",
        "out/example.ppm",
    ]))
    .expect("valid args with screenshot");
    assert_eq!(opts.screenshot.as_deref(), Some("out/example.ppm"));
    assert_eq!(opts.max_cycles, Some(100));
}

#[test]
fn parse_cli_args_rejects_screenshot_without_path() {
    let result = parse_cli_args(&args(&["prog", "rom.ch8", "--screenshot"]));
    assert!(result.is_err(), "missing --screenshot value should error");
}

#[test]
fn parse_cli_args_accepts_disassemble() {
    let opts = parse_cli_args(&args(&["prog", "rom.ch8", "--disassemble"]))
        .expect("valid disassemble args");

    assert!(opts.disassemble);
    assert_eq!(opts.disassemble_count, DEFAULT_DISASSEMBLE_COUNT);
    assert!(!opts.window);
}

#[test]
fn parse_cli_args_accepts_disassemble_count() {
    let opts = parse_cli_args(&args(&[
        "prog",
        "rom.ch8",
        "--disassemble",
        "--disassemble-count",
        "10",
    ]))
    .expect("valid disassemble count args");

    assert!(opts.disassemble);
    assert_eq!(opts.disassemble_count, 10);
}

#[test]
fn parse_cli_args_rejects_zero_disassemble_count() {
    let result = parse_cli_args(&args(&[
        "prog",
        "rom.ch8",
        "--disassemble",
        "--disassemble-count",
        "0",
    ]));

    assert!(result.is_err(), "zero disassemble count should error");
}

#[test]
fn parse_cli_args_rejects_window_with_disassemble() {
    let result = parse_cli_args(&args(&["prog", "rom.ch8", "--window", "--disassemble"]));

    assert!(
        result.is_err(),
        "--window and --disassemble should conflict"
    );
}

#[test]
fn headless_screenshot_is_saved_on_cpu_error() {
    // CPU-error runs should still write the requested screenshot.
    let bin = match std::env::var("CARGO_BIN_EXE_chip8_emulator") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("skipping: CARGO_BIN_EXE_chip8_emulator not set (run via `cargo test`)");
            return;
        }
    };

    let tmp = std::env::temp_dir().join(format!("chip8_errshot_test_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).expect("create temp dir");

    // 2-byte ROM: opcode 0x00FE (SUPER-CHIP high-res enable) -> rejected by base
    // CHIP-8 CPU as "Unsupported opcode: 0x00FE" at cycle 0.
    let rom_path = tmp.join("err.rom");
    std::fs::write(&rom_path, [0x00u8, 0xFEu8]).expect("write test rom");

    let shot_path = tmp.join("errshot.ppm");

    let output = std::process::Command::new(&bin)
        .arg(&rom_path)
        .arg("10")
        .arg("--screenshot")
        .arg(&shot_path)
        .output()
        .expect("run emulator binary");

    assert_eq!(
        output.status.code(),
        Some(1),
        "CPU error run should exit 1; stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Unsupported opcode: 0x00FE"),
        "expected CPU error for 0x00FE; stdout was: {stdout}"
    );

    assert!(
        shot_path.exists(),
        "screenshot PPM must be written even on CPU error"
    );
    let meta = std::fs::metadata(&shot_path).expect("stat screenshot");
    assert!(
        meta.len() > 0,
        "screenshot PPM must be non-empty; got {} bytes",
        meta.len()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn parse_cli_args_accepts_debugger() {
    let opts =
        parse_cli_args(&args(&["prog", "rom.ch8", "--debugger"])).expect("valid debugger args");
    assert!(opts.debugger);
    assert!(!opts.window);
    assert!(!opts.disassemble);
}

#[test]
fn parse_cli_args_rejects_debugger_with_window() {
    let result = parse_cli_args(&args(&["prog", "rom.ch8", "--debugger", "--window"]));
    assert!(result.is_err(), "--debugger and --window should conflict");
}

#[test]
fn parse_cli_args_rejects_debugger_with_disassemble() {
    let result = parse_cli_args(&args(&["prog", "rom.ch8", "--debugger", "--disassemble"]));
    assert!(
        result.is_err(),
        "--debugger and --disassemble should conflict"
    );
}
