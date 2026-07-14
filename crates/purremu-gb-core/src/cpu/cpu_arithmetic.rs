use crate::cpu::registers::CpuFlagsReg;


pub(crate) trait CpuArithmetic
where
    Self: Sized,
{
    fn cpu_add(&self, value: Self, carry_flag: bool) -> (Self, CpuFlagsReg);
    fn cpu_sub(&self, value: Self, borrow_flag: bool) -> (Self, CpuFlagsReg);
    fn cpu_and(&self, value: Self) -> (Self, CpuFlagsReg);
    fn cpu_or(&self, value: Self) -> (Self, CpuFlagsReg);
    fn cpu_xor(&self, value: Self) -> (Self, CpuFlagsReg);
    fn cpu_inc(&self) -> (Self, CpuFlagsReg);
    fn cpu_dec(&self) -> (Self, CpuFlagsReg);
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

    fn cpu_inc(&self) -> (Self, CpuFlagsReg) {
        self.cpu_add(1, false)
    }

    fn cpu_dec(&self) -> (Self, CpuFlagsReg) {
        self.cpu_sub(1, false)
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

    fn cpu_inc(&self) -> (Self, CpuFlagsReg) {
        return self.cpu_add(1, false);
    }

    fn cpu_dec(&self) -> (Self, CpuFlagsReg) {
        self.cpu_sub(1, false)
    }
}
