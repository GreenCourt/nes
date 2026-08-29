use crate::cartridge::Mirroring;

pub struct PPU {
    register_ctrl: u8,               // $2000 - PPU_CTRL (write)
    register_mask: u8,               // $2001 - PPU_MASK (write)
    register_status: u8,             // $2002 - PPU_STATUS (read)
    register_oam_addr: u8,           // $2003 - OAM_ADDR (write)
    register_oam_data: u8,           // $2004 - OAM_DATA (read/write)
    register_scroll: ScrollRegister, // $2005 - PPU_SCROLL (write * 2)
    register_addr: u16,              // $2006 - PPU_ADDR (write * 2)
    register_data: u8,               // $2007 - PPU_DATA (read/write)
    register_oam_dma: u8,            // $4014 - OAM_DMA (write)
    latch: bool,                     // shared by PPU_SCROLL and PPU_ADDR

    open_bus_value: u8,

    chr_rom: Vec<u8>,
    palette: [u8; 32],
    vram: [u8; 2048],
    _oam: [u8; 256],
    mirroring: Mirroring,

    scanline: u16,
    cycles: usize,
    pub nmi_interrupt: Option<u8>,
}

#[derive(Default)]
struct ScrollRegister {
    x: u8,
    y: u8,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            register_ctrl: 0,
            register_mask: 0,
            register_status: 0,
            register_oam_addr: 0,
            register_oam_data: 0,
            register_scroll: ScrollRegister::default(),
            register_addr: 0,
            register_data: 0,
            register_oam_dma: 0,
            latch: false,
            open_bus_value: 0,
            chr_rom,
            palette: [0; 32],
            vram: [0; 2048],
            _oam: [0; 256],
            mirroring,
            cycles: 0,
            scanline: 0,
            nmi_interrupt: None,
        }
    }

    const _CTRL_NAMETABLE1: u8 = 0b0000_0001;
    const _CTRL_NAMETABLE2: u8 = 0b0000_0010;
    const CTRL_VRAM_ADD_INCREMENT: u8 = 0b0000_0100;
    const _CTRL_SPRITE_PATTERN_ADDR: u8 = 0b0000_1000;
    const _CTRL_BACKGROUND_PATTERN_ADDR: u8 = 0b0001_0000;
    const _CTRL_SPRITE_SIZE: u8 = 0b0010_0000;
    const _CTRL_MASTER_SLAVE_SELECT: u8 = 0b0100_0000;
    const CTRL_GENERATE_NMI: u8 = 0b1000_0000;

    const _STATUS_SPRITE_OVERFLOW: u8 = 0b0010_0000;
    const STATUS_SPRITE_ZERO_HIT: u8 = 0b0100_0000;
    const STATUS_VBLANK: u8 = 0b1000_0000;

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => self.open_bus_value,
            0x2002 => self.register_status,
            0x2004 => self.register_oam_data,
            0x2007 => {
                self.open_bus_value = self.read_data();
                self.open_bus_value
            }
            0x4014 => {
                panic!("Attempt to read a write-only ppu register: 0x{:x}", addr);
            }
            _ => {
                panic!("Unknow address for the PPU registers: 0x{:x}", addr);
            }
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        self.open_bus_value = data;
        match addr {
            0x2000 => {
                let nmi = self.register_ctrl & PPU::CTRL_GENERATE_NMI != 0;
                self.register_ctrl = data;
                if !nmi
                    && (self.register_ctrl & PPU::CTRL_GENERATE_NMI != 0)
                    && (self.register_status & PPU::STATUS_VBLANK != 0)
                {
                    self.nmi_interrupt = Some(1);
                }
            }
            0x2001 => {
                self.register_mask = data;
            }
            0x2002 => {
                panic!(
                    "Attempt to write to a write-only ppu register: 0x{:x}",
                    addr
                );
            }
            0x2003 => {
                self.register_oam_addr = data;
            }
            0x2004 => {
                self.register_oam_data = data;
            }
            0x2005 => {
                if self.latch {
                    self.register_scroll.y = data;
                } else {
                    self.register_scroll.x = data;
                }
                self.latch = !self.latch;
            }
            0x2006 => {
                self.register_addr = if self.latch {
                    (self.register_addr & 0xff00) | data as u16
                } else {
                    ((data & 0x3f) as u16) << 8 | (self.register_addr & 0xff)
                };
                self.latch = !self.latch;
            }
            0x2007 => {
                self.register_data = data;
            }
            0x4014 => {
                self.register_oam_dma = data;
            }
            _ => {
                panic!("Unknow address for the PPU registers: 0x{:x}", addr);
            }
        }
    }

    fn increment_addr_register(&mut self) {
        let inc: u16 = if (self.register_ctrl & PPU::CTRL_VRAM_ADD_INCREMENT) != 0 {
            32
        } else {
            1
        };
        self.register_addr = self.register_addr.wrapping_add(inc) & 0x3fff;
    }

    fn read_data(&mut self) -> u8 {
        let addr = match self.register_addr {
            0x3000..=0x3eff => self.register_addr - 0x1000, // mirror to 0x2000..=0x2eff
            0x3f20..=0x3fff => self.register_addr & 0x3f1f, // mirror to 0x3f00..=0x3f1f
            _ => self.register_addr,
        };
        self.increment_addr_register();

        match addr {
            0..=0x1fff => {
                let ret = self.register_data;
                self.register_data = self.chr_rom[addr as usize];
                ret
            }
            0x2000..=0x2fff => {
                let ret = self.register_data;
                self.register_data = self.vram[self.mirror_vram_addr(addr) as usize];
                ret
            }
            0x3f00..=0x3f1f => self.palette[(addr - 0x3f00) as usize],
            _ => panic!("unexpected access to mirrored space: 0x{:x}", addr),
        }
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x400;
        match (&self.mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x800,
            (Mirroring::Horizontal, 2) => vram_index - 0x400,
            (Mirroring::Horizontal, 1) => vram_index - 0x400,
            (Mirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {
            self.cycles -= 341;
            self.scanline += 1;

            if self.scanline == 241 {
                self.register_status |= PPU::STATUS_VBLANK;
                self.register_status &= !PPU::STATUS_SPRITE_ZERO_HIT;

                if self.register_ctrl & PPU::CTRL_GENERATE_NMI != 0 {
                    self.nmi_interrupt = Some(1);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.nmi_interrupt = None;
                self.register_status &= !PPU::STATUS_SPRITE_ZERO_HIT;
                self.register_status &= !PPU::STATUS_VBLANK;
            }
        }
    }

    pub fn poll_nmi_interrupt(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl PPU {
        pub fn get_scanline(&self) -> u16 {
            self.scanline
        }
        pub fn get_cycles(&self) -> usize {
            self.cycles
        }

        pub fn peek_register(&self, addr: u16) -> u8 {
            // like read_register, but without mut
            match addr {
                0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => self.open_bus_value,
                0x2002 => self.register_status,
                0x2004 => self.register_oam_data,
                0x2007 => self.peek_data(),
                0x4014 => {
                    panic!("Attempt to read a write-only ppu register: 0x{:x}", addr);
                }
                _ => {
                    panic!("Unknow address for the PPU registers: 0x{:x}", addr);
                }
            }
        }

        fn peek_data(&self) -> u8 {
            // like read_data, but without mut
            let addr = match self.register_addr {
                0x3000..=0x3eff => self.register_addr - 0x1000, // mirror to 0x2000..=0x2eff
                0x3f20..=0x3fff => self.register_addr & 0x3f1f, // mirror to 0x3f00..=0x3f1f
                _ => self.register_addr,
            };

            match addr {
                0..=0x1fff => self.register_data,
                0x2000..=0x2fff => self.register_data,
                0x3f00..=0x3f1f => self.palette[(addr - 0x3f00) as usize],
                _ => panic!("unexpected access to mirrored space: 0x{:x}", addr),
            }
        }
    }

    #[test]
    fn test_latch() {
        let chr_rom = vec![];
        let mut ppu = PPU::new(chr_rom, Mirroring::Vertical);

        assert!(!ppu.latch);
        ppu.write_register(0x2006, 0x11);
        assert!(ppu.latch);
        ppu.write_register(0x2006, 0x22);
        assert!(!ppu.latch);
        assert_eq!(ppu.register_addr, 0x1122);
        ppu.write_register(0x2006, 0x54);
        assert!(ppu.latch);
        assert_eq!(ppu.register_addr, 0x1422); // masked by 0x3fff

        ppu.write_register(0x2005, 0x44); // write to y because latch is shared
        assert!(!ppu.latch);
        assert_eq!(ppu.register_scroll.y, 0x44);

        ppu.write_register(0x2005, 0x55);
        assert!(ppu.latch);
        assert_eq!(ppu.register_scroll.y, 0x44);
        assert_eq!(ppu.register_scroll.x, 0x55);
    }

    #[test]
    fn test_increment() {
        let chr_rom = vec![];
        let mut ppu = PPU::new(chr_rom, Mirroring::Vertical);

        assert!(!ppu.latch);
        ppu.write_register(0x2006, 0x11);
        assert!(ppu.latch);
        ppu.write_register(0x2006, 0x22);
        assert!(!ppu.latch);
        assert_eq!(ppu.register_addr, 0x1122);

        ppu.increment_addr_register();
        assert!(!ppu.latch);
        assert_eq!(ppu.register_addr, 0x1123);

        ppu.register_ctrl |= PPU::CTRL_VRAM_ADD_INCREMENT;

        ppu.increment_addr_register();
        assert!(!ppu.latch);
        assert_eq!(ppu.register_addr, 0x1143);
    }

    #[test]
    fn test_read_chr_rom() {
        let chr_rom = vec![0x11, 0x22, 0x33, 0x44, 0x55];
        let mut ppu = PPU::new(chr_rom, Mirroring::Vertical);

        ppu.write_register(0x2006, 0);
        ppu.write_register(0x2006, 1);
        // addr is incremented by read
        ppu.read_register(0x2007); // dummy read
        assert_eq!(ppu.read_register(0x2007), 0x22);
        assert_eq!(ppu.read_register(0x2007), 0x33);
        assert_eq!(ppu.read_register(0x2007), 0x44);
    }
}
