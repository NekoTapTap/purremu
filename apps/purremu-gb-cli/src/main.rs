use crossterm::event::{self, Event as CrossTermEvent, KeyCode, KeyEvent};
use crossterm::terminal;
use purremu_gb_core::{Event as GameBoyEvent, GameBoy, Joypad};
use std::env;
use std::fs;
use std::io;

fn handle_key_event(key_event: &KeyEvent, pressed_keys: &mut Joypad) {
    let pressed = if key_event.is_press() {
        Some(true)
    } else if key_event.is_release() {
        Some(false)
    } else {
        None
    };

    match pressed {
        Some(p) => {
            match key_event.code {
                KeyCode::Char('w') => pressed_keys.up = p,
                KeyCode::Char('s') => pressed_keys.down = p,
                KeyCode::Char('a') => pressed_keys.left = p,
                KeyCode::Char('d') => pressed_keys.right = p,
                KeyCode::Char('j') => pressed_keys.a = p,
                KeyCode::Char('k') => pressed_keys.b = p,
                KeyCode::Char('u') => pressed_keys.select = p,
                KeyCode::Char('i') => pressed_keys.start = p,
                _ => {}
            }

        }
        None => {}
    }
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;

    let rom_path = env::args()
        .nth(1)
        .expect("Please provide a ROM path as the first argument.");
    // read the ROM file into a Vec<u8>
    let rom_data = fs::read(rom_path).expect("Failed to read the ROM file.");

    let mut gameboy = GameBoy::new_post_boot(rom_data);
    let mut pressed_keys = Joypad::new();

    loop {
        if event::poll(std::time::Duration::from_millis(0))? {
            let crossterm_event = event::read()?;
            let mut key_event: Option<KeyEvent> = None;

            match crossterm_event {
                CrossTermEvent::Key(key) => key_event = Some(key),
                _ => {}
            }

            match key_event {
                Some(e) => {
                    if e.code == KeyCode::Esc {
                        terminal::disable_raw_mode()?;
                        return Ok(());
                    }

                    handle_key_event(&e, &mut pressed_keys);
                }
                _ => {}
            }
        }

        let gameboy_events = gameboy.step(&pressed_keys);
        for event in gameboy_events {
            match event {
                GameBoyEvent::SerialByte(byte) => {
                    print!("{}", byte as char);
                }
            }
        }
    }
}
