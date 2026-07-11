pub struct Serial {
    pub data: u8,
    pub control: u8,
    t_cycles: u16,
    bits_transferred: u8,
    transmitted: u8,
}

impl Serial {
    pub fn new() -> Self {
        Self {
            data: 0,
            control: 0b0000_0001,
            t_cycles: 0,
            bits_transferred: 0,
            transmitted: 0,
        }
    }

    pub fn reset(&mut self) {
        self.data = 0;
        self.control = 0b0000_0001;
        self.t_cycles = 0;
        self.bits_transferred = 0;
        self.transmitted = 0;
    }

    pub fn step(&mut self) -> Option<u8> {
        if self.control & 0b0000_0001 == 0 {
            return None;
        }

        self.t_cycles += 1;

        if self.t_cycles == 512 { // 128 M-cycles
            self.t_cycles = 0;
            self.bits_transferred += 1;
            self.transmitted = (self.transmitted << 1) | ((self.data & 0b1000_0000) >> 7);
            self.data = (self.data << 1) | 1;

            if self.bits_transferred == 8 {
                let transmitted = self.transmitted;

                self.reset();

                return Some(transmitted);
            }
        }

        None
    }
}
