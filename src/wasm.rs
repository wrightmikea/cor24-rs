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

    // ===== I/O Peripheral Access =====

    /// Get LED state (8 bits)
    pub fn get_leds(&self) -> u8 {
        self.cpu.io.leds
    }

    /// Get switch state (8 bits)
    pub fn get_switches(&self) -> u8 {
        self.cpu.io.switches
    }

    /// Set switch state (simulates external switch input)
    pub fn set_switches(&mut self, value: u8) {
        self.cpu.io.switches = value;
    }

    /// Toggle a specific switch bit
    pub fn toggle_switch(&mut self, bit: u8) {
        if bit < 8 {
            self.cpu.io.switches ^= 1 << bit;
        }
    }

    /// Get UART output buffer
    pub fn get_uart_output(&self) -> String {
        self.cpu.io.uart_output.clone()
    }

    /// Clear UART output buffer
    pub fn clear_uart_output(&mut self) {
        self.cpu.io.uart_output.clear();
    }

    /// Send a character to UART RX (simulates input)
    pub fn uart_send_char(&mut self, c: char) {
        self.cpu.io.uart_rx = c as u8;
        self.cpu.io.uart_rx_ready = true;
    }

    // ===== Additional accessors for Rust pipeline =====

    /// Get program counter (alias for pc())
    pub fn get_pc(&self) -> u32 {
        self.cpu.pc
    }

    /// Get condition flag (alias for get_c_flag())
    pub fn get_condition_flag(&self) -> bool {
        self.cpu.c
    }

    /// Get LED value (alias for get_leds())
    pub fn get_led_value(&self) -> u8 {
        self.cpu.io.leds
    }

    /// Get cycle count as u32 (truncated from u64)
    pub fn get_cycle_count(&self) -> u32 {
        self.cpu.cycles as u32
    }

    /// Read a byte from memory (alias for read_memory())
    pub fn read_byte(&self, addr: u32) -> u8 {
        self.cpu.read_byte(addr)
    }

    /// Get the current instruction disassembly
    pub fn get_current_instruction(&self) -> String {
        if self.cpu.halted {
            return "HALTED".to_string();
        }
        // Read opcode at PC
        let pc = self.cpu.pc;
        let opcode = self.cpu.read_byte(pc);

        // Get basic instruction name from opcode
        let name = match opcode {
            0x00..=0x02 => "add",
            0x03..=0x0B => "add/sub/mul",
            0x0C..=0x12 => "logic",
            0x13 => "bra",
            0x14 => "brf",
            0x15 => "brt",
            0x16..=0x43 => "mov/cmp",
            0x44..=0x4F => "lc",
            0x50..=0x5F => "mov",
            0x60..=0x6F => "jmp/jal",
            0x70..=0x7F => "push/pop",
            0x80..=0x9F => "sw/lw",
            0xA0..=0xBF => "sb/lb",
            0xC0..=0xCF => "la",
            0xD0..=0xD2 => "li",
            _ => "???",
        };
        format!("{:04X}: {:02X}  {}", pc, opcode, name)
    }

    /// Check if should stop for LED output (for animation purposes)
    /// Returns true after a reasonable number of cycles to prevent infinite loops
    pub fn should_stop_for_led(&self) -> bool {
        // Stop after 10000 cycles to prevent infinite loops in animation
        self.cpu.cycles >= 10000
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
