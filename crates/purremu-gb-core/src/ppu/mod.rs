use std::collections::VecDeque;

#[derive(PartialEq, Debug)]
enum PpuMode {
    OamSearch,
    PixelTransfer,
    HBlank,
    VBlank,
}

#[derive(Debug, Clone)]
pub struct Framebuffer(pub [[u8; 160]; 144]);

impl Framebuffer {
    pub fn new() -> Self {
        Framebuffer([[0; 160]; 144])
    }
}

struct TileData([u8; 0x1800]);

impl TileData {
    fn new() -> Self {
        TileData([0; 0x1800])
    }
}

struct OAM([u8; 0xA0]);

impl OAM {
    fn new() -> Self {
        OAM([0; 0xA0])
    }
}

struct TileMap([[u8; 32]; 32]);

impl TileMap {
    fn new() -> Self {
        TileMap([[0; 32]; 32])
    }

    fn get(&self, offset: usize) -> u8 {
        let tile_y = offset / 32;
        let tile_x = offset % 32;

        self.0[tile_y][tile_x]
    }

    fn set(&mut self, offset: usize, value: u8) {
        let tile_y = offset / 32;
        let tile_x = offset % 32;

        self.0[tile_y][tile_x] = value;
    }
}

pub(crate) enum PpuEvent {
    FrameReady(Framebuffer),
    InterruptRequested,
}

pub(crate) struct LcdControl {
    lcd_and_ppu_enable: bool,
    window_tile_map_area: bool,
    window_enable: bool,
    bg_and_window_tile_data_area: bool,
    bg_tile_map_area: bool,
    obj_size: bool,
    obj_enable: bool,
    bg_and_window_enable: bool,
}

impl LcdControl {
    pub(crate) fn new() -> Self {
        Self {
            lcd_and_ppu_enable: false,
            window_tile_map_area: false,
            window_enable: false,
            bg_and_window_tile_data_area: false,
            bg_tile_map_area: false,
            obj_size: false,
            obj_enable: false,
            bg_and_window_enable: false,
        }
    }
}

impl From<u8> for LcdControl {
    fn from(value: u8) -> Self {
        Self {
            lcd_and_ppu_enable: value & 0b1000_0000 != 0,
            window_tile_map_area: value & 0b0100_0000 != 0,
            window_enable: value & 0b0010_0000 != 0,
            bg_and_window_tile_data_area: value & 0b0001_0000 != 0,
            bg_tile_map_area: value & 0b0000_1000 != 0,
            obj_size: value & 0b0000_0100 != 0,
            obj_enable: value & 0b0000_0010 != 0,
            bg_and_window_enable: value & 0b0000_0001 != 0,
        }
    }
}

impl From<&LcdControl> for u8 {
    fn from(flags: &LcdControl) -> Self {
        (flags.lcd_and_ppu_enable as u8) << 7
            | (flags.window_tile_map_area as u8) << 6
            | (flags.window_enable as u8) << 5
            | (flags.bg_and_window_tile_data_area as u8) << 4
            | (flags.bg_tile_map_area as u8) << 3
            | (flags.obj_size as u8) << 2
            | (flags.obj_enable as u8) << 1
            | (flags.bg_and_window_enable as u8)
    }
}

enum FetcherState {
    FetchTileId,
    FetchTileDataLow,
    FetchTileDataHigh,
    Sleep,
    Push,
}

struct Fetcher {
    state: FetcherState,
    tile_id: u8,
    tile_data_low: u8,
    tile_data_high: u8,
    clock: u8,
    tile_x: u8,
}

impl Fetcher {
    fn new() -> Self {
        Self {
            state: FetcherState::FetchTileId,
            tile_id: 0,
            tile_data_low: 0,
            tile_data_high: 0,
            clock: 2,
            tile_x: 0,
        }
    }
}

