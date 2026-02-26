//! COR24 assembler
//!
//! Parses COR24 assembly language and produces machine code.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Assembly result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyResult {
    pub bytes: Vec<u8>,
    pub lines: Vec<AssembledLine>,
    pub errors: Vec<String>,
    pub labels: HashMap<String, u32>,
}

/// A single assembled line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledLine {
    pub address: u32,
    pub bytes: Vec<u8>,
    pub source: String,
    pub label: Option<String>,
}

/// COR24 Assembler
pub struct Assembler {
    /// Current address
    address: u32,
    /// Symbol table
    labels: HashMap<String, u32>,
    /// Forward references to resolve
    forward_refs: Vec<ForwardRef>,
    /// Output bytes
    output: Vec<u8>,
    /// Assembled lines
    lines: Vec<AssembledLine>,
    /// Errors
    errors: Vec<String>,
}

#[derive(Debug, Clone)]
struct ForwardRef {
    address: u32,
    label: String,
    ref_type: RefType,
    line_num: usize,
}

#[derive(Debug, Clone, Copy)]
enum RefType {
    Absolute24, // la instruction
    Relative8,  // branch instruction
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            address: 0,
            labels: HashMap::new(),
            forward_refs: Vec::new(),
            output: Vec::new(),
            lines: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Assemble source code
    pub fn assemble(&mut self, source: &str) -> AssemblyResult {
        self.address = 0;
        self.labels.clear();
        self.forward_refs.clear();
        self.output.clear();
        self.lines.clear();
        self.errors.clear();

        // First pass: collect labels and emit code
        for (line_num, line) in source.lines().enumerate() {
            self.assemble_line(line, line_num);
        }

        // Second pass: resolve forward references
        self.resolve_forward_refs();

        AssemblyResult {
            bytes: self.output.clone(),
            lines: self.lines.clone(),
            errors: self.errors.clone(),
            labels: self.labels.clone(),
        }
    }

    fn assemble_line(&mut self, line: &str, line_num: usize) {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            return;
        }

        let start_addr = self.address;
        let mut label = None;
        let mut instruction_part = line;

        // Check for label
        if let Some(colon_pos) = line.find(':') {
            let label_str = line[..colon_pos].trim();
            if !label_str.is_empty() {
                label = Some(label_str.to_string());
                self.labels.insert(label_str.to_string(), self.address);
            }
            instruction_part = line[colon_pos + 1..].trim();
        }

        // Handle directives
        if instruction_part.starts_with('.') {
            self.handle_directive(instruction_part, line_num);
            return;
        }

        // Skip if no instruction
        if instruction_part.is_empty() {
            if label.is_some() {
                self.lines.push(AssembledLine {
                    address: start_addr,
                    bytes: vec![],
                    source: line.to_string(),
                    label,
                });
            }
            return;
        }

        // Parse instruction
        let bytes = self.parse_instruction(instruction_part, line_num);

        self.lines.push(AssembledLine {
            address: start_addr,
            bytes: bytes.clone(),
            source: line.to_string(),
            label,
        });

