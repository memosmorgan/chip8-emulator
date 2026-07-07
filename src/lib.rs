//! chip8-emulator — a CHIP-8 emulator written in Rust.
//!
//! This crate exposes the core emulator modules. The binary entry point
//! lives in `main.rs` and reuses the library defined here.

pub mod cpu;
pub mod debug_repl;
pub mod debugger;
pub mod disassembler;
pub mod display;
pub mod input;
pub mod memory;
pub mod opcode;
pub mod rom;
pub mod runtime;
pub mod timer;
