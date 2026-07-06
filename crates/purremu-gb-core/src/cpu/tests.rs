use std::hash::RandomState;

use rand::RngExt;
use strum::IntoEnumIterator;

use crate::cpu::{Cpu, CpuInstruction, CpuPhase, CpuRegister, CpuRegisters};
use crate::memory_bus::MemoryBus;

impl From<CpuInstruction> for u8 {
    fn from(instruction: CpuInstruction) -> Self {
        match instruction {
            CpuInstruction::LdRImm8(CpuRegister::A) => 0x3E,
            CpuInstruction::LdRImm8(CpuRegister::B) => 0x06,
            CpuInstruction::LdRImm8(CpuRegister::C) => 0x0E,
            CpuInstruction::LdRImm8(CpuRegister::D) => 0x16,
            CpuInstruction::LdRImm8(CpuRegister::E) => 0x1E,
            CpuInstruction::LdRImm8(CpuRegister::H) => 0x26,
            CpuInstruction::LdRImm8(CpuRegister::L) => 0x2E,
        }
    }
}

#[test]
fn test_ld_r_imm8() {
    for register in CpuRegister::iter() {
        let mut bus = MemoryBus::new();
        let expected_register_value = rand::random_range(u8::MIN..=u8::MAX); // Random value for testing
        bus.rom[0x0000] = CpuInstruction::LdRImm8(register).into(); // LD r, imm8
        bus.rom[0x0001] = expected_register_value;

        let mut cpu = Cpu::new();
        assert_eq!(cpu.phase, CpuPhase::FetchOpcode);

        cpu.step_cycle(&bus);
        assert_eq!(
            cpu.phase,
            CpuPhase::InstructionDecode(CpuInstruction::LdRImm8(register)),
            "test failed for register {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0001);

        cpu.step_cycle(&bus);
        assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
        match register {
            CpuRegister::A => assert_eq!(cpu.registers.a, expected_register_value),
            CpuRegister::B => assert_eq!(cpu.registers.b, expected_register_value),
            CpuRegister::C => assert_eq!(cpu.registers.c, expected_register_value),
            CpuRegister::D => assert_eq!(cpu.registers.d, expected_register_value),
            CpuRegister::E => assert_eq!(cpu.registers.e, expected_register_value),
            CpuRegister::H => assert_eq!(cpu.registers.h, expected_register_value),
            CpuRegister::L => assert_eq!(cpu.registers.l, expected_register_value),
        }
        assert_eq!(cpu.registers.pc, 0x0002);
        assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    }
}
