use crate::memory_bus::MemoryBus;

#[cfg(test)]
mod tests;

pub(crate) mod cpu_arithmetic;
pub(crate) mod instructions;
pub(crate) mod registers;

use cpu_arithmetic::CpuArithmetic;
use instructions::{CpuCondition, CpuInstruction};
use registers::{CpuReg8, CpuReg16, CpuRegisters};

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
    FetchA16Mem(CpuInstruction, u16),
    FetchR8(CpuInstruction),
    ApplyRelativeJump(i8),
    ApplyAbsoluteJump(u16),
    ApplyAbsoluteJumpEnableInterrupts(u16),

    AddSpE8Low(u8),
    AddSpE8High(u8, u8),

    DecrementSpForWrite(CpuInstruction, u16),
    DecrementSp(CpuReg16),
    WriteSpMemHigh(CpuInstruction, u16),
    WriteSpMemLow(CpuInstruction, u16),

    ReadSpHigh(CpuInstruction, u8),
    ReadSpLow(CpuInstruction),

    PushR16Low(CpuReg16),
    PushR16High(CpuReg16),

    PopR16Low(CpuReg16),
    PopR16High(CpuReg16),

    DecrementR16(CpuReg16),
    IncrementR16(CpuReg16),

    CheckRetCondition(CpuCondition),
}

pub struct Cpu {
    t_cycles_until_step: u8,
    phase: CpuPhase,
    registers: CpuRegisters,
    instruction_set: [[CpuInstruction; 16]; 16],
    ime: bool, // Interrupt Master Enable flag
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            t_cycles_until_step: 3,
            phase: CpuPhase::FetchOpcode,
            registers: CpuRegisters::new(),
            instruction_set: Self::initialize_instruction_set(),
            ime: false,
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

