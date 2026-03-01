//! cor24-run: COR24 assembler and emulator CLI
//!
//! Usage:
//!   cor24-run --demo                              Run built-in LED demo
//!   cor24-run --demo --speed 50000 --time 10      Run at 50k IPS for 10 seconds
//!   cor24-run --run <file.s>                      Assemble and run
//!   cor24-run --assemble <in.s> <out.bin> <out.lst>  Assemble to binary + listing

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

/// Memory-mapped I/O address for LEDs
const IO_LEDSWDAT: u32 = 0xFF0000;

/// Default emulation speed (instructions per second)
/// Calibrated so spin loop of ~1000 iterations gives ~0.5s delay at default speed
const DEFAULT_SPEED: u64 = 100_000;

/// Default time limit in seconds
const DEFAULT_TIME_LIMIT: f64 = 10.0;

/// Minimal COR24 CPU state
struct Cpu {
    pc: u32,
    regs: [u32; 8],
    c: bool,
    mem: Vec<u8>,
    halted: bool,
    leds: u8,
    prev_leds: u8,
    cycles: u64,
}

impl Cpu {
    fn new() -> Self {
        let mut cpu = Self {
            pc: 0,
            regs: [0; 8],
            c: false,
            mem: vec![0; 65536],
            halted: false,
            leds: 0,
            prev_leds: 0xFF,
            cycles: 0,
        };
        // Initialize stack pointer to top of RAM
        cpu.regs[4] = 0xFE00; // sp = 0xFE00 (below I/O region)
        cpu
    }

    fn mask24(v: u32) -> u32 { v & 0xFFFFFF }

    fn sign_ext8(v: u8) -> u32 {
        if v & 0x80 != 0 { 0xFFFF00 | (v as u32) } else { v as u32 }
    }

    fn read_byte(&self, addr: u32) -> u8 {
        let addr = addr & 0xFFFFFF;
        if (addr & 0xFF0000) == 0xFF0000 { 0 }
        else { self.mem[(addr as usize) % self.mem.len()] }
    }

    fn write_byte(&mut self, addr: u32, val: u8) {
        let addr = addr & 0xFFFFFF;
        if addr == IO_LEDSWDAT {
            self.leds = val;
            if self.leds != self.prev_leds {
                print_leds(self.leds);
                self.prev_leds = self.leds;
            }
        } else if (addr & 0xFF0000) != 0xFF0000 {
            let len = self.mem.len();
            self.mem[(addr as usize) % len] = val;
        }
    }

    fn get_reg(&self, r: u8) -> u32 {
        if r == 5 { 0 } else { self.regs[(r & 7) as usize] & 0xFFFFFF }
    }

    fn set_reg(&mut self, r: u8, v: u32) {
        if r != 5 { self.regs[(r & 7) as usize] = v & 0xFFFFFF; }
    }

    fn load_program(&mut self, data: &[u8]) {
        for (i, &b) in data.iter().enumerate() {
            if i < self.mem.len() { self.mem[i] = b; }
        }
    }

