use crate::memory_bus::MemoryBus;

#[cfg(test)]
mod tests;

pub(crate) mod registers;
pub (crate) mod instructions;
pub(crate) mod cpu_arithmetic;

use registers::{CpuReg16, CpuReg8, CpuRegisters};
use instructions::CpuInstruction;
use cpu_arithmetic::CpuArithmetic;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CpuPhase {
    FetchOpcode,
    FetchImm8(CpuInstruction),
    FetchImm16Low(CpuInstruction),
    FetchImm16High(CpuInstruction),
    FetchR16(CpuInstruction),
    FetchE8(CpuInstruction),
    FetchA16Low(CpuInstruction),
    FetchA16High(CpuInstruction, u8),
    ApplyRelativeJump(i8),
    ApplyAbsoluteJump(u16),
}

pub struct Cpu {
    t_cycles_until_step: u8,
    phase: CpuPhase,
    registers: CpuRegisters,
    instruction_set: [[CpuInstruction; 16]; 16],
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            t_cycles_until_step: 3,
            phase: CpuPhase::FetchOpcode,
            registers: CpuRegisters::new(),
            instruction_set: Self::initialize_instruction_set(),
        }
    }

    // https://gbdev.io/pandocs/Power_Up_Sequence.html#console-state-after-boot-rom-hand-off
    pub fn new_post_boot() -> Self {
        let mut cpu = Self::new();
        cpu.registers.pc = 0x100;
        cpu.registers.sp = 0xFFFE;
        cpu.registers.a = 0x01;
        cpu.registers.f.zero = true;
        cpu.registers.f.subtract = false;
        cpu.registers.f.half_carry = true;
        cpu.registers.f.carry = true;
        cpu.registers.b = 0x00;
        cpu.registers.c = 0x13;
        cpu.registers.d = 0x00;
        cpu.registers.e = 0xD8;
        cpu.registers.h = 0x01;
        cpu.registers.l = 0x4D;

        cpu
    }

    fn decode_instruction(&self, opcode: u8) -> CpuInstruction {
        let row = (opcode >> 4) as usize;
        let col = (opcode & 0x0F) as usize;
        self.instruction_set[row][col]
    }

    fn encode_instruction(&self, instruction: CpuInstruction) -> u8 {
        let row = self
            .instruction_set
            .iter()
            .position(|r| r.contains(&instruction))
            .expect(&format!(
                "Instruction not found in instruction set: {:?}",
                instruction
            ));
        let col = self.instruction_set[row]
            .iter()
            .position(|&i| i == instruction)
            .expect(&format!(
                "Instruction not found in instruction set: {:?}",
                instruction
            ));
        ((row as u8) << 4) | (col as u8)
    }

    fn fetch_imm8(&mut self, instruction: CpuInstruction, bus: &mut MemoryBus) {
        match instruction {
            CpuInstruction::LdR8Imm8(register) => {
                let value = self.fetch8(bus);
                self.registers.set_r8(register, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::AddAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_add(value, self.registers.f.carry);
                self.registers.a = result;
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::SubAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_sub(value, self.registers.f.carry);
                self.registers.a = result;
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::AdcAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_add(value, self.registers.f.carry);
                self.registers.a = result;
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::SbcAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_sub(value, self.registers.f.carry);
                self.registers.a = result;
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::AndAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_and(value);
                self.registers.a = result;
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::OrAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_or(value);
                self.registers.a = result;
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::XorAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_xor(value);
                self.registers.a = result;
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdHlMemImm8 => {
                let hl_value = self.registers.get_r16(CpuReg16::HL);
                let imm_value = self.fetch8(bus);
                bus.write8(hl_value, imm_value);
                self.phase = CpuPhase::FetchOpcode;
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    // only 1 m-cycle instruction, so we can directly decode it
    fn phase_add_a_r8(&mut self, register: CpuReg8, carry_flag: bool) {
        let value = self.registers.get_r8(register);
        let (result, flags) = self.registers.a.cpu_add(value, carry_flag);
        self.registers.a = result;
        self.registers.f = flags;
        self.phase = CpuPhase::FetchOpcode;
    }

    // only 1 m-cycle instruction, so we can directly decode it
    fn phase_sub_a_r8(&mut self, register: CpuReg8, borrow_flag: bool) {
        let value = self.registers.get_r8(register);
        let (result, flags) = self.registers.a.cpu_sub(value, borrow_flag);
        self.registers.a = result;
        self.registers.f = flags;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn phase_and_a_r8(&mut self, register: CpuReg8) {
        let value = self.registers.get_r8(register);
        let (result, flags) = self.registers.a.cpu_and(value);
        self.registers.a = result;
        self.registers.f = flags;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn phase_or_a_r8(&mut self, register: CpuReg8) {
        let value = self.registers.get_r8(register);
        let (result, flags) = self.registers.a.cpu_or(value);
        self.registers.a = result;
        self.registers.f = flags;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn phase_xor_a_r8(&mut self, register: CpuReg8) {
        let value = self.registers.get_r8(register);
        let (result, flags) = self.registers.a.cpu_xor(value);
        self.registers.a = result;
        self.registers.f = flags;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn fetch_imm16_low(&mut self, instruction: CpuInstruction, bus: &MemoryBus) {
        let low_byte = self.fetch8(bus);
        match instruction {
            CpuInstruction::LdR16Imm16(register) => {
                self.phase = CpuPhase::FetchImm16High(instruction);
                self.registers.set_r16_low(register, low_byte);
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn fetch_imm16_high(&mut self, instruction: CpuInstruction, bus: &MemoryBus) {
        let high_byte = self.fetch8(bus);
        match instruction {
            CpuInstruction::LdR16Imm16(register) => {
                self.registers.set_r16_high(register, high_byte);
                self.phase = CpuPhase::FetchOpcode;
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn fetch_r16(&mut self, instruction: CpuInstruction, bus: &mut MemoryBus) {
        match instruction {
            CpuInstruction::AddHlR16(src) => {
                let src_value = self.registers.get_r16(src);
                let dest_value = self.registers.get_r16(CpuReg16::HL);
                let (result, flags) = dest_value.cpu_add(src_value, false);
                self.registers
                    .set_r16_high(CpuReg16::HL, (result >> 8) as u8);
                self.registers.set_r16_low(CpuReg16::HL, result as u8);
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdAR16mem(src) => {
                let src_value = self.registers.get_r16(src);
                let value = self.read8(bus, src_value);
                self.registers.set_r8(CpuReg8::A, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdR16memA(dest) => {
                let dest_value = self.registers.get_r16(dest);
                let src_value = self.registers.get_r8(CpuReg8::A);
                bus.write8(dest_value, src_value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdHlMemR8(src) => {
                let hl_value = self.registers.get_r16(CpuReg16::HL);
                let src_value = self.registers.get_r8(src);
                bus.write8(hl_value, src_value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdHlMemImm8 => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn fetch_e8(&mut self, instruction: CpuInstruction, bus: &MemoryBus) {
        let offset = self.fetch8(bus) as i8;
        let should_jump = match instruction {
            CpuInstruction::JrNzE8 => !self.registers.f.zero,
            CpuInstruction::JrZE8 => self.registers.f.zero,
            CpuInstruction::JrNcE8 => !self.registers.f.carry,
            CpuInstruction::JrCE8 => self.registers.f.carry,
            CpuInstruction::JrE8 => true,
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        };

        if !should_jump {
            self.phase = CpuPhase::FetchOpcode;
            return;
        }

        self.phase = CpuPhase::ApplyRelativeJump(offset);
    }

    fn fetch_a16_low(&mut self, instruction: CpuInstruction, bus: &MemoryBus) {
        let low_byte = self.fetch8(bus);

        match instruction {
            CpuInstruction::JpNzA16
            | CpuInstruction::JpZA16
            | CpuInstruction::JpNcA16
            | CpuInstruction::JpCA16
            | CpuInstruction::JpA16 => {
                self.phase = CpuPhase::FetchA16High(instruction, low_byte);
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn fetch_a16_high(&mut self, instruction: CpuInstruction, low_byte: u8, bus: &MemoryBus) {
        let high_byte = self.fetch8(bus);
        let addr = ((high_byte as u16) << 8) | (low_byte as u16);

        match instruction {
            CpuInstruction::JpNzA16 => {
                if !self.registers.f.zero {
                    self.phase = CpuPhase::ApplyAbsoluteJump(addr);
                    return
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::JpZA16 => {
                if self.registers.f.zero {
                    self.phase = CpuPhase::ApplyAbsoluteJump(addr);
                    return;
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::JpNcA16 => {
                if !self.registers.f.carry {
                    self.phase = CpuPhase::ApplyAbsoluteJump(addr);
                    return
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::JpCA16 => {
                if self.registers.f.carry {
                    self.phase = CpuPhase::ApplyAbsoluteJump(addr);
                    return
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::JpA16 => {
                self.phase = CpuPhase::ApplyAbsoluteJump(addr);
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn apply_relative_jump(&mut self, offset: i8) {
        self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
        self.phase = CpuPhase::FetchOpcode;
    }

    fn apply_absolute_jump(&mut self, addr: u16) {
        self.registers.pc = addr;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn phase_fetch_opcode(&mut self, bus: &MemoryBus) {
        let opcode = self.fetch8(bus);

        let instruction = self.decode_instruction(opcode);
        match instruction {
            CpuInstruction::LdR8Imm8(_) => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            CpuInstruction::AddAImm8 => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            CpuInstruction::AddAR8(register) => {
                self.phase_add_a_r8(register, false);
            }
            CpuInstruction::AdcAImm8 => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            CpuInstruction::AdcAR8(register) => {
                self.phase_add_a_r8(register, self.registers.f.carry);
            }
            CpuInstruction::AddHlR16(_) => {
                self.phase = CpuPhase::FetchR16(instruction);
            }
            CpuInstruction::SubAImm8 => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            CpuInstruction::SubAR8(register) => {
                self.phase_sub_a_r8(register, false);
            }
            CpuInstruction::SbcAImm8 => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            CpuInstruction::SbcAR8(register) => {
                self.phase_sub_a_r8(register, self.registers.f.carry);
            }
            CpuInstruction::LdR16Imm16(_) => {
                self.phase = CpuPhase::FetchImm16Low(instruction);
            }
            CpuInstruction::AndAImm8 => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            CpuInstruction::AndAR8(register) => {
                self.phase_and_a_r8(register);
            }
            CpuInstruction::OrAImm8 => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            CpuInstruction::OrAR8(register) => {
                self.phase_or_a_r8(register);
            }
            CpuInstruction::XorAImm8 => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            CpuInstruction::XorAR8(register) => {
                self.phase_xor_a_r8(register);
            }
            CpuInstruction::LdAR16mem(_) => {
                self.phase = CpuPhase::FetchR16(instruction);
            }
            CpuInstruction::LdR16memA(_) => {
                self.phase = CpuPhase::FetchR16(instruction);
            }
            CpuInstruction::LdHlMemR8(_) => {
                self.phase = CpuPhase::FetchR16(instruction);
            }
            CpuInstruction::LdHlMemImm8 => {
                self.phase = CpuPhase::FetchR16(instruction);
            }
            CpuInstruction::JrNzE8
            | CpuInstruction::JrZE8
            | CpuInstruction::JrNcE8
            | CpuInstruction::JrCE8
            | CpuInstruction::JrE8 => {
                self.phase = CpuPhase::FetchE8(instruction);
            }
            CpuInstruction::JpNzA16
            | CpuInstruction::JpZA16
            | CpuInstruction::JpNcA16
            | CpuInstruction::JpCA16
            | CpuInstruction::JpA16 => {
                self.phase = CpuPhase::FetchA16Low(instruction);
            }
            CpuInstruction::JpHl => {
                let hl_value = self.registers.get_r16(CpuReg16::HL);
                self.registers.pc = hl_value;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::Nop => {
                self.phase = CpuPhase::FetchOpcode;
            }
            _ => {
                panic!("No such instruction: {:?} ({:02X})", instruction, self.encode_instruction(instruction));
            }
        }
    }

    pub fn step(&mut self, bus: &mut MemoryBus) {
        if self.t_cycles_until_step != 0 {
            self.t_cycles_until_step -= 1;
            return;
        }

        self.t_cycles_until_step = 3;

        match self.phase {
            CpuPhase::FetchOpcode => {
                self.phase_fetch_opcode(bus);
            }
            CpuPhase::FetchImm8(instruction) => {
                self.fetch_imm8(instruction, bus);
            }
            CpuPhase::FetchImm16Low(instruction) => {
                self.fetch_imm16_low(instruction, bus);
            }
            CpuPhase::FetchImm16High(instruction) => {
                self.fetch_imm16_high(instruction, bus);
            }
            CpuPhase::FetchR16(instruction) => {
                self.fetch_r16(instruction, bus);
            }
            CpuPhase::FetchE8(instruction) => {
                self.fetch_e8(instruction, bus);
            }
            CpuPhase::FetchA16Low(instruction) => {
                self.fetch_a16_low(instruction, bus);
            }
            CpuPhase::FetchA16High(instruction, low_byte) => {
                self.fetch_a16_high(instruction, low_byte, bus);
            }
            CpuPhase::ApplyRelativeJump(offset) => {
                self.apply_relative_jump(offset);
            }
            CpuPhase::ApplyAbsoluteJump(addr) => {
                self.apply_absolute_jump(addr);
            }
        }
    }

    fn read8(&mut self, bus: &MemoryBus, addr: u16) -> u8 {
        bus.read8(addr)
    }

    fn fetch8(&mut self, bus: &MemoryBus) -> u8 {
        let value = bus.read8(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        value
    }
}
