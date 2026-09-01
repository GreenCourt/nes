use crate::cartridge::Mirroring;

pub struct PPU {
    // $2000 - PPU_CTRL (write)
    // $2001 - PPU_MASK (write)
    // $2002 - PPU_STATUS (read)
    // $2003 - OAM_ADDR (write)
    // $2004 - OAM_DATA (read/write)
    // $2005 - PPU_SCROLL (write * 2)
    // $2006 - PPU_ADDR (write * 2)
    // $2007 - PPU_DATA (read/write)
    ctrl: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: [u8; 256],
    scroll: ScrollRegister,
    addr: u16,
    data: u8,

    w: bool, // shared by PPU_SCROLL and PPU_ADDR
    open_bus_value: u8,

    chr_rom: Vec<u8>,
    palette: [u8; 32],
    vram: [u8; 2048],
    mirroring: Mirroring,

    scanline: u16,
    cycles: usize,
    pub nmi_interrupt: Option<u8>,

    frame: Frame,
    sprite_zero_hit_cycle: Option<u16>,
}

#[derive(Default)]
struct ScrollRegister {
    x: u8,
    y: u8,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: [0; 256],
            scroll: ScrollRegister::default(),
            addr: 0,
            data: 0,
            w: false,
            open_bus_value: 0,
            chr_rom,
            palette: [0; 32],
            vram: [0; 2048],
            mirroring,
            cycles: 0,
            scanline: 0,
            nmi_interrupt: None,
            frame: Frame::new(),
            sprite_zero_hit_cycle: None,
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

    const STATUS_SPRITE_OVERFLOW: u8 = 0b0010_0000;
    const STATUS_SPRITE_ZERO_HIT: u8 = 0b0100_0000;
    const STATUS_VBLANK: u8 = 0b1000_0000;

    const MASK_SHOW_BG_LEFT: u8 = 0b0000_0010;
    const MASK_SHOW_SPRITES_LEFT: u8 = 0b0000_0100;
    const MASK_SHOW_BG: u8 = 0b0000_1000;
    const MASK_SHOW_SPRITES: u8 = 0b0001_0000;

