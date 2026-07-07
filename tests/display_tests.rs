use chip8_emulator::display::{Display, DISPLAY_HEIGHT, DISPLAY_WIDTH};

#[test]
fn display_starts_cleared() {
    let display = Display::new();
    for y in 0..DISPLAY_HEIGHT {
        for x in 0..DISPLAY_WIDTH {
            assert!(
                !display.get_pixel(x, y).unwrap(),
                "pixel ({}, {}) should be off after new()",
                x,
                y
            );
        }
    }
}

#[test]
fn display_set_and_get_pixel() {
    let mut display = Display::new();
    display.set_pixel(3, 7, true).unwrap();
    assert!(display.get_pixel(3, 7).unwrap(), "pixel (3,7) should be on");
    assert!(
        !display.get_pixel(4, 7).unwrap(),
        "pixel (4,7) should remain off"
    );

    assert!(
        display.get_pixel(64, 0).is_err(),
        "get_pixel(64,0) should be out of bounds"
    );
    assert!(
        display.set_pixel(0, 32, true).is_err(),
        "set_pixel(0,32,true) should be out of bounds"
    );
}

#[test]
fn display_clear_turns_all_pixels_off() {
    let mut display = Display::new();
    let on_pixels = [(0, 0), (10, 10), (63, 31)];
    for &(x, y) in &on_pixels {
        display.set_pixel(x, y, true).unwrap();
        assert!(
            display.get_pixel(x, y).unwrap(),
            "pixel ({},{}) should be on before clear",
            x,
            y
        );
    }

    display.clear();

    for &(x, y) in &on_pixels {
        assert!(
            !display.get_pixel(x, y).unwrap(),
            "pixel ({},{}) should be off after clear",
            x,
            y
        );
    }
}

#[test]
fn draw_sprite_turns_pixels_on() {
    let mut display = Display::new();
    let sprite = [0b1111_0000];
    let collided = display.draw_sprite(0, 0, &sprite);

    assert!(!collided, "drawing on empty screen should not collide");
    assert!(display.get_pixel(0, 0).unwrap(), "(0,0) should be on");
    assert!(display.get_pixel(1, 0).unwrap(), "(1,0) should be on");
    assert!(display.get_pixel(2, 0).unwrap(), "(2,0) should be on");
    assert!(display.get_pixel(3, 0).unwrap(), "(3,0) should be on");
    assert!(!display.get_pixel(4, 0).unwrap(), "(4,0) should be off");
}

#[test]
fn draw_sprite_uses_xor() {
    let mut display = Display::new();
    let sprite = [0b1111_0000];

    let _ = display.draw_sprite(0, 0, &sprite);
    assert!(
        display.get_pixel(0, 0).unwrap(),
        "(0,0) on after first draw"
    );

    let _ = display.draw_sprite(0, 0, &sprite);
    assert!(
        !display.get_pixel(0, 0).unwrap(),
        "(0,0) should be off after XOR second draw"
    );
}

#[test]
fn draw_sprite_reports_collision() {
    let mut display = Display::new();
    let sprite = [0b1111_0000];

    let first = display.draw_sprite(0, 0, &sprite);
    assert!(!first, "first draw should not collide");

    let second = display.draw_sprite(0, 0, &sprite);
    assert!(second, "second draw should report collision");
}

#[test]
fn draw_sprite_wraps_at_screen_edges() {
    // Single ON bit at col 1 (bit 6) drawn at x=63 wraps to (0,31),
    // since actual_x = (63 + 1) % 64 = 0. (MSB at col 0 stays at x=63.)
    let mut display = Display::new();
    let sprite = [0b0100_0000];
    let _ = display.draw_sprite(63, 31, &sprite);
    assert!(
        display.get_pixel(0, 31).unwrap(),
        "bit should wrap from x=63 to x=0"
    );
    assert!(
        !display.get_pixel(63, 31).unwrap(),
        "(63,31) should remain off (bit wrapped away)"
    );

    // Two ON bits at x=63, y=0: col 0 -> (63,0), col 1 -> (0,0).
    let mut display2 = Display::new();
    let sprite2 = [0b1100_0000];
    let _ = display2.draw_sprite(63, 0, &sprite2);
    assert!(
        display2.get_pixel(63, 0).unwrap(),
        "first bit should be at (63,0)"
    );
    assert!(
        display2.get_pixel(0, 0).unwrap(),
        "second bit should wrap to (0,0)"
    );
}

#[test]
fn display_to_ascii_has_32_lines() {
    let display = Display::new();
    let ascii = display.to_ascii();
    // to_ascii ends each of the 32 rows with '\n', so lines() yields 32 lines.
    let lines: Vec<&str> = ascii.lines().collect();
    assert_eq!(
        lines.len(),
        DISPLAY_HEIGHT,
        "to_ascii must produce 32 lines"
    );
    for line in &lines {
        assert_eq!(line.len(), DISPLAY_WIDTH, "each line must be 64 chars wide");
    }
}

#[test]
fn display_to_ascii_marks_lit_pixels() {
    let mut display = Display::new();
    display.set_pixel(0, 0, true).unwrap();
    display.set_pixel(63, 31, true).unwrap();
    let ascii = display.to_ascii();
    let lines: Vec<&str> = ascii.lines().collect();
    // (0,0) is the first char of line 0.
    let first = lines[0].chars().next().unwrap();
    assert_eq!(first, '█', "(0,0) should be '█'");
    // The rest of line 0 should be spaces.
    let rest: Vec<char> = lines[0].chars().skip(1).collect();
    assert!(
        rest.iter().all(|c| *c == ' '),
        "rest of line 0 should be spaces"
    );
    // (63,31) is last char of last line.
    let last_line = lines[DISPLAY_HEIGHT - 1];
    let last = last_line.chars().last().unwrap();
    assert_eq!(last, '█', "(63,31) should be '█'");
    // First char of last line should be space.
    let first_last = last_line.chars().next().unwrap();
    assert_eq!(first_last, ' ', "(0,31) should be space");
}

#[test]
fn save_ppm_rejects_zero_scale() {
    let display = Display::new();
    let dir = std::env::temp_dir();
    let path = dir.join("chip8_test_zero_scale.ppm");
    let result = display.save_ppm(&path, 0);
    assert!(result.is_err(), "save_ppm with scale 0 should error");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn save_ppm_writes_valid_header() {
    let mut display = Display::new();
    display.set_pixel(0, 0, true).unwrap();
    let dir = std::env::temp_dir();
    let path = dir.join("chip8_test_ppm_header.ppm");
    display.save_ppm(&path, 4).expect("save_ppm should succeed");
    let content = std::fs::read_to_string(&path).expect("read ppm");
    let _ = std::fs::remove_file(&path);
    let mut lines = content.lines();
    let magic = lines.next().expect("first line");
    assert_eq!(magic, "P3", "first line must be the P3 magic number");
    let dims = lines.next().expect("second line");
    let parts: Vec<&str> = dims.split_whitespace().collect();
    assert_eq!(parts.len(), 2, "second line must have width and height");
    assert_eq!(parts[0], "256", "width must be 64*4=256");
    assert_eq!(parts[1], "128", "height must be 32*4=128");
    let maxval = lines.next().expect("third line");
    assert_eq!(maxval, "255", "third line must be the max color value");
}
