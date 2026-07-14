use strum::IntoEnumIterator;

use crate::cpu::instructions::{CpuCbInstruction, CpuCbOperand, CpuCbOperation, CpuCondition};
use crate::cpu::{Cpu, CpuArithmetic, CpuInstruction, CpuPhase, CpuReg8, CpuReg16};
use crate::memory_bus::MemoryBus;

fn rand_external_ram_addr() -> u16 {
    rand::random_range(0xA000..=0xFDFF)
}

fn cpu_step_n(cpu: &mut Cpu, bus: &mut MemoryBus, n: usize) {
    for _ in 0..n {
        cpu.step(bus);
    }
}

#[test]
fn test_cpu_advances_one_m_cycle_every_four_t_cycles() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();

    cpu_step_n(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.registers.pc, 0);

    cpu_step_n(&mut cpu, &mut bus, 1);
    assert_eq!(cpu.registers.pc, 1);

    cpu_step_n(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.registers.pc, 1);

    cpu_step_n(&mut cpu, &mut bus, 1);
    assert_eq!(cpu.registers.pc, 2);
}

#[test]
fn test_ld_r_imm8() {
    for register in CpuReg8::iter() {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let expected_register_value = rand::random_range(u8::MIN..=u8::MAX); // Random value for testing
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::LdR8Imm8(register)); // LD r, imm8
        bus.rom[0x0001] = expected_register_value;

        assert_eq!(cpu.phase, CpuPhase::FetchOpcode);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchImm8(CpuInstruction::LdR8Imm8(register)),
            "test failed for register {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0001);

        cpu_step_n(&mut cpu, &mut bus, 4);
        match register {
            CpuReg8::A => assert_eq!(cpu.registers.a, expected_register_value),
            CpuReg8::B => assert_eq!(cpu.registers.b, expected_register_value),
            CpuReg8::C => assert_eq!(cpu.registers.c, expected_register_value),
            CpuReg8::D => assert_eq!(cpu.registers.d, expected_register_value),
            CpuReg8::E => assert_eq!(cpu.registers.e, expected_register_value),
            CpuReg8::H => assert_eq!(cpu.registers.h, expected_register_value),
            CpuReg8::L => assert_eq!(cpu.registers.l, expected_register_value),
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
    let (result, flags) = a.cpu_add(b, false);
    assert_eq!(result, 0x30);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, false);
    assert_eq!(flags.half_carry, false);
    assert_eq!(flags.carry, false);

    // add with half carry
    let a = 0x0Fu8;
    let b = 0x01u8;
    let (result, flags) = a.cpu_add(b, false);
    assert_eq!(result, 0x10);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, false);
    assert_eq!(flags.half_carry, true);
    assert_eq!(flags.carry, false);

    // add with carry but no zero
    let a = 0xFFu8;
    let b = 0x02u8;
    let (result, flags) = a.cpu_add(b, false);
    assert_eq!(result, 0x01);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, false);
    assert_eq!(flags.half_carry, true);
    assert_eq!(flags.carry, true);

    // add with zero result
    let a = 0xFFu8;
    let b = 0x01u8;
    let (result, flags) = a.cpu_add(b, false);
    assert_eq!(result, 0x00);
    assert_eq!(flags.zero, true);
    assert_eq!(flags.subtract, false);
    assert_eq!(flags.half_carry, true);
    assert_eq!(flags.carry, true);

    // add with carry flag set
    let a = 0x10u8;
    let b = 0x20u8;
    let (result, flags) = a.cpu_add(b, true);
    assert_eq!(result, 0x31);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, false);
    assert_eq!(flags.half_carry, false);
    assert_eq!(flags.carry, false);
}

#[test]
fn test_add_a_imm8() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();

    let initial_a_value = 0x10;
    let imm_value = 0x20;
    bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::AddAImm8); // ADD A, imm8
    bus.rom[0x0001] = imm_value;
    cpu.registers.set_r8(CpuReg8::A, initial_a_value);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchImm8(CpuInstruction::AddAImm8),
        "test failed for ADD A, imm8"
    );
    assert_eq!(cpu.registers.pc, 0x0001);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(
        cpu.registers.a,
        initial_a_value.overflowing_add(imm_value).0
    );
    assert_eq!(cpu.registers.pc, 0x0002);
}

#[test]
fn test_add_a_r() {
    for register in CpuReg8::iter() {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let initial_a_value = rand::random_range(u8::MIN..=u8::MAX);
        let r_value = if register == CpuReg8::A {
            // if register is A, use the same value as initial_a_value to test adding A to itself
            initial_a_value
        } else {
            rand::random_range(u8::MIN..=u8::MAX) // Random value for testing
        };
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::AddAR8(register)); // ADD A, r

        cpu.registers.set_r8(CpuReg8::A, initial_a_value);
        cpu.registers.set_r8(register, r_value);
        cpu_step_n(&mut cpu, &mut bus, 4);
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

#[test]
fn test_cpu_sub() {
    // normal sub
    let a = 0x20u8;
    let b = 0x10u8;
    let (result, flags) = a.cpu_sub(b, false);
    assert_eq!(result, 0x10);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, true);
    assert_eq!(flags.half_carry, false);
    assert_eq!(flags.carry, false);

    // sub with half carry
    let a = 0x10u8;
    let b = 0x01u8;
    let (result, flags) = a.cpu_sub(b, false);
    assert_eq!(result, 0x0F);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, true);
    assert_eq!(flags.half_carry, true);
    assert_eq!(flags.carry, false);

    // sub with borrow but no zero
    let a = 0x01u8;
    let b = 0x02u8;
    let (result, flags) = a.cpu_sub(b, false);
    assert_eq!(result, 0xFF);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, true);
    assert_eq!(flags.half_carry, true);
    assert_eq!(flags.carry, true);

    // sub with zero result
    let a = 0x01u8;
    let b = 0x01u8;
    let (result, flags) = a.cpu_sub(b, false);
    assert_eq!(result, 0x00);
    assert_eq!(flags.zero, true);
    assert_eq!(flags.subtract, true);
    assert_eq!(flags.half_carry, false);
    assert_eq!(flags.carry, false);

    // sub with carry flag set
    let a = 0x20u8;
    let b = 0x10u8;
    let (result, flags) = a.cpu_sub(b, true);
    assert_eq!(result, 0x0F);
    assert_eq!(flags.zero, false);
    assert_eq!(flags.subtract, true);
    assert_eq!(flags.half_carry, true);
    assert_eq!(flags.carry, false);
}

#[test]
fn test_sub_a_imm8() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    let initial_a_value = 0x10;
    let imm_value = 0x20;
    bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::SubAImm8); // SUB A, imm8
    bus.rom[0x0001] = imm_value;
    cpu.registers.set_r8(CpuReg8::A, initial_a_value);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchImm8(CpuInstruction::SubAImm8),
        "test failed for SUB A, imm8"
    );
    assert_eq!(cpu.registers.pc, 0x0001);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(
        cpu.registers.a,
        initial_a_value.overflowing_sub(imm_value).0
    );
    assert_eq!(cpu.registers.pc, 0x0002);
}

#[test]
fn test_sub_a_r() {
    for register in CpuReg8::iter() {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let initial_a_value = rand::random_range(u8::MIN..=u8::MAX);
        let r_value = if register == CpuReg8::A {
            // if register is A, use the same value as initial_a_value to test subtracting A from itself
            initial_a_value
        } else {
            rand::random_range(u8::MIN..=u8::MAX) // Random value for testing
        };
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::SubAR8(register)); // SUB A, r

        cpu.registers.set_r8(CpuReg8::A, initial_a_value);
        cpu.registers.set_r8(register, r_value);
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for SUB A, {:?}",
            register
        );
        assert_eq!(
            cpu.registers.a,
            initial_a_value.overflowing_sub(r_value).0,
            "register: {:?}, initial_a_value: {}, r_value: {}, result: {}",
            register,
            initial_a_value,
            r_value,
            cpu.registers.a
        );
        assert_eq!(cpu.registers.pc, 0x0001);
    }
}

