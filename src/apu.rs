pub struct APU {
    status: u8,
    frame_counter: u8,
    cycles: usize,
    step: usize,
    frame_interrupt: Option<u8>,
    triangle: TriangleChannel,
}

const FRAME_PERIODS: [usize; 5] = [7457, 14913, 22371, 29829, 37281];

const STATUS_FRAME_INTERRUPT: u8 = 0b0100_0000;

impl APU {
    pub fn new() -> Self {
        APU {
            status: 0,
            frame_counter: 0,
            cycles: 0,
            step: 0,
            frame_interrupt: None,
            triangle: TriangleChannel::new(),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                self.status &= !STATUS_FRAME_INTERRUPT;
                self.status
            }
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4008 => self.triangle.write(addr, data),
            0x4009 => {}
            0x400A => self.triangle.write(addr, data),
            0x400B => self.triangle.write(addr, data),
            0x4015 => {
                self.status = (self.status & 0b1100_0000) | (data & 0b0001_1111);
                self.triangle.set_enabled(data & 0x04 != 0);
            }
            0x4017 => {
                self.frame_counter = (self.frame_counter & 0b0011_1111) | (data & 0b1100_0000);
                self.cycles = 0;
                self.step = 0;
                if self.mode() == 5 {
                    self.quarter_frame_tick();
                    self.half_frame_tick();
                }
            }
            _ => {}
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.triangle.tick_timer();

            self.cycles += 1;
            self.step_frame_counter();
        }
    }

    fn step_frame_counter(&mut self) {
        let period = FRAME_PERIODS[self.step];

        if self.cycles >= period {
            match self.mode() {
                4 => {
                    self.quarter_frame_tick();
                    if self.step == 1 || self.step == 3 {
                        self.half_frame_tick();
                    }
                    if self.step == 3 {
                        if self.irq_enabled() {
                            self.status |= STATUS_FRAME_INTERRUPT;
                            self.frame_interrupt = Some(1);
                        }
                        self.cycles = 0;
                        self.step = 0;
                        return;
                    }
                }
                5 => {
                    if self.step != 3 {
                        self.quarter_frame_tick();
                    }
                    if self.step == 1 || self.step == 4 {
                        self.half_frame_tick();
                    }
                    if self.step == 4 {
                        self.cycles = 0;
                        self.step = 0;
                        return;
                    }
                }
                _ => unreachable!(),
            }
            self.step += 1;
        }
    }

    fn quarter_frame_tick(&mut self) {
        self.triangle.tick_linear_counter();
        // TODO: pulse1/pulse2 envelope, noise envelope
    }

    fn half_frame_tick(&mut self) {
        self.triangle.tick_length_counter();
        // TODO: pulse1/pulse2 length_counter + sweep, noise length_counter
    }

    fn mode(&self) -> u8 {
        const BIT_MODE: u8 = 0b1000_0000;
        if self.frame_counter & BIT_MODE == 0 {
            4
        } else {
            5
        }
    }

    fn irq_enabled(&self) -> bool {
        const BIT_IRQ: u8 = 0b0100_0000;
        self.frame_counter & BIT_IRQ == 0
    }

    pub fn poll_frame_interrupt(&mut self) -> Option<u8> {
        self.frame_interrupt.take()
    }

    pub fn get_sample(&self) -> f32 {
        let t = self.triangle.output() as f32; // 0..15
        (t / 15.0) * 2.0 - 1.0 // normalize to -1.0..1.0
    }
}

impl Default for APU {
    fn default() -> Self {
        Self::new()
    }
}

const TRIANGLE_SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct TriangleChannel {
    enabled: bool,
    control_flag: bool,

    linear_counter_reload_value: u8,
    linear_counter: u8,
    linear_counter_reload_flag: bool,

    length_counter: u8,

    timer_period: u16,
    timer: u16,

    sequence_pos: u8,
}

impl TriangleChannel {
    pub fn new() -> Self {
        TriangleChannel {
            enabled: false,
            control_flag: false,
            linear_counter_reload_value: 0,
            linear_counter: 0,
            linear_counter_reload_flag: false,
            length_counter: 0,
            timer_period: 0,
            timer: 0,
            sequence_pos: 0,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4008 => {
                self.control_flag = (data & 0x80) != 0;
                self.linear_counter_reload_value = data & 0x7F;
            }
            0x400A => {
                self.timer_period = (self.timer_period & 0x0700) | data as u16;
            }
            0x400B => {
                self.timer_period = (self.timer_period & 0x00FF) | ((data as u16 & 0x07) << 8);
                if self.enabled {
                    self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
                }
                self.linear_counter_reload_flag = true;
            }
            _ => unreachable!(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    pub fn tick_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            if self.length_counter > 0 && self.linear_counter > 0 {
                self.sequence_pos = (self.sequence_pos + 1) % 32;
            }
        } else {
            self.timer -= 1;
        }
    }

    pub fn tick_linear_counter(&mut self) {
        if self.linear_counter_reload_flag {
            self.linear_counter = self.linear_counter_reload_value;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        if !self.control_flag {
            self.linear_counter_reload_flag = false;
        }
    }

    pub fn tick_length_counter(&mut self) {
        if !self.control_flag && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || self.timer_period < 2 {
            // mute inaudible frequency range
            return 0;
        }
        TRIANGLE_SEQUENCE[self.sequence_pos as usize]
    }
}

impl Default for TriangleChannel {
    fn default() -> Self {
        Self::new()
    }
}
