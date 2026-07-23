use std::collections::VecDeque;

use crate::memory_bus::InterruptType;

#[derive(PartialEq, Debug)]
enum PpuMode {
    HBlank,        // Mode 0
    VBlank,        // Mode 1
    OamSearch,     // Mode 2
    PixelTransfer, // Mode 3
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
    InterruptRequested(InterruptType),
}

pub(crate) struct LcdControl {
    lcd_and_ppu_enable: bool,
    window_tile_map_area: bool,
    window_enable: bool,
    use_unsigned_tile_addressing: bool,
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
            use_unsigned_tile_addressing: false,
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
            use_unsigned_tile_addressing: value & 0b0001_0000 != 0,
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
            | (flags.use_unsigned_tile_addressing as u8) << 4
            | (flags.bg_tile_map_area as u8) << 3
            | (flags.obj_size as u8) << 2
            | (flags.obj_enable as u8) << 1
            | (flags.bg_and_window_enable as u8)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LcdStatus {
    mode_oam_interrupt: bool,
    mode_v_blank_interrupt: bool,
    mode_h_blank_interrupt: bool,
    lyc_equals_ly_interrupt: bool,
}

impl From<u8> for LcdStatus {
    // PPU mode is read-only, so we don't include it here
    #[rustfmt::skip]
    fn from(value: u8) -> Self {
        Self {
            lyc_equals_ly_interrupt: value & 0b0100_0000 != 0, // bit 6
            mode_oam_interrupt:      value & 0b0010_0000 != 0, // bit 5
            mode_v_blank_interrupt:  value & 0b0001_0000 != 0, // bit 4
            mode_h_blank_interrupt:  value & 0b0000_1000 != 0, // bit 3
        }
    }
}

impl From<&LcdStatus> for u8 {
    #[rustfmt::skip]
    fn from(flags: &LcdStatus) -> Self {
        (flags.lyc_equals_ly_interrupt as u8) << 6
            | (flags.mode_oam_interrupt as u8) << 5
            | (flags.mode_v_blank_interrupt as u8) << 4
            | (flags.mode_h_blank_interrupt as u8) << 3
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
    pending_sprites: VecDeque<Sprite>,
    current_sprite: Option<Sprite>,
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
            pending_sprites: VecDeque::new(),
            current_sprite: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SpriteAttributes {
    priority: bool,
    y_flip: bool,
    x_flip: bool,
    palette_number: u8,
    #[allow(dead_code)]
    fetch_tile_from_bank_1: bool, // CGB only, if false, fetch tile from bank 0
    #[allow(dead_code)]
    cgb_palette_number: u8, // CGB only, 0-7
}

impl From<u8> for SpriteAttributes {
    fn from(value: u8) -> Self {
        Self {
            priority: value & 0b1000_0000 != 0,
            y_flip: value & 0b0100_0000 != 0,
            x_flip: value & 0b0010_0000 != 0,
            palette_number: (value & 0b0001_0000) >> 4,
            fetch_tile_from_bank_1: value & 0b0000_1000 != 0,
            cgb_palette_number: value & 0b0000_0111,
        }
    }
}

impl From<&SpriteAttributes> for u8 {
    fn from(flags: &SpriteAttributes) -> Self {
        (flags.priority as u8) << 7
            | (flags.y_flip as u8) << 6
            | (flags.x_flip as u8) << 5
            | (flags.palette_number & 0b0000_0001) << 4
            | (flags.fetch_tile_from_bank_1 as u8) << 3
            | (flags.cgb_palette_number & 0b0000_0111)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Sprite {
    y: u8,
    x: u8,
    tile_index: u8,
    attributes: SpriteAttributes,
}

impl From<[u8; 4]> for Sprite {
    fn from(data: [u8; 4]) -> Self {
        Self {
            y: data[0],
            x: data[1],
            tile_index: data[2],
            attributes: SpriteAttributes::from(data[3]),
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
    tile_map: TileMap,
    background_fifo: VecDeque<u8>,
    object_fifo: VecDeque<u8>,
    fetcher: Fetcher,
    screen_x: u8, // how many pixels have been drawn on the current line
    lyc: u8,      // LY Compare
    lcd_status: LcdStatus,
    scx: u8, // Scroll X
    scy: u8, // Scroll Y
    pixels_to_discard: u8,
    sprites_to_draw: Vec<Sprite>,
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
            tile_map: TileMap::new(),
            background_fifo: VecDeque::new(),
            object_fifo: VecDeque::new(),
            fetcher: Fetcher::new(),
            screen_x: 0,
            lyc: 0,
            lcd_status: LcdStatus::default(),
            scx: 0,
            scy: 0,
            pixels_to_discard: 0,
            sprites_to_draw: Vec::new(),
        }
    }

    fn get_sprite_at(&self, x: u8) -> Vec<Sprite> {
        self.sprites_to_draw
            .iter()
            .filter(|sprite| sprite.x == x)
            .copied()
            .collect()
    }

    pub fn tile_address(&self) -> usize {
        let mut line_in_tile = self.row.wrapping_add(self.scy) % 8; // which line of the tile we are currently drawing
        if let Some(sprite) = self.fetcher.current_sprite {
            if sprite.attributes.y_flip {
                line_in_tile = 7 - line_in_tile;
            }
        }

        if self.lcd_control.use_unsigned_tile_addressing {
            self.fetcher.tile_id as usize * 16 + // tile index
                     (line_in_tile as usize) * 2 // low byte, line of the tile it self
        } else {
            // https://gbdev.io/pandocs/Tile_Data.html#vram-tile-data
            // Tile data can store 384 tiles but tile map can only address 256 tiles
            (0x1000_isize + self.fetcher.tile_id as i8 as isize * 16) as usize + // tile index
                     (line_in_tile as usize) * 2 // low byte, line of the tile it self
        }
    }

    pub(crate) fn transfer_pixel(&mut self) {
        let background_pixel = self.background_fifo.pop_front();
        let object_pixel = self.object_fifo.pop_front();

        match background_pixel {
            Some(bg_color_id) => {
                match object_pixel {
                    Some(obj_color_id) => {
                        // If the object pixel is not transparent (color ID 0), it takes priority over the background pixel
                        if obj_color_id != 0 {
                            self.framebuffer.0[self.row as usize][self.screen_x as usize] =
                                obj_color_id;
                            self.col += 1;
                            return;
                        }

                        self.framebuffer.0[self.row as usize][self.screen_x as usize] = bg_color_id;
                        self.col += 1;
                    }
                    None => {
                        if self.pixels_to_discard > 0 {
                            self.pixels_to_discard -= 1;
                            return;
                        }

                        self.framebuffer.0[self.row as usize][self.screen_x as usize] = bg_color_id;
                        self.col += 1;
                    }
                }
            }
            None => {
                if let Some(obj_color_id) = object_pixel {
                    self.framebuffer.0[self.row as usize][self.screen_x as usize] = obj_color_id;
                    self.col += 1;
                }
            }
        }
    }

    fn check_and_push_sprite(&mut self) {
        let sprites = self.get_sprite_at(self.screen_x);
        sprites.iter().for_each(|sprite| {
            self.fetcher.pending_sprites.push_back(*sprite);
        });
    }

    pub fn step(&mut self) -> Vec<PpuEvent> {
        let mut events = Vec::<PpuEvent>::new();

        if !self.lcd_control.lcd_and_ppu_enable {
            return events;
        }

        // mode-specific behavior
        match self.mode {
            PpuMode::OamSearch => {
                if self.lyc == self.row && self.lcd_status.lyc_equals_ly_interrupt {
                    events.push(PpuEvent::InterruptRequested(InterruptType::LCD));
                }

                if self.col % 2 == 0 && self.sprites_to_draw.len() < 10 {
                    let sprite_index = self.col / 2;
                    let sprite = Sprite::from([
                        self.oam.0[sprite_index as usize * 4],
                        self.oam.0[sprite_index as usize * 4 + 1],
                        self.oam.0[sprite_index as usize * 4 + 2],
                        self.oam.0[sprite_index as usize * 4 + 3],
                    ]);

                    if sprite.y.wrapping_sub(16) <= self.row
                        && self.row < sprite.y.wrapping_sub(16).wrapping_add(8)
                        && self.sprites_to_draw.len() < 10
                    {
                        self.sprites_to_draw.push(sprite);
                    }
                }

                self.col += 1;
            }
            PpuMode::PixelTransfer => {
                self.transfer_pixel();

                match self.fetcher.state {
                    FetcherState::FetchTileId => {
                        if self.fetcher.clock > 0 {
                            self.fetcher.clock -= 1;
                        }

                        let sprites = self.get_sprite_at(self.screen_x);
                        if sprites.len() > 0 {
                            sprites.iter().for_each(|sprite| {
                                self.fetcher.pending_sprites.push_back(*sprite);
                            });

                            let sprite = self.fetcher.pending_sprites.pop_front();
                            if let Some(s) = sprite {
                                self.fetcher.current_sprite = Some(s);
                                self.fetcher.tile_id = s.tile_index;
                            }
                        } else {
                            let background_y = self.row.wrapping_add(self.scy);
                            let tile_y = usize::from(background_y / 8); // line of the tile map

                            let first_tile_x = usize::from(self.scx / 8); // skipped tiles at the start of the line
                            let tile_x = (first_tile_x + self.fetcher.tile_x as usize) % 32; // col of the tile map

                            self.fetcher.tile_id = self.tile_map.0[tile_y][tile_x];
                        }

                        self.fetcher.state = FetcherState::FetchTileDataLow;
                        self.fetcher.clock = 2;
                    }
                    FetcherState::FetchTileDataLow => {
                        if self.fetcher.clock > 0 {
                            self.fetcher.clock -= 1;
                        }

                        self.fetcher.tile_data_low = self.tile_data.0[self.tile_address()];
                        if let Some(sprite) = self.fetcher.current_sprite {
                            if sprite.attributes.x_flip {
                                self.fetcher.tile_data_low =
                                    self.fetcher.tile_data_low.reverse_bits();
                            }
                        }

                        self.check_and_push_sprite();

                        self.fetcher.state = FetcherState::FetchTileDataHigh;
                        self.fetcher.clock = 2;
                    }
                    FetcherState::FetchTileDataHigh => {
                        if self.fetcher.clock > 0 {
                            self.fetcher.clock -= 1;
                        }

                        self.fetcher.tile_data_high = self.tile_data.0[self.tile_address() + 1];
                        if let Some(sprite) = self.fetcher.current_sprite {
                            if sprite.attributes.x_flip {
                                self.fetcher.tile_data_high =
                                    self.fetcher.tile_data_high.reverse_bits();
                            }
                        }

                        self.check_and_push_sprite();

                        self.fetcher.state = FetcherState::Sleep;
                        self.fetcher.clock = 2;
                    }
                    FetcherState::Sleep => {
                        if self.fetcher.clock > 0 {
                            self.fetcher.clock -= 1;
                        }

                        self.check_and_push_sprite();

                        self.fetcher.state = FetcherState::Push;
                        self.fetcher.clock = 2;
                    }
                    FetcherState::Push => {
                        if self.fetcher.current_sprite.is_some() {
                            if self.object_fifo.len() < 8 {
                                for i in (0..8).rev() {
                                    let bit_low = (self.fetcher.tile_data_low >> i) & 1;
                                    let bit_high = (self.fetcher.tile_data_high >> i) & 1;
                                    let color_id = (bit_high << 1) | bit_low;

                                    if let Some(pixel) = self.object_fifo.get(i) {
                                        if *pixel == 0 {
                                            self.object_fifo[i] = color_id;
                                        }
                                    } else {
                                        self.object_fifo.push_back(color_id);
                                    }
                                }

                                self.check_and_push_sprite();

                                self.fetcher.state = FetcherState::FetchTileId;
                                self.fetcher.clock = 2;
                            }
                        } else if self.background_fifo.is_empty() {
                            for i in (0..8).rev() {
                                let bit_low = (self.fetcher.tile_data_low >> i) & 1;
                                let bit_high = (self.fetcher.tile_data_high >> i) & 1;
                                let color_id = (bit_high << 1) | bit_low;
                                self.background_fifo.push_back(color_id);
                            }

                            self.check_and_push_sprite();

                            self.fetcher.tile_x = self.fetcher.tile_x.wrapping_add(1) & 31;
                            self.fetcher.state = FetcherState::FetchTileId;
                            self.fetcher.clock = 2;
                        }
                    }
                }
            }
            _ => {
                self.col += 1;
            }
        }

        // state transitions
        match self.mode {
            PpuMode::OamSearch => {
                if self.col >= 80 {
                    self.sprites_to_draw.sort_by(|a, b| {
                        if a.x == b.x {
                            a.tile_index.cmp(&b.tile_index)
                        } else {
                            a.x.cmp(&b.x)
                        }
                    });
                    self.fetcher.tile_x = 0; // start of the tile map line
                    self.mode = PpuMode::PixelTransfer;
                    self.pixels_to_discard = self.scx % 8; // discard pixels from the FIFO based on SCX
                }
            }
            PpuMode::HBlank => {
                if self.col >= 456 {
                    self.col = 0;
                    self.row += 1;

                    if self.row >= 144 {
                        self.mode = PpuMode::VBlank;
                        events.push(PpuEvent::InterruptRequested(InterruptType::VBlank));

                        if self.lcd_status.mode_v_blank_interrupt {
                            events.push(PpuEvent::InterruptRequested(InterruptType::LCD));
                        }

                        events.push(PpuEvent::InterruptRequested(InterruptType::VBlank));
                        events.push(PpuEvent::FrameReady(self.framebuffer.clone()));
                    } else {
                        self.mode = PpuMode::OamSearch;

                        if self.lcd_status.mode_oam_interrupt {
                            events.push(PpuEvent::InterruptRequested(InterruptType::LCD));
                        }
                    }
                }
            }
            PpuMode::PixelTransfer => {
                if self.screen_x >= 160 {
                    self.mode = PpuMode::HBlank;
                    self.fetcher = Fetcher::new();
                    self.background_fifo.clear();
                    self.screen_x = 0;

                    if self.lcd_status.mode_h_blank_interrupt {
                        events.push(PpuEvent::InterruptRequested(InterruptType::LCD));
                    }
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
            0x9800..=0x9BFF => self.tile_map.get(addr as usize - 0x9800 as usize),
            _ => 0,
        }
    }

    pub(crate) fn write_vram_by_cpu(&mut self, addr: u16, value: u8) {
        if self.mode == PpuMode::PixelTransfer && self.lcd_control.lcd_and_ppu_enable {
            return;
        }

        match addr {
            0x8000..=0x97FF => self.tile_data.0[addr as usize - 0x8000 as usize] = value,
            0x9800..=0x9BFF => self.tile_map.set(addr as usize - 0x9800 as usize, value),
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

    pub(crate) fn read_lcd_status_by_cpu(&self) -> u8 {
        let mode_bits = match self.mode {
            PpuMode::OamSearch => 2,
            PpuMode::PixelTransfer => 3,
            PpuMode::HBlank => 0,
            PpuMode::VBlank => 1,
        };

        mode_bits | u8::from(&self.lcd_status) | ((self.row == self.lyc) as u8) << 2
    }

    pub(crate) fn write_lcd_status_by_cpu(&mut self, value: u8) {
        self.lcd_status = LcdStatus::from(value);
    }

    pub(crate) fn write_lyc_by_cpu(&mut self, value: u8) {
        self.lyc = value;
    }

    pub(crate) fn read_scroll_x_by_cpu(&self) -> u8 {
        self.scx
    }

    pub(crate) fn write_scroll_x_by_cpu(&mut self, value: u8) {
        self.scx = value;
    }

    pub(crate) fn read_scroll_y_by_cpu(&self) -> u8 {
        self.scy
    }

    pub(crate) fn write_scroll_y_by_cpu(&mut self, value: u8) {
        self.scy = value;
    }
}