    fn step(&mut self) -> bool {
        if self.halted { return false; }
        let b0 = self.read_byte(self.pc);
        self.cycles += 1;

        match b0 {
            // halt (la ir,0)
            0xC7 => {
                let addr = self.read_byte(self.pc+1) as u32
                    | ((self.read_byte(self.pc+2) as u32) << 8)
                    | ((self.read_byte(self.pc+3) as u32) << 16);
                if addr == 0 { self.halted = true; return false; }
                self.pc = addr;
            }
            // la ra,imm24
            0x29..=0x2F => {
                let ra = b0 - 0x29;
                let imm = self.read_byte(self.pc+1) as u32
                    | ((self.read_byte(self.pc+2) as u32) << 8)
                    | ((self.read_byte(self.pc+3) as u32) << 16);
                self.set_reg(ra, imm);
                self.pc = Self::mask24(self.pc + 4);
            }
            // lc ra,imm8
            0x44..=0x47 => {
                let ra = b0 - 0x44;
                let imm = self.read_byte(self.pc + 1);
                self.set_reg(ra, Self::sign_ext8(imm));
                self.pc = Self::mask24(self.pc + 2);
            }
            // lcu ra,imm8
            0x48..=0x4B => {
                let ra = b0 - 0x48;
                let imm = self.read_byte(self.pc + 1);
                self.set_reg(ra, imm as u32);
                self.pc = Self::mask24(self.pc + 2);
            }
            // add ra,rb (0x00-0x02 for r0, 0x08 for r1,r0, etc.)
            0x00..=0x02 => {
                let rb = b0;
                let v = Self::mask24(self.get_reg(0).wrapping_add(self.get_reg(rb)));
                self.set_reg(0, v);
                self.pc = Self::mask24(self.pc + 1);
            }
            // add r0,imm
            0x09 => {
                let imm = self.read_byte(self.pc + 1);
                let v = Self::mask24(self.get_reg(0).wrapping_add(Self::sign_ext8(imm)));
                self.set_reg(0, v);
                self.pc = Self::mask24(self.pc + 2);
            }
            // sub r0,rb (0x0A for sub r0,r1, 0x0B for sub r0,r2)
            0x0A..=0x0B => {
                let rb = b0 - 0x09; // 0x0A -> rb=1, 0x0B -> rb=2
                let ra_val = self.get_reg(0);
                let rb_val = self.get_reg(rb);
                let result = Self::mask24(ra_val.wrapping_sub(rb_val));
                self.set_reg(0, result);
                // Set condition flag: true if result > 0 (for brt to work as "branch if positive")
                self.c = result != 0 && (result & 0x800000) == 0;
                self.pc = Self::mask24(self.pc + 1);
            }
            // sub r2,r0 (0xA0 based on encoding)
            0xA0 => {
                let ra_val = self.get_reg(2);
                let rb_val = self.get_reg(0);
                let result = Self::mask24(ra_val.wrapping_sub(rb_val));
                self.set_reg(2, result);
                self.c = result != 0 && (result & 0x800000) == 0;
                self.pc = Self::mask24(self.pc + 1);
            }
            // add sp,imm
            0x21 => {
                let imm = self.read_byte(self.pc + 1);
                let v = Self::mask24(self.get_reg(4).wrapping_add(Self::sign_ext8(imm)));
                self.set_reg(4, v);
                self.pc = Self::mask24(self.pc + 2);
            }
            // mov ra,rb (0x30-0x42) and mov ra,c (0x34,0x3C,0x43)
            0x30..=0x32 => { self.set_reg(0, self.get_reg(b0 - 0x30)); self.pc += 1; }
            0x34 => { self.set_reg(0, if self.c {1} else {0}); self.pc += 1; }
            0x38..=0x3A => { self.set_reg(1, self.get_reg(b0 - 0x38)); self.pc += 1; }
            0x3C => { self.set_reg(1, if self.c {1} else {0}); self.pc += 1; }
            0x40..=0x42 => { self.set_reg(2, self.get_reg(b0 - 0x40)); self.pc += 1; }
            0x43 => { self.set_reg(2, if self.c {1} else {0}); self.pc += 1; }
            // mov fp,sp
            0x4C => { self.set_reg(3, self.get_reg(4)); self.pc += 1; }
            // mov sp,fp
            0x53 => { self.set_reg(4, self.get_reg(3)); self.pc += 1; }
            // and r0,rb
            0x03..=0x05 => {
                let rb = b0 - 0x03;
                self.set_reg(0, self.get_reg(0) & self.get_reg(rb));
                self.pc += 1;
            }
            // sb ra,imm(rb) - store byte
            0x80..=0x89 => {
                let idx = b0 - 0x80;
                let ra = idx / 8;
                let rb = idx % 8;
                let imm = self.read_byte(self.pc + 1);
                let addr = Self::mask24(self.get_reg(rb).wrapping_add(Self::sign_ext8(imm)));
                self.write_byte(addr, self.get_reg(ra) as u8);
                self.pc = Self::mask24(self.pc + 2);
            }
            // sw ra,imm(fp) - store word to stack frame
            0x8A..=0x8C => {
                let ra = b0 - 0x8A;
                let imm = self.read_byte(self.pc + 1);
                let addr = Self::mask24(self.get_reg(3).wrapping_add(Self::sign_ext8(imm)));
                let v = self.get_reg(ra);
                self.write_byte(addr, (v & 0xFF) as u8);
                self.write_byte(addr + 1, ((v >> 8) & 0xFF) as u8);
                self.write_byte(addr + 2, ((v >> 16) & 0xFF) as u8);
                self.pc = Self::mask24(self.pc + 2);
            }
            // lw ra,imm(fp) - load word from stack frame
            0x92..=0x94 => {
                let ra = b0 - 0x92;
                let imm = self.read_byte(self.pc + 1);
                let addr = Self::mask24(self.get_reg(3).wrapping_add(Self::sign_ext8(imm)));
                let b0 = self.read_byte(addr) as u32;
                let b1 = self.read_byte(addr + 1) as u32;
                let b2 = self.read_byte(addr + 2) as u32;
                self.set_reg(ra, b0 | (b1 << 8) | (b2 << 16));
                self.pc = Self::mask24(self.pc + 2);
            }
            // ceq ra,z
            0x15 => { self.c = self.get_reg(0) == 0; self.pc += 1; }
            0x16 => { self.c = self.get_reg(0) == self.get_reg(1); self.pc += 1; }
            // clu ra,rb
            0x1E..=0x20 => {
                let rb = b0 - 0x1E;
                self.c = self.get_reg(0) < self.get_reg(rb);
                self.pc += 1;
            }
            // bra
            0x13 => {
                let imm = self.read_byte(self.pc + 1);
                let next = Self::mask24(self.pc + 2);
                self.pc = Self::mask24(next.wrapping_add(Self::sign_ext8(imm)));
            }
            // brf
            0x14 => {
                let imm = self.read_byte(self.pc + 1);
                let next = Self::mask24(self.pc + 2);
                if !self.c { self.pc = Self::mask24(next.wrapping_add(Self::sign_ext8(imm))); }
                else { self.pc = next; }
            }
            // brt
            0x12 => {
                let imm = self.read_byte(self.pc + 1);
                let next = Self::mask24(self.pc + 2);
                if self.c { self.pc = Self::mask24(next.wrapping_add(Self::sign_ext8(imm))); }
                else { self.pc = next; }
            }
            // push ra (0x64-0x6F)
            0x64..=0x6F => {
                let ra = (b0 - 0x64) / 2;
                let sp = self.get_reg(4).wrapping_sub(3);
                self.set_reg(4, sp);
                let v = self.get_reg(ra);
                self.write_byte(sp, (v & 0xFF) as u8);
                self.write_byte(sp+1, ((v>>8) & 0xFF) as u8);
                self.write_byte(sp+2, ((v>>16) & 0xFF) as u8);
                self.pc += 1;
            }
            // pop ra (0x70-0x7B)
            0x70..=0x7B => {
                let ra = (b0 - 0x70) / 2;
                let sp = self.get_reg(4);
                let v = self.read_byte(sp) as u32
                    | ((self.read_byte(sp+1) as u32) << 8)
                    | ((self.read_byte(sp+2) as u32) << 16);
                self.set_reg(ra, v);
                self.set_reg(4, sp.wrapping_add(3));
                self.pc += 1;
            }
            _ => {
                eprintln!("Unknown opcode 0x{:02X} at PC=0x{:04X}", b0, self.pc);
                self.halted = true;
                return false;
            }
        }
        true
    }
}

