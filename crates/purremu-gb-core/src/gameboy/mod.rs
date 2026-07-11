use crate::cpu::Cpu;
use crate::memory_bus::MemoryBus;
use crate::ppu::Ppu;

#[cfg(test)]
mod tests;

pub struct GameBoy {
    pub cpu: Cpu,
    pub memory_bus: MemoryBus,
    pub ppu: Ppu,
}

pub enum Event {
    SerialByte(u8),
}

impl GameBoy {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            cpu: Cpu::new(),
            memory_bus: MemoryBus::new(rom_data),
            ppu: Ppu::new(),
        }
    }

    pub fn step(&mut self) -> Vec<Event> {
        let mut events = Vec::new();

        // TODO: read from joypad

        if let Some(byte) = self.memory_bus.serial.step() {
            events.push(Event::SerialByte(byte));
        }
        self.cpu.step(&mut self.memory_bus);
        self.ppu.step();

        events
    }
}
