use crate::memory_bus::MemoryBus;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::serial::Serial;
use std::io;

pub struct GameBoy {
    pub cpu: Cpu,
    pub memory_bus: MemoryBus,
    pub ppu: Ppu,
    pub serial: Serial,
}

pub enum Event {
    SerialByte(u8),
}

impl GameBoy {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            memory_bus: MemoryBus::new(),
            ppu: Ppu::new(),
            serial: Serial::new(),
        }
    }

    pub fn step(&mut self) -> Vec<Event> {
        let mut events = Vec::new();

        // TODO: read from joypad

        if let Some(byte) = self.serial.step() {
            events.push(Event::SerialByte(byte));
        }
        self.cpu.step(&mut self.memory_bus);
        self.ppu.step();

        events
    }
}