fn print_leds(leds: u8) {
    print!("\rLEDs: ");
    for i in (0..8).rev() {
        if (leds >> i) & 1 == 1 { print!("\x1b[91m●\x1b[0m"); }
        else { print!("○"); }
    }
    print!("  (0x{:02X})  ", leds);
    std::io::stdout().flush().ok();
}

/// Run CPU with timing control
/// - speed: instructions per second (0 = unlimited)
/// - time_limit: maximum wall-clock time in seconds
fn run_with_timing(cpu: &mut Cpu, speed: u64, time_limit: f64) -> u64 {
    let start = Instant::now();
    let time_limit_duration = Duration::from_secs_f64(time_limit);

    // Calculate batch size and sleep interval
    // Execute in batches to reduce overhead
    let batch_size: u64 = if speed == 0 { 10000 } else { (speed / 100).max(1) };
    let batch_duration = if speed == 0 {
        Duration::ZERO
    } else {
        Duration::from_secs_f64(batch_size as f64 / speed as f64)
    };

    let mut total_instructions: u64 = 0;
    let mut batch_start = Instant::now();

    loop {
        // Check time limit
        if start.elapsed() >= time_limit_duration {
            break;
        }

        // Execute a batch of instructions
        for _ in 0..batch_size {
            if !cpu.step() {
                return total_instructions;
            }
            total_instructions += 1;
        }

        // Pace execution if speed is set
        if speed > 0 {
            let elapsed = batch_start.elapsed();
            if elapsed < batch_duration {
                thread::sleep(batch_duration - elapsed);
            }
            batch_start = Instant::now();
        }
    }

    total_instructions
}