#[test]
fn test_ld_r16_imm16() {
    for register in [CpuReg16::BC, CpuReg16::DE, CpuReg16::HL] {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let low_byte = rand::random_range(u8::MIN..=u8::MAX);
        let high_byte = rand::random_range(u8::MIN..=u8::MAX);
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::LdR16Imm16(register)); // LD rr, imm16
        bus.rom[0x0001] = low_byte;
        bus.rom[0x0002] = high_byte;

        assert_eq!(cpu.phase, CpuPhase::FetchOpcode);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchImm16Low(CpuInstruction::LdR16Imm16(register)),
            "test failed for register {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0001);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchImm16High(CpuInstruction::LdR16Imm16(register)),
            "test failed for register {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0002);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for register {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0003);
        assert_eq!(
            cpu.registers.get_r16(register),
            ((high_byte as u16) << 8) | (low_byte as u16),
            "test failed for register {:?}",
            register
        );
    }
}

#[test]
fn test_and_a_r8() {
    for register in CpuReg8::iter() {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let initial_a_value = rand::random_range(u8::MIN..=u8::MAX);
        let r_value = if register == CpuReg8::A {
            // if register is A, use the same value as initial_a_value to test subtracting A from itself
            initial_a_value
        } else {
            rand::random_range(u8::MIN..=u8::MAX) // Random value for testing
        };
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::AndAR8(register)); // AND A, r

        cpu.registers.set_r8(CpuReg8::A, initial_a_value);
        cpu.registers.set_r8(register, r_value);
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for AND A, {:?}",
            register
        );
        assert_eq!(
            cpu.registers.a,
            initial_a_value & r_value,
            "register: {:?}
 initial_a_value: {:08b}
 r_value:         {:08b}
 result:          {:08b}",
            register,
            initial_a_value,
            r_value,
            cpu.registers.a
        );
        assert_eq!(cpu.registers.pc, 0x0001);
    }
}

#[test]
fn test_or_a_r8() {
    for register in CpuReg8::iter() {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let initial_a_value = rand::random_range(u8::MIN..=u8::MAX);
        let r_value = if register == CpuReg8::A {
            // if register is A, use the same value as initial_a_value to test subtracting A from itself
            initial_a_value
        } else {
            rand::random_range(u8::MIN..=u8::MAX) // Random value for testing
        };
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::OrAR8(register)); // OR A, r

        cpu.registers.set_r8(CpuReg8::A, initial_a_value);
        cpu.registers.set_r8(register, r_value);
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for OR A, {:?}",
            register
        );
        assert_eq!(
            cpu.registers.a,
            initial_a_value | r_value,
            "register: {:?}
 initial_a_value: {:08b}
 r_value:         {:08b}
 result:          {:08b}",
            register,
            initial_a_value,
            r_value,
            cpu.registers.a
        );
        assert_eq!(cpu.registers.pc, 0x0001);
    }
}

#[test]
fn test_xor_a_r8() {
    for register in CpuReg8::iter() {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let initial_a_value = rand::random_range(u8::MIN..=u8::MAX);
        let r_value = if register == CpuReg8::A {
            // if register is A, use the same value as initial_a_value to test subtracting A from itself
            initial_a_value
        } else {
            rand::random_range(u8::MIN..=u8::MAX) // Random value for testing
        };
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::XorAR8(register)); // XOR A, r

        cpu.registers.set_r8(CpuReg8::A, initial_a_value);
        cpu.registers.set_r8(register, r_value);
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for XOR A, {:?}",
            register
        );
        assert_eq!(
            cpu.registers.a,
            initial_a_value ^ r_value,
            "register: {:?}
 initial_a_value: {:08b}
 r_value:         {:08b}
 result:          {:08b}",
            register,
            initial_a_value,
            r_value,
            cpu.registers.a
        );
        assert_eq!(cpu.registers.pc, 0x0001);
    }
}

#[test]
fn test_and_a_imm8() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();

    let initial_a_value = rand::random_range(u8::MIN..=u8::MAX);
    let imm_value = rand::random_range(u8::MIN..=u8::MAX);
    bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::AndAImm8); // AND A, imm8
    bus.rom[0x0001] = imm_value;

    cpu.registers.set_r8(CpuReg8::A, initial_a_value);
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchImm8(CpuInstruction::AndAImm8),
        "test failed for AND A, imm8"
    );
    assert_eq!(cpu.registers.pc, 0x0001);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(
        cpu.registers.a,
        initial_a_value & imm_value,
        "initial_a_value: {:08b}
 r_value:         {:08b}
 result:          {:08b}",
        initial_a_value,
        imm_value,
        cpu.registers.a
    );
    assert_eq!(cpu.registers.pc, 0x0002);
}

#[test]
fn test_or_a_imm8() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();

    let initial_a_value = rand::random_range(u8::MIN..=u8::MAX);
    let imm_value = rand::random_range(u8::MIN..=u8::MAX);
    bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::OrAImm8); // OR A, imm8
    bus.rom[0x0001] = imm_value;

    cpu.registers.set_r8(CpuReg8::A, initial_a_value);
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchImm8(CpuInstruction::OrAImm8),
        "test failed for OR A, imm8"
    );
    assert_eq!(cpu.registers.pc, 0x0001);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(
        cpu.registers.a,
        initial_a_value | imm_value,
        "initial_a_value: {:08b}
 r_value:         {:08b}
 result:          {:08b}",
        initial_a_value,
        imm_value,
        cpu.registers.a
    );
    assert_eq!(cpu.registers.pc, 0x0002);
}

#[test]
fn test_xor_a_imm8() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();

    let initial_a_value = rand::random_range(u8::MIN..=u8::MAX);
    let imm_value = rand::random_range(u8::MIN..=u8::MAX);
    bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::XorAImm8); // XOR A, imm8
    bus.rom[0x0001] = imm_value;

    cpu.registers.set_r8(CpuReg8::A, initial_a_value);
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchImm8(CpuInstruction::XorAImm8),
        "test failed for XOR A, imm8"
    );
    assert_eq!(cpu.registers.pc, 0x0001);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(
        cpu.registers.a,
        initial_a_value ^ imm_value,
        "initial_a_value: {:08b}
 r_value:         {:08b}
 result:          {:08b}",
        initial_a_value,
        imm_value,
        cpu.registers.a
    );
    assert_eq!(cpu.registers.pc, 0x0002);
}

#[test]
fn test_add_hl_r16() {
    for register in [CpuReg16::BC, CpuReg16::DE, CpuReg16::HL] {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let src_value = rand::random_range(u16::MIN..=u16::MAX);
        let dest_value = if register == CpuReg16::HL {
            // If the register is HL, we want to test adding HL to itself, so we set src_value to dest_value
            src_value
        } else {
            rand::random_range(u16::MIN..=u16::MAX)
        };
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::AddHlR16(register)); // ADD HL, rr

        cpu.registers.set_r16(CpuReg16::HL, dest_value);
        cpu.registers.set_r16(register, src_value);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchR16(CpuInstruction::AddHlR16(register)),
            "test failed for ADD HL, {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0001);
        assert_eq!(
            cpu.registers.get_r16(CpuReg16::HL),
            dest_value,
            "don't touch the HL register yet, we need to simulate this 1 byte instruction taking 2 cycles"
        );

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for ADD HL, {:?}",
            register
        );
        assert_eq!(
            cpu.registers.get_r16(CpuReg16::HL),
            dest_value.overflowing_add(src_value).0,
            "dest_register: {:?}, dest_value: {}, src_register: {:?}, src_value: {}, result: {}",
            CpuReg16::HL,
            dest_value,
            register,
            src_value,
            cpu.registers.get_r16(CpuReg16::HL)
        );
        assert_eq!(cpu.registers.pc, 0x0001);
    }
}

