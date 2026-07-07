//! Headless 64x32 monochrome display buffer for the CHIP-8 emulator.
//!
//! Implements the core video memory used by the CHIP-8 CPU to render
//! sprites: a simple 64 by 32 grid of boolean pixels with no rendering
//! backend (headless). The surrounding crate may read the buffer out and
//! display it however it likes.

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

pub struct Display {
    pixels: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl Display {
    pub fn new() -> Self {
        Display {
            pixels: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        for row in self.pixels.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Result<bool, String> {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return Err(format!("get_pixel: ({}, {}) out of bounds", x, y));
        }
        Ok(self.pixels[y][x])
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) -> Result<(), String> {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return Err(format!("set_pixel: ({}, {}) out of bounds", x, y));
        }
        self.pixels[y][x] = value;
        Ok(())
    }

    pub fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut collided = false;
        for (row, &byte) in sprite.iter().enumerate() {
            let actual_y = (y as usize + row) % DISPLAY_HEIGHT;
            for col in 0..8 {
                let bit = (byte >> (7 - col)) & 0x01 == 1;
                if !bit {
                    continue;
                }
                let actual_x = (x as usize + col) % DISPLAY_WIDTH;
                let current = self.pixels[actual_y][actual_x];
                let new_pixel = current ^ bit;
                if current && !new_pixel {
                    collided = true;
                }
                self.pixels[actual_y][actual_x] = new_pixel;
            }
        }
        collided
    }

    /// Render the display buffer as an ASCII string for terminal debugging.
    ///
    /// Each lit pixel becomes '█' and each unlit pixel becomes ' ' (space).
    /// Each row ends with '\n' (including the last row), so the result has
    /// exactly DISPLAY_HEIGHT lines. This is a debug/terminal dump only — it
    /// is NOT a renderer.
    pub fn to_ascii(&self) -> String {
        let mut out = String::with_capacity(DISPLAY_WIDTH * DISPLAY_HEIGHT + DISPLAY_HEIGHT);
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                out.push(if self.pixels[y][x] { '█' } else { ' ' });
            }
            out.push('\n');
        }
        out
    }

    /// Save the display buffer as an ASCII (P3) PPM image file.
    ///
    /// Each lit pixel is white (`255 255 255`), each unlit pixel is black
    /// (`0 0 0`). The image dimensions are `(64*scale)` by `(32*scale)`.
    /// The PPM P3 (ASCII) format is written with `std::fs` — no external
    /// image dependency.
    ///
    /// Errors: returns `Err(String)` if `scale == 0` or if the file cannot be
    /// created/written.
    pub fn save_ppm<P: AsRef<std::path::Path>>(&self, path: P, scale: usize) -> Result<(), String> {
        if scale == 0 {
            return Err("save_ppm: scale must be greater than 0".to_string());
        }
        let width = DISPLAY_WIDTH * scale;
        let height = DISPLAY_HEIGHT * scale;
        let mut out = String::with_capacity(32 + (width * height * 12));
        out.push_str("P3\n");
        out.push_str(&format!("{width} {height}\n"));
        out.push_str("255\n");
        for y in 0..DISPLAY_HEIGHT {
            for _dy in 0..scale {
                for x in 0..DISPLAY_WIDTH {
                    let on = self.pixels[y][x];
                    for _dx in 0..scale {
                        if on {
                            out.push_str("255 255 255 ");
                        } else {
                            out.push_str("0 0 0 ");
                        }
                    }
                }
                out.push('\n');
            }
        }
        std::fs::write(path, out).map_err(|e| format!("save_ppm: failed to write file: {e}"))
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}
