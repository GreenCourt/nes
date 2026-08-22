use super::cartridge::Cartridge;
use super::ppu::PPU;

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
    cycles: usize,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            cpu_ram: [0; 2048],
            ppu: PPU::new(cartridge.chr_rom, cartridge.screen_mirroring),
            prg_rom: cartridge.prg_rom,
            cycles: 0,
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
        self.cycles += cycles as usize;
        self.ppu.tick(cycles * 3);
    }
}

impl Mem for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_MIRROS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_ram[mirror_down_addr as usize]
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END | 0x4014 => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.ppu.read_register(mirror_down_addr)
            }
            ROM_START..=ROM_END => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring mem access at 0x{:X}", addr);
                todo!("out-of-range addr 0x{:X}", addr);
            }
        }
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_MIRROS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_ram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS_START..=PPU_REGISTERS_MIRRORS_END | 0x4014 => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.ppu.write_register(mirror_down_addr, data);
            }
            ROM_START..=ROM_END => {
                panic!("Attempt to write to Cartridge ROM space");
            }
            _ => {
                println!("Ignoring mem write-access at 0x{:X}", addr);
                todo!("out-of-range addr 0x{:X}", addr);
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
        pub fn get_cycles(&self) -> usize {
            self.cycles
        }
        pub fn get_ppu_scanline(&self) -> u16 {
            self.ppu.get_scanline()
        }
        pub fn get_ppu_cycles(&self) -> usize {
            self.ppu.get_cycles()
        }
    }
}
