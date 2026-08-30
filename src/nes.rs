use super::bus::Bus;
use super::cartridge::Cartridge;
use super::cpu::CPU;
use super::ppu::Frame;

const CPU_CLOCK_HZ: f64 = 1_789_773.0;

pub struct Nes {
    cpu: CPU,
    overshoot_cycles: usize,
}

impl Nes {
    pub fn new(rom: &[u8]) -> Result<Nes, String> {
        let mut cpu = CPU::new(Bus::new(Cartridge::new(rom)?));
        cpu.reset();
        Ok(Nes {
            cpu,
            overshoot_cycles: 0,
        })
    }

    pub fn step(&mut self, time_sec: f64) {
        let cycles_to_elapse: usize = (time_sec * CPU_CLOCK_HZ) as usize + self.overshoot_cycles;
        let mut elapsed_cycles: usize = 0;
        while elapsed_cycles < cycles_to_elapse {
            elapsed_cycles += self.cpu.execute_single_instruction();
        }
        self.overshoot_cycles = elapsed_cycles - cycles_to_elapse;
    }

    pub fn get_frame(&self) -> Frame {
        self.cpu.bus.get_frame()
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.overshoot_cycles = 0;
    }
}
