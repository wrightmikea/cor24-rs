//! COR24 instruction execution

use super::instruction::{DecodedInstruction, InstructionFormat, Opcode};
use super::state::{CpuState, DecodeRom};

/// Execute result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteResult {
    /// Instruction executed successfully
    Ok,
    /// CPU halted
    Halted,
    /// Invalid instruction
    InvalidInstruction(u8),
    /// Memory access error
    MemoryError(u32),
}

/// COR24 instruction executor
#[derive(Clone)]
pub struct Executor {
    decode_rom: DecodeRom,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            decode_rom: DecodeRom::new(),
        }
    }

    /// Execute a single instruction
    pub fn step(&self, cpu: &mut CpuState) -> ExecuteResult {
        if cpu.halted {
            return ExecuteResult::Halted;
        }

        // Fetch instruction byte
        let inst_byte = cpu.read_byte(cpu.pc);

        // Check for HALT (using 0x00 at address 0 as halt)
        if inst_byte == 0x00 && cpu.pc == 0 {
            cpu.halted = true;
            return ExecuteResult::Halted;
        }

        // Decode instruction
        let decoded_value = self.decode_rom.decode(inst_byte);
        if decoded_value == 0xFFF {
            return ExecuteResult::InvalidInstruction(inst_byte);
        }

        let inst = DecodedInstruction::from_decoded(decoded_value);
        let format = inst.opcode.format();

        // Fetch additional bytes based on format
        let imm8 = match format {
            InstructionFormat::TwoBytes | InstructionFormat::FourBytes => {
                cpu.read_byte(cpu.pc.wrapping_add(1))
            }
            _ => 0,
        };

        let imm24 = match format {
            InstructionFormat::FourBytes => {
                let b0 = cpu.read_byte(cpu.pc.wrapping_add(1)) as u32;
                let b1 = cpu.read_byte(cpu.pc.wrapping_add(2)) as u32;
                let b2 = cpu.read_byte(cpu.pc.wrapping_add(3)) as u32;
                b0 | (b1 << 8) | (b2 << 16)
            }
            _ => 0,
        };

        // Calculate next PC
        let inst_size = match format {
            InstructionFormat::SingleByte => 1,
            InstructionFormat::TwoBytes => 2,
            InstructionFormat::FourBytes => 4,
        };
        let next_pc = CpuState::mask_24(cpu.pc.wrapping_add(inst_size));

        // Execute instruction
        match inst.opcode {
            Opcode::AddReg => {
                // add ra,rb
                let ra_val = cpu.get_reg(inst.ra);
                let rb_val = cpu.get_reg(inst.rb);
                let result = CpuState::mask_24(ra_val.wrapping_add(rb_val));
                cpu.set_reg(inst.ra, result);
                cpu.pc = next_pc;
            }

            Opcode::AddImm => {
                // add ra,dd
                let ra_val = cpu.get_reg(inst.ra);
                let imm = CpuState::sign_extend_8(imm8);
                let result = CpuState::mask_24(ra_val.wrapping_add(imm));
                cpu.set_reg(inst.ra, result);
                cpu.pc = next_pc;
            }

            Opcode::And => {
                // and ra,rb
                let ra_val = cpu.get_reg(inst.ra);
                let rb_val = cpu.get_reg(inst.rb);
                cpu.set_reg(inst.ra, ra_val & rb_val);
                cpu.pc = next_pc;
            }

            Opcode::Bra => {
                // bra dd (always branch)
                let offset = CpuState::sign_extend_8(imm8);
                cpu.pc = CpuState::mask_24(next_pc.wrapping_add(offset));
            }

            Opcode::Brf => {
                // brf dd (branch if false)
                if !cpu.c {
                    let offset = CpuState::sign_extend_8(imm8);
                    cpu.pc = CpuState::mask_24(next_pc.wrapping_add(offset));
                } else {
                    cpu.pc = next_pc;
                }
            }

            Opcode::Brt => {
                // brt dd (branch if true)
                if cpu.c {
                    let offset = CpuState::sign_extend_8(imm8);
                    cpu.pc = CpuState::mask_24(next_pc.wrapping_add(offset));
                } else {
                    cpu.pc = next_pc;
                }
            }

            Opcode::Ceq => {
                // ceq ra,rb (compare equal)
                let ra_val = cpu.get_reg(inst.ra);
                let rb_val = cpu.get_reg(inst.rb);
                cpu.c = ra_val == rb_val;
                cpu.pc = next_pc;
            }

            Opcode::Cls => {
                // cls ra,rb (compare less signed)
                let ra_val = cpu.get_reg(inst.ra) as i32;
                let rb_val = cpu.get_reg(inst.rb) as i32;
                // Sign extend from 24-bit
                let ra_signed = if ra_val & 0x800000 != 0 {
                    ra_val | 0xFF000000u32 as i32
                } else {
                    ra_val
                };
                let rb_signed = if rb_val & 0x800000 != 0 {
                    rb_val | 0xFF000000u32 as i32
                } else {
                    rb_val
                };
                cpu.c = ra_signed < rb_signed;
                cpu.pc = next_pc;
            }

            Opcode::Clu => {
                // clu ra,rb (compare less unsigned)
                let ra_val = cpu.get_reg(inst.ra);
                let rb_val = cpu.get_reg(inst.rb);
                cpu.c = ra_val < rb_val;
                cpu.pc = next_pc;
            }

            Opcode::Jal => {
                // jal ra,(rb) - jump and link
                let target = cpu.get_reg(inst.rb);
                cpu.set_reg(inst.ra, next_pc);
                cpu.pc = target;
            }

            Opcode::Jmp => {
                // jmp (ra)
                let target = cpu.get_reg(inst.ra);
                cpu.pc = target;
            }

            Opcode::La => {
                // la ra,dddddd
                if inst.ra == 7 {
                    // jmp dddddd (absolute jump)
                    cpu.pc = imm24;
                } else {
                    cpu.set_reg(inst.ra, imm24);
                    cpu.pc = next_pc;
                }
            }

            Opcode::Lb => {
                // lb ra,dd(rb) - load byte signed
                let base = cpu.get_reg(inst.rb);
                let offset = CpuState::sign_extend_8(imm8);
                let addr = CpuState::mask_24(base.wrapping_add(offset));
                let value = cpu.read_byte(addr);
                cpu.set_reg(inst.ra, CpuState::sign_extend_8(value));
                cpu.pc = next_pc;
            }

            Opcode::Lbu => {
                // lbu ra,dd(rb) - load byte unsigned
                let base = cpu.get_reg(inst.rb);
                let offset = CpuState::sign_extend_8(imm8);
                let addr = CpuState::mask_24(base.wrapping_add(offset));
                let value = cpu.read_byte(addr);
                cpu.set_reg(inst.ra, value as u32);
                cpu.pc = next_pc;
            }

            Opcode::Lc => {
                // lc ra,dd - load constant signed
                cpu.set_reg(inst.ra, CpuState::sign_extend_8(imm8));
                cpu.pc = next_pc;
            }

            Opcode::Lcu => {
                // lcu ra,dd - load constant unsigned
                cpu.set_reg(inst.ra, imm8 as u32);
                cpu.pc = next_pc;
            }

            Opcode::Lw => {
                // lw ra,dd(rb) - load word
                let base = cpu.get_reg(inst.rb);
                let offset = CpuState::sign_extend_8(imm8);
                let addr = CpuState::mask_24(base.wrapping_add(offset));
                let value = cpu.read_word(addr);
                cpu.set_reg(inst.ra, value);
                cpu.pc = next_pc;
            }

            Opcode::Mov => {
                // mov ra,rb or mov ra,c
                if inst.rb == 5 {
                    // mov ra,c
                    cpu.set_reg(inst.ra, if cpu.c { 1 } else { 0 });
                } else {
                    cpu.set_reg(inst.ra, cpu.get_reg(inst.rb));
                }
                cpu.pc = next_pc;
            }

            Opcode::Mul => {
                // mul ra,rb
                let ra_val = cpu.get_reg(inst.ra);
                let rb_val = cpu.get_reg(inst.rb);
                let result = CpuState::mask_24(ra_val.wrapping_mul(rb_val));
                cpu.set_reg(inst.ra, result);
                cpu.pc = next_pc;
            }

            Opcode::Or => {
                // or ra,rb
                let ra_val = cpu.get_reg(inst.ra);
                let rb_val = cpu.get_reg(inst.rb);
                cpu.set_reg(inst.ra, ra_val | rb_val);
                cpu.pc = next_pc;
            }

            Opcode::Pop => {
                // pop ra - load from stack, increment sp
                let sp = cpu.get_reg(4);
                let value = cpu.read_word(sp);
                cpu.set_reg(inst.ra, value);
                cpu.set_reg(4, CpuState::mask_24(sp.wrapping_add(3)));
                cpu.pc = next_pc;
            }

            Opcode::Push => {
                // push ra - decrement sp, store to stack
                let sp = cpu.get_reg(4);
                let new_sp = CpuState::mask_24(sp.wrapping_sub(3));
                cpu.set_reg(4, new_sp);
                let value = cpu.get_reg(inst.ra);
                cpu.write_word(new_sp, value);
                cpu.pc = next_pc;
            }

            Opcode::Sb => {
                // sb ra,dd(rb) - store byte
                let base = cpu.get_reg(inst.rb);
                let offset = CpuState::sign_extend_8(imm8);
                let addr = CpuState::mask_24(base.wrapping_add(offset));
                let value = cpu.get_reg(inst.ra) as u8;
                cpu.write_byte(addr, value);
                cpu.pc = next_pc;
            }

            Opcode::Shl => {
                // shl ra,rb - shift left
                let ra_val = cpu.get_reg(inst.ra);
                let shift = cpu.get_reg(inst.rb) & 0x1F;
                let result = CpuState::mask_24(ra_val << shift);
                cpu.set_reg(inst.ra, result);
                cpu.pc = next_pc;
            }

            Opcode::Sra => {
                // sra ra,rb - shift right arithmetic
                let ra_val = cpu.get_reg(inst.ra);
                let shift = cpu.get_reg(inst.rb) & 0x1F;
                // Sign extend for arithmetic shift
                let signed_val = if ra_val & 0x800000 != 0 {
                    ra_val | 0xFF000000
                } else {
                    ra_val
                };
                let result = CpuState::mask_24((signed_val as i32 >> shift) as u32);
                cpu.set_reg(inst.ra, result);
                cpu.pc = next_pc;
            }

            Opcode::Srl => {
                // srl ra,rb - shift right logical
                let ra_val = cpu.get_reg(inst.ra);
                let shift = cpu.get_reg(inst.rb) & 0x1F;
                let result = ra_val >> shift;
                cpu.set_reg(inst.ra, result);
                cpu.pc = next_pc;
            }

            Opcode::Sub => {
                // sub ra,rb
                let ra_val = cpu.get_reg(inst.ra);
                let rb_val = cpu.get_reg(inst.rb);
                let result = CpuState::mask_24(ra_val.wrapping_sub(rb_val));
                cpu.set_reg(inst.ra, result);
                cpu.pc = next_pc;
            }

            Opcode::SubSp => {
                // sub sp,dddddd
                let sp = cpu.get_reg(4);
                let result = CpuState::mask_24(sp.wrapping_sub(imm24));
                cpu.set_reg(4, result);
                cpu.pc = next_pc;
            }

            Opcode::Sw => {
                // sw ra,dd(rb) - store word
                let base = cpu.get_reg(inst.rb);
                let offset = CpuState::sign_extend_8(imm8);
                let addr = CpuState::mask_24(base.wrapping_add(offset));
                let value = cpu.get_reg(inst.ra);
                cpu.write_word(addr, value);
                cpu.pc = next_pc;
            }

            Opcode::Sxt => {
                // sxt ra,rb - sign extend byte
                let rb_val = cpu.get_reg(inst.rb);
                cpu.set_reg(inst.ra, CpuState::sign_extend_8(rb_val as u8));
                cpu.pc = next_pc;
            }

            Opcode::Xor => {
                // xor ra,rb
                let ra_val = cpu.get_reg(inst.ra);
                let rb_val = cpu.get_reg(inst.rb);
                cpu.set_reg(inst.ra, ra_val ^ rb_val);
                cpu.pc = next_pc;
            }

            Opcode::Zxt => {
                // zxt ra,rb - zero extend byte
                let rb_val = cpu.get_reg(inst.rb);
                cpu.set_reg(inst.ra, rb_val & 0xFF);
                cpu.pc = next_pc;
            }

            Opcode::Invalid => {
                return ExecuteResult::InvalidInstruction(inst_byte);
            }
        }

        cpu.cycles += 1;
        cpu.instructions += 1;

        ExecuteResult::Ok
    }

    /// Run until halted or max cycles reached
    pub fn run(&self, cpu: &mut CpuState, max_cycles: u64) -> ExecuteResult {
        let start_cycles = cpu.cycles;
        while cpu.cycles - start_cycles < max_cycles {
            match self.step(cpu) {
                ExecuteResult::Ok => continue,
                result => return result,
            }
        }
        ExecuteResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_immediate() {
        let mut cpu = CpuState::new();
        let executor = Executor::new();

        // add r0,5 (assuming 0x09 is add r0,imm)
        cpu.write_byte(0, 0x09);
        cpu.write_byte(1, 0x05);
        cpu.set_reg(0, 10);

        executor.step(&mut cpu);

        assert_eq!(cpu.get_reg(0), 15);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn test_lc() {
        let mut cpu = CpuState::new();
        let executor = Executor::new();

        // lc r0,42 (0x44 = lc r0)
        cpu.write_byte(0, 0x44);
        cpu.write_byte(1, 42);

        executor.step(&mut cpu);

        assert_eq!(cpu.get_reg(0), 42);
        assert_eq!(cpu.pc, 2);
    }
}
