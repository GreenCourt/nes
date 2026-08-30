pub struct Controller {
    strobe: bool,
    button_index: u8,
    button_status: u8,
}

impl Controller {
    pub const BUTTON_RIGHT: u8 = 0b1000_0000;
    pub const BUTTON_LEFT: u8 = 0b0100_0000;
    pub const BUTTON_DOWN: u8 = 0b0010_0000;
    pub const BUTTON_UP: u8 = 0b0001_0000;
    pub const BUTTON_START: u8 = 0b0000_1000;
    pub const BUTTON_SELECT: u8 = 0b0000_0100;
    pub const BUTTON_A: u8 = 0b0000_0010;
    pub const BUTTON_B: u8 = 0b0000_0001;

    pub fn new() -> Self {
        Controller {
            strobe: false,
            button_index: 0,
            button_status: 0,
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }
        let res = (self.button_status & (1 << self.button_index)) >> self.button_index;
        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }
        res
    }

    pub fn update_button_status(&mut self, pushed: bool, button_bit: u8) {
        self.button_status = if pushed {
            self.button_status | button_bit
        } else {
            self.button_status & !button_bit
        }
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}
