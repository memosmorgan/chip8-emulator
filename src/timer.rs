//! Timer constants for the CHIP-8 emulator.
//!
//! Holds the timer constants for the CHIP-8 delay and sound timers. The
//! actual wall-clock decrement is driven by `runtime.rs` (window mode) and
//! the rough cadence in `main.rs` (headless); this module only provides the
//! constants.

pub const TIMER_HZ: u32 = 60;
