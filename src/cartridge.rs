use std::fs;

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

impl Cartridge {
    pub fn new(raw: &[u8]) -> Result<Cartridge, String> {
        if raw[0..4] != NES_TAG {
            return Err("File is not in iNES file format".to_string());
        }

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        // A size of zero for chr_rom is interpreted
        // as having CHR_RAM instead of CHR_ROM.
        let has_chr_ram = chr_rom_size == 0;

        let _has_battery = raw[6] & 0b10 != 0;

        Ok(Cartridge {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: if has_chr_ram {
                vec![0u8; 8192] // 8KiB as CHR-RAM
            } else {
                raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec()
            },
            prg_ram: vec![0u8; 8192], // always prepare 8KiB prg_ram for the iNES format.
            mapper,
            screen_mirroring,
        })
    }

    pub fn from_file(path: &str) -> Result<Cartridge, String> {
        let bytes: Vec<u8> = fs::read(path).map_err(|e| e.to_string())?;
        Cartridge::new(&bytes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl Cartridge {
        pub fn from_opcodes(ops: &[u8]) -> Self {
            let mut prg_rom = vec![0; 0xFFFF - 0x8000 + 1];
            prg_rom[..ops.len()].copy_from_slice(ops);
            let reset = 0xFFFC - 0x8000;
            prg_rom[reset] = 0x00;
            prg_rom[reset + 1] = 0x80;
            Cartridge {
                prg_rom,
                chr_rom: Vec::<u8>::new(),
                prg_ram: Vec::<u8>::new(),
                mapper: 0,
                screen_mirroring: Mirroring::FourScreen,
            }
        }
    }
}