// =============================================================================
// Assembler
// =============================================================================

struct Assembler {
    labels: HashMap<String, u32>,
    output: Vec<u8>,
    listing: Vec<String>,
}

impl Assembler {
    fn new() -> Self {
        Self { labels: HashMap::new(), output: Vec::new(), listing: Vec::new() }
    }

    fn assemble(&mut self, source: &str) -> Result<(), String> {
        // Pass 1: collect labels
        let mut addr = 0u32;
        for line in source.lines() {
            let line = line.split(';').next().unwrap_or("").trim();
            if line.is_empty() { continue; }
            if let Some(label) = line.strip_suffix(':') {
                self.labels.insert(label.trim().to_string(), addr);
                continue;
            }
            addr += self.estimate_size(line);
        }

        // Pass 2: generate code
        for line in source.lines() {
            let orig = line;
            let line = line.split(';').next().unwrap_or("").trim();
            if line.is_empty() {
                self.listing.push(format!("                    {}", orig));
                continue;
            }
            if line.ends_with(':') {
                self.listing.push(format!("{:04X}:               {}", self.output.len(), orig));
                continue;
            }
            let start = self.output.len();
            self.emit(line)?;
            let bytes: Vec<String> = self.output[start..].iter().map(|b| format!("{:02X}", b)).collect();
            self.listing.push(format!("{:04X}: {:14} {}", start, bytes.join(" "), orig));
        }
        Ok(())
    }

    fn estimate_size(&self, line: &str) -> u32 {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { return 0; }
        match parts[0].to_lowercase().as_str() {
            "la" | "halt" => 4,
            "push" | "pop" | "mov" | "and" | "ceq" | "clu" | "cls" | "sub" => 1,
            _ => 2,
        }
    }

    fn emit(&mut self, line: &str) -> Result<(), String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { return Ok(()); }
        let mnemonic = parts[0].to_lowercase();
        let operand_str = if parts.len() > 1 { parts[1..].join(" ") } else { String::new() };
        let operands: Vec<&str> = operand_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