    pub fn read(&mut self, addr: u16) -> u8 {
        // Don't forget to fix the peek function if you fix this function!
        match addr {
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => self.open_bus_value,
            0x2002 => {
                let status = self.status;
                self.status &= !PPU::STATUS_VBLANK;
                self.w = false;
                status
            }
            0x2004 => self.oam_data[self.oam_addr as usize],
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
                let nmi = self.ctrl & PPU::CTRL_GENERATE_NMI != 0;
                self.ctrl = data;
                if !nmi
                    && (self.ctrl & PPU::CTRL_GENERATE_NMI != 0)
                    && (self.status & PPU::STATUS_VBLANK != 0)
                {
                    self.nmi_interrupt = Some(1);
                }
            }
            0x2001 => {
                self.mask = data;
            }
            0x2002 => (),
            0x2003 => {
                self.oam_addr = data;
            }
            0x2004 => {
                self.oam_data[self.oam_addr as usize] = data;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            0x2005 => {
                if self.w {
                    self.scroll.y = data;
                } else {
                    self.scroll.x = data;
                }
                self.w = !self.w;
            }
            0x2006 => {
                self.addr = if self.w {
                    (self.addr & 0xff00) | data as u16
                } else {
                    ((data & 0x3f) as u16) << 8 | (self.addr & 0xff)
                };
                self.w = !self.w;
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
        let inc: u16 = if (self.ctrl & PPU::CTRL_VRAM_ADD_INCREMENT) != 0 {
            32
        } else {
            1
        };
        self.addr = self.addr.wrapping_add(inc) & 0x3fff;
    }

    fn read_data(&mut self) -> u8 {
        // Don't forget to fix peek_data if you fix this function!
        let addr = match self.addr {
            0x3000..=0x3eff => self.addr - 0x1000, // mirror to 0x2000..=0x2eff
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.addr - 0x10, // mirror to 0x3f00/0x3f04/0x3f08/0x3f0c
            0x3f20..=0x3fff => self.addr & 0x3f1f,                 // mirror to 0x3f00..=0x3f1f
            _ => self.addr,
        };
        self.increment_addr_register();

        match addr {
            0..=0x1fff => {
                let ret = self.data;
                self.data = self.chr_rom[addr as usize];
                ret
            }
            0x2000..=0x2fff => {
                let ret = self.data;
                self.data = self.vram[self.mirror_vram_addr(addr) as usize];
                ret
            }
            0x3f00..=0x3f1f => self.palette[(addr - 0x3f00) as usize],
            _ => panic!("unexpected access to mirrored space: 0x{:x}", addr),
        }
    }

    fn write_data(&mut self, value: u8) {
        let addr = match self.addr {
            0x3000..=0x3eff => self.addr - 0x1000, // mirror to 0x2000..=0x2eff
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.addr - 0x10, // mirror to 0x3f00/0x3f04/0x3f08/0x3f0c
            0x3f20..=0x3fff => self.addr & 0x3f1f,                 // mirror to 0x3f00..=0x3f1f
            _ => self.addr,
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
        let mut remaining = cycles as usize;

        while remaining > 0 {
            // loop for each scanline
            let cycles_for_current_scanline = remaining.min(341 - self.cycles);

            let old_cycles = self.cycles;
            self.cycles += cycles_for_current_scanline;
            remaining -= cycles_for_current_scanline;

            if self.status & PPU::STATUS_SPRITE_ZERO_HIT == 0
                && let Some(hit_cycle) = self.sprite_zero_hit_cycle
                && old_cycles < hit_cycle as usize
                && self.cycles >= hit_cycle as usize
            {
                self.status |= PPU::STATUS_SPRITE_ZERO_HIT;
            }

            if self.cycles >= 341 {
                // proceed to next scanline
                self.cycles -= 341;

                if self.scanline < 240 {
                    self.draw_scanline_bg(false);
                    self.draw_scanline_sprites();
                    self.update_sprite_overflow_flag();
                }

                self.scanline += 1;

                if self.scanline == 241 {
                    self.status |= PPU::STATUS_VBLANK;
                    if self.ctrl & PPU::CTRL_GENERATE_NMI != 0 {
                        self.nmi_interrupt = Some(1);
                    }
                }

                if self.scanline >= 262 {
                    self.scanline = 0;
                    self.nmi_interrupt = None;
                    self.status &= !PPU::STATUS_SPRITE_ZERO_HIT;
                    self.status &= !PPU::STATUS_SPRITE_OVERFLOW;
                    self.status &= !PPU::STATUS_VBLANK;
                }

                self.sprite_zero_hit_cycle = self.calc_sprite_zero_hit_cycle();
            }
        }
    }

    fn draw_scanline_bg(&mut self, calc_only: bool) -> Vec<bool> {
        if self.mask & PPU::MASK_SHOW_BG == 0 {
            let backdrop = SYSTEM_PALETTE[self.palette[0] as usize];
            for x in 0..256usize {
                self.frame.set_pixel(x, self.scanline as usize, backdrop);
            }
            return vec![false; 256];
        }

        let tile_row = (self.scanline / 8) as usize;
        let y_in_tile = (self.scanline % 8) as usize;

        let mut opaque = vec![false; 256];

        // draw background
        let bank = if self.ctrl & PPU::CTRL_BACKGROUND_PATTERN_ADDR != 0 {
            0x1000
        } else {
            0x0000
        };
        for tile_col in 0..32usize {
            let tile_number = self.vram[tile_row * 32 + tile_col] as u16;
            let tile = &self.chr_rom
                [(bank + tile_number * 16) as usize..=(bank + tile_number * 16 + 15) as usize];
            let palette = self.bg_palette(tile_col, tile_row);

            let mut upper = tile[y_in_tile];
            let mut lower = tile[y_in_tile + 8];

            for x_in_tile in (0..=7).rev() {
                let mut value = (1 & upper) << 1 | (1 & lower);
                upper >>= 1;
                lower >>= 1;

                let x_in_frame = tile_col * 8 + x_in_tile;
                if x_in_frame > 256 {
                    continue;
                }

                if self.mask & PPU::MASK_SHOW_BG_LEFT == 0 && x_in_frame < 8 {
                    value = 0;
                }

                let rgb = match value {
                    0 => SYSTEM_PALETTE[self.palette[0] as usize],
                    1 => SYSTEM_PALETTE[palette[1] as usize],
                    2 => SYSTEM_PALETTE[palette[2] as usize],
                    3 => SYSTEM_PALETTE[palette[3] as usize],
                    _ => panic!("unreachable"),
                };

                opaque[x_in_frame] = value != 0;
                if !calc_only {
                    self.frame
                        .set_pixel(x_in_frame, self.scanline as usize, rgb);
                }
            }
        }
        opaque
    }

    fn draw_scanline_sprites(&mut self) {
        if self.mask & PPU::MASK_SHOW_SPRITES == 0 {
            return;
        }

        let bank: u16 = (self.ctrl & PPU::CTRL_SPRITE_PATTERN_ADDR) as u16;
        let mut candidates: Vec<usize> = Vec::with_capacity(8);
        for i in (0..self.oam_data.len()).step_by(4) {
            let y = self.oam_data[i] as u16;
            if self.scanline >= y && self.scanline < y + 8 {
                candidates.push(i);
                if candidates.len() == 8 {
                    break; // max 8 sprites for single line
                }
            }
        }

        for &i in candidates.iter().rev() {
            let tile_number = self.oam_data[i + 1] as u16;
            let sprite_y = self.oam_data[i] as u16;
            let sprite_x = self.oam_data[i + 3] as usize;
            let attr = self.oam_data[i + 2];

            let flip_vertical = attr >> 7 & 1 == 1;
            let flip_horizontal = attr >> 6 & 1 == 1;
            let pallette_idx = attr & 0b11;
            let sprite_palette = self.sprite_palette(pallette_idx);

            let y_in_sprite = if flip_vertical {
                7 - (self.scanline - sprite_y)
            } else {
                self.scanline - sprite_y
            } as usize;

            let tile = &self.chr_rom
                [(bank + tile_number * 16) as usize..=(bank + tile_number * 16 + 15) as usize];
            let mut upper = tile[y_in_sprite];
            let mut lower = tile[y_in_sprite + 8];

            for x in (0..=7).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper >>= 1;
                lower >>= 1;
                if value == 0 {
                    continue; // transparent pixels
                }

                let x_in_frame = if flip_horizontal {
                    sprite_x + 7 - x
                } else {
                    sprite_x + x
                };
                if x_in_frame >= 256 {
                    continue;
                }

                if x_in_frame < 8 && self.mask & PPU::MASK_SHOW_SPRITES_LEFT == 0 {
                    continue;
                }

                let rgb = match value {
                    1 => SYSTEM_PALETTE[sprite_palette[1] as usize],
                    2 => SYSTEM_PALETTE[sprite_palette[2] as usize],
                    3 => SYSTEM_PALETTE[sprite_palette[3] as usize],
                    _ => unreachable!(),
                };

                self.frame
                    .set_pixel(x_in_frame, self.scanline as usize, rgb);
            }
        }
    }

    fn calc_sprite_zero_hit_cycle(&mut self) -> Option<u16> {
        if self.scanline >= 240 {
            return None;
        }

        if self.mask & PPU::MASK_SHOW_BG == 0 || self.mask & PPU::MASK_SHOW_SPRITES == 0 {
            return None;
        }

        let sprite_y = self.oam_data[0] as u16;
        if self.scanline < sprite_y || self.scanline >= sprite_y + 8 {
            // sprite zero is not on current scanline
            return None;
        }

        let tile_number = self.oam_data[1] as u16;
        let attr = self.oam_data[2];
        let sprite_x = self.oam_data[3] as usize;
        let flip_vertical = attr >> 7 & 1 == 1;
        let flip_horizontal = attr >> 6 & 1 == 1;

        let bank: u16 = (self.ctrl & PPU::CTRL_SPRITE_PATTERN_ADDR) as u16;
        let y_in_sprite = if flip_vertical {
            7 - (self.scanline - sprite_y)
        } else {
            self.scanline - sprite_y
        } as usize;

        let tile = &self.chr_rom
            [(bank + tile_number * 16) as usize..=(bank + tile_number * 16 + 15) as usize];
        let mut upper = tile[y_in_sprite];
        let mut lower = tile[y_in_sprite + 8];

        let bg_opaque = self.draw_scanline_bg(true);

        for x in (0..=7).rev() {
            let value = (1 & lower) << 1 | (1 & upper);
            upper >>= 1;
            lower >>= 1;
            if value == 0 {
                continue;
            }

            let screen_x = if flip_horizontal {
                sprite_x + 7 - x
            } else {
                sprite_x + x
            };

            if screen_x >= 255 {
                continue; // not hit on x=255
            }
            if screen_x < 8 {
                if self.mask & PPU::MASK_SHOW_BG_LEFT == 0 {
                    continue;
                }
                if self.mask & PPU::MASK_SHOW_SPRITES_LEFT == 0 {
                    continue;
                }
            }

            if bg_opaque[screen_x] {
                return Some((screen_x + 1) as u16); // +1 is requied
            }
        }
        None
    }

    fn update_sprite_overflow_flag(&mut self) {
        if (self.mask & (PPU::MASK_SHOW_BG | PPU::MASK_SHOW_SPRITES)) == 0 {
            return;
        }

        let mut count = 0;
        for i in (0..self.oam_data.len()).step_by(4) {
            let y = self.oam_data[i] as u16;
            if self.scanline >= y && self.scanline < y + 8 {
                count += 1;
                if count > 8 {
                    self.status |= PPU::STATUS_SPRITE_OVERFLOW;
                    return;
                }
            }
        }
    }

    pub fn poll_nmi_interrupt(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }

    pub fn get_frame(&self) -> &Frame {
        &self.frame
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
                0x2002 => self.status,
                0x2004 => self.oam_data[self.oam_addr as usize],
                0x2007 => self.peek_data(),
                _ => {
                    panic!("Unknow address for the PPU registers: 0x{:x}", addr);
                }
            }
        }

        fn peek_data(&self) -> u8 {
            // like read_data, but without mut
            // Don't forget to fix read_data if you fix this function!
            let addr = match self.addr {
                0x3000..=0x3eff => self.addr - 0x1000, // mirror to 0x2000..=0x2eff
                0x3f20..=0x3fff => self.addr & 0x3f1f, // mirror to 0x3f00..=0x3f1f
                _ => self.addr,
            };

            match addr {
                0..=0x1fff => self.data,
                0x2000..=0x2fff => self.data,
                0x3f00..=0x3f1f => self.palette[(addr - 0x3f00) as usize],
                _ => panic!("unexpected access to mirrored space: 0x{:x}", addr),
            }
        }

        pub fn trace(&self) -> String {
            format!(
                "CTRL:0x{:x} MASK:0x{:x} STATUS:0x{:x} OAM_ADDR:0x{:x} SCROLL_X:0x{:x} SCROLL_Y:0x{:x} ADDR:0x{:x} DATA:0x{:x} W:{}",
                self.ctrl,
                self.mask,
                self.status,
                self.oam_addr,
                self.scroll.x,
                self.scroll.y,
                self.addr,
                self.data,
                self.w as usize,
            )
        }
    }

