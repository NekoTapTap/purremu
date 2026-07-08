use crate::memory_bus::MemoryBus;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CpuPhase {
    FetchOpcode,
    FetchImmediateNumber(CpuInstruction),
}

#[rustfmt::skip]
#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum CpuRegister {
    A,
    B, C,
    D, E,
    H, L,
}

#[rustfmt::skip]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CpuInstruction {
    LdR8N8(CpuRegister),
    LdR8R8(CpuRegister, CpuRegister),
    LdR16N16(CpuRegister, CpuRegister),
    LdR16R8(CpuRegister, CpuRegister, CpuRegister),
    LdR8R16(CpuRegister, CpuRegister, CpuRegister),

    IncR8(CpuRegister),
    IncR16(CpuRegister, CpuRegister),
    DecR8(CpuRegister),
    DecR16(CpuRegister, CpuRegister),

    AddAN8,
    AddAR8(CpuRegister),
    AddR16R16(CpuRegister, CpuRegister, CpuRegister, CpuRegister),
    AdcAN8,
    AdcAR8(CpuRegister),

    SubAN8,
    SubAR8(CpuRegister),
    SbcAN8,
    SbcAR8(CpuRegister),

    AndAN8,
    AndAR8(CpuRegister),
    OrAN8,
    OrAR8(CpuRegister),
    XorAN8,
    XorAR8(CpuRegister),

    CpAN8,
    CpAR8(CpuRegister),

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
        use CpuRegister::*;

        [
            [NoImpl     , LdR16N16(B,C), LdR16R8(B,C,A), IncR16(B,C), IncR8(B)   , DecR8(B)   , LdR8N8(B), NoImpl     , NoImpl     , AddR16R16(H,L,B,C), LdR8R16(A,B,C), DecR16(B,C), IncR8(C)   , DecR8(C)   , LdR8N8(C), NoImpl     ],
            [NoImpl     , LdR16N16(D,E), LdR16R8(D,E,A), IncR16(D,E), IncR8(D)   , DecR8(D)   , LdR8N8(D), NoImpl     , NoImpl     , AddR16R16(H,L,D,E), LdR8R16(A,D,E), DecR16(D,E), IncR8(E)   , DecR8(E)   , LdR8N8(E), NoImpl     ],
            [NoImpl     , LdR16N16(H,L), LdR16R8(H,L,A), IncR16(H,L), IncR8(H)   , DecR8(H)   , LdR8N8(H), NoImpl     , NoImpl     , AddR16R16(H,L,H,L), LdR8R16(A,H,L), DecR16(H,L), IncR8(L)   , DecR8(L)   , LdR8N8(L), NoImpl     ],
            [NoImpl     , NoImpl       , NoImpl        , NoImpl     , NoImpl     , NoImpl     , NoImpl   , NoImpl     , NoImpl     , NoImpl            , NoImpl        , NoImpl     , IncR8(A)   , DecR8(A)   , LdR8N8(A), NoImpl     ],

            [LdR8R8(B,B), LdR8R8(B,C)  , LdR8R8(B,D)   , LdR8R8(B,E), LdR8R8(B,H), LdR8R8(B,L), NoImpl   , LdR8R8(B,A), LdR8R8(C,B), LdR8R8(C,C)       , LdR8R8(C,D)   , LdR8R8(C,E), LdR8R8(C,H), LdR8R8(C,L), NoImpl   , LdR8R8(C,A)],
            [LdR8R8(D,B), LdR8R8(D,C)  , LdR8R8(D,D)   , LdR8R8(D,E), LdR8R8(D,H), LdR8R8(D,L), NoImpl   , LdR8R8(D,A), LdR8R8(E,B), LdR8R8(E,C)       , LdR8R8(E,D)   , LdR8R8(E,E), LdR8R8(E,H), LdR8R8(E,L), NoImpl   , LdR8R8(E,A)],
            [LdR8R8(H,B), LdR8R8(H,C)  , LdR8R8(H,D)   , LdR8R8(H,E), LdR8R8(H,H), LdR8R8(H,L), NoImpl   , LdR8R8(H,A), LdR8R8(L,B), LdR8R8(L,C)       , LdR8R8(L,D)   , LdR8R8(L,E), LdR8R8(L,H), LdR8R8(L,L), NoImpl   , LdR8R8(L,A)],
            [NoImpl     , NoImpl       , NoImpl        , NoImpl     , NoImpl     , NoImpl     , NoImpl   , NoImpl     , LdR8R8(A,B), LdR8R8(A,C)       , LdR8R8(A,D)   , LdR8R8(A,E), LdR8R8(A,H), LdR8R8(A,L), NoImpl   , LdR8R8(A,A)],

            [AddAR8(B)  , AddAR8(C)    , AddAR8(D)     , AddAR8(E)  , AddAR8(H)  , AddAR8(L)  , NoImpl   , AddAR8(A)  , AdcAR8(B)  , AdcAR8(C)         , AdcAR8(D)     , AdcAR8(E)  , AdcAR8(H)  , AdcAR8(L)  , NoImpl   , AdcAR8(A)  ],
            [SubAR8(B)  , SubAR8(C)    , SubAR8(D)     , SubAR8(E)  , SubAR8(H)  , SubAR8(L)  , NoImpl   , SubAR8(A)  , SbcAR8(B)  , SbcAR8(C)         , SbcAR8(D)     , SbcAR8(E)  , SbcAR8(H)  , SbcAR8(L)  , NoImpl   , SbcAR8(A)  ],
            [AndAR8(B)  , AndAR8(C)    , AndAR8(D)     , AndAR8(E)  , AndAR8(H)  , AndAR8(L)  , NoImpl   , AndAR8(A)  , XorAR8(B)  , XorAR8(C)         , XorAR8(D)     , XorAR8(E)  , XorAR8(H)  , XorAR8(L)  , NoImpl   , XorAR8(A)  ],
            [OrAR8(B)   , OrAR8(C)     , OrAR8(D)      , OrAR8(E)   , OrAR8(H)   , OrAR8(L)   , NoImpl   , OrAR8(A)   , CpAR8(B)   , CpAR8(C)          , CpAR8(D)      , CpAR8(E)   , CpAR8(H)   , CpAR8(L)   , NoImpl   , CpAR8(A)   ],

            [NoImpl     , NoImpl       , NoImpl        , NoImpl     , NoImpl     , NoImpl     , AddAN8   , NoImpl     , NoImpl     , NoImpl            , NoImpl        , NoImpl     , NoImpl     , NoImpl     , AdcAN8   , NoImpl     ],
            [NoImpl     , NoImpl       , NoImpl        , Illegal    , NoImpl     , NoImpl     , SubAN8   , NoImpl     , NoImpl     , NoImpl            , NoImpl        , Illegal    , NoImpl     , Illegal    , SbcAN8   , NoImpl     ],
            [NoImpl     , NoImpl       , NoImpl        , Illegal    , Illegal    , NoImpl     , AndAN8   , NoImpl     , NoImpl     , NoImpl            , NoImpl        , Illegal    , Illegal    , Illegal    , XorAN8   , NoImpl     ],
            [NoImpl     , NoImpl       , NoImpl        , NoImpl     , Illegal    , NoImpl     , OrAN8    , NoImpl     , NoImpl     , NoImpl            , NoImpl        , NoImpl     , Illegal    , Illegal    , CpAN8    , NoImpl     ],
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
            CpuInstruction::LdR8N8(register) => {
                let value = self.fetch8(bus);
                self.registers.set(register, value);
                self.phase = CpuPhase::FetchOpcode;
            }
            CpuInstruction::AddAN8 => {
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

                let instruction = self.decode_instruction(opcode);
                match instruction {
                    CpuInstruction::LdR8N8(_) => {
                        self.phase = CpuPhase::FetchImmediateNumber(instruction);
                    }
                    CpuInstruction::AddAN8 => {
                        self.phase = CpuPhase::FetchImmediateNumber(instruction);
                    }
                    CpuInstruction::AddAR8(register) => {
                        // only 1 m-cycle instruction, so we can directly decode it
                        let value = self.registers.get(register);
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
            CpuPhase::FetchImmediateNumber(instruction) => {
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