        match mnemonic.as_str() {
            "la" => {
                let ra = self.reg(&operands[0])?;
                let imm = self.imm24(&operands[1])?;
                self.output.push(0x29 + ra);
                self.output.push((imm & 0xFF) as u8);
                self.output.push(((imm >> 8) & 0xFF) as u8);
                self.output.push(((imm >> 16) & 0xFF) as u8);
            }
            "lc" => {
                let ra = self.reg(&operands[0])?;
                let imm = self.imm8(&operands[1])?;
                self.output.push(0x44 + ra);
                self.output.push(imm as u8);
            }
            "lcu" => {
                let ra = self.reg(&operands[0])?;
                let imm = self.imm8(&operands[1])?;
                self.output.push(0x48 + ra);
                self.output.push(imm as u8);
            }
            "add" => {
                let ra = self.reg(&operands[0])?;
                let op2 = operands[1].trim();
                let is_reg = op2.starts_with("r") || op2 == "sp" || op2 == "fp" || op2 == "z" || op2 == "iv" || op2 == "ir";
                if is_reg && !op2.starts_with("-") && !op2.chars().next().unwrap_or('x').is_ascii_digit() {
                    let rb = self.reg(op2)?;
                    self.output.push(ra * 8 + rb);
                } else if ra == 4 {
                    let imm = self.imm8(op2)?;
                    self.output.push(0x21);
                    self.output.push(imm as u8);
                } else {
                    let imm = self.imm8(op2)?;
                    self.output.push(0x09 + ra * 8);
                    self.output.push(imm as u8);
                }
            }
            "sub" => {
                let ra = self.reg(&operands[0])?;
                let rb = self.reg(&operands[1])?;
                // sub r0,r1 = 0x0A, sub r0,r2 = 0x0B, sub r2,r0 = 0xA0
                if ra == 0 && rb == 1 {
                    self.output.push(0x0A);
                } else if ra == 0 && rb == 2 {
                    self.output.push(0x0B);
                } else if ra == 2 && rb == 0 {
                    self.output.push(0xA0);
                } else {
                    return Err(format!("Unsupported sub encoding: r{},r{}", ra, rb));
                }
            }
            "mov" => {
                let ra = self.reg(&operands[0])?;
                let op2 = operands[1].trim();
                if op2 == "c" {
                    self.output.push(0x34 + ra * 8);
                } else if op2 == "sp" && ra == 3 {
                    self.output.push(0x4C);
                } else if op2 == "fp" && ra == 4 {
                    self.output.push(0x53);
                } else {
                    let rb = self.reg(op2)?;
                    self.output.push(0x30 + ra * 8 + rb);
                }
            }
            "and" => {
                let ra = self.reg(&operands[0])?;
                let rb = self.reg(&operands[1])?;
                self.output.push(0x03 + ra * 8 + rb);
            }
            "sb" => {
                let ra = self.reg(&operands[0])?;
                let (imm, rb) = self.mem(&operands[1])?;
                self.output.push(0x80 + ra * 8 + rb);
                self.output.push(imm as u8);
            }
            "sw" => {
                let ra = self.reg(&operands[0])?;
                let (imm, rb) = self.mem(&operands[1])?;
                if rb == 3 {
                    self.output.push(0x8A + ra);
                } else {
                    self.output.push(0x8D + ra * 8 + rb);
                }
                self.output.push(imm as u8);
            }
            "lw" => {
                let ra = self.reg(&operands[0])?;
                let (imm, rb) = self.mem(&operands[1])?;
                if rb == 3 {
                    self.output.push(0x92 + ra);
                } else {
                    self.output.push(0x95 + ra * 8 + rb);
                }
                self.output.push(imm as u8);
            }
            "ceq" => {
                let ra = self.reg(&operands[0])?;
                let rb = self.reg(&operands[1])?;
                if rb == 5 {
                    self.output.push(0x15 + ra);
                } else {
                    self.output.push(0x15 + ra + rb);
                }
            }
            "clu" => {
                let ra = self.reg(&operands[0])?;
                let rb = self.reg(&operands[1])?;
                self.output.push(0x1E + ra * 3 + rb);
            }
            "bra" => {
                self.output.push(0x13);
                let off = self.branch(&operands[0])?;
                self.output.push(off as u8);
            }
            "brf" => {
                self.output.push(0x14);
                let off = self.branch(&operands[0])?;
                self.output.push(off as u8);
            }
            "brt" => {
                self.output.push(0x12);
                let off = self.branch(&operands[0])?;
                self.output.push(off as u8);
            }
            "push" => {
                let ra = self.reg(&operands[0])?;
                self.output.push(0x64 + ra * 2);
            }
            "pop" => {
                let ra = self.reg(&operands[0])?;
                self.output.push(0x70 + ra * 2);
            }
            "halt" => {
                self.output.extend_from_slice(&[0xC7, 0, 0, 0]);
            }
            _ => return Err(format!("Unknown: {}", mnemonic)),
        }
        Ok(())
    }

    fn reg(&self, s: &str) -> Result<u8, String> {
        let s = s.trim();
        match s.to_lowercase().as_str() {
            "r0" => Ok(0), "r1" => Ok(1), "r2" => Ok(2),
            "r3"|"fp" => Ok(3), "r4"|"sp" => Ok(4), "r5"|"z" => Ok(5),
            "r6"|"iv" => Ok(6), "r7"|"ir" => Ok(7),
            _ => Err(format!("Bad reg: '{}'", s))
        }
    }

    fn imm8(&self, s: &str) -> Result<i32, String> {
        let s = s.trim();
        if let Some(h) = s.strip_prefix("0x") { i32::from_str_radix(h, 16).map_err(|e| e.to_string()) }
        else { s.parse().map_err(|e: std::num::ParseIntError| e.to_string()) }
    }

    fn imm24(&self, s: &str) -> Result<u32, String> {
        let s = s.trim();
        if let Some(h) = s.strip_prefix("0x") { u32::from_str_radix(h, 16).map_err(|e| e.to_string()) }
        else { s.parse::<i64>().map(|v| (v as u32) & 0xFFFFFF).map_err(|e| e.to_string()) }
    }

    fn mem(&self, s: &str) -> Result<(i32, u8), String> {
        if let Some(p) = s.find('(') {
            let off = if p == 0 { 0 } else { self.imm8(&s[..p])? };
            let reg = s[p+1..].trim_end_matches(')');
            Ok((off, self.reg(reg)?))
        } else { Err(format!("Bad mem: {}", s)) }
    }

    fn branch(&mut self, target: &str) -> Result<i32, String> {
        let cur = self.output.len() as i32 + 1;
        if let Some(&addr) = self.labels.get(target.trim()) {
            Ok((addr as i32) - cur)
        } else { Err(format!("Undefined: {}", target)) }
    }
}

