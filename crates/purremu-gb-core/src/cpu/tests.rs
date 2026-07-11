use strum::IntoEnumIterator;

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
fn test_ld_r_imm8() {
    for register in CpuReg8::iter() {
        let mut bus = MemoryBus::new();
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
    let mut bus = MemoryBus::new();
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
        let mut bus = MemoryBus::new();
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
    let mut bus = MemoryBus::new();
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
        let mut bus = MemoryBus::new();
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
    for register in CpuReg16::iter() {
        let mut bus = MemoryBus::new();
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
        let mut bus = MemoryBus::new();
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
        let mut bus = MemoryBus::new();
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
        let mut bus = MemoryBus::new();
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
    let mut bus = MemoryBus::new();
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
    let mut bus = MemoryBus::new();
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
    let mut bus = MemoryBus::new();
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
    for register in CpuReg16::iter() {
        let mut bus = MemoryBus::new();
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

    for r16 in CpuReg16::iter() {
        let mut bus = MemoryBus::new();
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
        let mut bus = MemoryBus::new();
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
    let mut bus = MemoryBus::new();
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