pub(crate) struct Ppu {
    row: u8,
    col: u16,
    mode: PpuMode,
    framebuffer: Framebuffer,
    tile_data: TileData,
    lcd_control: LcdControl,
    oam: OAM,
    tile_map_1: TileMap,
    // tile_map_2: TileMap,
    background_fifo: VecDeque<u8>,
    fetcher: Fetcher,
    screen_x: u8, // how many pixels have been popped from the FIFO
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            mode: PpuMode::OamSearch,
            row: 0,
            col: 0,
            lcd_control: LcdControl::new(),
            framebuffer: Framebuffer::new(),
            tile_data: TileData::new(),
            oam: OAM::new(),
            tile_map_1: TileMap::new(),
            // tile_map_2: TileMap::new(),
            background_fifo: VecDeque::new(),
            fetcher: Fetcher::new(),
            screen_x: 0
        }
    }

    pub fn step(&mut self) -> Vec<PpuEvent> {
        let mut events = Vec::<PpuEvent>::new();

        if !self.lcd_control.lcd_and_ppu_enable {
            return events;
        }

        match self.mode {
            PpuMode::OamSearch => {}
            PpuMode::PixelTransfer => {
                // TODO: mix
                let pixel = self.background_fifo.pop_front();
                if let Some(color_id) = pixel {
                    self.framebuffer.0[self.row as usize][self.screen_x as usize] = color_id;
                    self.screen_x += 1;
                }

                match self.fetcher.state {
                    FetcherState::FetchTileId => {
                        let tile_y = usize::from(self.row / 8); // line of the tile map
                        let tile_x = usize::from(self.fetcher.tile_x);

                        self.fetcher.tile_id = self.tile_map_1.0[tile_y][tile_x];

                        self.fetcher.clock -= 1;
                        if self.fetcher.clock == 0 {
                            self.fetcher.state = FetcherState::FetchTileDataLow;
                            self.fetcher.clock = 2;
                        }
                    }
                    FetcherState::FetchTileDataLow => {
                        self.fetcher.tile_data_low = self.tile_data.0[
                            self.fetcher.tile_id as usize * 16 + // tile index
                            (self.row as usize % 8) * 2 // low byte, line of the tile it self
                            ];
                        self.fetcher.clock -= 1;
                        if self.fetcher.clock == 0 {
                            self.fetcher.state = FetcherState::FetchTileDataHigh;
                            self.fetcher.clock = 2;
                        }
                    }
                    FetcherState::FetchTileDataHigh => {
                        self.fetcher.tile_data_high = self.tile_data.0[
                            self.fetcher.tile_id as usize * 16 + // tile index
                             (self.row as usize % 8) * 2 + 1 // high byte, line of the tile
                             ];
                        self.fetcher.clock -= 1;
                        if self.fetcher.clock == 0 {
                            self.fetcher.state = FetcherState::Sleep;
                            self.fetcher.clock = 2;
                        }
                    }
                    FetcherState::Sleep => {
                        self.fetcher.clock -= 1;
                        if self.fetcher.clock == 0 {
                            self.fetcher.state = FetcherState::Push;
                            self.fetcher.clock = 2;
                        }
                    }
                    FetcherState::Push => {
                        if self.background_fifo.is_empty() {
                            for i in (0..8).rev() {
                                let bit_low = (self.fetcher.tile_data_low >> i) & 1;
                                let bit_high = (self.fetcher.tile_data_high >> i) & 1;
                                let color_id = (bit_high << 1) | bit_low;
                                self.background_fifo.push_back(color_id);
                            }

                            self.fetcher.tile_x = self.fetcher.tile_x.wrapping_add(1) & 31;
                            self.fetcher.state = FetcherState::FetchTileId;
                            self.fetcher.clock = 2;
                        }
                    }
                }
            }
            PpuMode::HBlank => {}
            PpuMode::VBlank => {}
        }

        self.col += 1;

        match self.mode {
            PpuMode::OamSearch => {
                if self.col >= 80 {
                    self.fetcher.tile_x = 0; // start of the tile map line
                    self.mode = PpuMode::PixelTransfer;
                }
            }
            PpuMode::HBlank => {
                if self.col >= 456 {
                    self.col = 0;
                    self.row += 1;

                    if self.row >= 144 {
                        self.mode = PpuMode::VBlank;
                        events.push(PpuEvent::InterruptRequested);
                        events.push(PpuEvent::FrameReady(self.framebuffer.clone()));
                    } else {
                        self.mode = PpuMode::OamSearch;
                    }
                }
            }
            PpuMode::PixelTransfer => {
                if self.screen_x >= 160 {
                    self.mode = PpuMode::HBlank;
                    self.fetcher = Fetcher::new();
                    self.background_fifo.clear();
                    self.screen_x = 0;
                }
            }

            PpuMode::VBlank => {
                if self.col >= 456 {
                    self.col = 0;
                    self.row += 1;
                }

                if self.row >= 154 {
                    self.row = 0;
                    self.mode = PpuMode::OamSearch;
                    self.framebuffer = Framebuffer::new();
                }
            }
        }

        events
    }

    pub(crate) fn read_vram_by_cpu(&self, addr: u16) -> u8 {
        if self.mode == PpuMode::PixelTransfer && self.lcd_control.lcd_and_ppu_enable {
            return 0;
        }

        match addr {
            0x8000..=0x97FF => self.tile_data.0[addr as usize - 0x8000 as usize],
            0x9800..=0x9BFF => self.tile_map_1.get(addr as usize - 0x9800 as usize),
            // 0x9C00..=0x9FFF => self.tile_map_2.0[addr as usize - 0x9C00 as usize],
            _ => 0,
        }
    }

    pub(crate) fn write_vram_by_cpu(&mut self, addr: u16, value: u8) {
        if self.mode == PpuMode::PixelTransfer && self.lcd_control.lcd_and_ppu_enable {
            return;
        }

        match addr {
            0x8000..=0x97FF => self.tile_data.0[addr as usize - 0x8000 as usize] = value,
            0x9800..=0x9BFF => self.tile_map_1.set(addr as usize - 0x9800 as usize, value),
            // 0x9C00..=0x9FFF => self.tile_map_2.0[addr as usize - 0x9C00 as usize] = value,
            _ => {}
        }
    }

    pub(crate) fn read_oam_by_cpu(&self, addr: u16) -> u8 {
        if self.mode == PpuMode::PixelTransfer
            || self.mode == PpuMode::OamSearch && self.lcd_control.lcd_and_ppu_enable
        {
            return 0;
        }

        return self.oam.0[addr as usize - 0xFE00 as usize];
    }

    pub(crate) fn write_oam_by_cpu(&mut self, addr: u16, value: u8) {
        if self.mode == PpuMode::PixelTransfer
            || self.mode == PpuMode::OamSearch && self.lcd_control.lcd_and_ppu_enable
        {
            return;
        }

        self.oam.0[addr as usize - 0xFE00 as usize] = value;
    }

    pub(crate) fn read_lcd_control_by_cpu(&self) -> u8 {
        u8::from(&self.lcd_control)
    }

    pub(crate) fn read_ly_by_cpu(&self) -> u8 {
        self.row
    }

    pub(crate) fn write_lcd_control_by_cpu(&mut self, value: u8) {
        self.lcd_control = LcdControl::from(value);

        if self.lcd_control.lcd_and_ppu_enable {
            self.mode = PpuMode::OamSearch;
            self.framebuffer = Framebuffer::new();
            self.col = 0;
            self.row = 0;
            self.screen_x = 0;
        }
    }
}