// =============================================================================
// Demo program with spin loop delay
// =============================================================================

/// LED counter demo with spin loop delay
/// Counts 0-255 on LEDs with configurable delay between updates
const DEMO_SOURCE: &str = r#"
; LED Counter Demo with Spin Loop Delay
; Counts 0-255 on LEDs, loops forever

        push    fp
        mov     fp, sp
        add     sp, -3          ; allocate 1 word for counter

        la      r1, 0xFF0000    ; LED I/O address
        lc      r0, 0
        sw      r0, 0(fp)       ; counter = 0

main_loop:
        ; Write counter to LEDs
        lw      r0, 0(fp)       ; load counter
        sb      r0, 0(r1)       ; write to LEDs

        ; Spin loop delay
        ; ~15000 iterations, 3 instructions each = 45k instructions
        ; At 100k IPS default speed: ~0.45 seconds per LED change
        la      r2, 15000
delay:
        lc      r0, 1
        sub     r2, r0
        brt     delay

        ; Increment counter (wraps at 256)
        lw      r0, 0(fp)
        lc      r2, 1
        add     r0, r2
        sw      r0, 0(fp)

        bra     main_loop
"#;

// =============================================================================
// Main
// =============================================================================

fn parse_args() -> (String, u64, f64, Option<String>) {
    let args: Vec<String> = env::args().collect();
    let mut command = String::new();
    let mut speed = DEFAULT_SPEED;
    let mut time_limit = DEFAULT_TIME_LIMIT;
    let mut file = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--demo" => command = "demo".to_string(),
            "--run" => {
                command = "run".to_string();
                if i + 1 < args.len() {
                    file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--assemble" => {
                command = "assemble".to_string();
                // Collect remaining args for assemble
            }
            "--speed" | "-s" => {
                if i + 1 < args.len() {
                    speed = args[i + 1].parse().unwrap_or(DEFAULT_SPEED);
                    i += 1;
                }
            }
            "--time" | "-t" => {
                if i + 1 < args.len() {
                    time_limit = args[i + 1].parse().unwrap_or(DEFAULT_TIME_LIMIT);
                    i += 1;
                }
            }
            _ => {
                if command.is_empty() && !args[i].starts_with('-') {
                    file = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    (command, speed, time_limit, file)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("cor24-run: COR24 assembler and emulator\n");
        println!("Usage:");
        println!("  cor24-run --demo [options]        Run built-in LED demo");
        println!("  cor24-run --run <file.s> [opts]   Assemble and run");
        println!("  cor24-run --assemble <in.s> <out.bin> <out.lst>");
        println!();
        println!("Options:");
        println!("  --speed, -s <ips>    Instructions per second (default: {})", DEFAULT_SPEED);
        println!("  --time, -t <secs>    Time limit in seconds (default: {})", DEFAULT_TIME_LIMIT);
        println!();
        println!("Example:");
        println!("  cor24-run --demo --speed 100000 --time 10");
        return;
    }

    let (command, speed, time_limit, file) = parse_args();

    match command.as_str() {
        "demo" => {
            println!("=== COR24 LED Demo ===\n");
            println!("Binary counter 0-255 on LEDs with spin loop delay");
            println!("Speed: {} instructions/sec, Time limit: {}s\n", speed, time_limit);

            // Assemble the demo source
            let mut asm = Assembler::new();
            if let Err(e) = asm.assemble(DEMO_SOURCE) {
                eprintln!("Assembly error: {}", e);
                return;
            }

            println!("Assembled {} bytes\n", asm.output.len());

            // Show listing
            println!("Program listing:");
            for line in &asm.listing {
                if !line.trim().is_empty() {
                    println!("{}", line);
                }
            }
            println!();

            // Run with timing
            println!("Running (Ctrl+C to stop)...\n");
            let mut cpu = Cpu::new();
            cpu.load_program(&asm.output);

            let instructions = run_with_timing(&mut cpu, speed, time_limit);

            println!("\n\nExecuted {} instructions in {:.1}s", instructions, time_limit);
            println!("Effective speed: {:.0} IPS", instructions as f64 / time_limit);
        }

        "run" => {
            let filename = match file {
                Some(f) => f,
                None => {
                    eprintln!("Usage: cor24-run --run <file.s>");
                    return;
                }
            };

            let source = fs::read_to_string(&filename).expect("Cannot read file");
            let mut asm = Assembler::new();
            if let Err(e) = asm.assemble(&source) {
                eprintln!("Assembly error: {}", e);
                return;
            }

            println!("Assembled {} bytes\n", asm.output.len());
            println!("Listing:");
            for line in &asm.listing { println!("{}", line); }
            println!("\nRunning (speed: {} IPS, time limit: {}s)...\n", speed, time_limit);

            let mut cpu = Cpu::new();
            cpu.load_program(&asm.output);

            let instructions = run_with_timing(&mut cpu, speed, time_limit);

            println!("\n\nExecuted {} instructions", instructions);
        }

        "assemble" => {
            if args.len() < 5 {
                eprintln!("Usage: cor24-run --assemble <in.s> <out.bin> <out.lst>");
                return;
            }
            let source = fs::read_to_string(&args[2]).expect("Cannot read file");
            let mut asm = Assembler::new();
            if let Err(e) = asm.assemble(&source) {
                eprintln!("Assembly error: {}", e);
                return;
            }
            fs::write(&args[3], &asm.output).expect("Cannot write .bin");
            let mut lst_file = fs::File::create(&args[4]).expect("Cannot write .lst");
            for line in &asm.listing {
                writeln!(lst_file, "{}", line).ok();
            }
            println!("Wrote {} bytes to {}", asm.output.len(), args[3]);
            println!("Wrote listing to {}", args[4]);
        }

        _ => {
            eprintln!("Unknown command. Use --demo, --run, or --assemble");
        }
    }
}
