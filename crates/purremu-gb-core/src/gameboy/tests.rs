use super::{Event, GameBoy};

#[test]
fn emits_a_serial_event_from_the_memory_mapped_serial_device() {
    let mut gameboy = GameBoy::new(vec![0; 0x8000]);
    gameboy.memory_bus.write8(0xFF01, b'H');
    gameboy.memory_bus.write8(0xFF02, 0x81);

    let mut serial_bytes = Vec::new();
    for _ in 0..4096 {
        serial_bytes.extend(gameboy.step().into_iter().map(|event| match event {
            Event::SerialByte(byte) => byte,
        }));
    }

    assert_eq!(serial_bytes, [b'H']);
}
