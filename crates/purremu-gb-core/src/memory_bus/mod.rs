pub struct MemoryBus {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
}

impl MemoryBus {
    pub fn new() -> Self {
        Self {
            rom: vec![0; 0x8000],
            ram: vec![0; 0x2000],
        }
    }

    // https://gbdev.io/pandocs/Memory_Map.html
    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize],
            0x8000..=0x9FFF => 0, // vram
            0xA000..=0xBFFF => self.ram[(addr - 0xA000) as usize],
            0xC000..=0xDFFF => self.ram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => self.ram[(addr - 0xE000) as usize],
            // 0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            _ => 0,
        }
    }

    pub fn write8(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize] = value,
            0x8000..=0x9FFF => {} // vram
            0xA000..=0xBFFF => self.ram[(addr - 0xA000) as usize] = value,
            0xC000..=0xDFFF => self.ram[(addr - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.ram[(addr - 0xE000) as usize] = value,
            // 0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = value,
            _ => {}
        }
    }
}
