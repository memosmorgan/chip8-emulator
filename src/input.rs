//! Headless 16-key keypad input state for the CHIP-8 emulator.
//!
//! Tracks the pressed/released state of the CHIP-8 hex keypad. It is
//! intentionally headless and testable: there is no real window, SDL2, or
//! keyboard event loop here. The surrounding crate drives the state by
//! calling [`Input::press_key`] and [`Input::release_key`] directly.

pub const KEY_COUNT: usize = 16;

#[derive(Debug, Clone)]
pub struct Input {
    keys: [bool; KEY_COUNT],
}

impl Input {
    pub fn new() -> Self {
        Input {
            keys: [false; KEY_COUNT],
        }
    }

    pub fn press_key(&mut self, key: u8) -> Result<(), String> {
        if key > 0x0F {
            return Err(format!("press_key: invalid key {:#04X}", key));
        }
        self.keys[key as usize] = true;
        Ok(())
    }

    pub fn release_key(&mut self, key: u8) -> Result<(), String> {
        if key > 0x0F {
            return Err(format!("release_key: invalid key {:#04X}", key));
        }
        self.keys[key as usize] = false;
        Ok(())
    }

    pub fn is_pressed(&self, key: u8) -> Result<bool, String> {
        if key > 0x0F {
            return Err(format!("is_pressed: invalid key {:#04X}", key));
        }
        Ok(self.keys[key as usize])
    }

    pub fn first_pressed_key(&self) -> Option<u8> {
        for index in 0..KEY_COUNT {
            if self.keys[index] {
                return Some(index as u8);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        for key in self.keys.iter_mut() {
            *key = false;
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
