use purremu_gb_core::{Event, GameBoy};
use std::env;
use std::fs;

fn main() {
    let rom_path = env::args()
        .nth(1)
        .expect("Please provide a ROM path as the first argument.");
    // read the ROM file into a Vec<u8>
    let rom_data = fs::read(rom_path).expect("Failed to read the ROM file.");

    let mut gameboy = GameBoy::new_post_boot(rom_data);

    loop {
        let events = gameboy.step();
        for event in events {
            match event {
                Event::SerialByte(byte) => {
                    print!("{}", byte as char);
                }
            }
        }
    }
}
