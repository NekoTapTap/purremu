use crate::{cpu::CpuReg8::C, memory_bus::MemoryBus};

#[cfg(test)]
mod tests;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CpuPhase {
    FetchOpcode,
    FetchImm8(CpuInstruction),
}

#[rustfmt::skip]
#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum CpuReg8 {
    A,
    B, C,
    D, E,
    H, L,
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum CpuReg16 {
    BC,
    DE,
    HL,
    SP,
}

#[rustfmt::skip]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CpuInstruction {
    LdR8Imm8(CpuReg8),
    LdR8R8(CpuReg8, CpuReg8),
    LdR16Imm16(CpuReg8, CpuReg8),
    LdR16memR8(CpuReg16, CpuReg8),
    LdR8R16mem(CpuReg8, CpuReg16),

    IncR8(CpuReg8),
    IncR16(CpuReg8, CpuReg8),
    DecR8(CpuReg8),
    DecR16(CpuReg16),

    AddAImm8,
    AddAR8(CpuReg8),
    AddR16R16(CpuReg16, CpuReg16),
    AdcAImm8,
    AdcAR8(CpuReg8),

    SubAImm8,
    SubAR8(CpuReg8),
    SbcAImm8,
    SbcAR8(CpuReg8),

    AndAImm8,
    AndAR8(CpuReg8),
    OrAImm8,
    OrAR8(CpuReg8),
    XorAImm8,
    XorAR8(CpuReg8),

    CpAImm8,
    CpAR8(CpuReg8),

    Illegal,
    NoImpl,
}

#[rustfmt::skip]
struct CpuRegisters {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,
    f: CpuFlagsReg,
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
            f: CpuFlagsReg::new(),
        }
    }


    fn set_r8(&mut self, register: CpuReg8, value: u8) {
        match register {
            CpuReg8::A => self.a = value,
            CpuReg8::B => self.b = value,
            CpuReg8::C => self.c = value,
            CpuReg8::D => self.d = value,
            CpuReg8::E => self.e = value,
            CpuReg8::H => self.h = value,
            CpuReg8::L => self.l = value,
        }
    }

    fn get_r8(&self, register: CpuReg8) -> u8 {
        match register {
            CpuReg8::A => self.a,
            CpuReg8::B => self.b,
            CpuReg8::C => self.c,
            CpuReg8::D => self.d,
            CpuReg8::E => self.e,
            CpuReg8::H => self.h,
            CpuReg8::L => self.l,
        }
    }
}

struct CpuFlagsReg {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

impl CpuFlagsReg {
    fn new() -> Self {
        Self {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: false,
        }
    }
}

impl From<u8> for CpuFlagsReg {
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

impl From<CpuFlagsReg> for u8 {
    #[rustfmt::skip]
    fn from(flags: CpuFlagsReg) -> Self {
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
    fn cpu_add(&self, value: u8) -> (Self, CpuFlagsReg);
    fn cpu_sub(&self, value: u8) -> (Self, CpuFlagsReg);
}

impl CpuArithmetic for u8 {
    fn cpu_add(&self, value: u8) -> (Self, CpuFlagsReg) {
        let (result, carry) = self.overflowing_add(value);

        let flags = CpuFlagsReg {
            zero: result == 0,
            subtract: false,
            half_carry: (*self & 0x0F) + (value & 0x0F) > 0x0F, // For BCD
            carry: carry,
        };
        (result, flags)
    }

    fn cpu_sub(&self, value: u8) -> (Self, CpuFlagsReg) {
        let (result, borrow) = self.overflowing_sub(value);

        let flags = CpuFlagsReg {
            zero: result == 0,
            subtract: true,
            half_carry: (*self & 0x0F) < (value & 0x0F), // For BCD
            carry: borrow,
        };
        (result, flags)
    }
}

pub struct Cpu {
    divider: u8,
    phase: CpuPhase,
    registers: CpuRegisters,
    instruction_set: [[CpuInstruction; 16]; 16],
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            divider: 0,
            phase: CpuPhase::FetchOpcode,
            registers: CpuRegisters::new(),
            instruction_set: Self::initialize_instruction_set(),
        }
    }

