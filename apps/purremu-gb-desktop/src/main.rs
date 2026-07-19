use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use purremu_gb_core::{Event as GameBoyEvent, GameBoy, Joypad};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const FRAMEBUFFER_WIDTH: u32 = 160;
const FRAMEBUFFER_HEIGHT: u32 = 144;
const INITIAL_SCALE: f64 = 4.0;
const T_CYCLES_PER_FRAME: usize = 70_224;
const T_CYCLES_PER_SECOND: u64 = 4_194_304;
const FRAME_DURATION: Duration =
    Duration::from_nanos(1_000_000_000 * T_CYCLES_PER_FRAME as u64 / T_CYCLES_PER_SECOND);
const PALETTE: [[u8; 4]; 4] = [
    [0xe0, 0xf8, 0xd0, 0xff],
    [0x88, 0xc0, 0x70, 0xff],
    [0x34, 0x68, 0x56, 0xff],
    [0x08, 0x18, 0x20, 0xff],
];

struct App {
    gameboy: GameBoy,
    joypad: Joypad,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    next_frame_at: Option<Instant>,
    runtime_error: Option<String>,
}

impl App {
    fn new(rom_data: Vec<u8>) -> Self {
        Self {
            gameboy: GameBoy::new_post_boot(rom_data),
            joypad: Joypad::new(),
            window: None,
            pixels: None,
            next_frame_at: None,
            runtime_error: None,
        }
    }

    fn handle_keyboard_input(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
        let pressed = event.state == ElementState::Pressed;

        match event.physical_key {
            PhysicalKey::Code(KeyCode::Escape) if pressed => event_loop.exit(),
            PhysicalKey::Code(KeyCode::KeyW) => self.joypad.up = pressed,
            PhysicalKey::Code(KeyCode::KeyS) => self.joypad.down = pressed,
            PhysicalKey::Code(KeyCode::KeyA) => self.joypad.left = pressed,
            PhysicalKey::Code(KeyCode::KeyD) => self.joypad.right = pressed,
            PhysicalKey::Code(KeyCode::KeyJ) => self.joypad.a = pressed,
            PhysicalKey::Code(KeyCode::KeyK) => self.joypad.b = pressed,
            PhysicalKey::Code(KeyCode::KeyU) => self.joypad.select = pressed,
            PhysicalKey::Code(KeyCode::KeyI) => self.joypad.start = pressed,
            _ => {}
        }
    }

    fn emulate_frame(&mut self) -> io::Result<()> {
        let mut serial_output = Vec::new();

        for _ in 0..T_CYCLES_PER_FRAME {
            for event in self.gameboy.step(&self.joypad) {
                match event {
                    GameBoyEvent::SerialByte(byte) => serial_output.push(byte),
                    GameBoyEvent::FrameReady(framebuffer) => {
                        let Some(pixels) = self.pixels.as_mut() else {
                            continue;
                        };

                        for (&color_id, pixel) in framebuffer
                            .0
                            .iter()
                            .flatten()
                            .zip(pixels.frame_mut().chunks_exact_mut(4))
                        {
                            pixel.copy_from_slice(&PALETTE[usize::from(color_id)]);
                        }
                    }
                }
            }
        }

        if serial_output.is_empty() {
            return Ok(());
        }

        io::stdout().write_all(&serial_output)
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = match self.window.as_ref() {
            Some(window) => Arc::clone(window),
            None => {
                let initial_size = LogicalSize::new(
                    f64::from(FRAMEBUFFER_WIDTH) * INITIAL_SCALE,
                    f64::from(FRAMEBUFFER_HEIGHT) * INITIAL_SCALE,
                );
                let attributes = Window::default_attributes()
                    .with_title("Purremu")
                    .with_inner_size(initial_size)
                    .with_min_inner_size(LogicalSize::new(
                        f64::from(FRAMEBUFFER_WIDTH),
                        f64::from(FRAMEBUFFER_HEIGHT),
                    ));
                let window = match event_loop.create_window(attributes) {
                    Ok(window) => Arc::new(window),
                    Err(error) => {
                        self.runtime_error = Some(format!("failed to create window: {error}"));
                        event_loop.exit();
                        return;
                    }
                };
                self.window = Some(Arc::clone(&window));
                window
            }
        };

        if self.pixels.is_some() {
            return;
        }

        let size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(size.width.max(1), size.height.max(1), Arc::clone(&window));
        let pixels = match Pixels::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT, surface_texture) {
            Ok(pixels) => pixels,
            Err(error) => {
                self.runtime_error = Some(format!("failed to create pixel renderer: {error}"));
                event_loop.exit();
                return;
            }
        };

        window.request_redraw();
        self.pixels = Some(pixels);
        self.next_frame_at = Some(Instant::now());
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.joypad = Joypad::new();
        self.pixels = None;
        self.next_frame_at = None;
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref().cloned() else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event, event_loop);
            }
            WindowEvent::Focused(false) => self.joypad = Joypad::new(),
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }

                let Some(pixels) = self.pixels.as_mut() else {
                    return;
                };
                if let Err(error) = pixels.resize_surface(size.width, size.height) {
                    self.runtime_error = Some(format!("failed to resize pixel renderer: {error}"));
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                if self.next_frame_at.is_some_and(|deadline| now >= deadline) {
                    if let Err(error) = self.emulate_frame() {
                        self.runtime_error =
                            Some(format!("failed to write serial output: {error}"));
                        event_loop.exit();
                        return;
                    }

                    let next_frame_at = self.next_frame_at.expect("frame deadline must exist");
                    let next_frame_at = next_frame_at + FRAME_DURATION;
                    let finished_at = Instant::now();
                    self.next_frame_at = Some(if next_frame_at <= finished_at {
                        finished_at + FRAME_DURATION
                    } else {
                        next_frame_at
                    });
                }

                let Some(pixels) = self.pixels.as_ref() else {
                    return;
                };
                if let Err(error) = pixels.render() {
                    self.runtime_error = Some(format!("failed to render frame: {error}"));
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(window), Some(next_frame_at)) = (&self.window, self.next_frame_at) else {
            return;
        };

        if Instant::now() >= next_frame_at {
            window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame_at));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_else(|| "purremu-gb-desktop".into());
    let rom_path = args.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("usage: {} <ROM_PATH>", program.to_string_lossy()),
        )
    })?;
    if args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("usage: {} <ROM_PATH>", program.to_string_lossy()),
        )
        .into());
    }
    let rom_data = fs::read(&rom_path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read ROM {}: {error}", rom_path.to_string_lossy()),
        )
    })?;

    let event_loop = EventLoop::new()?;
    let mut app = App::new(rom_data);
    event_loop.run_app(&mut app)?;

    if let Some(error) = app.runtime_error {
        return Err(io::Error::other(error).into());
    }

    Ok(())
}
