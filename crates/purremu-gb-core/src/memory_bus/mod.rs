use crate::{joypad::Joypad, ppu::Ppu, serial::Serial};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptType {
    VBlank,
    LCD,
    Timer,
    Serial,
    Joypad,
}

impl From<InterruptType> for u16 {
    fn from(interrupt: InterruptType) -> Self {
        match interrupt {
            InterruptType::VBlank => 0x0040,
            InterruptType::LCD => 0x0048,
            InterruptType::Timer => 0x0050,
            InterruptType::Serial => 0x0058,
            InterruptType::Joypad => 0x0060,
        }
    }
}

pub(crate) struct PendingInterrupts(pub(crate) Vec<InterruptType>);

impl From<u8> for PendingInterrupts {
    fn from(value: u8) -> Self {
        let mut interrupts = Vec::new();
        if value & 0b0000_0001 != 0 {
            interrupts.push(InterruptType::VBlank);
        }
        if value & 0b0000_0010 != 0 {
            interrupts.push(InterruptType::LCD);
        }
        if value & 0b0000_0100 != 0 {
            interrupts.push(InterruptType::Timer);
        }
        if value & 0b0000_1000 != 0 {
            interrupts.push(InterruptType::Serial);
        }
        if value & 0b0001_0000 != 0 {
            interrupts.push(InterruptType::Joypad);
        }
        Self(interrupts)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Interrupt {
    pub v_blank: bool,
    pub lcd: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

impl Interrupt {
    pub fn new() -> Self {
        Self {
            v_blank: false,
            lcd: false,
            timer: false,
            serial: false,
            joypad: false,
        }
    }
}

impl From<u8> for Interrupt {
    fn from(value: u8) -> Self {
        Self {
            v_blank: value & 0b0000_0001 != 0,
            lcd: value & 0b0000_0010 != 0,
            timer: value & 0b0000_0100 != 0,
            serial: value & 0b0000_1000 != 0,
            joypad: value & 0b0001_0000 != 0,
        }
    }
}

impl From<Interrupt> for u8 {
    fn from(flags: Interrupt) -> Self {
        (flags.v_blank as u8)
            | ((flags.lcd as u8) << 1)
            | ((flags.timer as u8) << 2)
            | ((flags.serial as u8) << 3)
            | ((flags.joypad as u8) << 4)
    }
}

pub struct MemoryBus {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub hram: Vec<u8>,
    pub serial: Serial,
    pub interrupt_enable: Interrupt,
    pub interrupt_flags: Interrupt,
    pub(crate) joypad: Joypad,
    pub(crate) ppu: Ppu,
}

impl MemoryBus {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            rom: rom_data,
            ram: vec![0; 0x2000],
            hram: vec![0; 0x7F],
            serial: Serial::new(),
            interrupt_enable: Interrupt::new(),
            interrupt_flags: Interrupt::new(),
            joypad: Joypad::new(),
            ppu: Ppu::new(),
        }
    }

    // https://gbdev.io/pandocs/Memory_Map.html
    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize],
            0xA000..=0xBFFF => self.ram[(addr - 0xA000) as usize],
            0xC000..=0xDFFF => self.ram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => self.ram[(addr - 0xE000) as usize],
            0xFF00 => self.joypad.get_by_cpu(),
            0xFF01 => self.serial.data,
            0xFF02 => self.serial.control,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFFFF => u8::from(self.interrupt_enable),
            0xFF0F => u8::from(self.interrupt_flags),

            0xFE00..=0xFE9F => self.ppu.read_oam_by_cpu(addr),
            0x8000..=0x9FFF => self.ppu.read_vram_by_cpu(addr),
            0xFF40 => self.ppu.read_lcd_control_by_cpu(),
            0xFF41 => self.ppu.read_lcd_status_by_cpu(),
            0xFF44 => self.ppu.read_ly_by_cpu(),
            _ => 0,
        }
    }

    pub fn write8(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize] = value,
            0xA000..=0xBFFF => self.ram[(addr - 0xA000) as usize] = value,
            0xC000..=0xDFFF => self.ram[(addr - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.ram[(addr - 0xE000) as usize] = value,
            0xFF00 => self.joypad.set_by_cpu(value),
            0xFF01 => self.serial.data = value,
            0xFF02 => self.serial.control = value,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,
            0xFFFF => self.interrupt_enable = Interrupt::from(value),
            0xFF0F => self.interrupt_flags = Interrupt::from(value),

            0xFE00..=0xFE9F => self.ppu.write_oam_by_cpu(addr, value),
            0x8000..=0x9FFF => self.ppu.write_vram_by_cpu(addr, value),
            0xFF40 => self.ppu.write_lcd_control_by_cpu(value),
            0xFF41 => self.ppu.write_lcd_status_by_cpu(value),
            0xFF45 => self.ppu.write_lyc_by_cpu(value),
            _ => {}
        }
    }

    fn set_interrupt_flag(&mut self, interrupt: InterruptType, set: bool) {
        match interrupt {
            InterruptType::VBlank => self.interrupt_flags.v_blank = set,
            InterruptType::LCD => self.interrupt_flags.lcd = set,
            InterruptType::Timer => self.interrupt_flags.timer = set,
            InterruptType::Serial => self.interrupt_flags.serial = set,
            InterruptType::Joypad => self.interrupt_flags.joypad = set,
        }
    }

    pub fn request_interrupt(&mut self, interrupt: InterruptType) {
        self.set_interrupt_flag(interrupt, true);
    }

    pub fn clear_interrupt_flag(&mut self, interrupt: InterruptType) {
        self.set_interrupt_flag(interrupt, false);
    }
}
