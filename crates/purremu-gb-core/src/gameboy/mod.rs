use crate::cpu::Cpu;
use crate::memory_bus::MemoryBus;
use crate::ppu::Ppu;
use crate::joypad::Joypad;

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

    pub fn new_post_boot(rom_data: Vec<u8>) -> Self {
        Self {
            cpu: Cpu::new_post_boot(),
            memory_bus: MemoryBus::new(rom_data),
            ppu: Ppu::new(),
        }
    }

    pub fn step(&mut self, joypad: &Joypad) -> Vec<Event> {
        let mut events = Vec::new();
        self.memory_bus.joypad = joypad.clone();

        if let Some(byte) = self.memory_bus.serial.step() {
            events.push(Event::SerialByte(byte));
        }
        self.cpu.step(&mut self.memory_bus);
        self.ppu.step();

        events
    }
}