    #[test]
    fn test_latch() {
        let chr_rom = vec![];
        let mut ppu = PPU::new(chr_rom, Mirroring::Vertical);

        assert!(!ppu.w);
        ppu.write(0x2006, 0x11);
        assert!(ppu.w);
        ppu.write(0x2006, 0x22);
        assert!(!ppu.w);
        assert_eq!(ppu.addr, 0x1122);
        ppu.write(0x2006, 0x54);
        assert!(ppu.w);
        assert_eq!(ppu.addr, 0x1422); // masked by 0x3fff

        ppu.write(0x2005, 0x44); // write to y because w is shared
        assert!(!ppu.w);
        assert_eq!(ppu.scroll.y, 0x44);

        ppu.write(0x2005, 0x55);
        assert!(ppu.w);
        assert_eq!(ppu.scroll.y, 0x44);
        assert_eq!(ppu.scroll.x, 0x55);

        ppu.read(0x2002); // reading status register clears the w
        assert!(!ppu.w);
    }

    #[test]
    fn test_increment() {
        let chr_rom = vec![];
        let mut ppu = PPU::new(chr_rom, Mirroring::Vertical);

        assert!(!ppu.w);
        ppu.write(0x2006, 0x11);
        assert!(ppu.w);
        ppu.write(0x2006, 0x22);
        assert!(!ppu.w);
        assert_eq!(ppu.addr, 0x1122);

        ppu.increment_addr_register();
        assert!(!ppu.w);
        assert_eq!(ppu.addr, 0x1123);

        ppu.ctrl |= PPU::CTRL_VRAM_ADD_INCREMENT;

        ppu.increment_addr_register();
        assert!(!ppu.w);
        assert_eq!(ppu.addr, 0x1143);
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
        assert_eq!(ppu.oam_addr, 12);
        ppu.write(0x2004, 55);
        assert_eq!(ppu.oam_addr, 13);
        assert_eq!(ppu.oam_data[12], 55);
    }
}
