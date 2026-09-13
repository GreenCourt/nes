use super::bus::Bus;
use super::cartridge::Cartridge;
use super::controller::Controller;
use super::cpu::CPU;
use super::ppu::Frame;

// CPU clock = 19_687_500 / 11 Hz (NTSC)
const CPU_CLOCK_NUMERATOR: u64 = 19_687_500;
const CPU_CLOCK_DENOMINATOR: u64 = 11;

pub struct Nes {
    cpu: CPU,
    audio_sample_cycle_remainder_scaled: u64,
}

impl Nes {
    pub fn new(rom: &[u8]) -> Result<Nes, String> {
        let mut cpu = CPU::new(Bus::new(Cartridge::new(rom)?));
        cpu.reset();
        Ok(Nes {
            cpu,
            audio_sample_cycle_remainder_scaled: 0,
        })
    }

    pub fn step(&mut self, num_audio_samples: usize, sample_rate: u32) -> Vec<f32> {
        let mut samples = Vec::with_capacity(num_audio_samples);

        //
        // Track the cycle/sample ratio as an exact rational number to avoid floating-point drift.
        // Cycles per sample = CPU_CLOCK_HZ / sample_rate
        //                    = CPU_CLOCK_NUMERATOR / (CPU_CLOCK_DENOMINATOR * sample_rate)
        //
        // So `threshold` is the numerator, and `scale` is the denominator.
        //
        let scale: u64 = sample_rate as u64 * CPU_CLOCK_DENOMINATOR;
        let threshold: u64 = CPU_CLOCK_NUMERATOR;

        while samples.len() < num_audio_samples {
            let cycles = self.cpu.execute_single_instruction() as u64;
            self.audio_sample_cycle_remainder_scaled += cycles * scale;

            while self.audio_sample_cycle_remainder_scaled >= threshold
                && samples.len() < num_audio_samples
            {
                samples.push(self.cpu.bus.get_audio_sample());
                self.audio_sample_cycle_remainder_scaled -= threshold;
            }
        }
        samples
    }

    pub fn get_frame(&self) -> &Frame {
        self.cpu.bus.get_frame()
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.audio_sample_cycle_remainder_scaled = 0;
    }

    pub fn update_button_right(&mut self, pushed: bool) {
        self.cpu
            .bus
            .update_button_status(pushed, Controller::BUTTON_RIGHT);
    }

    pub fn update_button_left(&mut self, pushed: bool) {
        self.cpu
            .bus
            .update_button_status(pushed, Controller::BUTTON_LEFT);
    }

    pub fn update_button_down(&mut self, pushed: bool) {
        self.cpu
            .bus
            .update_button_status(pushed, Controller::BUTTON_DOWN);
    }

    pub fn update_button_up(&mut self, pushed: bool) {
        self.cpu
            .bus
            .update_button_status(pushed, Controller::BUTTON_UP);
    }

    pub fn update_button_start(&mut self, pushed: bool) {
        self.cpu
            .bus
            .update_button_status(pushed, Controller::BUTTON_START);
    }

    pub fn update_button_select(&mut self, pushed: bool) {
        self.cpu
            .bus
            .update_button_status(pushed, Controller::BUTTON_SELECT);
    }

    pub fn update_button_a(&mut self, pushed: bool) {
        self.cpu
            .bus
            .update_button_status(pushed, Controller::BUTTON_A);
    }

    pub fn update_button_b(&mut self, pushed: bool) {
        self.cpu
            .bus
            .update_button_status(pushed, Controller::BUTTON_B);
    }
}