        for b in bytes {
            self.output.push(b);
            self.address += 1;
        }
    }

    fn handle_directive(&mut self, directive: &str, _line_num: usize) {
        let parts: Vec<&str> = directive.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0].to_lowercase().as_str() {
            ".org" => {
                if parts.len() > 1
                    && let Some(addr) = self.parse_number(parts[1])
                {
                    // Pad output to reach the new address
                    while self.output.len() < addr as usize {
                        self.output.push(0);
                    }
                    self.address = addr;
                }
            }
            ".byte" | ".db" => {
                for part in &parts[1..] {
                    if let Some(val) = self.parse_number(part.trim_matches(',')) {
                        self.output.push(val as u8);
                        self.address += 1;
                    }
                }
            }
            ".word" | ".dw" => {
                for part in &parts[1..] {
                    if let Some(val) = self.parse_number(part.trim_matches(',')) {
                        // 24-bit word, little-endian
                        self.output.push((val & 0xFF) as u8);
                        self.output.push(((val >> 8) & 0xFF) as u8);
                        self.output.push(((val >> 16) & 0xFF) as u8);
                        self.address += 3;
                    }
                }
            }
            ".ascii" | ".asciz" => {
                // Extract string between quotes
                if let Some(start) = directive.find('"')
                    && let Some(end) = directive.rfind('"')
                    && end > start
                {
                    let s = &directive[start + 1..end];
                    for c in s.bytes() {
                        self.output.push(c);
                        self.address += 1;
                    }
                    if parts[0].to_lowercase() == ".asciz" {
                        self.output.push(0);
                        self.address += 1;
                    }
                }
            }
            _ => {}
        }
    }

    fn parse_instruction(&mut self, inst: &str, line_num: usize) -> Vec<u8> {
        let parts: Vec<&str> = inst.split_whitespace().collect();
        if parts.is_empty() {
            return vec![];
        }

        let mnemonic = parts[0].to_lowercase();
        let operands_str = if parts.len() > 1 {
            parts[1..].join(" ")
        } else {
            String::new()
        };
        let operands: Vec<&str> = if !operands_str.is_empty() {
            operands_str.split(',').map(|s| s.trim()).collect()
        } else {
            vec![]
        };

        match mnemonic.as_str() {
            // Stack operations
            "push" => self.encode_push(&operands, line_num),
            "pop" => self.encode_pop(&operands, line_num),

            // Move operations
            "mov" => self.encode_mov(&operands, line_num),

            // Arithmetic
            "add" => self.encode_add(&operands, line_num),
            "sub" => self.encode_sub(&operands, line_num),
            "mul" => self.encode_mul(&operands, line_num),

            // Logic
            "and" => self.encode_logic(&operands, 0x02, line_num),
            "or" => self.encode_logic(&operands, 0x13, line_num),
            "xor" => self.encode_logic(&operands, 0x1E, line_num),

            // Shifts
            "shl" => self.encode_shift(&operands, 0x17, line_num),
            "sra" => self.encode_shift(&operands, 0x18, line_num),
            "srl" => self.encode_shift(&operands, 0x19, line_num),

            // Compares
            "ceq" => self.encode_compare(&operands, "ceq", line_num),
            "cls" => self.encode_compare(&operands, "cls", line_num),
            "clu" => self.encode_compare(&operands, "clu", line_num),

            // Branches
            "bra" => self.encode_branch(&operands, 0x13, line_num),
            "brf" => self.encode_branch(&operands, 0x14, line_num),
            "brt" => self.encode_branch(&operands, 0x15, line_num),

            // Jumps
            "jmp" => self.encode_jmp(&operands, line_num),
            "jal" => self.encode_jal(&operands, line_num),

            // Load operations
            "la" => self.encode_la(&operands, line_num),
            "lc" => self.encode_lc(&operands, false, line_num),
            "lcu" => self.encode_lc(&operands, true, line_num),
            "lb" => self.encode_load_store(&operands, 0x0C, false, line_num),
            "lbu" => self.encode_load_store(&operands, 0x0D, false, line_num),
            "lw" => self.encode_load_store(&operands, 0x10, false, line_num),

            // Store operations
            "sb" => self.encode_load_store(&operands, 0x16, true, line_num),
            "sw" => self.encode_load_store(&operands, 0x1C, true, line_num),

            // Extensions
            "sxt" => self.encode_extension(&operands, 0xB0, line_num),
            "zxt" => self.encode_extension(&operands, 0xF0, line_num),

            // Pseudo-instructions
            "halt" => vec![0x00], // Jump to address 0 (infinite loop)
            "nop" => vec![0x00],  // add r0,r0

            _ => {
                self.errors.push(format!(
                    "Line {}: Unknown instruction '{}'",
                    line_num + 1,
                    mnemonic
                ));
                vec![]
            }
        }
    }

    fn parse_register(&self, s: &str) -> Option<u8> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "r0" => Some(0),
            "r1" => Some(1),
            "r2" => Some(2),
            "r3" | "fp" => Some(3),
            "r4" | "sp" => Some(4),
            "r5" | "z" => Some(5),
            "r6" | "iv" => Some(6),
            "r7" | "ir" => Some(7),
            "c" => Some(5), // Condition flag accessed as r5 in mov
            _ => None,
        }
    }

    fn parse_number(&self, s: &str) -> Option<u32> {
        let s = s.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            u32::from_str_radix(&s[2..], 16).ok()
        } else if s.starts_with('-') {
            s.parse::<i32>().ok().map(|v| v as u32)
        } else {
            s.parse::<u32>().ok()
        }
    }

    fn encode_push(&mut self, operands: &[&str], line_num: usize) -> Vec<u8> {
        if operands.is_empty() {
            self.errors
                .push(format!("Line {}: push requires operand", line_num + 1));
            return vec![];
        }

        if let Some(ra) = self.parse_register(operands[0]) {
            // push r0=7D, r1=7E, r2=7F, fp=80
            vec![0x7D + ra]
        } else {
            self.errors.push(format!(
                "Line {}: Invalid register '{}'",
                line_num + 1,
                operands[0]
            ));
            vec![]
        }
    }

    fn encode_pop(&mut self, operands: &[&str], line_num: usize) -> Vec<u8> {
        if operands.is_empty() {
            self.errors
                .push(format!("Line {}: pop requires operand", line_num + 1));
            return vec![];
        }

        if let Some(ra) = self.parse_register(operands[0]) {
            // pop r0=78, r1=7A, r2=7B, fp=7C (r0 uses different encoding)
            if ra == 0 { vec![0x78] } else { vec![0x79 + ra] }
        } else {
            self.errors.push(format!(
                "Line {}: Invalid register '{}'",
                line_num + 1,
                operands[0]
            ));
            vec![]
        }
    }

    fn encode_mov(&mut self, operands: &[&str], line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors
                .push(format!("Line {}: mov requires two operands", line_num + 1));
            return vec![];
        }

        let ra = self.parse_register(operands[0]);
        let rb = self.parse_register(operands[1]);

        match (ra, rb) {
            (Some(ra), Some(rb)) => {
                // Common mov encodings
                match (ra, rb) {
                    (3, 4) => vec![0x65], // mov fp,sp
                    (4, 3) => vec![0x69], // mov sp,fp
                    (0, 2) => vec![0x57], // mov r0,r2
                    (0, 5) => vec![0x62], // mov r0,c (c accessed as r5)
                    _ => {
                        // Generic encoding: 0x50 + (ra << 3) + rb (approximation)
                        vec![0x50 + (ra << 3) + rb]
                    }
                }
            }
            _ => {
                self.errors
                    .push(format!("Line {}: Invalid mov operands", line_num + 1));
                vec![]
            }
        }
    }

    fn encode_add(&mut self, operands: &[&str], line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors
                .push(format!("Line {}: add requires two operands", line_num + 1));
            return vec![];
        }

        let ra = self.parse_register(operands[0]);

        // Check if second operand is immediate
        if let Some(imm) = self.parse_number(operands[1]) {
            // add ra,imm
            if let Some(ra) = ra {
                vec![0x08 + ra, imm as u8]
            } else {
                self.errors
                    .push(format!("Line {}: Invalid register", line_num + 1));
                vec![]
            }
        } else if let Some(rb) = self.parse_register(operands[1]) {
            // add ra,rb
            if let Some(ra) = ra {
                if ra == 0 {
                    vec![rb] // add r0,rb encoded as just rb
                } else {
                    // Other add encodings
                    vec![(ra << 3) + rb]
                }
            } else {
                self.errors
                    .push(format!("Line {}: Invalid register", line_num + 1));
                vec![]
            }
        } else {
            self.errors
                .push(format!("Line {}: Invalid add operand", line_num + 1));
            vec![]
        }
    }

    fn encode_sub(&mut self, operands: &[&str], line_num: usize) -> Vec<u8> {
        // For now, simple sub ra,rb encoding
        if operands.len() < 2 {
            self.errors
                .push(format!("Line {}: sub requires two operands", line_num + 1));
            return vec![];
        }

        // Check for sub sp,imm24 pattern
        if operands[0].to_lowercase() == "sp"
            && let Some(imm) = self.parse_number(operands[1])
        {
            // sub sp,dddddd (4 bytes)
            return vec![
                0xD8, // sub sp encoding
                (imm & 0xFF) as u8,
                ((imm >> 8) & 0xFF) as u8,
                ((imm >> 16) & 0xFF) as u8,
            ];
        }

        self.errors
            .push(format!("Line {}: sub not fully implemented", line_num + 1));
        vec![]
    }

    fn encode_mul(&mut self, operands: &[&str], line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors
                .push(format!("Line {}: mul requires two operands", line_num + 1));
            return vec![];
        }
        // Placeholder
        vec![0x90]
    }

    fn encode_logic(&mut self, operands: &[&str], _base_opcode: u8, line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors.push(format!(
                "Line {}: logic op requires two operands",
                line_num + 1
            ));
            return vec![];
        }

        let ra = self.parse_register(operands[0]);
        let rb = self.parse_register(operands[1]);

        match (ra, rb) {
            (Some(0), Some(1)) => vec![0x0D], // and r0,r1
            (Some(0), Some(2)) => vec![0x0E], // and r0,r2 (approximate)
            _ => {
                self.errors.push(format!(
                    "Line {}: Logic operation encoding not found",
                    line_num + 1
                ));
                vec![]
            }
        }
    }

    fn encode_shift(&mut self, operands: &[&str], _opcode: u8, line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors.push(format!(
                "Line {}: shift requires two operands",
                line_num + 1
            ));
            return vec![];
        }
        // Placeholder
        vec![0xB0]
    }

    fn encode_compare(&mut self, operands: &[&str], cmp_type: &str, line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors.push(format!(
                "Line {}: compare requires two operands",
                line_num + 1
            ));
            return vec![];
        }

        let ra = self.parse_register(operands[0]);
        let rb = self.parse_register(operands[1]);

        match (ra, rb, cmp_type) {
            (Some(0), Some(1), "ceq") => vec![0x16],
            (Some(0), Some(2), "ceq") => vec![0x17],
            (Some(2), Some(0), "cls") => vec![0x1D],
            (Some(1), Some(0), "cls") => vec![0x1B],
            (Some(0), Some(5), "cls") => vec![0xCB], // cls r0,z
            (Some(5), Some(0), "clu") => vec![0xCE], // clu z,r0
            _ => {
                self.errors.push(format!(
                    "Line {}: Compare encoding not found for {} {:?},{:?}",
                    line_num + 1,
                    cmp_type,
                    ra,
                    rb
                ));
                vec![]
            }
        }
    }

    fn encode_branch(&mut self, operands: &[&str], opcode: u8, line_num: usize) -> Vec<u8> {
        if operands.is_empty() {
            self.errors
                .push(format!("Line {}: branch requires target", line_num + 1));
            return vec![];
        }

        let target = operands[0].trim();

        // Check if it's a label
        if let Some(&addr) = self.labels.get(target) {
            // Calculate relative offset
            let next_pc = self.address + 2;
            let offset = (addr as i32) - (next_pc as i32);
            if !(-128..=127).contains(&offset) {
                self.errors
                    .push(format!("Line {}: Branch target too far", line_num + 1));
                return vec![];
            }
            vec![opcode, offset as u8]
        } else if let Some(imm) = self.parse_number(target) {
            // Direct offset
            vec![opcode, imm as u8]
        } else {
            // Forward reference
            self.forward_refs.push(ForwardRef {
                address: self.address + 1,
                label: target.to_string(),
                ref_type: RefType::Relative8,
                line_num,
            });
            vec![opcode, 0x00] // Placeholder
        }
    }

    fn encode_jmp(&mut self, operands: &[&str], line_num: usize) -> Vec<u8> {
        if operands.is_empty() {
            self.errors
                .push(format!("Line {}: jmp requires target", line_num + 1));
            return vec![];
        }

        let target = operands[0].trim();

        // Check for (ra) syntax - indirect jump
        if target.starts_with('(') && target.ends_with(')') {
            let reg = &target[1..target.len() - 1];
            if let Some(ra) = self.parse_register(reg) {
                return match ra {
                    1 => vec![0x27], // jmp (r1)
                    7 => vec![0x2F], // jmp (ir)
                    _ => vec![0x20 + ra],
                };
            }
        }

        self.errors
            .push(format!("Line {}: Invalid jmp syntax", line_num + 1));
        vec![]
    }

    fn encode_jal(&mut self, operands: &[&str], line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors
                .push(format!("Line {}: jal requires ra,(rb)", line_num + 1));
            return vec![];
        }

        let ra = self.parse_register(operands[0]);
        let rb_str = operands[1].trim();

        // Parse (rb) syntax
        if rb_str.starts_with('(') && rb_str.ends_with(')') {
            let reg = &rb_str[1..rb_str.len() - 1];
            if let Some(rb) = self.parse_register(reg)
                && ra == Some(1)
                && rb == 0
            {
                return vec![0x25]; // jal r1,(r0)
            }
        }

        self.errors
            .push(format!("Line {}: Invalid jal syntax", line_num + 1));
        vec![]
    }

    fn encode_la(&mut self, operands: &[&str], line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors.push(format!(
                "Line {}: la requires register and address",
                line_num + 1
            ));
            return vec![];
        }

        let ra = self.parse_register(operands[0]);
        let target = operands[1].trim();

        if let Some(ra) = ra {
            let opcode = 0x28 + ra; // la r0=0x28, r1=0x29, r2=0x2A, etc.

            if let Some(addr) = self.parse_number(target) {
                // Immediate address
                return vec![
                    opcode,
                    (addr & 0xFF) as u8,
                    ((addr >> 8) & 0xFF) as u8,
                    ((addr >> 16) & 0xFF) as u8,
                ];
            } else if let Some(&addr) = self.labels.get(target) {
                // Known label
                return vec![
                    opcode,
                    (addr & 0xFF) as u8,
                    ((addr >> 8) & 0xFF) as u8,
                    ((addr >> 16) & 0xFF) as u8,
                ];
            } else {
                // Forward reference
                self.forward_refs.push(ForwardRef {
                    address: self.address + 1,
                    label: target.to_string(),
                    ref_type: RefType::Absolute24,
                    line_num,
                });
                return vec![opcode, 0x00, 0x00, 0x00];
            }
        }

        self.errors
            .push(format!("Line {}: Invalid la operand", line_num + 1));
        vec![]
    }

    fn encode_lc(&mut self, operands: &[&str], unsigned: bool, line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors.push(format!(
                "Line {}: lc requires register and constant",
                line_num + 1
            ));
            return vec![];
        }

        let ra = self.parse_register(operands[0]);
        let imm = self.parse_number(operands[1]);

        match (ra, imm) {
            (Some(ra), Some(imm)) => {
                // lc ra,dd: 0x44+ra for signed, 0x3C+ra for unsigned
                let opcode = if unsigned { 0x3C + ra } else { 0x44 + ra };
                vec![opcode, imm as u8]
            }
            _ => {
                self.errors
                    .push(format!("Line {}: Invalid lc operands", line_num + 1));
                vec![]
            }
        }
    }

    fn encode_load_store(
        &mut self,
        operands: &[&str],
        _opcode: u8,
        _is_store: bool,
        line_num: usize,
    ) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors.push(format!(
                "Line {}: load/store requires operands",
                line_num + 1
            ));
            return vec![];
        }

        let ra = self.parse_register(operands[0]);
        let addr_part = operands[1].trim();

        // Parse offset(rb) syntax
        if let Some(paren_pos) = addr_part.find('(') {
            let offset_str = &addr_part[..paren_pos];
            let rb_str = &addr_part[paren_pos + 1..].trim_end_matches(')');

            let offset = if offset_str.is_empty() {
                Some(0)
            } else {
                self.parse_number(offset_str)
            };
            let rb = self.parse_register(rb_str);

            if let (Some(ra), Some(rb), Some(offset)) = (ra, rb, offset) {
                // Simplified encoding - would need full ROM decode
                // lw r0,dd(fp) = 0x4D, lw r1,dd(fp) = 0x51, etc.
                if rb == 3 {
                    // fp
                    let base = 0x4D + (ra * 4);
                    return vec![base, offset as u8];
                }
                // Default encoding attempt
                return vec![0x4D, offset as u8];
            }
        }

        self.errors
            .push(format!("Line {}: Invalid load/store syntax", line_num + 1));
        vec![]
    }

    fn encode_extension(&mut self, operands: &[&str], base: u8, line_num: usize) -> Vec<u8> {
        if operands.len() < 2 {
            self.errors.push(format!(
                "Line {}: extension requires two operands",
                line_num + 1
            ));
            return vec![];
        }

        let ra = self.parse_register(operands[0]);
        let rb = self.parse_register(operands[1]);

        match (ra, rb) {
            (Some(ra), Some(rb)) => {
                vec![base + (ra << 3) + rb]
            }
            _ => {
                self.errors
                    .push(format!("Line {}: Invalid extension operands", line_num + 1));
                vec![]
            }
        }
    }

    fn resolve_forward_refs(&mut self) {
        for fref in &self.forward_refs {
            if let Some(&target_addr) = self.labels.get(&fref.label) {
                match fref.ref_type {
                    RefType::Absolute24 => {
                        let addr = fref.address as usize;
                        if addr + 2 < self.output.len() {
                            self.output[addr] = (target_addr & 0xFF) as u8;
                            self.output[addr + 1] = ((target_addr >> 8) & 0xFF) as u8;
                            self.output[addr + 2] = ((target_addr >> 16) & 0xFF) as u8;
                        }
                    }
                    RefType::Relative8 => {
                        let next_pc = fref.address + 1;
                        let offset = (target_addr as i32) - (next_pc as i32);
                        if (-128..=127).contains(&offset) {
                            let addr = fref.address as usize;
                            if addr < self.output.len() {
                                self.output[addr] = offset as u8;
                            }
                        } else {
                            self.errors.push(format!(
                                "Line {}: Branch target '{}' too far",
                                fref.line_num + 1,
                                fref.label
                            ));
                        }
                    }
                }
            } else {
                self.errors.push(format!(
                    "Line {}: Undefined label '{}'",
                    fref.line_num + 1,
                    fref.label
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assembly() {
        let mut asm = Assembler::new();
        let result = asm.assemble("lc r0,42");
        assert!(result.errors.is_empty());
        assert_eq!(result.bytes, vec![0x44, 42]);
    }

    #[test]
    fn test_push_pop() {
        let mut asm = Assembler::new();
        let result = asm.assemble("push r0\npop r1");
        assert!(result.errors.is_empty());
        assert_eq!(result.bytes, vec![0x7D, 0x7A]);
    }
}
