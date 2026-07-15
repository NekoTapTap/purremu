#[derive(Debug, Clone, Copy)]
pub struct Joypad {
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub select_buttons: bool,
    pub select_d_pad: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            a: false,
            b: false,
            start: false,
            select: false,
            up: false,
            down: false,
            left: false,
            right: false,
            select_buttons: false,
            select_d_pad: false,
        }
    }

    pub fn set_by_bus(&mut self, input: &Joypad) {
        self.a = input.a;
        self.b = input.b;
        self.start = input.start;
        self.select = input.select;
        self.up = input.up;
        self.down = input.down;
        self.left = input.left;
        self.right = input.right;
    }

    pub fn set_by_cpu(&mut self, buttons: u8) {
        self.select_buttons = !buttons & 0b0010_0000 != 0;
        self.select_d_pad = !buttons & 0b0001_0000 != 0;

        // other values are dropped, because the joypad register is write-only for the upper 4 bits
    }

    pub fn get_by_cpu(&self) -> u8 {
        let mut result = 0b0000_0000;

        if self.select_d_pad {
            // direction buttons
            if self.right {
                result |= 0b0000_0001;
            }
            if self.left {
                result |= 0b0000_0010;
            }
            if self.up {
                result |= 0b0000_0100;
            }
            if self.down {
                result |= 0b0000_1000;
            }
        }

        if self.select_buttons {
            // action buttons
            if self.a {
                result |= 0b0000_0001;
            }
            if self.b {
                result |= 0b0000_0010;
            }
            if self.select {
                result |= 0b0000_0100;
            }
            if self.start {
                result |= 0b0000_1000;
            }
        }

        !result
    }
}
