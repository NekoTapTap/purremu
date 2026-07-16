use crossterm::event::{
    self, Event as CrossTermEvent, KeyCode, KeyEvent, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal;
use purremu_gb_core::{Event as GameBoyEvent, GameBoy, Joypad};
use std::env;
use std::fs;
use std::io::{self, Error};
use std::time::Duration;

struct TerminalSession {
    keyboard_enhancement_enabled: bool,
}

impl TerminalSession {
    fn start() -> io::Result<Self> {
        let keyboard_enhancement_enabled = terminal::supports_keyboard_enhancement()?;

        #[cfg(unix)]
        if !keyboard_enhancement_enabled {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "the terminal does not support the enhanced keyboard protocol required for key-release events",
            ));
        }

        terminal::enable_raw_mode()?;

        if keyboard_enhancement_enabled {
            let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;
            let mut stdout = io::stdout();
            if let Err(error) = execute!(stdout, PushKeyboardEnhancementFlags(flags)) {
                terminal::disable_raw_mode()?;
                return Err(error);
            }
        }

        Ok(Self {
            keyboard_enhancement_enabled,
        })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let enhancement_error = if self.keyboard_enhancement_enabled {
            let mut stdout = io::stdout();
            execute!(stdout, PopKeyboardEnhancementFlags).err()
        } else {
            None
        };
        let raw_mode_error = terminal::disable_raw_mode().err();

        if let Some(error) = enhancement_error {
            eprintln!("failed to disable the enhanced keyboard protocol: {error}");
        }
        if let Some(error) = raw_mode_error {
            eprintln!("failed to disable terminal raw mode: {error}");
        }
    }
}

fn handle_key_event(key_event: &KeyEvent, pressed_keys: &mut Joypad) {
    let pressed = if key_event.is_press() {
        Some(true)
    } else if key_event.is_release() {
        Some(false)
    } else {
        None
    };

    match pressed {
        Some(p) => match key_event.code {
            KeyCode::Char('w') => pressed_keys.up = p,
            KeyCode::Char('s') => pressed_keys.down = p,
            KeyCode::Char('a') => pressed_keys.left = p,
            KeyCode::Char('d') => pressed_keys.right = p,
            KeyCode::Char('j') => pressed_keys.a = p,
            KeyCode::Char('k') => pressed_keys.b = p,
            KeyCode::Char('u') => pressed_keys.select = p,
            KeyCode::Char('i') => pressed_keys.start = p,
            _ => {}
        },
        None => {}
    }
}

fn poll_keyboard_event(pressed_keys: &mut Joypad) -> io::Result<()> {
    if event::poll(Duration::from_millis(0))? {
        let crossterm_event = event::read()?;
        let mut key_event: Option<KeyEvent> = None;

        match crossterm_event {
            CrossTermEvent::Key(key) => key_event = Some(key),
            _ => {}
        }

        match key_event {
            Some(e) => {
                if e.code == KeyCode::Esc && e.is_press() {
                    return Err(Error::new(io::ErrorKind::Other, "Escape key pressed"));
                }

                handle_key_event(&e, pressed_keys);
            }
            _ => {}
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let rom_path = env::args()
        .nth(1)
        .expect("Please provide a ROM path as the first argument.");
    // read the ROM file into a Vec<u8>
    let rom_data = fs::read(rom_path).expect("Failed to read the ROM file.");

    let mut gameboy = GameBoy::new_post_boot(rom_data);
    let mut pressed_keys = Joypad::new();
    let _terminal_session = TerminalSession::start()?;
    let mut poll_keyboard_event_until = 1024;

    loop {
        if poll_keyboard_event_until == 0 {
            poll_keyboard_event_until = 1024;
            if let Err(e) = poll_keyboard_event(&mut pressed_keys) {
                eprintln!("Error polling keyboard event: {e}");

                return Ok(())
            }
        } else {
            poll_keyboard_event_until -= 1;
        }

        let gameboy_events = gameboy.step(&pressed_keys);
        for event in gameboy_events {
            match event {
                GameBoyEvent::SerialByte(byte) => {
                    print!("{}", byte as char);
                }
                GameBoyEvent::FrameReady(_) => {
                    // TODO: ignore it now, ignore it when cli runs in headless mode
                }
            }
        }
    }
}
