use crate::memory_bus::MemoryBus;

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
}

impl From<u8> for CpuInstruction {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x3E => CpuInstruction::LdRImm8(CpuRegister::A), // LD A, d8
            0x06 => CpuInstruction::LdRImm8(CpuRegister::B), // LD B, d8
            0x0E => CpuInstruction::LdRImm8(CpuRegister::C), // LD C, d8
            0x16 => CpuInstruction::LdRImm8(CpuRegister::D), // LD D, d8
            0x1E => CpuInstruction::LdRImm8(CpuRegister::E), // LD E, d8
            0x26 => CpuInstruction::LdRImm8(CpuRegister::H), // LD H, d8
            0x2E => CpuInstruction::LdRImm8(CpuRegister::L), // LD L, d8
            _ => panic!("Opcode {:02X} not implemented", opcode),
        }
    }
}

struct CpuRegisters {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
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
            registers: CpuRegisters {
                pc: 0,
                sp: 0,
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                h: 0,
                l: 0,
            },
        }
    }

    fn step_cycle(&mut self, bus: &MemoryBus) {
        match self.phase {
            CpuPhase::FetchOpcode => {
                let opcode = self.fetch8(bus);
                self.phase = CpuPhase::InstructionDecode(CpuInstruction::from(opcode));
            }
            CpuPhase::InstructionDecode(CpuInstruction::LdRImm8(register)) => {
                let value = self.fetch8(bus);
                match register {
                    CpuRegister::A => self.registers.a = value,
                    CpuRegister::B => self.registers.b = value,
                    CpuRegister::C => self.registers.c = value,
                    CpuRegister::D => self.registers.d = value,
                    CpuRegister::E => self.registers.e = value,
                    CpuRegister::H => self.registers.h = value,
                    CpuRegister::L => self.registers.l = value,
                }
                self.phase = CpuPhase::FetchOpcode;
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
