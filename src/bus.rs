use super::cartridge::Cartridge;
use super::controller::Controller;
use super::ppu::{Frame, PPU};

const RAM_START: u16 = 0x0000;
const RAM_MIRROS_END: u16 = 0x1FFF;
const PPU_REGISTERS_START: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const ROM_START: u16 = 0x8000;
const ROM_END: u16 = 0xFFFF;

pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
    fn mem_read_u16(&mut self, pos: u16) -> u16;
    fn mem_write_u16(&mut self, pos: u16, data: u16);
}

pub struct Bus {
    cpu_ram: [u8; 2048],
    ppu: PPU,
    prg_rom: Vec<u8>,
    controller: Controller,
    cycles: usize,

    dma_active: bool,
    dma_page: u8,
    dma_step: u16,
    dma_buffer: u8,
    dma_total_cycles: u16,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            cpu_ram: [0; 2048],
            ppu: PPU::new(cartridge.chr_rom, cartridge.screen_mirroring),
            prg_rom: cartridge.prg_rom,
            controller: Controller::new(),
            cycles: 0,
            dma_active: false,
            dma_page: 0,
            dma_step: 0,
            dma_buffer: 0,
            dma_total_cycles: 0,
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= ROM_START;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // mirror
            addr %= 0x4000;
        }
        self.prg_rom[addr as usize]
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles = self.cycles.wrapping_add(cycles as usize);
        if self.dma_active {
            self.process_dma(cycles);
        }
        self.ppu.tick(cycles * 3);
    }

    pub fn poll_nmi_interrupt(&mut self) -> Option<u8> {
        self.ppu.poll_nmi_interrupt()
    }

    fn process_dma(&mut self, cycles: u8) {
        assert!(self.dma_active);

        let dummy_cycles = self.dma_total_cycles - 512;

        for _ in 0..=cycles {
            if self.dma_step >= dummy_cycles {
                let transfer_step = self.dma_step - dummy_cycles;
                if transfer_step.is_multiple_of(2) {
                    // even step: read data from cpu_ram and store it to the buffer
                    let addr = ((self.dma_page as u16) << 8) | (transfer_step / 2);
                    self.dma_buffer = self.mem_read(addr);
                } else {
                    // odd step: write the buffer data to ppu
                    self.ppu.write(0x2004, self.dma_buffer)
                }
            } else {
                // nothing to do
            }

            self.dma_step += 1;

            if self.dma_step >= self.dma_total_cycles {
                self.dma_active = false;
                break;
            }
        }
    }

    pub fn dma_is_active(&self) -> bool {
        self.dma_active
    }

    pub fn get_cycles(&self) -> usize {
        self.cycles
    }

    pub fn get_frame(&self) -> Frame {
        self.ppu.get_frame()
    }

    pub fn update_button_status(&mut self, pushed: bool, button_bit: u8) {
        self.controller.update_button_status(pushed, button_bit);
    }
}

impl Mem for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        // Don't forget to fix mem_peek if you fix this function!
        match addr {
            RAM_START..=RAM_MIRROS_END => {
                let mirror_down_addr = addr & 0x7FF;
                self.cpu_ram[mirror_down_addr as usize]
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0x2007;
                self.ppu.read(mirror_down_addr)
            }
            0x4016 => self.controller.read(),
            0x4017 => {
                0 // 2nd controller is not implemented
            }
            ROM_START..=ROM_END => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring mem access at 0x{:X}", addr);
                0 // TODO: dummy
            }
        }
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        // Don't forget to fix mem_peek_u16 if you fix this function!
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_MIRROS_END => {
                let mirror_down_addr = addr & 0x7FF;
                self.cpu_ram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0x2007;
                self.ppu.write(mirror_down_addr, data);
            }
            0x4014 => {
                // start OAM DMA
                self.dma_page = data;
                self.dma_active = true;
                self.dma_step = 0;
                self.dma_total_cycles = if self.cycles % 2 == 1 { 514 } else { 513 };
            }
            0x4016 => self.controller.write(data),
            0x4017 => {
                // 2nd controller is not implemented
            }
            ROM_START..=ROM_END => {
                panic!("Attempt to write to Cartridge ROM space");
            }
            _ => {
                // TODO
                println!("Ignoring mem write-access at 0x{:X}", addr);
            }
        }
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos.wrapping_add(1), hi);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    impl Bus {
        pub fn get_ppu_scanline(&self) -> u16 {
            self.ppu.get_scanline()
        }
        pub fn get_ppu_cycles(&self) -> usize {
            self.ppu.get_cycles()
        }
        pub fn mem_peek(&self, addr: u16) -> u8 {
            // like mem_read but without mut
            // Don't forget to fix mem_read if you fix this function!
            match addr {
                RAM_START..=RAM_MIRROS_END => {
                    let mirror_down_addr = addr & 0x7FF;
                    self.cpu_ram[mirror_down_addr as usize]
                }
                PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END => {
                    let mirror_down_addr = addr & 0x2007;
                    self.ppu.peek(mirror_down_addr)
                }
                0x4016 => {
                    0 // dummy
                }
                0x4017 => {
                    0 // 2nd controller is not implemented
                }
                ROM_START..=ROM_END => self.read_prg_rom(addr),
                _ => {
                    0 // TODO: dummy
                }
            }
        }
        pub fn mem_peek_u16(&self, pos: u16) -> u16 {
            // like mem_read but without mut
            // Don't forget to fix mem_read_u16 if you fix this function!
            let lo = self.mem_peek(pos) as u16;
            let hi = self.mem_peek(pos.wrapping_add(1)) as u16;
            (hi << 8) | lo
        }
        pub fn trace_ppu(&self) -> String {
            self.ppu.trace()
        }
    }
}
