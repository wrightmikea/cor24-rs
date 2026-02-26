//! COR24 CPU emulator
//!
//! The COR24 is a C-Oriented RISC 24-bit architecture with:
//! - 8 general-purpose 24-bit registers (r0-r7)
//! - Special register aliases: fp=r3, sp=r4, z=r5, iv=r6, ir=r7
//! - Single condition flag (C)
//! - Variable-length instructions (1-4 bytes)
//! - Little-endian byte ordering

pub mod executor;
pub mod instruction;
pub mod state;

pub use executor::{ExecuteResult, Executor};
pub use instruction::{DecodedInstruction, InstructionFormat, Opcode, REG_NAMES};
pub use state::{CpuState, DecodeRom, INITIAL_SP, MEMORY_SIZE, RESET_ADDRESS};