    #[cfg(test)]
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
            CpuInstruction::LdhA8A => {
                // LDH = LD (FF00 + a8), A
                let addr = 0xFF00 | (self.fetch8(bus) as u16);
                let value = self.registers.get_r8(CpuReg8::A);
                bus.write8(addr, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdhAA8 => {
                let addr = 0xFF00 | (self.fetch8(bus) as u16);
                let value = bus.read8(addr);
                self.registers.set_r8(CpuReg8::A, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdhAC | CpuInstruction::LdhCA => {
                self.phase = CpuPhase::FetchR8(instruction)
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn phase_fetch_r8(&mut self, instruction: CpuInstruction, bus: &mut MemoryBus) {
        use CpuReg8::*;

        match instruction {
            CpuInstruction::LdhCA => {
                // LDH (FF00 + C), A
                let addr = 0xFF00 | (self.registers.get_r8(C) as u16);
                let value = self.registers.get_r8(A);
                bus.write8(addr, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdhAC => {
                // LDH A, (FF00 + C)
                let addr = 0xFF00 | (self.registers.get_r8(C) as u16);
                let value = bus.read8(addr);
                self.registers.set_r8(A, value);
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
                self.registers.set_r16_low(register, low_byte);
                self.phase = CpuPhase::FetchImm16High(instruction);
            }
            CpuInstruction::LdSpImm16 => {
                self.registers.sp = low_byte as u16; // set low byte of SP
                self.phase = CpuPhase::FetchImm16High(instruction);
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
            CpuInstruction::LdSpImm16 => {
                self.registers.sp |= (high_byte as u16) << 8; // set high byte of SP
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
        let raw_offset = self.fetch8(bus);
        if instruction == CpuInstruction::AddSpE8 {
            self.phase = CpuPhase::AddSpE8Low(raw_offset);
            return;
        }

        let offset = raw_offset as i8;
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

    fn add_sp_e8_low(&mut self, raw_offset: u8) {
        let [sp_low, _] = self.registers.sp.to_le_bytes();
        let (result_low, reg) = sp_low.cpu_add(raw_offset, false);
        let adjustment = if raw_offset & 0x80 != 0 { 0xFF } else { 0x00 };

        // There is no signed types in the Gameboy CPU,
        // so carry and half-carry flags still calculated as if the offset was unsigned.
        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = reg.half_carry;
        self.registers.f.carry = reg.carry;
        self.phase = CpuPhase::AddSpE8High(result_low, adjustment);
    }

    fn add_sp_e8_high(&mut self, result_low: u8, adjustment: u8) {
        let [_, sp_high] = self.registers.sp.to_le_bytes();
        let result_high = sp_high
            .wrapping_add(adjustment)
            .wrapping_add(u8::from(self.registers.f.carry));

        self.registers.sp = u16::from_le_bytes([result_low, result_high]);
        self.phase = CpuPhase::FetchOpcode;
    }

    fn fetch_a16_low(&mut self, instruction: CpuInstruction, bus: &MemoryBus) {
        let low_byte = self.fetch8(bus);

        match instruction {
            CpuInstruction::JpNzA16
            | CpuInstruction::JpZA16
            | CpuInstruction::JpNcA16
            | CpuInstruction::JpCA16
            | CpuInstruction::JpA16
            | CpuInstruction::CallA16
            | CpuInstruction::CallZA16
            | CpuInstruction::CallCA16
            | CpuInstruction::CallNcA16
            | CpuInstruction::CallNzA16
            | CpuInstruction::LdAAddr
            | CpuInstruction::LdAddrA => {
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
                    return;
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
                    return;
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::JpCA16 => {
                if self.registers.f.carry {
                    self.phase = CpuPhase::ApplyAbsoluteJump(addr);
                    return;
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::JpA16 => {
                self.phase = CpuPhase::ApplyAbsoluteJump(addr);
            }
            CpuInstruction::CallA16 => {
                self.phase = CpuPhase::DecrementSpForWrite(instruction, addr);
            }
            CpuInstruction::CallNzA16 => {
                if !self.registers.f.zero {
                    self.phase = CpuPhase::DecrementSpForWrite(instruction, addr);
                    return;
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::CallZA16 => {
                if self.registers.f.zero {
                    self.phase = CpuPhase::DecrementSpForWrite(instruction, addr);
                    return;
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::CallNcA16 => {
                if !self.registers.f.carry {
                    self.phase = CpuPhase::DecrementSpForWrite(instruction, addr);
                    return;
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::CallCA16 => {
                if self.registers.f.carry {
                    self.phase = CpuPhase::DecrementSpForWrite(instruction, addr);
                    return;
                }
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdAAddr | CpuInstruction::LdAddrA => {
                self.phase = CpuPhase::FetchA16Mem(instruction, addr);
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

    fn check_return_condition(&mut self, condition: CpuCondition) {
        let should_return = match condition {
            CpuCondition::NZ => !self.registers.f.zero,
            CpuCondition::Z => self.registers.f.zero,
            CpuCondition::NC => !self.registers.f.carry,
            CpuCondition::C => self.registers.f.carry,
            CpuCondition::None => true,
        };

        if !should_return {
            self.phase = CpuPhase::FetchOpcode;
            return;
        }

        self.phase = CpuPhase::ReadSpLow(CpuInstruction::Ret(condition));
    }

    fn phase_fetch_opcode(&mut self, bus: &MemoryBus) {
        let opcode = self.fetch8(bus);

        let instruction = self.decode_instruction(opcode);
        match instruction {
            CpuInstruction::LdR8Imm8(_)
            | CpuInstruction::AdcAImm8
            | CpuInstruction::LdhAA8
            | CpuInstruction::LdhA8A
            | CpuInstruction::AddAImm8
            | CpuInstruction::SubAImm8
            | CpuInstruction::SbcAImm8
            | CpuInstruction::AndAImm8
            | CpuInstruction::OrAImm8
            | CpuInstruction::XorAImm8 => {
                self.phase = CpuPhase::FetchImm8(instruction);
            }
            CpuInstruction::LdR8R8(dest, src) => {
                let value = self.registers.get_r8(src);
                self.registers.set_r8(dest, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::AddAR8(register) => {
                self.phase_add_a_r8(register, false);
            }
            CpuInstruction::AdcAR8(register) => {
                self.phase_add_a_r8(register, self.registers.f.carry);
            }
            CpuInstruction::AddHlR16(_) => {
                self.phase = CpuPhase::FetchR16(instruction);
            }
            CpuInstruction::SubAR8(register) => {
                self.phase_sub_a_r8(register, false);
            }
            CpuInstruction::SbcAR8(register) => {
                self.phase_sub_a_r8(register, self.registers.f.carry);
            }
            CpuInstruction::LdR16Imm16(_) | CpuInstruction::LdSpImm16 => {
                self.phase = CpuPhase::FetchImm16Low(instruction);
            }
            CpuInstruction::AndAR8(register) => {
                self.phase_and_a_r8(register);
            }
            CpuInstruction::OrAR8(register) => {
                self.phase_or_a_r8(register);
            }
            CpuInstruction::XorAR8(register) => {
                self.phase_xor_a_r8(register);
            }
            CpuInstruction::LdAR16mem(_)
            | CpuInstruction::LdR16memA(_)
            | CpuInstruction::LdHlMemImm8
            | CpuInstruction::LdHlMemR8(_) => {
                self.phase = CpuPhase::FetchR16(instruction);
            }
            CpuInstruction::JrNzE8
            | CpuInstruction::JrZE8
            | CpuInstruction::JrNcE8
            | CpuInstruction::JrCE8
            | CpuInstruction::JrE8
            | CpuInstruction::AddSpE8 => {
                self.phase = CpuPhase::FetchE8(instruction);
            }
            CpuInstruction::JpNzA16
            | CpuInstruction::JpZA16
            | CpuInstruction::JpNcA16
            | CpuInstruction::JpCA16
            | CpuInstruction::JpA16
            | CpuInstruction::CallA16
            | CpuInstruction::CallNzA16
            | CpuInstruction::CallZA16
            | CpuInstruction::CallNcA16
            | CpuInstruction::CallCA16
            | CpuInstruction::LdAAddr
            | CpuInstruction::LdAddrA => {
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
            CpuInstruction::DI => {
                self.ime = false;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::EI => {
                self.ime = true;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::Ret(CpuCondition::None) => {
                self.phase = CpuPhase::ReadSpLow(instruction);
            }
            CpuInstruction::Ret(cond) => {
                self.phase = CpuPhase::CheckRetCondition(cond);
            }
            CpuInstruction::RetI => {
                self.phase = CpuPhase::ReadSpLow(instruction);
            }
            CpuInstruction::DecR16(r16) => {
                self.phase = CpuPhase::DecrementR16(r16);
            }
            CpuInstruction::IncR16(r16) => {
                self.phase = CpuPhase::IncrementR16(r16);
            }
            CpuInstruction::DecR8(r8) => {
                self.decrement_r8(r8);
            }
            CpuInstruction::IncR8(r8) => {
                self.increment_r8(r8);
            }
            CpuInstruction::PushR16(r16) => {
                self.phase = CpuPhase::DecrementSp(r16);
            }
            CpuInstruction::PopR16(r16) => {
                self.phase = CpuPhase::PopR16Low(r16);
            }
            CpuInstruction::Rst(addr) => {
                self.phase = CpuPhase::DecrementSpForWrite(instruction, addr);
            }
            _ => {
                panic!("No such instruction: {:?} (0X{:02X})", instruction, opcode);
            }
        }
    }

    fn decrement_sp_for_write(&mut self, instruction: CpuInstruction, addr: u16) {
        self.registers.sp -= 1;
        self.phase = CpuPhase::WriteSpMemHigh(instruction, addr);
    }

    fn decrement_sp(&mut self, register: CpuReg16) {
        self.registers.sp -= 1;
        self.phase = CpuPhase::PushR16High(register);
    }

    fn decrement_r8(&mut self, register: CpuReg8) {
        let value = self.registers.get_r8(register);
        let (result, flags) = value.cpu_sub(1, false);
        self.registers.set_r8(register, result);
        self.registers.f = flags;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn increment_r8(&mut self, register: CpuReg8) {
        let value = self.registers.get_r8(register);
        let (result, flags) = value.cpu_add(1, false);
        self.registers.set_r8(register, result);
        self.registers.f = flags;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn increment_r16(&mut self, register: CpuReg16) {
        let value = self.registers.get_r16(register);
        let (result, flags) = value.cpu_add(1, false);
        self.registers.set_r16_high(register, (result >> 8) as u8);
        self.registers.set_r16_low(register, result as u8);
        self.registers.f = flags;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn decrement_r16(&mut self, register: CpuReg16) {
        let value = self.registers.get_r16(register);
        let (result, flags) = value.cpu_sub(1, false);
        self.registers.set_r16_high(register, (result >> 8) as u8);
        self.registers.set_r16_low(register, result as u8);
        self.registers.f = flags;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn write_sp_high(&mut self, instruction: CpuInstruction, addr: u16, bus: &mut MemoryBus) {
        match instruction {
            CpuInstruction::CallA16
            | CpuInstruction::CallCA16
            | CpuInstruction::CallZA16
            | CpuInstruction::CallNcA16
            | CpuInstruction::CallNzA16
            | CpuInstruction::Rst(_) => {
                let high_byte = (self.registers.pc >> 8) as u8;
                bus.write8(self.registers.sp, high_byte);
                self.registers.sp -= 1;
                self.phase = CpuPhase::WriteSpMemLow(instruction, addr);
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn write_sp_low(&mut self, instruction: CpuInstruction, addr: u16, bus: &mut MemoryBus) {
        match instruction {
            CpuInstruction::CallA16
            | CpuInstruction::CallCA16
            | CpuInstruction::CallZA16
            | CpuInstruction::CallNcA16
            | CpuInstruction::CallNzA16
            | CpuInstruction::Rst(_) => {
                let low_byte = self.registers.pc as u8;
                bus.write8(self.registers.sp, low_byte);
                self.registers.pc = addr;
                self.phase = CpuPhase::FetchOpcode;
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn read_sp_low(&mut self, instruction: CpuInstruction, bus: &MemoryBus) {
        match instruction {
            CpuInstruction::Ret(_) | CpuInstruction::RetI => {
                let low_byte = self.read8(bus, self.registers.sp);
                self.registers.sp += 1;
                self.phase = CpuPhase::ReadSpHigh(instruction, low_byte);
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn read_sp_high(&mut self, instruction: CpuInstruction, low_byte: u8, bus: &MemoryBus) {
        let high_byte = self.read8(bus, self.registers.sp);
        self.registers.sp += 1;

        let addr = ((high_byte as u16) << 8) | (low_byte as u16);

        match instruction {
            CpuInstruction::Ret(_) => self.phase = CpuPhase::ApplyAbsoluteJump(addr),
            CpuInstruction::RetI => self.phase = CpuPhase::ApplyAbsoluteJumpEnableInterrupts(addr),

            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn push_r16_high(&mut self, register: CpuReg16, bus: &mut MemoryBus) {
        let value = self.registers.get_r16(register);
        bus.write8(self.registers.sp, (value >> 8) as u8);
        self.registers.sp -= 1;
        self.phase = CpuPhase::PushR16Low(register);
    }

    fn push_r16_low(&mut self, register: CpuReg16, bus: &mut MemoryBus) {
        let value = self.registers.get_r16(register);
        bus.write8(self.registers.sp, value as u8);
        self.registers.sp -= 1;
        self.phase = CpuPhase::FetchOpcode;
    }

    fn pop_r16_low(&mut self, register: CpuReg16, bus: &mut MemoryBus) {
        let low_byte = self.read8(bus, self.registers.sp);
        self.registers.sp += 1;
        self.registers.set_r16_low(register, low_byte);
        self.phase = CpuPhase::PopR16High(register);
    }

    fn pop_r16_high(&mut self, register: CpuReg16, bus: &mut MemoryBus) {
        let high_byte = self.read8(bus, self.registers.sp);
        self.registers.sp += 1;
        self.registers.set_r16_high(register, high_byte);
        self.phase = CpuPhase::FetchOpcode;
    }

    fn fetch_a16_mem(&mut self, instruction: CpuInstruction, addr: u16, bus: &mut MemoryBus) {
        match instruction {
            CpuInstruction::LdAAddr => {
                let value = self.read8(bus, addr);
                self.registers.set_r8(CpuReg8::A, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::LdAddrA => {
                let value = self.registers.get_r8(CpuReg8::A);
                bus.write8(addr, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
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
            CpuPhase::FetchOpcode => self.phase_fetch_opcode(bus),
            CpuPhase::FetchImm8(instruction) => self.fetch_imm8(instruction, bus),
            CpuPhase::FetchImm16Low(instruction) => self.fetch_imm16_low(instruction, bus),
            CpuPhase::FetchImm16High(instruction) => self.fetch_imm16_high(instruction, bus),
            CpuPhase::FetchR16(instruction) => self.fetch_r16(instruction, bus),
            CpuPhase::FetchE8(instruction) => self.fetch_e8(instruction, bus),
            CpuPhase::FetchA16Low(instruction) => self.fetch_a16_low(instruction, bus),
            CpuPhase::FetchA16High(instruction, low_byte) => {
                self.fetch_a16_high(instruction, low_byte, bus)
            }
            CpuPhase::DecrementSpForWrite(instruction, addr) => {
                self.decrement_sp_for_write(instruction, addr)
            }
            CpuPhase::IncrementR16(register) => self.increment_r16(register),
            CpuPhase::DecrementR16(register) => self.decrement_r16(register),
            CpuPhase::WriteSpMemHigh(instruction, addr) => {
                self.write_sp_high(instruction, addr, bus)
            }
            CpuPhase::WriteSpMemLow(instruction, addr) => self.write_sp_low(instruction, addr, bus),
            CpuPhase::ReadSpHigh(instruction, low_byte) => {
                self.read_sp_high(instruction, low_byte, bus)
            }
            CpuPhase::CheckRetCondition(condition) => self.check_return_condition(condition),
            CpuPhase::ReadSpLow(instruction) => self.read_sp_low(instruction, bus),
            CpuPhase::ApplyRelativeJump(offset) => self.apply_relative_jump(offset),
            CpuPhase::ApplyAbsoluteJump(addr) => self.apply_absolute_jump(addr),
            CpuPhase::ApplyAbsoluteJumpEnableInterrupts(addr) => {
                self.apply_absolute_jump(addr);
                self.ime = true;
            }
            CpuPhase::PushR16High(register) => self.push_r16_high(register, bus),
            CpuPhase::PushR16Low(register) => self.push_r16_low(register, bus),
            CpuPhase::DecrementSp(register) => self.decrement_sp(register),
            CpuPhase::PopR16Low(register) => self.pop_r16_low(register, bus),
            CpuPhase::PopR16High(register) => self.pop_r16_high(register, bus),
            CpuPhase::FetchA16Mem(instruction, addr) => self.fetch_a16_mem(instruction, addr, bus),
            CpuPhase::FetchR8(instruction) => self.phase_fetch_r8(instruction, bus),
            CpuPhase::AddSpE8Low(raw_offset) => self.add_sp_e8_low(raw_offset),
            CpuPhase::AddSpE8High(result_low, adjustment) => {
                self.add_sp_e8_high(result_low, adjustment)
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
