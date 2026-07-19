use crate::cpu::Cpu;
use crate::memory_bus::MemoryBus;
use crate::ppu::PpuEvent::InterruptRequested;
use crate::ppu::{Framebuffer, PpuEvent};
use crate::joypad::Joypad;

#[cfg(test)]
mod tests;

pub struct GameBoy {
    cpu: Cpu,
    memory_bus: MemoryBus,
}

pub enum Event {
    SerialByte(u8),
    FrameReady(Framebuffer)
}

impl GameBoy {
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            cpu: Cpu::new(),
            memory_bus: MemoryBus::new(rom_data),
        }
    }

    pub fn new_post_boot(rom_data: Vec<u8>) -> Self {
        Self {
            cpu: Cpu::new_post_boot(),
            memory_bus: MemoryBus::new(rom_data),
        }
    }

    pub fn step(&mut self, joypad: &Joypad) -> Vec<Event> {
        let mut events = Vec::new();
        self.memory_bus.joypad.set_by_bus(joypad);

        if let Some(byte) = self.memory_bus.serial.step() {
            events.push(Event::SerialByte(byte));
        }
        self.cpu.step(&mut self.memory_bus);

        let ppu_events = self.memory_bus.ppu.step();
        for ppu_event in ppu_events {
            match ppu_event {
                PpuEvent::FrameReady(framebuffer) => events.push(Event::FrameReady(framebuffer)),
                InterruptRequested(interrupt_type) => self.memory_bus.request_interrupt(interrupt_type),
            }
        }

        events
    }
}