#[test]
fn test_ld_a_r16mem() {
    use CpuReg8::A;

    for r16 in [CpuReg16::BC, CpuReg16::DE] {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let r16_value = rand_external_ram_addr(); // Random value in the range of external RAM
        let mem_value = rand::random_range(u8::MIN..=u8::MAX);
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::LdAR16mem(r16)); // LD A, (rr)
        bus.write8(r16_value, mem_value);

        cpu.registers.set_r16(r16, r16_value);
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchR16(CpuInstruction::LdAR16mem(r16)),
            "test failed for LD {:?}, ({:?})",
            A,
            r16
        );
        assert_eq!(cpu.registers.pc, 0x0001);
        assert_eq!(
            cpu.registers.get_r8(A),
            0,
            "don't touch the {:?} register yet, we need to simulate this 1 byte instruction taking 2 cycles",
            A
        );

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for LD {:?}, ({:?})",
            A,
            r16
        );
        assert_eq!(
            cpu.registers.get_r8(A),
            mem_value,
            "register: {:?}, r16: {:?}, r16_value: {}, mem_value: {}, result: {}",
            A,
            r16,
            r16_value,
            mem_value,
            cpu.registers.get_r8(A)
        );
        assert_eq!(cpu.registers.pc, 0x0001);
    }
}

#[test]
fn test_ld_hl_mem_r8() {
    for register in CpuReg8::iter() {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        let hl_value = rand_external_ram_addr(); // Random value in the range of external RAM
        let r_value = if register == CpuReg8::H {
            (hl_value >> 8) as u8 // Use the high byte of hl_value for testing
        } else if register == CpuReg8::L {
            (hl_value & 0xFF) as u8 // Use the low byte of hl_value for testing
        } else {
            rand::random_range(u8::MIN..=u8::MAX)
        };
        bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::LdHlMemR8(register)); // LD (HL), r

        cpu.registers.set_r16(CpuReg16::HL, hl_value);
        cpu.registers.set_r8(register, r_value);
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchR16(CpuInstruction::LdHlMemR8(register)),
            "test failed for LD (HL), {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0001);
        assert_eq!(
            bus.read8(hl_value),
            0,
            "don't touch the memory at HL yet, we need to simulate this 1 byte instruction taking 2 cycles"
        );

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for LD (HL), {:?}",
            register
        );
        assert_eq!(
            bus.read8(hl_value),
            r_value,
            "register: {:?}, r_value: {}, hl: {}, result: {}",
            register,
            r_value,
            hl_value,
            bus.read8(hl_value)
        );
        assert_eq!(cpu.registers.pc, 0x0001);
    }
}

#[test]
fn test_ld_hl_mem_imm8() {
    let instruction = CpuInstruction::LdHlMemImm8;
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();

    let hl_value = rand_external_ram_addr(); // Random value in the range of external RAM
    let imm_value = rand::random_range(u8::MIN..=u8::MAX);
    bus.rom[0x0000] = cpu.encode_instruction(instruction); // LD (HL), imm8
    bus.rom[0x0001] = imm_value;

    cpu.registers.set_r16(CpuReg16::HL, hl_value);
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchR16(instruction),
        "test failed for LD (HL), imm8"
    );
    assert_eq!(cpu.registers.pc, 0x0001);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchImm8(instruction),
        "test failed for LD (HL), imm8"
    );
    assert_eq!(bus.read8(hl_value), 0);
    assert_eq!(cpu.registers.pc, 0x0001);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchOpcode,
        "test failed for LD (HL), imm8"
    );
    assert_eq!(bus.read8(hl_value), imm_value);
    assert_eq!(cpu.registers.pc, 0x0002);
}

#[test]
fn test_jr_e8() {
    // jump forward
    {
        let instruction = CpuInstruction::JrE8;
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        bus.rom[0x0000] = cpu.encode_instruction(instruction); // JR e8
        bus.rom[0x0001] = 2;
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchE8(instruction),
            "test failed for JR e8"
        );
        assert_eq!(cpu.registers.pc, 0x0001);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_ne!(cpu.phase, CpuPhase::FetchOpcode, "test failed for JR e8");
        assert_eq!(cpu.registers.pc, 0x0002);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(cpu.phase, CpuPhase::FetchOpcode, "test failed for JR e8");
        assert_eq!(cpu.registers.pc, 0x0004);
    }
    // jump backward
    {
        let instruction = CpuInstruction::JrE8;
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();

        bus.rom[0x0000] = cpu.encode_instruction(instruction); // JR e8
        bus.rom[0x0001] = (-2i8) as u8; // -2 in two's complement
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchE8(instruction),
            "test failed for JR e8"
        );
        assert_eq!(cpu.registers.pc, 0x0001);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_ne!(cpu.phase, CpuPhase::FetchOpcode, "test failed for JR e8");
        assert_eq!(cpu.registers.pc, 0x0002);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(cpu.phase, CpuPhase::FetchOpcode, "test failed for JR e8");
        assert_eq!(cpu.registers.pc, 0);
    }
}

#[test]
fn test_jr_cc_e8() {
    let conditions = [
        (CpuInstruction::JrNzE8, false),
        (CpuInstruction::JrZE8, true),
        (CpuInstruction::JrNcE8, false),
        (CpuInstruction::JrCE8, true),
    ];

    for (instruction, flag_when_met) in conditions {
        for condition_met in [true, false] {
            let mut bus = MemoryBus::new(vec![0; 0x8000]);
            let mut cpu = Cpu::new();
            let flag = condition_met == flag_when_met;

            match instruction {
                CpuInstruction::JrNzE8 | CpuInstruction::JrZE8 => cpu.registers.f.zero = flag,
                CpuInstruction::JrNcE8 | CpuInstruction::JrCE8 => {
                    cpu.registers.f.carry = flag;
                }
                _ => unreachable!(),
            }

            bus.rom[0x0000] = cpu.encode_instruction(instruction);
            bus.rom[0x0001] = 2;
            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(
                cpu.phase,
                CpuPhase::FetchE8(instruction),
                "test failed for {instruction:?}, condition_met={condition_met}"
            );
            assert_eq!(cpu.registers.pc, 0x0001);

            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(cpu.registers.pc, 0x0002);

            if condition_met {
                assert_ne!(
                    cpu.phase,
                    CpuPhase::FetchOpcode,
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );
                cpu_step_n(&mut cpu, &mut bus, 4);
                assert_eq!(cpu.registers.pc, 0x0004);
            }

            assert_eq!(
                cpu.phase,
                CpuPhase::FetchOpcode,
                "test failed for {instruction:?}, condition_met={condition_met}"
            );
        }
    }
}

#[test]
fn test_jp_a16() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    bus.rom[0x0000] = cpu.encode_instruction(CpuInstruction::JpA16);
    bus.rom[0x0001] = 0x34;
    bus.rom[0x0002] = 0x12;

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0001);
    assert_eq!(cpu.phase, CpuPhase::FetchA16Low(CpuInstruction::JpA16));

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0002);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchA16High(CpuInstruction::JpA16, 0x34),
    );

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0003);
    assert_eq!(cpu.phase, CpuPhase::ApplyAbsoluteJump(0x1234));

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
}