    #[rustfmt::skip]
    pub fn initialize_instruction_set() -> [[CpuInstruction; 16]; 16] {
        use CpuInstruction::*;
        use CpuReg8::*;
        use CpuReg16::*;

        [
            [NoImpl     , LdR16Imm16(B,C), LdR16memR8(BC,A), IncR16(B,C), IncR8(B)   , DecR8(B)   , LdR8Imm8(B), NoImpl     , NoImpl     , AddR16R16(HL,BC), LdR8R16mem(A,BC), DecR16(BC), IncR8(C)   , DecR8(C)   , LdR8Imm8(C), NoImpl     ],
            [NoImpl     , LdR16Imm16(D,E), LdR16memR8(DE,A), IncR16(D,E), IncR8(D)   , DecR8(D)   , LdR8Imm8(D), NoImpl     , NoImpl     , AddR16R16(HL,DE), LdR8R16mem(A,DE), DecR16(DE), IncR8(E)   , DecR8(E)   , LdR8Imm8(E), NoImpl     ],
            [NoImpl     , LdR16Imm16(H,L), LdR16memR8(HL,A), IncR16(H,L), IncR8(H)   , DecR8(H)   , LdR8Imm8(H), NoImpl     , NoImpl     , AddR16R16(HL,HL), LdR8R16mem(A,HL), DecR16(HL), IncR8(L)   , DecR8(L)   , LdR8Imm8(L), NoImpl     ],
            [NoImpl     , NoImpl       , NoImpl        , NoImpl     , NoImpl     , NoImpl     , NoImpl   , NoImpl     , NoImpl     , NoImpl            , NoImpl        , NoImpl     , IncR8(A)   , DecR8(A)   , LdR8Imm8(A), NoImpl     ],

            [LdR8R8(B,B), LdR8R8(B,C)  , LdR8R8(B,D)   , LdR8R8(B,E), LdR8R8(B,H), LdR8R8(B,L), NoImpl   , LdR8R8(B,A), LdR8R8(C,B), LdR8R8(C,C)       , LdR8R8(C,D)   , LdR8R8(C,E), LdR8R8(C,H), LdR8R8(C,L), NoImpl   , LdR8R8(C,A)],
            [LdR8R8(D,B), LdR8R8(D,C)  , LdR8R8(D,D)   , LdR8R8(D,E), LdR8R8(D,H), LdR8R8(D,L), NoImpl   , LdR8R8(D,A), LdR8R8(E,B), LdR8R8(E,C)       , LdR8R8(E,D)   , LdR8R8(E,E), LdR8R8(E,H), LdR8R8(E,L), NoImpl   , LdR8R8(E,A)],
            [LdR8R8(H,B), LdR8R8(H,C)  , LdR8R8(H,D)   , LdR8R8(H,E), LdR8R8(H,H), LdR8R8(H,L), NoImpl   , LdR8R8(H,A), LdR8R8(L,B), LdR8R8(L,C)       , LdR8R8(L,D)   , LdR8R8(L,E), LdR8R8(L,H), LdR8R8(L,L), NoImpl   , LdR8R8(L,A)],
            [NoImpl     , NoImpl       , NoImpl        , NoImpl     , NoImpl     , NoImpl     , NoImpl   , NoImpl     , LdR8R8(A,B), LdR8R8(A,C)       , LdR8R8(A,D)   , LdR8R8(A,E), LdR8R8(A,H), LdR8R8(A,L), NoImpl   , LdR8R8(A,A)],

            [AddAR8(B)  , AddAR8(C)    , AddAR8(D)     , AddAR8(E)  , AddAR8(H)  , AddAR8(L)  , NoImpl   , AddAR8(A)  , AdcAR8(B)  , AdcAR8(C)         , AdcAR8(D)     , AdcAR8(E)  , AdcAR8(H)  , AdcAR8(L)  , NoImpl   , AdcAR8(A)  ],
            [SubAR8(B)  , SubAR8(C)    , SubAR8(D)     , SubAR8(E)  , SubAR8(H)  , SubAR8(L)  , NoImpl   , SubAR8(A)  , SbcAR8(B)  , SbcAR8(C)         , SbcAR8(D)     , SbcAR8(E)  , SbcAR8(H)  , SbcAR8(L)  , NoImpl   , SbcAR8(A)  ],
            [AndAR8(B)  , AndAR8(C)    , AndAR8(D)     , AndAR8(E)  , AndAR8(H)  , AndAR8(L)  , NoImpl   , AndAR8(A)  , XorAR8(B)  , XorAR8(C)         , XorAR8(D)     , XorAR8(E)  , XorAR8(H)  , XorAR8(L)  , NoImpl   , XorAR8(A)  ],
            [OrAR8(B)   , OrAR8(C)     , OrAR8(D)      , OrAR8(E)   , OrAR8(H)   , OrAR8(L)   , NoImpl   , OrAR8(A)   , CpAR8(B)   , CpAR8(C)          , CpAR8(D)      , CpAR8(E)   , CpAR8(H)   , CpAR8(L)   , NoImpl   , CpAR8(A)   ],

            [NoImpl     , NoImpl       , NoImpl        , NoImpl     , NoImpl     , NoImpl     , AddAImm8   , NoImpl     , NoImpl     , NoImpl            , NoImpl        , NoImpl     , NoImpl     , NoImpl     , AdcAImm8   , NoImpl     ],
            [NoImpl     , NoImpl       , NoImpl        , Illegal    , NoImpl     , NoImpl     , SubAImm8   , NoImpl     , NoImpl     , NoImpl            , NoImpl        , Illegal    , NoImpl     , Illegal    , SbcAImm8   , NoImpl     ],
            [NoImpl     , NoImpl       , NoImpl        , Illegal    , Illegal    , NoImpl     , AndAImm8   , NoImpl     , NoImpl     , NoImpl            , NoImpl        , Illegal    , Illegal    , Illegal    , XorAImm8   , NoImpl     ],
            [NoImpl     , NoImpl       , NoImpl        , NoImpl     , Illegal    , NoImpl     , OrAImm8    , NoImpl     , NoImpl     , NoImpl            , NoImpl        , NoImpl     , Illegal    , Illegal    , CpAImm8    , NoImpl     ],
        ]
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
            .expect("Instruction not found in instruction set");
        let col = self.instruction_set[row]
            .iter()
            .position(|&i| i == instruction)
            .expect("Instruction not found in instruction set");
        ((row as u8) << 4) | (col as u8)
    }

