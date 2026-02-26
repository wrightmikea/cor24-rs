//! COR24 CPU state

use serde::{Deserialize, Serialize};

/// Memory size: 64KB for emulation (addresses 0x000000-0x00FFFF)
pub const MEMORY_SIZE: usize = 65536;

/// Default reset address (embedded block RAM start)
pub const RESET_ADDRESS: u32 = 0x000000;

/// Stack pointer initial value
pub const INITIAL_SP: u32 = 0x00FC00;

/// COR24 CPU state
#[derive(Clone, Serialize, Deserialize)]
pub struct CpuState {
    /// Program counter (24-bit)
    pub pc: u32,
    /// Register file (8 x 24-bit registers)
    pub registers: [u32; 8],
    /// Condition flag
    pub c: bool,
    /// Memory (byte-addressable)
    pub memory: Vec<u8>,
    /// Halted flag
    pub halted: bool,
    /// Cycle count
    pub cycles: u64,
    /// Instruction count
    pub instructions: u64,
}

impl Default for CpuState {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuState {
    /// Create a new CPU state with default values
    pub fn new() -> Self {
        let mut state = Self {
            pc: RESET_ADDRESS,
            registers: [0; 8],
            c: false,
            memory: vec![0; MEMORY_SIZE],
            halted: false,
            cycles: 0,
            instructions: 0,
        };
        // Initialize stack pointer
        state.registers[4] = INITIAL_SP;
        state
    }

    /// Reset CPU to initial state (preserves memory)
    pub fn reset(&mut self) {
        self.pc = RESET_ADDRESS;
        self.registers = [0; 8];
        self.registers[4] = INITIAL_SP;
        self.c = false;
        self.halted = false;
        self.cycles = 0;
        self.instructions = 0;
    }

    /// Hard reset (clears memory too)
    pub fn hard_reset(&mut self) {
        self.reset();
        self.memory.fill(0);
    }

    /// Read a byte from memory
    pub fn read_byte(&self, addr: u32) -> u8 {
        let addr = (addr as usize) % MEMORY_SIZE;
        self.memory[addr]
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, addr: u32, value: u8) {
        let addr = (addr as usize) % MEMORY_SIZE;
        self.memory[addr] = value;
    }

    /// Read a 24-bit word from memory (little-endian)
    pub fn read_word(&self, addr: u32) -> u32 {
        let b0 = self.read_byte(addr) as u32;
        let b1 = self.read_byte(addr.wrapping_add(1)) as u32;
        let b2 = self.read_byte(addr.wrapping_add(2)) as u32;
        b0 | (b1 << 8) | (b2 << 16)
    }

    /// Write a 24-bit word to memory (little-endian)
    pub fn write_word(&mut self, addr: u32, value: u32) {
        self.write_byte(addr, (value & 0xFF) as u8);
        self.write_byte(addr.wrapping_add(1), ((value >> 8) & 0xFF) as u8);
        self.write_byte(addr.wrapping_add(2), ((value >> 16) & 0xFF) as u8);
    }

    /// Get register value (masked to 24 bits)
    pub fn get_reg(&self, reg: u8) -> u32 {
        self.registers[(reg & 0x07) as usize] & 0xFFFFFF
    }

    /// Set register value (masked to 24 bits)
    pub fn set_reg(&mut self, reg: u8, value: u32) {
        self.registers[(reg & 0x07) as usize] = value & 0xFFFFFF;
    }

    /// Sign extend 8-bit to 24-bit
    pub fn sign_extend_8(value: u8) -> u32 {
        if value & 0x80 != 0 {
            0xFFFF00 | (value as u32)
        } else {
            value as u32
        }
    }

    /// Sign extend 24-bit result
    pub fn mask_24(value: u32) -> u32 {
        value & 0xFFFFFF
    }

    /// Load program into memory at given address
    pub fn load_program(&mut self, start_addr: u32, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(start_addr + i as u32, byte);
        }
    }
}

/// Instruction decode ROM
/// Maps 8-bit instruction bytes to 12-bit decoded values: [opcode(5):ra(3):rb(3)]
#[derive(Clone)]
pub struct DecodeRom {
    entries: [u16; 256],
}

impl Default for DecodeRom {
    fn default() -> Self {
        Self::new()
    }
}