#[test]
fn test_jp_cc_a16() {
    let conditions = [
        (CpuInstruction::JpNzA16, false),
        (CpuInstruction::JpZA16, true),
        (CpuInstruction::JpNcA16, false),
        (CpuInstruction::JpCA16, true),
    ];

    for (instruction, flag_when_met) in conditions {
        for condition_met in [true, false] {
            let mut bus = MemoryBus::new(vec![0; 0x8000]);
            let mut cpu = Cpu::new();
            let flag = condition_met == flag_when_met;

            match instruction {
                CpuInstruction::JpNzA16 | CpuInstruction::JpZA16 => cpu.registers.f.zero = flag,
                CpuInstruction::JpNcA16 | CpuInstruction::JpCA16 => {
                    cpu.registers.f.carry = flag;
                }
                _ => unreachable!(),
            }

            bus.rom[0x0000] = cpu.encode_instruction(instruction);
            bus.rom[0x0001] = 0x34;
            bus.rom[0x0002] = 0x12;

            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(
                cpu.phase,
                CpuPhase::FetchA16Low(instruction),
                "test failed for {instruction:?}, condition_met={condition_met}"
            );
            assert_eq!(cpu.registers.pc, 0x0001);

            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(cpu.registers.pc, 0x0002);
            assert_eq!(
                cpu.phase,
                CpuPhase::FetchA16High(instruction, 0x34),
                "test failed for {instruction:?}, condition_met={condition_met}"
            );

            if condition_met {
                cpu_step_n(&mut cpu, &mut bus, 4);
                assert_eq!(cpu.registers.pc, 0x0003);
                assert_eq!(
                    cpu.phase,
                    CpuPhase::ApplyAbsoluteJump(0x1234),
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );
                cpu_step_n(&mut cpu, &mut bus, 4);
                assert_eq!(cpu.registers.pc, 0x1234);
                assert_eq!(
                    cpu.phase,
                    CpuPhase::FetchOpcode,
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );

                continue;
            }

            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(cpu.registers.pc, 0x0003);
            assert_eq!(
                cpu.phase,
                CpuPhase::FetchOpcode,
                "test failed for {instruction:?}, condition_met={condition_met}"
            );
        }
    }
}

#[test]
fn test_jp_hl() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    let hl_value = 0xC000;
    cpu.registers.set_r16(CpuReg16::HL, hl_value);
    bus.rom[0x0000] = 0xE9;

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, hl_value);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
}

#[test]
fn test_call_a16() {
    let instruction = CpuInstruction::CallA16;

    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new_post_boot();
    bus.rom[0x0100] = cpu.encode_instruction(instruction);
    bus.rom[0x0101] = 0x34;
    bus.rom[0x0102] = 0x12;

    // M1: Fetch opcode
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0101);
    assert_eq!(cpu.phase, CpuPhase::FetchA16Low(instruction));

    // M2: Fetch low byte of address
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0102);
    assert_eq!(cpu.phase, CpuPhase::FetchA16High(instruction, 0x34));

    // M3: Fetch high byte of address
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0103);
    assert_eq!(
        cpu.phase,
        CpuPhase::DecrementSpForWrite(instruction, 0x1234),
        "test failed for CALL a16"
    );

    // M4: SP -= 1
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0103);
    assert_eq!(cpu.registers.sp, 0xFFFD);
    assert_eq!(
        cpu.phase,
        CpuPhase::WriteSpMemHigh(instruction, 0x1234),
        "test failed for CALL a16"
    );

    // M5: Set [SP] to high byte of return address, SP -= 1
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0103);
    assert_eq!(cpu.registers.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFD), 0x01);
    assert_eq!(
        cpu.phase,
        CpuPhase::WriteSpMemLow(instruction, 0x1234),
        "test failed for CALL a16"
    );

    // M6: Set [SP] to low byte of return address, PC = a16, Phase = FetchOpcode
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFC), 0x03);
    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
}

#[test]
fn test_call_with_condition() {
    let conditions = [
        (CpuInstruction::CallNzA16, false),
        (CpuInstruction::CallZA16, true),
        (CpuInstruction::CallNcA16, false),
        (CpuInstruction::CallCA16, true),
    ];

    for (instruction, flag_when_met) in conditions {
        for condition_met in [true, false] {
            let mut bus = MemoryBus::new(vec![0; 0x8000]);
            let mut cpu = Cpu::new_post_boot();
            let flag = condition_met == flag_when_met;

            match instruction {
                CpuInstruction::CallNzA16 | CpuInstruction::CallZA16 => cpu.registers.f.zero = flag,
                CpuInstruction::CallNcA16 | CpuInstruction::CallCA16 => {
                    cpu.registers.f.carry = flag;
                }
                _ => unreachable!(),
            }

            bus.rom[0x0100] = cpu.encode_instruction(instruction);
            bus.rom[0x0101] = 0x34;
            bus.rom[0x0102] = 0x12;

            // M1: Fetch opcode
            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(
                cpu.phase,
                CpuPhase::FetchA16Low(instruction),
                "test failed for {instruction:?}, condition_met={condition_met}"
            );
            assert_eq!(cpu.registers.pc, 0x0101);

            // M2: Fetch low byte of address
            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(
                cpu.phase,
                CpuPhase::FetchA16High(instruction, 0x34),
                "test failed for {instruction:?}, condition_met={condition_met}"
            );
            assert_eq!(cpu.registers.pc, 0x0102);

            // M3: Fetch high byte of address
            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(cpu.registers.pc, 0x0103);
            if condition_met {
                assert_eq!(
                    cpu.phase,
                    CpuPhase::DecrementSpForWrite(instruction, 0x1234),
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );
                // M4: SP -= 1
                cpu_step_n(&mut cpu, &mut bus, 4);
                assert_eq!(cpu.registers.pc, 0x0103);
                assert_eq!(cpu.registers.sp, 0xFFFD);
                assert_eq!(
                    cpu.phase,
                    CpuPhase::WriteSpMemHigh(instruction, 0x1234),
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );

                // M5: Set [SP] to high byte of return address, SP -= 1
                cpu_step_n(&mut cpu, &mut bus, 4);
                assert_eq!(cpu.registers.pc, 0x0103);
                assert_eq!(cpu.registers.sp, 0xFFFC);
                assert_eq!(bus.read8(0xFFFD), 0x01);
                assert_eq!(
                    cpu.phase,
                    CpuPhase::WriteSpMemLow(instruction, 0x1234),
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );

                // M6: Set [SP] to low byte of return address, PC = a16, Phase = FetchOpcode
                cpu_step_n(&mut cpu, &mut bus, 4);
                assert_eq!(cpu.registers.sp, 0xFFFC);
                assert_eq!(bus.read8(0xFFFC), 0x03);
                assert_eq!(cpu.registers.pc, 0x1234);
                assert_eq!(
                    cpu.phase,
                    CpuPhase::FetchOpcode,
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );

                continue;
            }

            assert_eq!(
                cpu.phase,
                CpuPhase::FetchOpcode,
                "test failed for {instruction:?}, condition_met={condition_met}"
            );
            assert_eq!(cpu.registers.pc, 0x0103);
        }
    }
}

