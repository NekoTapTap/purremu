use std::hash::RandomState;

use rand::RngExt;
use strum::IntoEnumIterator;

use crate::cpu::{Cpu, CpuArithmetic, CpuInstruction, CpuPhase, CpuRegister, CpuRegisters};
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
            CpuInstruction::AddAImm8 => 0xC6,
            CpuInstruction::AddAR(CpuRegister::A) => 0x87,
            CpuInstruction::AddAR(CpuRegister::B) => 0x80,
            CpuInstruction::AddAR(CpuRegister::C) => 0x81,
            CpuInstruction::AddAR(CpuRegister::D) => 0x82,
            CpuInstruction::AddAR(CpuRegister::E) => 0x83,
            CpuInstruction::AddAR(CpuRegister::H) => 0x84,
            CpuInstruction::AddAR(CpuRegister::L) => 0x85,
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

#[test]
fn test_cpu_add() {
    // normal add
    let a = 0x10u8;
    let b = 0x20u8;
    let (result, flags) = a.cpu_add(b);
    assert_eq!(result, 0x30);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, false);
    assert_eq!(flags.half_carry, false);
    assert_eq!(flags.carry, false);

    // add with half carry
    let a = 0x0Fu8;
    let b = 0x01u8;
    let (result, flags) = a.cpu_add(b);
    assert_eq!(result, 0x10);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, false);
    assert_eq!(flags.half_carry, true);
    assert_eq!(flags.carry, false);

    // add with carry but no zero
    let a = 0xFFu8;
    let b = 0x02u8;
    let (result, flags) = a.cpu_add(b);
    assert_eq!(result, 0x01);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, false);
    assert_eq!(flags.half_carry, true);
    assert_eq!(flags.carry, true);

    // add with zero result
    let a = 0xFFu8;
    let b = 0x01u8;
    let (result, flags) = a.cpu_add(b);
    assert_eq!(result, 0x00);
    assert_eq!(flags.zero, true);
    assert_eq!(flags.subtract, false);
    assert_eq!(flags.half_carry, true);
    assert_eq!(flags.carry, true);
}

#[test]
fn test_add_a_imm8() {
    let mut bus = MemoryBus::new();
    let initial_a_value = 0x10;
    let imm_value = 0x20;
    bus.rom[0x0000] = CpuInstruction::AddAImm8.into(); // ADD A, imm8
    bus.rom[0x0001] = imm_value;
    let mut cpu = Cpu::new();
    cpu.registers.set(CpuRegister::A, initial_a_value);

    cpu.step_cycle(&bus);
    assert_eq!(
        cpu.phase,
        CpuPhase::InstructionDecode(CpuInstruction::AddAImm8),
        "test failed for ADD A, imm8"
    );
    assert_eq!(cpu.registers.pc, 0x0001);

    cpu.step_cycle(&bus);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(
        cpu.registers.a,
        initial_a_value.overflowing_add(imm_value).0
    );
    assert_eq!(cpu.registers.pc, 0x0002);
}

#[test]
fn test_add_a_r() {
    for register in CpuRegister::iter() {
        let mut bus = MemoryBus::new();
        let initial_a_value = rand::random_range(u8::MIN..=u8::MAX);
        let r_value = if register == CpuRegister::A { // if register is A, use the same value as initial_a_value to test adding A to itself
            initial_a_value
        } else {
            rand::random_range(u8::MIN..=u8::MAX) // Random value for testing
        };
        bus.rom[0x0000] = CpuInstruction::AddAR(register).into(); // ADD A, r

        let mut cpu = Cpu::new();
        cpu.registers.set(CpuRegister::A, initial_a_value);
        cpu.registers.set(register, r_value);
        cpu.step_cycle(&bus);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for ADD A, {:?}",
            register
        );
        assert_eq!(
            cpu.registers.a,
            initial_a_value.overflowing_add(r_value).0,
            "register: {:?}, initial_a_value: {}, r_value: {}, result: {}",
            register,
            initial_a_value,
            r_value,
            cpu.registers.a
        );
        assert_eq!(cpu.registers.pc, 0x0001);
    }
}
