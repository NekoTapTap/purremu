use crate::memory_bus::MemoryBus;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CpuPhase {
    FetchOpcode,
    FetchImm8(CpuInstruction),
    FetchImm16Low(CpuInstruction),
    FetchImm16High(CpuInstruction),
    FetchR16(CpuInstruction),
    FetchE8(CpuInstruction),
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
}

#[rustfmt::skip]
#[derive(PartialEq, Debug, Clone, Copy)]
// TODO: use another model to represent instructions, because this model represents too much operations but SM83 don't have that many operations.
pub enum CpuInstruction {
    LdR8Imm8(CpuReg8),
    LdR8R8(CpuReg8, CpuReg8),
    LdR16Imm16(CpuReg16),
    LdR16memA(CpuReg16),
    LdAR16mem(CpuReg16),
    LdHlMemR8(CpuReg8),
    LdHlMemImm8,

    IncR8(CpuReg8),
    IncR16(CpuReg8, CpuReg8),
    DecR8(CpuReg8),
    DecR16(CpuReg16),

    AddAImm8,
    AddAR8(CpuReg8),
    AddHlR16(CpuReg16),
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

    JrNzE8,
    JrZE8,
    JrNcE8,
    JrCE8,
    JrE8,
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

    fn set_r16_low(&mut self, register: CpuReg16, value: u8) {
        match register {
            CpuReg16::BC => self.c = value,
            CpuReg16::DE => self.e = value,
            CpuReg16::HL => self.l = value,
        }
    }

    fn set_r16_high(&mut self, register: CpuReg16, value: u8) {
        match register {
            CpuReg16::BC => self.b = value,
            CpuReg16::DE => self.d = value,
            CpuReg16::HL => self.h = value,
        }
    }

    fn set_r16(&mut self, register: CpuReg16, value: u16) {
        let high = (value >> 8) as u8;
        let low = (value & 0xFF) as u8;
        self.set_r16_high(register, high);
        self.set_r16_low(register, low);
    }