#[test]
fn test_ret() {
    let instruction = CpuInstruction::Ret(CpuCondition::None);
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new_post_boot();
    cpu.registers.sp = 0xFFFC;

    bus.rom[0x0100] = cpu.encode_instruction(instruction);
    bus.write8(0xFFFC, 0x34); // low byte of return address
    bus.write8(0xFFFD, 0x12); // high byte of return address

    // M1: Fetch opcode
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::ReadSpLow(instruction),
        "test failed for RET"
    );
    assert_eq!(cpu.registers.pc, 0x0101);
    assert_eq!(cpu.registers.sp, 0xFFFC);

    // M2: Read low byte of return address from [SP], SP += 1
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::ReadSpHigh(instruction, 0x34),
        "test failed for RET"
    );
    assert_eq!(cpu.registers.pc, 0x0101);
    assert_eq!(cpu.registers.sp, 0xFFFD);

    // M3: Read high byte of return address from [SP], SP += 1
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::ApplyAbsoluteJump(0x1234),
        "test failed for RET"
    );
    assert_eq!(cpu.registers.sp, 0xFFFE);
    assert_eq!(cpu.registers.pc, 0x0101);

    // M4: Set PC to return address, Phase = FetchOpcode
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.sp, 0xFFFE);
    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
}

#[test]
fn test_ret_i() {
    let instruction = CpuInstruction::RetI;
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new_post_boot();
    cpu.registers.sp = 0xFFFC;

    bus.rom[0x0100] = cpu.encode_instruction(instruction);
    bus.write8(0xFFFC, 0x34); // low byte of return address
    bus.write8(0xFFFD, 0x12); // high byte of return address

    // M1: Fetch opcode
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::ReadSpLow(instruction),
        "test failed for RETI"
    );
    assert_eq!(cpu.registers.pc, 0x0101);
    assert_eq!(cpu.registers.sp, 0xFFFC);

    // M2: Read low byte of return address from [SP], SP += 1
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::ReadSpHigh(instruction, 0x34),
        "test failed for RETI"
    );
    assert_eq!(cpu.registers.pc, 0x0101);
    assert_eq!(cpu.registers.sp, 0xFFFD);

    // M3: Read high byte of return address from [SP], SP += 1
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::ApplyAbsoluteJumpEnableInterrupts(0x1234),
        "test failed for RETI"
    );
    assert_eq!(cpu.registers.sp, 0xFFFE);
    assert_eq!(cpu.registers.pc, 0x0101);

    // M4: Set PC to return address, Phase = FetchOpcode, and enable interrupts
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.sp, 0xFFFE);
    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.ime, true);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
}

#[test]
fn test_ret_with_condition() {
    let conditions = [
        (CpuCondition::NZ, false),
        (CpuCondition::Z, true),
        (CpuCondition::NC, false),
        (CpuCondition::C, true),
    ];

    for (condition, flag_when_met) in conditions {
        for condition_met in [true, false] {
            let mut bus = MemoryBus::new(vec![0; 0x8000]);
            let mut cpu = Cpu::new_post_boot();
            cpu.registers.sp = 0xFFFC;
            let flag = condition_met == flag_when_met;

            match condition {
                CpuCondition::NZ | CpuCondition::Z => cpu.registers.f.zero = flag,
                CpuCondition::NC | CpuCondition::C => {
                    cpu.registers.f.carry = flag;
                }
                _ => unreachable!(),
            }

            let instruction = CpuInstruction::Ret(condition);

            bus.rom[0x0100] = cpu.encode_instruction(instruction);
            bus.write8(0xFFFC, 0x34); // low byte of return address
            bus.write8(0xFFFD, 0x12); // high byte of return address

            // M1: Fetch opcode
            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(cpu.registers.pc, 0x0101);
            assert_eq!(cpu.registers.sp, 0xFFFC);
            assert_eq!(
                cpu.phase,
                CpuPhase::CheckRetCondition(condition),
                "test failed for {instruction:?}, condition_met={condition_met}"
            );

            // M2: Check condition
            cpu_step_n(&mut cpu, &mut bus, 4);
            assert_eq!(cpu.registers.pc, 0x0101);
            assert_eq!(cpu.registers.sp, 0xFFFC);

            if condition_met {
                // M3: Read low byte of return address from [SP], SP += 1
                cpu_step_n(&mut cpu, &mut bus, 4);
                assert_eq!(
                    cpu.phase,
                    CpuPhase::ReadSpHigh(instruction, 0x34),
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );
                assert_eq!(cpu.registers.pc, 0x0101);
                assert_eq!(cpu.registers.sp, 0xFFFD);

                // M4: Read high byte of return address from [SP], SP += 1
                cpu_step_n(&mut cpu, &mut bus, 4);
                assert_eq!(
                    cpu.phase,
                    CpuPhase::ApplyAbsoluteJump(0x1234),
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );
                assert_eq!(cpu.registers.sp, 0xFFFE);
                assert_eq!(cpu.registers.pc, 0x0101);

                // M5: Set PC to return address, Phase = FetchOpcode
                cpu_step_n(&mut cpu, &mut bus, 4);
                assert_eq!(cpu.registers.sp, 0xFFFE);
                assert_eq!(cpu.registers.pc, 0x1234);
                assert_eq!(
                    cpu.phase,
                    CpuPhase::FetchOpcode,
                    "test failed for {instruction:?}, condition_met={condition_met}"
                );
                continue;
            }

            // M2: condition not met, so we skip the return and just fetch the next opcode
            assert_eq!(
                cpu.phase,
                CpuPhase::FetchOpcode,
                "test failed for {instruction:?}, condition_met={condition_met}"
            );
        }
    }
}

#[test]
fn test_ld_sp_imm16() {
    let instruction = CpuInstruction::LdSpImm16;
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new_post_boot();
    bus.rom[0x0100] = cpu.encode_instruction(instruction);
    bus.rom[0x0101] = 0x34;
    bus.rom[0x0102] = 0x12;

    // M1: Fetch opcode
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0101);
    assert_eq!(cpu.phase, CpuPhase::FetchImm16Low(instruction));

    // M2: Fetch low byte of immediate value
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0102);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchImm16High(instruction),
        "test failed for LD SP, imm16"
    );

    // M3: Fetch high byte of immediate value and set SP
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.pc, 0x0103);
    assert_eq!(
        cpu.registers.sp, 0x1234,
        "test failed for LD SP, imm16: expected SP=0x1234 but got SP={:04X}",
        cpu.registers.sp
    );
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
}

#[test]
fn test_push_r16() {
    for register in [CpuReg16::BC, CpuReg16::DE, CpuReg16::HL] {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new_post_boot();
        let r16_value = rand::random_range(u16::MIN..=u16::MAX);
        cpu.registers.set_r16(register, r16_value);
        cpu.registers.sp = 0xFFFC;
        bus.rom[0x0100] = cpu.encode_instruction(CpuInstruction::PushR16(register));

        // M1: Fetch opcode
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::DecrementSp(register),
            "test failed for PUSH {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(cpu.registers.sp, 0xFFFC);

        // M2: SP -= 1
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::PushR16High(register),
            "test failed for PUSH {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(cpu.registers.sp, 0xFFFB);

        // M3: Write high byte of r16 to [SP], SP -= 1
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::PushR16Low(register),
            "test failed for PUSH {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(cpu.registers.sp, 0xFFFA);
        assert_eq!(
            bus.read8(0xFFFB),
            (r16_value >> 8) as u8,
            "test failed for PUSH {:?}: expected high byte at [SP+1] to be {:02X} but got {:02X}",
            register,
            (r16_value >> 8) as u8,
            bus.read8(0xFFFB)
        );

        // M4: Write low byte of r16 to [SP], SP = SP + 0, PC = PC + 1
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for PUSH {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0101);
    }
}

