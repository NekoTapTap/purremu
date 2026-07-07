use crate::memory_bus::MemoryBus;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CpuPhase {
    FetchOpcode,
    InstructionDecode(CpuInstruction),
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum CpuRegister {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CpuInstruction {
    LdRImm8(CpuRegister),
    AddAImm8,
    AddAR(CpuRegister),
}

impl From<u8> for CpuInstruction {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x3E => CpuInstruction::LdRImm8(CpuRegister::A), // LD A, imm8
            0x06 => CpuInstruction::LdRImm8(CpuRegister::B), // LD B, imm8
            0x0E => CpuInstruction::LdRImm8(CpuRegister::C), // LD C, imm8
            0x16 => CpuInstruction::LdRImm8(CpuRegister::D), // LD D, imm8
            0x1E => CpuInstruction::LdRImm8(CpuRegister::E), // LD E, imm8
            0x26 => CpuInstruction::LdRImm8(CpuRegister::H), // LD H, imm8
            0x2E => CpuInstruction::LdRImm8(CpuRegister::L), // LD L, imm8
            0xC6 => CpuInstruction::AddAImm8,                // ADD A, imm8
            0x80 => CpuInstruction::AddAR(CpuRegister::B), // ADD A, B
            0x81 => CpuInstruction::AddAR(CpuRegister::C), // ADD A, C
            0x82 => CpuInstruction::AddAR(CpuRegister::D), // ADD A, D
            0x83 => CpuInstruction::AddAR(CpuRegister::E), // ADD A, E
            0x84 => CpuInstruction::AddAR(CpuRegister::H), // ADD A, H
            0x85 => CpuInstruction::AddAR(CpuRegister::L), // ADD A, L
            0x87 => CpuInstruction::AddAR(CpuRegister::A), // ADD A, A
            _ => panic!("Opcode {:02X} not implemented", opcode),
        }
    }
}

#[rustfmt::skip]
struct CpuRegisters {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,
    f: CpuFlagsRegister,
}

impl CpuRegisters {
    #[rustfmt::skip]
    fn new() -> Self {
        Self {
            pc: 0,
            sp: 0,
            a: 0,
            b: 0, c: 0,
            d: 0, e: 0,
            h: 0, l: 0,
            f: CpuFlagsRegister::new(),
        }
    }

    fn set(&mut self, register: CpuRegister, value: u8) {
        match register {
            CpuRegister::A => self.a = value,
            CpuRegister::B => self.b = value,
            CpuRegister::C => self.c = value,
            CpuRegister::D => self.d = value,
            CpuRegister::E => self.e = value,
            CpuRegister::H => self.h = value,
            CpuRegister::L => self.l = value,
        }
    }

    fn get(&self, register: CpuRegister) -> u8 {
        match register {
            CpuRegister::A => self.a,
            CpuRegister::B => self.b,
            CpuRegister::C => self.c,
            CpuRegister::D => self.d,
            CpuRegister::E => self.e,
            CpuRegister::H => self.h,
            CpuRegister::L => self.l,
        }
    }
}

struct CpuFlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

impl CpuFlagsRegister {
    fn new() -> Self {
        Self {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: false,
        }
    }
}

impl From<u8> for CpuFlagsRegister {
    /**
     * For better understanding, I use binary literals to represent the flags in the 8-bit register.
     */
    #[rustfmt::skip]
    fn from(value: u8) -> Self {
        Self {
            zero:       value & 0b1000_0000 != 0,
            subtract:   value & 0b0100_0000 != 0,
            half_carry: value & 0b0010_0000 != 0,
            carry:      value & 0b0001_0000 != 0,
        }
    }
}

impl From<CpuFlagsRegister> for u8 {
    #[rustfmt::skip]
    fn from(flags: CpuFlagsRegister) -> Self {
        (if flags.zero             { 0b1000_0000 } else { 0 })
            | (if flags.subtract   { 0b0100_0000 } else { 0 })
            | (if flags.half_carry { 0b0010_0000 } else { 0 })
            | (if flags.carry      { 0b0001_0000 } else { 0 })
    }
}

trait CpuArithmetic
where
    Self: Sized,
{
    fn cpu_add(&self, value: u8) -> (Self, CpuFlagsRegister);
}

impl CpuArithmetic for u8 {
    fn cpu_add(&self, value: u8) -> (Self, CpuFlagsRegister) {
        let (result, carry) = self.overflowing_add(value);

        let flags = CpuFlagsRegister {
            zero: result == 0,
            subtract: false,
            half_carry: (*self & 0x0F) + (value & 0x0F) > 0x0F,
            carry: carry,
        };
        (result, flags)
    }
}

pub struct Cpu {
    divider: u8,
    phase: CpuPhase,
    registers: CpuRegisters,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            divider: 0,
            phase: CpuPhase::FetchOpcode,
            registers: CpuRegisters::new(),
        }
    }

    fn decode_instruction(&mut self, instruction: CpuInstruction, bus: &MemoryBus) {
        match instruction {
            CpuInstruction::LdRImm8(register) => {
                let value = self.fetch8(bus);
                self.registers.set(register, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::AddAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_add(value);
                self.registers.a = result;
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
    }

    fn step_cycle(&mut self, bus: &MemoryBus) {
        match self.phase {
            CpuPhase::FetchOpcode => {
                let opcode = self.fetch8(bus);

                let instruction = CpuInstruction::from(opcode);
                match instruction {
                    CpuInstruction::LdRImm8(_) => {
                        self.phase = CpuPhase::InstructionDecode(instruction);
                    }
                    CpuInstruction::AddAImm8 => {
                        self.phase = CpuPhase::InstructionDecode(instruction);
                    }
                    CpuInstruction::AddAR(register) => {
                        // only 1 m-cycle instruction, so we can directly decode it
                        let value = self.registers.get(register);
                        let (result, flags) = self.registers.a.cpu_add(value);
                        self.registers.a = result;
                        self.registers.f = flags;
                        self.phase = CpuPhase::FetchOpcode;
                    }
                }
            }
            CpuPhase::InstructionDecode(instruction) => {
                self.decode_instruction(instruction, bus);
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