    fn fetch_immediate_number(&mut self, instruction: CpuInstruction, bus: &MemoryBus) {
        match instruction {
            CpuInstruction::LdR8Imm8(register) => {
                let value = self.fetch8(bus);
                self.registers.set_r8(register, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::AddAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_add(value);
                self.registers.a = result;
                self.registers.f = flags;
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::SubAImm8 => {
                let value = self.fetch8(bus);
                let (result, flags) = self.registers.a.cpu_sub(value);
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

                let instruction = self.decode_instruction(opcode);
                match instruction {
                    CpuInstruction::LdR8Imm8(_) => {
                        self.phase = CpuPhase::FetchImm8(instruction);
                    }
                    CpuInstruction::AddAImm8 => {
                        self.phase = CpuPhase::FetchImm8(instruction);
                    }
                    CpuInstruction::AddAR8(register) => {
                        // only 1 m-cycle instruction, so we can directly decode it
                        let value = self.registers.get_r8(register);
                        let (result, flags) = self.registers.a.cpu_add(value);
                        self.registers.a = result;
                        self.registers.f = flags;
                        self.phase = CpuPhase::FetchOpcode;
                    }
                    CpuInstruction::SubAImm8 => {
                        self.phase = CpuPhase::FetchImm8(instruction);
                    }
                    CpuInstruction::SubAR8(register) => {
                        let value = self.registers.get_r8(register);
                        let (result, flags) = self.registers.a.cpu_sub(value);
                        self.registers.a = result;
                        self.registers.f = flags;
                        self.phase = CpuPhase::FetchOpcode;
                    }
                    _ => {
                        panic!("No such instruction: {:?}", instruction);
                    }
                }
            }
            CpuPhase::FetchImm8(instruction) => {
                self.fetch_immediate_number(instruction, bus);
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