#[test]
fn test_pop_r16() {
    for register in [CpuReg16::BC, CpuReg16::DE, CpuReg16::HL, CpuReg16::AF] {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new_post_boot();
        let r16_value = rand::random_range(u16::MIN..=u16::MAX);
        cpu.registers.sp = 0xFFFC;
        bus.write8(0xFFFC, (r16_value & 0xFF) as u8); // low byte
        bus.write8(0xFFFD, (r16_value >> 8) as u8); // high byte
        bus.rom[0x0100] = cpu.encode_instruction(CpuInstruction::PopR16(register));

        // M1: Fetch opcode
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::PopR16Low(register),
            "test failed for POP {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0101);

        // M2: Read low byte from [SP], SP += 1
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::PopR16High(register),
            "test failed for POP {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(
            cpu.registers.sp, 0xFFFD,
            "test failed for POP {:?}: expected SP=0xFFFD but got SP={:04X}",
            register, cpu.registers.sp
        );

        if register == CpuReg16::AF {
            // For AF, the low byte (F) should have its lower 4 bits cleared
            assert_eq!(
                cpu.registers.get_r16(register) & 0x00FF,
                r16_value & 0x00F0,
                "test failed for POP {:?}: expected low byte of r16={:02X} but got {:02X}",
                register,
                r16_value & 0x00F0,
                cpu.registers.get_r16(register) & 0x00FF
            );
        } else {
            assert_eq!(
                cpu.registers.get_r16(register) & 0x00FF,
                r16_value & 0x00FF,
                "test failed for POP {:?}: expected low byte of r16={:02X} but got {:02X}",
                register,
                r16_value & 0x00FF,
                cpu.registers.get_r16(register) & 0x00FF
            );
        }

        if register == CpuReg16::AF {
            // For AF, the low byte (F) should have its lower 4 bits cleared
            assert_eq!(
                cpu.registers.get_r16(register) & 0x00FF,
                r16_value & 0x00F0,
                "test failed for POP {:?}: expected low byte of r16={:02X} but got {:02X}",
                register,
                r16_value & 0x00F0,
                cpu.registers.get_r16(register) & 0x00FF
            );
        } else {
            assert_eq!(
                cpu.registers.get_r16(register) & 0x00FF,
                r16_value & 0x00FF,
                "test failed for POP {:?}: expected low byte of r16={:02X} but got {:02X}",
                register,
                r16_value & 0x00FF,
                cpu.registers.get_r16(register) & 0x00FF
            );
        }

        // M3: Read high byte from [SP], SP += 1, set r16 to the value read
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(
            cpu.phase,
            CpuPhase::FetchOpcode,
            "test failed for POP {:?}",
            register
        );
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(
            cpu.registers.sp, 0xFFFE,
            "test failed for POP {:?}: expected SP=0xFFFE but got SP={:04X}",
            register, cpu.registers.sp
        );

        if register == CpuReg16::AF {
            // For AF, the low byte (F) should have its lower 4 bits cleared
            assert_eq!(
                cpu.registers.get_r16(register) & 0x00FF,
                r16_value & 0x00F0,
                "test failed for POP {:?}: expected low byte of r16={:02X} but got {:02X}",
                register,
                r16_value & 0x00F0,
                cpu.registers.get_r16(register) & 0x00FF
            );
        } else {
            assert_eq!(
                cpu.registers.get_r16(register) & 0x00FF,
                r16_value & 0x00FF,
                "test failed for POP {:?}: expected low byte of r16={:02X} but got {:02X}",
                register,
                r16_value & 0x00FF,
                cpu.registers.get_r16(register) & 0x00FF
            );
        }
    }
}

#[test]
fn test_ld_addr_a() {
    let instruction = CpuInstruction::LdAddrA;
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new_post_boot();
    let a16_value = rand_external_ram_addr(); // Random value in the range of external RAM
    let a_value = rand::random_range(u8::MIN..=u8::MAX);
    bus.rom[0x0100] = cpu.encode_instruction(instruction);
    bus.rom[0x0101] = (a16_value & 0xFF) as u8; // low byte
    bus.rom[0x0102] = (a16_value >> 8) as u8; // high byte

    cpu.registers.set_r8(CpuReg8::A, a_value);

    // M1: Fetch opcode
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchA16Low(instruction),
        "test failed for LD (a16), A"
    );
    assert_eq!(cpu.registers.pc, 0x0101);

    // M2: Fetch low byte of address
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchA16High(instruction, (a16_value & 0xFF) as u8),
        "test failed for LD (a16), A"
    );
    assert_eq!(cpu.registers.pc, 0x0102);

    // M3: Fetch high byte of address
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchA16Mem(instruction, a16_value),
        "test failed for LD (a16), A"
    );
    assert_eq!(cpu.registers.pc, 0x0103);

    // M4: Write A to memory at a16
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        bus.read8(a16_value),
        a_value,
        "test failed for LD (a16), A: expected memory at {:04X} to be {:02X} but got {:02X}",
        a16_value,
        a_value,
        bus.read8(a16_value)
    );
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchOpcode,
        "test failed for LD (a16), A"
    );
}

#[test]
fn test_ld_a_addr() {
    let instruction = CpuInstruction::LdAAddr;
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new_post_boot();
    let a16_value = rand_external_ram_addr(); // Random value in the range of external RAM
    let mem_value = rand::random_range(u8::MIN..=u8::MAX);
    bus.rom[0x0100] = cpu.encode_instruction(instruction);
    bus.rom[0x0101] = (a16_value & 0xFF) as u8; // low byte
    bus.rom[0x0102] = (a16_value >> 8) as u8; // high byte
    bus.write8(a16_value, mem_value);

    // M1: Fetch opcode
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchA16Low(instruction),
        "test failed for LD A, (a16)"
    );
    assert_eq!(cpu.registers.pc, 0x0101);

    // M2: Fetch low byte of address
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchA16High(instruction, (a16_value & 0xFF) as u8),
        "test failed for LD A, (a16)"
    );
    assert_eq!(cpu.registers.pc, 0x0102);

    // M3: Fetch high byte of address
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchA16Mem(instruction, a16_value),
        "test failed for LD A, (a16)"
    );
    assert_eq!(cpu.registers.pc, 0x0103);

    // M4: Read memory at a16 into A
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(
        cpu.registers.get_r8(CpuReg8::A),
        mem_value,
        "test failed for LD A, (a16): expected A to be {:02X} but got {:02X}",
        mem_value,
        cpu.registers.get_r8(CpuReg8::A)
    );
    assert_eq!(
        cpu.phase,
        CpuPhase::FetchOpcode,
        "test failed for LD A, (a16)"
    );
}

#[test]
fn test_add_sp_e8() {
    let instruction = CpuInstruction::AddSpE8;
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new_post_boot();
    cpu.registers.sp = 0xE008;
    bus.rom[0x0100] = cpu.encode_instruction(instruction);
    bus.rom[0x0101] = 0xF8; // -8

    // M1: Fetch opcode
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchE8(instruction));
    assert_eq!(cpu.registers.pc, 0x0101);
    assert_eq!(cpu.registers.sp, 0xE008);

    // M2: Fetch e8
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_ne!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(cpu.registers.pc, 0x0102);
    assert_eq!(cpu.registers.sp, 0xE008);

    // M3: Add the low bytes and update flags
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_ne!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(cpu.registers.pc, 0x0102);
    assert_eq!(cpu.registers.sp, 0xE008);
    assert!(!cpu.registers.f.zero);
    assert!(!cpu.registers.f.subtract);
    assert!(cpu.registers.f.half_carry);
    assert!(cpu.registers.f.carry);

    // M4: Add the high bytes and commit SP
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(cpu.registers.pc, 0x0102);
    assert_eq!(cpu.registers.sp, 0xE000);
}