    fn get_r16(&self, register: CpuReg16) -> u16 {
        match register {
            CpuReg16::BC => ((self.b as u16) << 8) | (self.c as u16),
            CpuReg16::DE => ((self.d as u16) << 8) | (self.e as u16),
            CpuReg16::HL => ((self.h as u16) << 8) | (self.l as u16),
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
    fn cpu_add(&self, value: Self, carry_flag: bool) -> (Self, CpuFlagsReg);
    fn cpu_sub(&self, value: Self, borrow_flag: bool) -> (Self, CpuFlagsReg);
    fn cpu_and(&self, value: Self) -> (Self, CpuFlagsReg);
    fn cpu_or(&self, value: Self) -> (Self, CpuFlagsReg);
    fn cpu_xor(&self, value: Self) -> (Self, CpuFlagsReg);
}

impl CpuArithmetic for u8 {
    fn cpu_add(&self, value: u8, carry_flag: bool) -> (Self, CpuFlagsReg) {
        let carry = u8::from(carry_flag);
        let result_u16 = *self as u16 + value as u16 + carry as u16;
        let result_carry = result_u16 > 0xFF;
        let result = result_u16 as u8;

        let flags = CpuFlagsReg {
            zero: result == 0,
            subtract: false,
            half_carry: ((*self & 0x0F) + (value & 0x0F) + carry) > 0x0F, // For BCD
            carry: result_carry,
        };
        (result, flags)
    }

    fn cpu_sub(&self, value: u8, borrow_flag: bool) -> (Self, CpuFlagsReg) {
        let borrow = u8::from(borrow_flag);
        let result_u16 = (*self as u16).wrapping_sub(value as u16 + borrow as u16);
        let result = result_u16 as u8;
        let borrow_occurred = (*self as u16) < (value as u16 + borrow as u16);

        let flags = CpuFlagsReg {
            zero: result == 0,
            subtract: true,
            half_carry: (*self & 0x0F) < (value & 0x0F) + borrow, // For BCD
            carry: borrow_occurred,
        };
        (result, flags)
    }

    fn cpu_and(&self, value: u8) -> (Self, CpuFlagsReg) {
        let result = *self & value;
        let flags = CpuFlagsReg {
            zero: result == 0,
            subtract: false,
            half_carry: true, // For AND operation, H flag is set
            carry: false,
        };
        (result, flags)
    }

    fn cpu_or(&self, value: u8) -> (Self, CpuFlagsReg) {
        let result = *self | value;
        let flags = CpuFlagsReg {
            zero: result == 0,
            subtract: false,
            half_carry: false,
            carry: false,
        };
        (result, flags)
    }

    fn cpu_xor(&self, value: u8) -> (Self, CpuFlagsReg) {
        let result = *self ^ value;
        let flags = CpuFlagsReg {
            zero: result == 0,
            subtract: false,
            half_carry: false,
            carry: false,
        };
        (result, flags)
    }
}

impl CpuArithmetic for u16 {
    fn cpu_add(&self, value: u16, carry_flag: bool) -> (Self, CpuFlagsReg) {
        let carry = u16::from(carry_flag);
        let result_u32 = *self as u32 + value as u32 + carry as u32;
        let result_carry = result_u32 > 0xFFFF;
        let result = result_u32 as u16;

        let flags = CpuFlagsReg {
            zero: false, // 16-bit operations do not affect the Z flag
            subtract: false,
            half_carry: ((*self & 0x0FFF) + (value & 0x0FFF) + carry) > 0x0FFF, // For BCD
            carry: result_carry,
        };
        (result, flags)
    }

    fn cpu_sub(&self, value: u16, borrow_flag: bool) -> (Self, CpuFlagsReg) {
        let borrow = u16::from(borrow_flag);
        let result_u32 = (*self as u32).wrapping_sub(value as u32 + borrow as u32);
        let result = result_u32 as u16;
        let borrow_occurred = (*self as u32) < (value as u32 + borrow as u32);

        let flags = CpuFlagsReg {
            zero: false, // 16-bit operations do not affect the Z flag
            subtract: true,
            half_carry: (*self & 0x0FFF) < (value & 0x0FFF) + borrow, // For BCD
            carry: borrow_occurred,
        };
        (result, flags)
    }

    fn cpu_and(&self, _value: u16) -> (Self, CpuFlagsReg) {
        unimplemented!("AND operation is not defined for 16-bit values");
    }

    fn cpu_or(&self, _value: u16) -> (Self, CpuFlagsReg) {
        unimplemented!("OR operation is not defined for 16-bit values");
    }

    fn cpu_xor(&self, _value: u16) -> (Self, CpuFlagsReg) {
        unimplemented!("XOR operation is not defined for 16-bit values");
    }
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
            t_cycles_until_step: 0,
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
            [NoImpl     , LdR16Imm16(BC), LdR16memA(BC), IncR16(B,C), IncR8(B)   , DecR8(B)   , LdR8Imm8(B), NoImpl     , NoImpl     , AddHlR16(BC), LdAR16mem(BC), DecR16(BC), IncR8(C)   , DecR8(C)   , LdR8Imm8(C), NoImpl     ],
            [NoImpl     , LdR16Imm16(DE), LdR16memA(DE), IncR16(D,E), IncR8(D)   , DecR8(D)   , LdR8Imm8(D), NoImpl     , JrE8     , AddHlR16(DE), LdAR16mem(DE), DecR16(DE), IncR8(E)   , DecR8(E)   , LdR8Imm8(E), NoImpl     ],
            [JrNzE8     , LdR16Imm16(HL), LdR16memA(HL), IncR16(H,L), IncR8(H)   , DecR8(H)   , LdR8Imm8(H), NoImpl     , JrZE8     , AddHlR16(HL), LdAR16mem(HL), DecR16(HL), IncR8(L)   , DecR8(L)   , LdR8Imm8(L), NoImpl     ],
            [JrNcE8     , NoImpl       , NoImpl        , NoImpl     , NoImpl     , NoImpl     , LdHlMemImm8, NoImpl     , JrCE8     , NoImpl            , NoImpl        , NoImpl     , IncR8(A)   , DecR8(A)   , LdR8Imm8(A), NoImpl     ],

            [LdR8R8(B,B), LdR8R8(B,C)  , LdR8R8(B,D)   , LdR8R8(B,E), LdR8R8(B,H), LdR8R8(B,L), NoImpl   , LdR8R8(B,A), LdR8R8(C,B), LdR8R8(C,C)       , LdR8R8(C,D)   , LdR8R8(C,E), LdR8R8(C,H), LdR8R8(C,L), NoImpl   , LdR8R8(C,A)],
            [LdR8R8(D,B), LdR8R8(D,C)  , LdR8R8(D,D)   , LdR8R8(D,E), LdR8R8(D,H), LdR8R8(D,L), NoImpl   , LdR8R8(D,A), LdR8R8(E,B), LdR8R8(E,C)       , LdR8R8(E,D)   , LdR8R8(E,E), LdR8R8(E,H), LdR8R8(E,L), NoImpl   , LdR8R8(E,A)],
            [LdR8R8(H,B), LdR8R8(H,C)  , LdR8R8(H,D)   , LdR8R8(H,E), LdR8R8(H,H), LdR8R8(H,L), NoImpl   , LdR8R8(H,A), LdR8R8(L,B), LdR8R8(L,C)       , LdR8R8(L,D)   , LdR8R8(L,E), LdR8R8(L,H), LdR8R8(L,L), NoImpl   , LdR8R8(L,A)],
            [LdHlMemR8(B), LdHlMemR8(C), LdHlMemR8(D), LdHlMemR8(E) , LdHlMemR8(H),LdHlMemR8(L), NoImpl  , LdHlMemR8(A)     , LdR8R8(A,B), LdR8R8(A,C)       , LdR8R8(A,D)   , LdR8R8(A,E), LdR8R8(A,H), LdR8R8(A,L), NoImpl   , LdR8R8(A,A)],

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
        match instruction {
            CpuInstruction::JrNzE8 => {
                if !self.registers.f.zero {
                    self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                }
            }
            CpuInstruction::JrZE8 => {
                if self.registers.f.zero {
                    self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                }
            }
            CpuInstruction::JrNcE8 => {
                if !self.registers.f.carry {
                    self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                }
            }
            CpuInstruction::JrCE8 => {
                if self.registers.f.carry {
                    self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                }
            }
            CpuInstruction::JrE8 => {
                self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
            }
            _ => {
                panic!("No such instruction: {:?}", instruction);
            }
        }
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
            CpuInstruction::JrNzE8 | CpuInstruction::JrZE8 | CpuInstruction::JrNcE8 | CpuInstruction::JrCE8 | CpuInstruction::JrE8 => {
                self.phase = CpuPhase::FetchE8(instruction);
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

        self.t_cycles_until_step = 4;

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
