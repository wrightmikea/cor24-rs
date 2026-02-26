//! WASM bindings for the COR24 CPU simulator
//!
//! This module provides JavaScript-accessible interfaces to the CPU.

use crate::assembler::{Assembler, AssemblyResult};
use crate::challenge::get_challenges;
use crate::cpu::{CpuState, ExecuteResult, Executor};
use wasm_bindgen::prelude::*;
use web_sys::console;

/// WASM-accessible CPU wrapper
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmCpu {
    cpu: CpuState,
    executor: Executor,
    last_result: Option<AssemblyResult>,
}

impl Default for WasmCpu {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmCpu {
    /// Create a new CPU
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set panic hook for better error messages
        console_error_panic_hook::set_once();

        Self {
            cpu: CpuState::new(),
            executor: Executor::new(),
            last_result: None,
        }
    }

    /// Reset the CPU to initial state (preserves memory)
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    /// Hard reset - clears memory too
    pub fn hard_reset(&mut self) {
        self.cpu.hard_reset();
        self.last_result = None;
    }

    /// Assemble source code and load into memory
    pub fn assemble(&mut self, source: &str) -> Result<JsValue, JsValue> {
        let mut assembler = Assembler::new();
        let result = assembler.assemble(source);

        if result.errors.is_empty() {
            // Load program into memory at address 0
            self.cpu.hard_reset();
            self.cpu.load_program(0, &result.bytes);

            console::log_1(&JsValue::from_str(&format!(
                "Loaded {} bytes into memory",
                result.bytes.len()
            )));

            self.last_result = Some(result.clone());

            // Return assembly output as JSON
            serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
        } else {
            Err(JsValue::from_str(&result.errors.join("\n")))
        }
    }

    /// Get the assembled lines for display
    pub fn get_assembled_lines(&self) -> Vec<String> {
        if let Some(result) = &self.last_result {
            result
                .lines
                .iter()
                .map(|line| {
                    if line.bytes.is_empty() {
                        format!("       {}", line.source)
                    } else {
                        let bytes_str: String = line
                            .bytes
                            .iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        format!("{:04X}: {:12} {}", line.address, bytes_str, line.source)
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Execute one instruction
    pub fn step(&mut self) -> Result<bool, JsValue> {
        match self.executor.step(&mut self.cpu) {
            ExecuteResult::Ok => Ok(true),
            ExecuteResult::Halted => Ok(false),
            ExecuteResult::InvalidInstruction(byte) => Err(JsValue::from_str(&format!(
                "Invalid instruction: 0x{:02X}",
                byte
            ))),
            ExecuteResult::MemoryError(addr) => Err(JsValue::from_str(&format!(
                "Memory error at address: 0x{:06X}",
                addr
            ))),
        }
    }

    /// Run until halt or error
    pub fn run(&mut self) -> Result<(), JsValue> {
        match self.executor.run(&mut self.cpu, 100000) {
            ExecuteResult::Ok => Ok(()),
            ExecuteResult::Halted => Ok(()),
            ExecuteResult::InvalidInstruction(byte) => Err(JsValue::from_str(&format!(
                "Invalid instruction: 0x{:02X}",
                byte
            ))),
            ExecuteResult::MemoryError(addr) => Err(JsValue::from_str(&format!(
                "Memory error at address: 0x{:06X}",
                addr
            ))),
        }
    }

    /// Check if CPU is halted
    pub fn is_halted(&self) -> bool {
        self.cpu.halted
    }

    /// Get program counter
    pub fn pc(&self) -> u32 {
        self.cpu.pc
    }

    /// Get cycle count
    pub fn cycle_count(&self) -> u64 {
        self.cpu.cycles
    }

    /// Get instruction count
    pub fn instruction_count(&self) -> u64 {
        self.cpu.instructions
    }

    /// Get condition flag
    pub fn get_c_flag(&self) -> bool {
        self.cpu.c
    }

    /// Read a register value
    pub fn read_register(&self, reg: u8) -> u32 {
        self.cpu.get_reg(reg)
    }

    /// Get all register values as an array
    pub fn get_registers(&self) -> Vec<u32> {
        (0..8).map(|i| self.cpu.get_reg(i)).collect()
    }

    /// Read memory byte
    pub fn read_memory(&self, addr: u32) -> u8 {
        self.cpu.read_byte(addr)
    }

    /// Get memory slice as bytes
    pub fn get_memory_slice(&self, start: u32, len: u32) -> Vec<u8> {
        (0..len).map(|i| self.cpu.read_byte(start + i)).collect()
    }
}

// ===== Challenge System =====

/// Get number of available challenges
#[wasm_bindgen]
pub fn get_challenge_count() -> usize {
    get_challenges().len()
}

/// Validate a solution for a challenge
/// Returns true if the solution passes
#[wasm_bindgen]
pub fn validate_challenge(challenge_id: usize, source: &str) -> Result<bool, JsValue> {
    // Find the challenge
    let challenges = get_challenges();
    let challenge = challenges
        .iter()
        .find(|c| c.id == challenge_id)
        .ok_or_else(|| JsValue::from_str(&format!("Challenge {} not found", challenge_id)))?;

    // Assemble the source code
    let mut assembler = Assembler::new();
    let result = assembler.assemble(source);

    if !result.errors.is_empty() {
        return Err(JsValue::from_str(&result.errors.join("\n")));
    }

    // Create a new CPU and load the program
    let mut cpu = CpuState::new();
    cpu.load_program(0, &result.bytes);

    // Run the program
    let executor = Executor::new();
    match executor.run(&mut cpu, 100000) {
        ExecuteResult::Ok | ExecuteResult::Halted => {
            // Validate the result
            Ok((challenge.validator)(&cpu))
        }
        ExecuteResult::InvalidInstruction(byte) => Err(JsValue::from_str(&format!(
            "Invalid instruction: 0x{:02X}",
            byte
        ))),
        ExecuteResult::MemoryError(addr) => Err(JsValue::from_str(&format!(
            "Memory error at address: 0x{:06X}",
            addr
        ))),
    }
}

/// Initialize the WASM module and mount Yew app
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages in browser console
    console_error_panic_hook::set_once();

    // Mount the Yew app
    yew::Renderer::<crate::app::App>::new().render();
}