#[test]
fn test_rst() {
    let addresses = [0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38];

    for addr in addresses {
        let instruction = CpuInstruction::Rst(addr);
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new_post_boot();
        cpu.registers.sp = 0xFFFC;
        bus.rom[0x0100] = cpu.encode_instruction(CpuInstruction::Rst(addr));

        // M1: Fetch opcode
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(cpu.phase, CpuPhase::DecrementSpForWrite(instruction, addr));

        // M2: Decrement SP
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(cpu.registers.sp, 0xFFFB);
        assert_eq!(cpu.phase, CpuPhase::WriteSpMemHigh(instruction, addr));

        // M3: Write high byte of return address to [SP]
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(cpu.registers.pc, 0x0101);
        assert_eq!(cpu.registers.sp, 0xFFFA);
        assert_eq!(bus.read8(0xFFFB), 0x01);
        assert_eq!(cpu.phase, CpuPhase::WriteSpMemLow(instruction, addr));

        // M4: Write low byte of return address to [SP], set PC to addr
        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(cpu.registers.sp, 0xFFFA);
        assert_eq!(bus.read8(0xFFFA), 0x01);
        assert_eq!(cpu.registers.pc, addr);
        assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    }
}

#[test]
fn test_cb_decoder_covers_all_opcodes() {
    for opcode in u8::MIN..=u8::MAX {
        let instruction = CpuCbInstruction::decode(opcode);
        let expected_operand = match opcode & 0b111 {
            0 => CpuCbOperand::Register(CpuReg8::B),
            1 => CpuCbOperand::Register(CpuReg8::C),
            2 => CpuCbOperand::Register(CpuReg8::D),
            3 => CpuCbOperand::Register(CpuReg8::E),
            4 => CpuCbOperand::Register(CpuReg8::H),
            5 => CpuCbOperand::Register(CpuReg8::L),
            6 => CpuCbOperand::HlMem,
            7 => CpuCbOperand::Register(CpuReg8::A),
            _ => unreachable!(),
        };
        let operation_index = (opcode >> 3) & 0b111;
        let expected_operation = match opcode >> 6 {
            0 => [
                CpuCbOperation::Rlc,
                CpuCbOperation::Rrc,
                CpuCbOperation::Rl,
                CpuCbOperation::Rr,
                CpuCbOperation::Sla,
                CpuCbOperation::Sra,
                CpuCbOperation::Swap,
                CpuCbOperation::Srl,
            ][operation_index as usize],
            1 => CpuCbOperation::Bit(operation_index),
            2 => CpuCbOperation::Res(operation_index),
            3 => CpuCbOperation::Set(operation_index),
            _ => unreachable!(),
        };

        assert_eq!(
            instruction.operand, expected_operand,
            "opcode CB {opcode:02X}"
        );
        assert_eq!(
            instruction.operation, expected_operation,
            "opcode CB {opcode:02X}"
        );
    }
}

#[test]
fn test_cb_rotate_and_shift_registers() {
    let cases = [
        (0x00, 0x81, false, 0x03, false, true),
        (0x08, 0x01, false, 0x80, false, true),
        (0x10, 0x80, true, 0x01, false, true),
        (0x18, 0x01, true, 0x80, false, true),
        (0x20, 0x81, false, 0x02, false, true),
        (0x28, 0x81, false, 0xC0, false, true),
        (0x30, 0xF0, true, 0x0F, false, false),
        (0x38, 0x01, false, 0x00, true, true),
    ];

    for (opcode, input, input_carry, expected, zero, carry) in cases {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();
        bus.rom[0] = 0xCB;
        bus.rom[1] = opcode;
        cpu.registers.b = input;
        cpu.registers.f.carry = input_carry;

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(cpu.phase, CpuPhase::FetchCbOpcode);
        assert_eq!(cpu.registers.pc, 1);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
        assert_eq!(cpu.registers.pc, 2);
        assert_eq!(cpu.registers.b, expected, "opcode CB {opcode:02X}");
        assert_eq!(cpu.registers.f.zero, zero, "opcode CB {opcode:02X}");
        assert!(!cpu.registers.f.subtract, "opcode CB {opcode:02X}");
        assert!(!cpu.registers.f.half_carry, "opcode CB {opcode:02X}");
        assert_eq!(cpu.registers.f.carry, carry, "opcode CB {opcode:02X}");
    }
}

#[test]
fn test_cb_bit_res_and_set_register_flags() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    bus.rom[0] = 0xCB;
    bus.rom[1] = 0x7F; // BIT 7,A
    cpu.registers.a = 0x80;
    cpu.registers.f.carry = true;

    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.registers.a, 0x80);
    assert!(!cpu.registers.f.zero);
    assert!(!cpu.registers.f.subtract);
    assert!(cpu.registers.f.half_carry);
    assert!(cpu.registers.f.carry);

    bus.rom[2] = 0xCB;
    bus.rom[3] = 0xBF; // RES 7,A
    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.registers.a, 0x00);
    assert!(!cpu.registers.f.zero);
    assert!(!cpu.registers.f.subtract);
    assert!(cpu.registers.f.half_carry);
    assert!(cpu.registers.f.carry);

    bus.rom[4] = 0xCB;
    bus.rom[5] = 0xC7; // SET 0,A
    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.registers.a, 0x01);
    assert!(!cpu.registers.f.zero);
    assert!(!cpu.registers.f.subtract);
    assert!(cpu.registers.f.half_carry);
    assert!(cpu.registers.f.carry);
}

#[test]
fn test_cb_hl_memory_timing() {
    let addr = 0xC000;
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    cpu.registers.set_r16(CpuReg16::HL, addr);
    bus.write8(addr, 0x01);
    bus.rom[0] = 0xCB;
    bus.rom[1] = 0x46; // BIT 0,[HL]

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchCbOpcode);
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::ReadCbHl(CpuCbOperation::Bit(0)));
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(bus.read8(addr), 0x01);
    assert!(!cpu.registers.f.zero);

    bus.rom[2] = 0xCB;
    bus.rom[3] = 0x86; // RES 0,[HL]
    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.phase, CpuPhase::ReadCbHl(CpuCbOperation::Res(0)));
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::WriteMem(addr, 0x00));
    assert_eq!(bus.read8(addr), 0x01);
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::FetchOpcode);
    assert_eq!(bus.read8(addr), 0x00);
}

#[test]
fn test_base_decoder_only_marks_undefined_opcodes_as_illegal() {
    let cpu = Cpu::new();
    let undefined = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in u8::MIN..=u8::MAX {
        assert_eq!(
            cpu.decode_instruction(opcode) == CpuInstruction::Illegal,
            undefined.contains(&opcode),
            "opcode {opcode:02X}"
        );
    }
}

#[test]
fn test_accumulator_rotates_clear_zero_flag() {
    let cases = [
        (0x07, 0x80, false, 0x01, true),
        (0x0F, 0x01, false, 0x80, true),
        (0x17, 0x80, true, 0x01, true),
        (0x1F, 0x01, true, 0x80, true),
    ];

    for (opcode, input, input_carry, expected, expected_carry) in cases {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();
        bus.rom[0] = opcode;
        cpu.registers.a = input;
        cpu.registers.f.zero = true;
        cpu.registers.f.subtract = true;
        cpu.registers.f.half_carry = true;
        cpu.registers.f.carry = input_carry;

        cpu_step_n(&mut cpu, &mut bus, 4);

        assert_eq!(cpu.registers.a, expected, "opcode {opcode:02X}");
        assert!(!cpu.registers.f.zero, "opcode {opcode:02X}");
        assert!(!cpu.registers.f.subtract, "opcode {opcode:02X}");
        assert!(!cpu.registers.f.half_carry, "opcode {opcode:02X}");
        assert_eq!(cpu.registers.f.carry, expected_carry, "opcode {opcode:02X}");
    }
}

