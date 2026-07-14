#[rustfmt::skip]
#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub(crate) enum CpuReg8 {
    A,
    B, C,
    D, E,
    H, L,
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub(crate) enum CpuReg16 {
    BC,
    DE,
    HL,
    AF,
    SP,
}

pub(crate) struct CpuFlagsReg {
    pub(crate) zero: bool,
    pub(crate) subtract: bool,
    pub(crate) half_carry: bool,
    pub(crate) carry: bool,
}

impl CpuFlagsReg {
    pub(crate) fn new() -> Self {
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

impl From<&CpuFlagsReg> for u8 {
    #[rustfmt::skip]
    fn from(flags: &CpuFlagsReg) -> Self {
        (if flags.zero             { 0b1000_0000 } else { 0 })
            | (if flags.subtract   { 0b0100_0000 } else { 0 })
            | (if flags.half_carry { 0b0010_0000 } else { 0 })
            | (if flags.carry      { 0b0001_0000 } else { 0 })
    }
}

#[rustfmt::skip]
pub(crate) struct CpuRegisters {
    pub(crate) pc: u16,
    pub(crate) sp: u16,
    pub(crate) a: u8,
    pub(crate) b: u8, pub(crate) c: u8,
    pub(crate) d: u8, pub(crate) e: u8,
    pub(crate) h: u8, pub(crate) l: u8,
    pub(crate) f: CpuFlagsReg,
}

impl CpuRegisters {
    #[rustfmt::skip]
    pub(crate) fn new() -> Self {
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

    pub(crate) fn set_r8(&mut self, register: CpuReg8, value: u8) {
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

    pub(crate) fn get_r8(&self, register: CpuReg8) -> u8 {
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

    pub(crate) fn set_r16_low(&mut self, register: CpuReg16, value: u8) {
        match register {
            CpuReg16::BC => self.c = value,
            CpuReg16::DE => self.e = value,
            CpuReg16::HL => self.l = value,
            CpuReg16::AF => self.f = CpuFlagsReg::from(value),
            CpuReg16::SP => self.sp = (self.sp & 0xFF00) | (value as u16),
        }
    }

    pub(crate) fn set_r16_high(&mut self, register: CpuReg16, value: u8) {
        match register {
            CpuReg16::BC => self.b = value,
            CpuReg16::DE => self.d = value,
            CpuReg16::HL => self.h = value,
            CpuReg16::AF => self.a = value,
            CpuReg16::SP => self.sp = (self.sp & 0x00FF) | ((value as u16) << 8),
        }
    }

    pub(crate) fn set_r16(&mut self, register: CpuReg16, value: u16) {
        let high = (value >> 8) as u8;
        let low = (value & 0xFF) as u8;
        self.set_r16_high(register, high);
        self.set_r16_low(register, low);
    }

    pub(crate) fn get_r16(&self, register: CpuReg16) -> u16 {
        match register {
            CpuReg16::BC => ((self.b as u16) << 8) | (self.c as u16),
            CpuReg16::DE => ((self.d as u16) << 8) | (self.e as u16),
            CpuReg16::HL => ((self.h as u16) << 8) | (self.l as u16),
            CpuReg16::AF => ((self.a as u16) << 8) | (u8::from(&self.f) as u16),
            CpuReg16::SP => self.sp,
        }
    }
}