impl DecodeRom {
    /// Create decode ROM with instruction mappings
    pub fn new() -> Self {
        let mut entries = [0xFFFu16; 256]; // Default to invalid

        // Build decode table from known instruction encodings
        // Format: entries[byte] = (opcode << 6) | (ra << 3) | rb

        // Single-byte register operations (from listings analysis)
        // add r0,r1 = 01, add r0,r2 = 02, etc.
        // opcode=0x00=AddReg, ra=0, rb varies
        for rb in 0..8u8 {
            entries[rb as usize] = rb as u16; // add r0,rb
        }

        // cls ra,rb patterns (opcode=0x07=Cls)
        entries[0x1A] = (0x07 << 6) | 2; // cls r0,r2 (ra=0, rb=2)
        entries[0x1B] = (0x07 << 6) | (1 << 3); // cls r1,r0 (ra=1, rb=0)
        entries[0x1D] = (0x07 << 6) | (2 << 3); // cls r2,r0 (ra=2, rb=0)

        // jal and jmp patterns
        entries[0x25] = (0x09 << 6) | (1 << 3); // jal r1,(r0) (ra=1, rb=0)
        entries[0x27] = (0x0A << 6) | (1 << 3); // jmp (r1) (ra=1, rb=0)

        // mov patterns (opcode=0x11=Mov)
        entries[0x57] = (0x11 << 6) | 2; // mov r0,r2 (ra=0, rb=2)
        entries[0x65] = (0x11 << 6) | (3 << 3) | 4; // mov fp,sp
        entries[0x69] = (0x11 << 6) | (4 << 3) | 3; // mov sp,fp

        // pop patterns (opcode=0x14=Pop, rb=4 for sp)
        entries[0x78] = (0x14 << 6) | 4; // pop r0 (ra=0, rb=4)
        entries[0x79] = (0x14 << 6) | 4; // pop r0 (alt?)
        entries[0x7A] = (0x14 << 6) | (1 << 3) | 4; // pop r1
        entries[0x7B] = (0x14 << 6) | (2 << 3) | 4; // pop r2
        entries[0x7C] = (0x14 << 6) | (3 << 3) | 4; // pop fp

        // push patterns (opcode=0x15=Push, rb=4 for sp)
        entries[0x7D] = (0x15 << 6) | 4; // push r0 (ra=0, rb=4)
        entries[0x7E] = (0x15 << 6) | (1 << 3) | 4; // push r1
        entries[0x7F] = (0x15 << 6) | (2 << 3) | 4; // push r2
        entries[0x80] = (0x15 << 6) | (3 << 3) | 4; // push fp

        // clu patterns
        entries[0xCE] = (0x08 << 6) | (5 << 3); // clu z,r0 (ra=5, rb=0)

        // Two-byte instructions (first byte encodes opcode + ra)
        // add ra,imm8: 08-0F range
        for ra in 0..8u8 {
            entries[(0x08 + ra) as usize] = (0x01 << 6) | ((ra as u16) << 3);
        }

        // bra/brf/brt: 13/14/15
        entries[0x13] = 0x03 << 6; // bra (ra=0, rb=0)
        entries[0x14] = 0x04 << 6; // brf (ra=0, rb=0)
        entries[0x15] = 0x05 << 6; // brt (ra=0, rb=0)

        // lc ra,imm8: 44-47 range
        for ra in 0..8u8 {
            entries[(0x44 + ra) as usize] = (0x0E << 6) | ((ra as u16) << 3);
        }

        // lcu ra,imm8: 3C-3F range
        for ra in 0..8u8 {
            entries[(0x3C + ra) as usize] = (0x0F << 6) | ((ra as u16) << 3);
        }

        // lb ra,dd(rb): various patterns
        entries[0x2C] = 0x0C << 6; // lb r0,(r0) (ra=0, rb=0)
        entries[0x2E] = (0x0C << 6) | 2; // lb r0,(r2) (ra=0, rb=2)

        // lw ra,dd(rb): 4D, 51, 55 patterns
        entries[0x4D] = (0x10 << 6) | 3; // lw r0,dd(fp) (ra=0, rb=3)
        entries[0x51] = (0x10 << 6) | (1 << 3) | 3; // lw r1,dd(fp)
        entries[0x55] = (0x10 << 6) | (2 << 3) | 3; // lw r2,dd(fp)

        // sb ra,dd(rb): 82, 84 patterns
        entries[0x82] = (0x16 << 6) | 2; // sb r0,(r2) (ra=0, rb=2)
        entries[0x84] = (0x16 << 6) | (1 << 3); // sb r1,(r0) (ra=1, rb=0)

        // sw ra,dd(rb): A6 pattern
        entries[0xA6] = (0x1C << 6) | 3; // sw r0,dd(fp) (ra=0, rb=3)

        // Four-byte instructions
        // la ra,addr24: 29-2F range
        for ra in 0..8u8 {
            entries[(0x28 + ra) as usize] = (0x0B << 6) | ((ra as u16) << 3);
        }

        Self { entries }
    }

    /// Decode an instruction byte
    pub fn decode(&self, byte: u8) -> u16 {
        self.entries[byte as usize]
    }

    /// Check if an instruction byte is valid
    pub fn is_valid(&self, byte: u8) -> bool {
        self.entries[byte as usize] != 0xFFF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_state_new() {
        let cpu = CpuState::new();
        assert_eq!(cpu.pc, RESET_ADDRESS);
        assert_eq!(cpu.registers[4], INITIAL_SP);
        assert!(!cpu.halted);
    }

    #[test]
    fn test_memory_operations() {
        let mut cpu = CpuState::new();

        cpu.write_byte(0x100, 0x42);
        assert_eq!(cpu.read_byte(0x100), 0x42);

        cpu.write_word(0x200, 0x123456);
        assert_eq!(cpu.read_word(0x200), 0x123456);
    }

    #[test]
    fn test_sign_extend() {
        assert_eq!(CpuState::sign_extend_8(0x7F), 0x00007F);
        assert_eq!(CpuState::sign_extend_8(0x80), 0xFFFF80);
        assert_eq!(CpuState::sign_extend_8(0xFF), 0xFFFFFF);
    }
}