#[test]
fn test_daa_adjusts_addition_and_subtraction_results() {
    let cases = [
        (0x3C, false, false, false, 0x42, false, false),
        (0x9A, false, false, false, 0x00, true, true),
        (0x0F, true, true, false, 0x09, false, false),
    ];

    for (input, subtract, half_carry, carry, expected, zero, expected_carry) in cases {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();
        bus.rom[0] = 0x27;
        cpu.registers.a = input;
        cpu.registers.f.subtract = subtract;
        cpu.registers.f.half_carry = half_carry;
        cpu.registers.f.carry = carry;

        cpu_step_n(&mut cpu, &mut bus, 4);

        assert_eq!(cpu.registers.a, expected);
        assert_eq!(cpu.registers.f.zero, zero);
        assert_eq!(cpu.registers.f.subtract, subtract);
        assert!(!cpu.registers.f.half_carry);
        assert_eq!(cpu.registers.f.carry, expected_carry);
    }
}

#[test]
fn test_cpl_scf_and_ccf_preserve_their_unaffected_flags() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    bus.rom[..3].copy_from_slice(&[0x2F, 0x37, 0x3F]);
    cpu.registers.a = 0x0F;
    cpu.registers.f.zero = true;
    cpu.registers.f.carry = false;

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.a, 0xF0);
    assert!(cpu.registers.f.zero);
    assert!(cpu.registers.f.subtract);
    assert!(cpu.registers.f.half_carry);
    assert!(!cpu.registers.f.carry);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert!(cpu.registers.f.zero);
    assert!(!cpu.registers.f.subtract);
    assert!(!cpu.registers.f.half_carry);
    assert!(cpu.registers.f.carry);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert!(cpu.registers.f.zero);
    assert!(!cpu.registers.f.subtract);
    assert!(!cpu.registers.f.half_carry);
    assert!(!cpu.registers.f.carry);
}

#[test]
fn test_alu_a_hl_memory_group() {
    let cases = [
        (0x86, 0x0F, 0x01, false, 0x10, false, false, true, false),
        (0x8E, 0x0F, 0x00, true, 0x10, false, false, true, false),
        (0x96, 0x10, 0x01, false, 0x0F, false, true, true, false),
        (0x9E, 0x10, 0x00, true, 0x0F, false, true, true, false),
        (0xA6, 0xF0, 0x0F, false, 0x00, true, false, true, false),
        (0xAE, 0xF0, 0xFF, false, 0x0F, false, false, false, false),
        (0xB6, 0xF0, 0x0F, false, 0xFF, false, false, false, false),
        (0xBE, 0x10, 0x10, false, 0x10, true, true, false, false),
    ];

    for (opcode, a, value, carry, expected, zero, subtract, half_carry, result_carry) in cases {
        let mut bus = MemoryBus::new(vec![0; 0x8000]);
        let mut cpu = Cpu::new();
        bus.rom[0] = opcode;
        cpu.registers.a = a;
        cpu.registers.f.carry = carry;
        cpu.registers.set_r16(CpuReg16::HL, 0xC000);
        bus.write8(0xC000, value);

        cpu_step_n(&mut cpu, &mut bus, 4);
        assert!(matches!(cpu.phase, CpuPhase::FetchR16(_)));
        cpu_step_n(&mut cpu, &mut bus, 4);

        assert_eq!(cpu.phase, CpuPhase::FetchOpcode, "opcode {opcode:02X}");
        assert_eq!(cpu.registers.a, expected, "opcode {opcode:02X}");
        assert_eq!(cpu.registers.f.zero, zero, "opcode {opcode:02X}");
        assert_eq!(cpu.registers.f.subtract, subtract, "opcode {opcode:02X}");
        assert_eq!(
            cpu.registers.f.half_carry, half_carry,
            "opcode {opcode:02X}"
        );
        assert_eq!(cpu.registers.f.carry, result_carry, "opcode {opcode:02X}");
    }
}

#[test]
fn test_remaining_load_instructions() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    bus.rom[..8].copy_from_slice(&[0x08, 0x00, 0xC0, 0xF9, 0xE2, 0xF2, 0x00, 0x00]);
    cpu.registers.sp = 0x1234;
    cpu.registers.set_r16(CpuReg16::HL, 0xABCD);
    cpu.registers.a = 0x5A;
    cpu.registers.c = 0x80;

    cpu_step_n(&mut cpu, &mut bus, 20);
    assert_eq!(bus.read8(0xC000), 0x34);
    assert_eq!(bus.read8(0xC001), 0x12);

    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.registers.sp, 0xABCD);

    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(bus.read8(0xFF80), 0x5A);
    bus.write8(0xFF80, 0xA5);
    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.registers.a, 0xA5);
}

#[test]
fn test_flag_rules_corrected_for_inc_dec_and_add_hl() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    bus.rom[..3].copy_from_slice(&[0x04, 0x0B, 0x19]); // INC B; DEC BC; ADD HL,DE
    cpu.registers.b = 0x0F;
    cpu.registers.c = 0x00;
    cpu.registers.set_r16(CpuReg16::HL, 1);
    cpu.registers.set_r16(CpuReg16::DE, 1);
    cpu.registers.f.zero = true;
    cpu.registers.f.carry = true;

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.registers.b, 0x10);
    assert!(cpu.registers.f.carry);

    let flags_before_dec = u8::from(&cpu.registers.f);
    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.registers.get_r16(CpuReg16::BC), 0x0FFF);
    assert_eq!(u8::from(&cpu.registers.f), flags_before_dec);

    cpu.registers.f.zero = true;
    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.registers.get_r16(CpuReg16::HL), 2);
    assert!(cpu.registers.f.zero);
}

#[test]
fn test_add_and_sub_immediates_do_not_use_carry_flag() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    bus.rom[..4].copy_from_slice(&[0xC6, 0x01, 0xD6, 0x01]);
    cpu.registers.a = 1;
    cpu.registers.f.carry = true;

    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.registers.a, 2);

    cpu.registers.f.carry = true;
    cpu_step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.registers.a, 1);
}

#[test]
fn test_ld_hl_sp_e8_uses_low_byte_for_carry_flags() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    bus.rom[..2].copy_from_slice(&[0xF8, 0x01]);
    cpu.registers.sp = 0x00FF;

    cpu_step_n(&mut cpu, &mut bus, 12);

    assert_eq!(cpu.registers.get_r16(CpuReg16::HL), 0x0100);
    assert!(!cpu.registers.f.zero);
    assert!(!cpu.registers.f.subtract);
    assert!(cpu.registers.f.half_carry);
    assert!(cpu.registers.f.carry);
}

#[test]
fn test_ei_enables_interrupts_after_the_following_instruction() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    bus.rom[..3].copy_from_slice(&[0xFB, 0x00, 0x00]);

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert!(!cpu.ime);
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert!(!cpu.ime);
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert!(cpu.ime);
}

#[test]
fn test_halt_and_stop_enter_explicit_cpu_states() {
    let mut bus = MemoryBus::new(vec![0; 0x8000]);
    let mut cpu = Cpu::new();
    bus.rom[0] = 0x76;

    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::Halted);
    assert_eq!(cpu.registers.pc, 1);

    let mut cpu = Cpu::new();
    bus.rom[..2].copy_from_slice(&[0x10, 0x00]);
    cpu_step_n(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.phase, CpuPhase::Stopped);
    assert_eq!(cpu.registers.pc, 2);
}
