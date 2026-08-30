use crate::cartridge::Mirroring;

pub struct PPU {
    register_ctrl: u8,               // $2000 - PPU_CTRL (write)
    register_mask: u8,               // $2001 - PPU_MASK (write)
    register_status: u8,             // $2002 - PPU_STATUS (read)
    register_oam_addr: u8,           // $2003 - OAM_ADDR (write)
    oam_data: [u8; 256],             // $2004 - OAM_DATA (read/write)
    register_scroll: ScrollRegister, // $2005 - PPU_SCROLL (write * 2)
    register_addr: u16,              // $2006 - PPU_ADDR (write * 2)
    register_data: u8,               // $2007 - PPU_DATA (read/write)

    internal_w: bool, // shared by PPU_SCROLL and PPU_ADDR
    open_bus_value: u8,

    chr_rom: Vec<u8>,
    palette: [u8; 32],
    vram: [u8; 2048],
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
            oam_data: [0; 256],
            register_scroll: ScrollRegister::default(),
            register_addr: 0,
            register_data: 0,
            internal_w: false,
            open_bus_value: 0,
            chr_rom,
            palette: [0; 32],
            vram: [0; 2048],
            mirroring,
            cycles: 0,
            scanline: 0,
            nmi_interrupt: None,
        }
    }

    const _CTRL_NAMETABLE1: u8 = 0b0000_0001;
    const _CTRL_NAMETABLE2: u8 = 0b0000_0010;
    const CTRL_VRAM_ADD_INCREMENT: u8 = 0b0000_0100;
    const CTRL_SPRITE_PATTERN_ADDR: u8 = 0b0000_1000;
    const CTRL_BACKGROUND_PATTERN_ADDR: u8 = 0b0001_0000;
    const _CTRL_SPRITE_SIZE: u8 = 0b0010_0000;
    const _CTRL_MASTER_SLAVE_SELECT: u8 = 0b0100_0000;
    const CTRL_GENERATE_NMI: u8 = 0b1000_0000;

    const _STATUS_SPRITE_OVERFLOW: u8 = 0b0010_0000;
    const STATUS_SPRITE_ZERO_HIT: u8 = 0b0100_0000;
    const STATUS_VBLANK: u8 = 0b1000_0000;

    pub fn read(&mut self, addr: u16) -> u8 {
        // Don't forget to fix the peek function if you fix this function!
        match addr {
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => self.open_bus_value,
            0x2002 => {
                let status = self.register_status;
                self.register_status &= !PPU::STATUS_VBLANK;
                self.internal_w = false;
                status
            }
            0x2004 => self.oam_data[self.register_oam_addr as usize],
            0x2007 => {
                self.open_bus_value = self.read_data();
                self.open_bus_value
            }
            _ => {
                panic!("Unknow address for the PPU registers: 0x{:x}", addr);
            }
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
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
            0x2002 => (),
            0x2003 => {
                self.register_oam_addr = data;
            }
            0x2004 => {
                self.oam_data[self.register_oam_addr as usize] = data;
                self.register_oam_addr = self.register_oam_addr.wrapping_add(1);
            }
            0x2005 => {
                if self.internal_w {
                    self.register_scroll.y = data;
                } else {
                    self.register_scroll.x = data;
                }
                self.internal_w = !self.internal_w;
            }
            0x2006 => {
                self.register_addr = if self.internal_w {
                    (self.register_addr & 0xff00) | data as u16
                } else {
                    ((data & 0x3f) as u16) << 8 | (self.register_addr & 0xff)
                };
                self.internal_w = !self.internal_w;
            }
            0x2007 => {
                self.write_data(data);
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
        // Don't forget to fix peek_data if you fix this function!
        let addr = match self.register_addr {
            0x3000..=0x3eff => self.register_addr - 0x1000, // mirror to 0x2000..=0x2eff
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.register_addr - 0x10, // mirror to 0x3f00/0x3f04/0x3f08/0x3f0c
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

    fn write_data(&mut self, value: u8) {
        let addr = match self.register_addr {
            0x3000..=0x3eff => self.register_addr - 0x1000, // mirror to 0x2000..=0x2eff
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.register_addr - 0x10, // mirror to 0x3f00/0x3f04/0x3f08/0x3f0c
            0x3f20..=0x3fff => self.register_addr & 0x3f1f, // mirror to 0x3f00..=0x3f1f
            _ => self.register_addr,
        };
        self.increment_addr_register();

        match addr {
            0..=0x1fff => println!("Attempt to write to chr rom space: 0x{:x}", addr),
            0x2000..=0x2fff => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3f00..=0x3fff => {
                self.palette[(addr - 0x3f00) as usize] = value;
            }
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

    pub fn get_frame(&self) -> Frame {
        //self.render_tiles()
        self.render_nametable()
    }

    pub fn render_nametable(&self) -> Frame {
        let mut frame = Frame::new();

        // draw background
        let bank = if self.register_ctrl & PPU::CTRL_BACKGROUND_PATTERN_ADDR != 0 {
            0x1000
        } else {
            0x0000
        };
        for i in 0..0x03c0 {
            let tile_number = self.vram[i] as u16;
            let tile_x = i % 32;
            let tile_y = i / 32;
            let tile = &self.chr_rom
                [(bank + tile_number * 16) as usize..=(bank + tile_number * 16 + 15) as usize];
            let palette = self.bg_palette(tile_x, tile_y);

            for y in 0..=7 {
                let mut upper = tile[y];
                let mut lower = tile[y + 8];

                for x in (0..=7).rev() {
                    let value = (1 & upper) << 1 | (1 & lower);
                    upper >>= 1;
                    lower >>= 1;
                    let rgb = match value {
                        0 => SYSTEM_PALETTE[self.palette[0] as usize],
                        1 => SYSTEM_PALETTE[palette[1] as usize],
                        2 => SYSTEM_PALETTE[palette[2] as usize],
                        3 => SYSTEM_PALETTE[palette[3] as usize],
                        _ => panic!("unreachable"),
                    };
                    frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb);
                }
            }
        }

        // draw sprites
        for i in (0..self.oam_data.len()).step_by(4).rev() {
            let tile_number = self.oam_data[i + 1] as u16;
            let tile_x = self.oam_data[i + 3] as usize;
            let tile_y = self.oam_data[i] as usize;

            let flip_vertical = self.oam_data[i + 2] >> 7 & 1 == 1;
            let flip_horizontal = self.oam_data[i + 2] >> 6 & 1 == 1;
            let pallette_idx = self.oam_data[i + 2] & 0b11;
            let sprite_palette = self.sprite_palette(pallette_idx);

            let bank: u16 = (self.register_ctrl & PPU::CTRL_SPRITE_PATTERN_ADDR) as u16;

            let tile = &self.chr_rom
                [(bank + tile_number * 16) as usize..=(bank + tile_number * 16 + 15) as usize];

            for y in 0..=7 {
                let mut upper = tile[y];
                let mut lower = tile[y + 8];
                'xfor: for x in (0..=7).rev() {
                    let value = (1 & lower) << 1 | (1 & upper);
                    upper >>= 1;
                    lower >>= 1;
                    let rgb = match value {
                        0 => continue 'xfor,
                        1 => SYSTEM_PALETTE[sprite_palette[1] as usize],
                        2 => SYSTEM_PALETTE[sprite_palette[2] as usize],
                        3 => SYSTEM_PALETTE[sprite_palette[3] as usize],
                        _ => panic!("unreachable"),
                    };
                    match (flip_horizontal, flip_vertical) {
                        (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
                        (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
                        (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
                        (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
                    }
                }
            }
        }
        frame
    }

    pub fn render_tiles(&self) -> Frame {
        let mut frame = Frame::new();
        let mut pos_x: usize = 0;
        let mut pos_y: usize = 0;

        for &bank in &[0x0000, 0x1000] {
            for tile_number in 0..=255 {
                // Tiles are delimited by 16-bit boundaries.
                let tile =
                    &self.chr_rom[(bank + tile_number * 16)..=(bank + tile_number * 16 + 15)];

                for y in 0..=7 {
                    let mut upper = tile[y];
                    let mut lower = tile[y + 8];

                    for x in (0..=7).rev() {
                        let value = (1 & upper) << 1 | (1 & lower);
                        upper >>= 1;
                        lower >>= 1;
                        let rgb = match value {
                            0 => SYSTEM_PALETTE[0x01],
                            1 => SYSTEM_PALETTE[0x23],
                            2 => SYSTEM_PALETTE[0x27],
                            3 => SYSTEM_PALETTE[0x30],
                            _ => panic!("unreachable"),
                        };
                        frame.set_pixel(pos_x + x, pos_y + y, rgb)
                    }
                }
                if pos_x + 9 + 9 < 256 {
                    pos_x += 9;
                } else {
                    pos_x = 0;
                    pos_y += 9;
                }
            }
            pos_x = 0;
            pos_y += 24;
        }
        frame
    }

    fn bg_palette(&self, tile_column: usize, tile_row: usize) -> [u8; 4] {
        let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
        let attr_byte = self.vram[0x3c0 + attr_table_idx]; // note: still using hardcoded first nametable

        let palette_idx = match (tile_column % 4 / 2, tile_row % 4 / 2) {
            (0, 0) => attr_byte & 0b11,
            (1, 0) => (attr_byte >> 2) & 0b11,
            (0, 1) => (attr_byte >> 4) & 0b11,
            (1, 1) => (attr_byte >> 6) & 0b11,
            (_, _) => panic!("should not happen"),
        };

        let palette_start: usize = 1 + (palette_idx as usize) * 4;
        [
            self.palette[0],
            self.palette[palette_start],
            self.palette[palette_start + 1],
            self.palette[palette_start + 2],
        ]
    }

    fn sprite_palette(&self, pallete_idx: u8) -> [u8; 4] {
        let start = 0x11 + (pallete_idx * 4) as usize;
        [
            0,
            self.palette[start],
            self.palette[start + 1],
            self.palette[start + 2],
        ]
    }
}

pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Frame {
            // w * h * rgb
            data: vec![0; Frame::WIDTH * Frame::HEIGHT * 3],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = y * 3 * Frame::WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

pub static SYSTEM_PALETTE: [(u8, u8, u8); 64] = [
    (0x80, 0x80, 0x80),
    (0x00, 0x3D, 0xA6),
    (0x00, 0x12, 0xB0),
    (0x44, 0x00, 0x96),
    (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28),
    (0xBA, 0x06, 0x00),
    (0x8C, 0x17, 0x00),
    (0x5C, 0x2F, 0x00),
    (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00),
    (0x00, 0x47, 0x2E),
    (0x00, 0x41, 0x66),
    (0x00, 0x00, 0x00),
    (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05),
    (0xC7, 0xC7, 0xC7),
    (0x00, 0x77, 0xFF),
    (0x21, 0x55, 0xFF),
    (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5),
    (0xFF, 0x29, 0x50),
    (0xFF, 0x22, 0x00),
    (0xD6, 0x32, 0x00),
    (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00),
    (0x05, 0x8F, 0x00),
    (0x00, 0x8A, 0x55),
    (0x00, 0x99, 0xCC),
    (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09),
    (0x09, 0x09, 0x09),
    (0xFF, 0xFF, 0xFF),
    (0x0F, 0xD7, 0xFF),
    (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF),
    (0xFF, 0x45, 0xF3),
    (0xFF, 0x61, 0x8B),
    (0xFF, 0x88, 0x33),
    (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20),
    (0x9F, 0xE3, 0x0E),
    (0x2B, 0xF0, 0x35),
    (0x0C, 0xF0, 0xA4),
    (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E),
    (0x0D, 0x0D, 0x0D),
    (0x0D, 0x0D, 0x0D),
    (0xFF, 0xFF, 0xFF),
    (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF),
    (0xDA, 0xAB, 0xEB),
    (0xFF, 0xA8, 0xF9),
    (0xFF, 0xAB, 0xB3),
    (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6),
    (0xFF, 0xF7, 0x9C),
    (0xD7, 0xE8, 0x95),
    (0xA6, 0xED, 0xAF),
    (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC),
    (0xDD, 0xDD, 0xDD),
    (0x11, 0x11, 0x11),
    (0x11, 0x11, 0x11),
];

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

        pub fn peek(&self, addr: u16) -> u8 {
            // like read, but without mut
            // Don't forget to fix the read function if you fix this function!
            match addr {
                0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => self.open_bus_value,
                0x2002 => self.register_status,
                0x2004 => self.oam_data[self.register_oam_addr as usize],
                0x2007 => self.peek_data(),
                _ => {
                    panic!("Unknow address for the PPU registers: 0x{:x}", addr);
                }
            }
        }

        fn peek_data(&self) -> u8 {
            // like read_data, but without mut
            // Don't forget to fix read_data if you fix this function!
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

        pub fn trace(&self) -> String {
            format!(
                "CTRL:0x{:x} MASK:0x{:x} STATUS:0x{:x} OAM_ADDR:0x{:x} SCROLL_X:0x{:x} SCROLL_Y:0x{:x} ADDR:0x{:x} DATA:0x{:x} W:{}",
                self.register_ctrl,
                self.register_mask,
                self.register_status,
                self.register_oam_addr,
                self.register_scroll.x,
                self.register_scroll.y,
                self.register_addr,
                self.register_data,
                self.internal_w as usize,
            )
        }
    }

    #[test]
    fn test_latch() {
        let chr_rom = vec![];
        let mut ppu = PPU::new(chr_rom, Mirroring::Vertical);

        assert!(!ppu.internal_w);
        ppu.write(0x2006, 0x11);
        assert!(ppu.internal_w);
        ppu.write(0x2006, 0x22);
        assert!(!ppu.internal_w);
        assert_eq!(ppu.register_addr, 0x1122);
        ppu.write(0x2006, 0x54);
        assert!(ppu.internal_w);
        assert_eq!(ppu.register_addr, 0x1422); // masked by 0x3fff

        ppu.write(0x2005, 0x44); // write to y because internal_w is shared
        assert!(!ppu.internal_w);
        assert_eq!(ppu.register_scroll.y, 0x44);

        ppu.write(0x2005, 0x55);
        assert!(ppu.internal_w);
        assert_eq!(ppu.register_scroll.y, 0x44);
        assert_eq!(ppu.register_scroll.x, 0x55);

        ppu.read(0x2002); // reading status register clears the internal_w
        assert!(!ppu.internal_w);
    }

    #[test]
    fn test_increment() {
        let chr_rom = vec![];
        let mut ppu = PPU::new(chr_rom, Mirroring::Vertical);

        assert!(!ppu.internal_w);
        ppu.write(0x2006, 0x11);
        assert!(ppu.internal_w);
        ppu.write(0x2006, 0x22);
        assert!(!ppu.internal_w);
        assert_eq!(ppu.register_addr, 0x1122);

        ppu.increment_addr_register();
        assert!(!ppu.internal_w);
        assert_eq!(ppu.register_addr, 0x1123);

        ppu.register_ctrl |= PPU::CTRL_VRAM_ADD_INCREMENT;

        ppu.increment_addr_register();
        assert!(!ppu.internal_w);
        assert_eq!(ppu.register_addr, 0x1143);
    }

    #[test]
    fn test_read_chr_rom() {
        let chr_rom = vec![0x11, 0x22, 0x33, 0x44, 0x55];
        let mut ppu = PPU::new(chr_rom, Mirroring::Vertical);

        ppu.write(0x2006, 0);
        ppu.write(0x2006, 1);
        // addr is incremented by read
        ppu.read(0x2007); // dummy read
        assert_eq!(ppu.read(0x2007), 0x22);
        assert_eq!(ppu.read(0x2007), 0x33);
        assert_eq!(ppu.read(0x2007), 0x44);
    }

    #[test]
    fn test_oam_rw() {
        let chr_rom = vec![0x11, 0x22, 0x33, 0x44, 0x55];
        let mut ppu = PPU::new(chr_rom, Mirroring::Vertical);

        ppu.write(0x2003, 12);
        assert_eq!(ppu.register_oam_addr, 12);
        ppu.write(0x2004, 55);
        assert_eq!(ppu.register_oam_addr, 13);
        assert_eq!(ppu.oam_data[12], 55);
    }
}
