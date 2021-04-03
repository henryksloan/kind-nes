// https://wiki.nesdev.com/w/index.php/INES
// https://wiki.nesdev.com/w/index.php/NES_2.0#Header
pub struct CartridgeMetadata {
    pub is_nes2: bool,
    pub n_prg_banks: u16,
    pub n_chr_banks: u16,
    pub hardwired_mirroring: Mirroring,
    pub has_battery: bool,
    pub has_trainer: bool,
    pub mapper_num: u16,

    // Uncommon features
    pub submapper_num: Option<u16>,
    pub console_type: ConsoleType,
    pub prg_ram_bytes: usize,
    pub prg_nvram_bytes: usize,
    pub chr_ram_bytes: usize,
    pub chr_nvram_bytes: usize,
    pub timing: ClockTiming,
    pub vs_system_type: Option<u8>,
    pub n_misc_roms: Option<u8>,
    pub default_expansion_device: Option<u8>,
}

impl CartridgeMetadata {
    pub fn from_header(header: Vec<u8>) -> Result<Self, &'static str> {
        if header[0..=3] != [b'N', b'E', b'S', 0x1A] {
            return Err("header does not begin with NES<EOF> identifier");
        }

        let is_nes2 = (header[7] & 0b1100) == 0b1000;

        let mut n_prg_banks = header[4] as u16;
        let mut n_chr_banks = header[5] as u16;

        let hardwired_mirroring = if ((header[6] >> 3) & 1) == 1 {
            Mirroring::FourScreen
        } else {
            match header[6] & 1 {
                0 => Mirroring::Horizontal,
                _ => Mirroring::Vertical,
            }
        };
        let has_battery = ((header[6] >> 1) & 1) == 1;
        let has_trainer = ((header[6] >> 2) & 1) == 1;
        let mut mapper_num = (header[6] >> 4) as u16;

        let mut submapper_num = None;
        let mut timing = ClockTiming::NTSC;
        let mut vs_system_type = None;
        let mut n_misc_roms = None;
        let mut default_expansion_device = None;

        let (console_type, prg_ram_bytes, prg_nvram_bytes, chr_ram_bytes, chr_nvram_bytes) =
            if is_nes2 {
                let console_type = match header[7] & 0b11 {
                    0 => ConsoleType::NESFamicom,
                    1 => ConsoleType::VsSystem,
                    2 => ConsoleType::Playchoice10,
                    _ => match header[13] & 0b1111 {
                        0x0 => ConsoleType::NESFamicom,
                        0x1 => ConsoleType::VsSystem,
                        0x2 => ConsoleType::Playchoice10,
                        0x3 => ConsoleType::DecimalFamiclone,
                        0x4 => ConsoleType::VT01Monochrome,
                        0x5 => ConsoleType::VT01STN,
                        0x6 => ConsoleType::VT02,
                        0x7 => ConsoleType::VT03,
                        0x8 => ConsoleType::VT09,
                        0x9 => ConsoleType::VT32,
                        0xA => ConsoleType::VT369,
                        0xB => ConsoleType::UM6578,
                        _ => ConsoleType::Other,
                    },
                };

                mapper_num |= ((header[7] & 0xF0) as u16) | (((header[8] & 0x0F) as u16) << 8);
                submapper_num = Some((header[8] >> 4) as u16);

                n_prg_banks |= ((header[9] & 0x0F) as u16) << 8;
                n_chr_banks |= ((header[9] & 0xF0) as u16) << 4;

                let prg_ram_bytes = 64usize << (header[10] & 0x0F);
                let prg_nvram_bytes = 64usize << ((header[10] & 0xF0) >> 4);
                let chr_ram_bytes = 64usize << (header[11] & 0x0F);
                let chr_nvram_bytes = 64usize << ((header[11] & 0xF0) >> 4);

                timing = match header[12] & 0b11 {
                    0 => ClockTiming::NTSC,
                    1 => ClockTiming::PAL,
                    2 => ClockTiming::MultiRegion,
                    _ => ClockTiming::Dendy,
                };

                // This could be an enum if it's ever used
                vs_system_type = Some(header[13]);

                n_misc_roms = Some(header[14] & 0b11);
                default_expansion_device = Some(header[14] & 0b11_1111);

                (
                    console_type,
                    prg_ram_bytes,
                    prg_nvram_bytes,
                    chr_ram_bytes,
                    chr_nvram_bytes,
                )
            } else {
                let console_type = match header[7] & 0b11 {
                    0 => ConsoleType::NESFamicom,
                    1 => ConsoleType::VsSystem,
                    _ => ConsoleType::Playchoice10,
                };

                // Either PRG-RAM or PRG-NVRAM, depending on the battery flag in byte 6
                let mut some_prg_ram_bytes = 0x2000;

                // "A general rule of thumb: if the last 4 bytes are not all zero, and the header is not marked for NES 2.0 format,
                // an emulator should either mask off the upper 4 bits of the mapper number or simply refuse to load the ROM."
                if header[12] == 0 && header[13] == 0 && header[14] == 0 && header[15] == 0 {
                    mapper_num |= (header[7] & 0xF0) as u16;

                    // "Size of PRG RAM in 8 KB units (Value 0 infers 8 KB for compatibility; see PRG RAM circuit)"
                    some_prg_ram_bytes *= std::cmp::max(header[8], 1) as usize;
                }

                let chr_ram_bytes = if n_chr_banks == 0 { 0x2000 } else { 0 };

                let (prg_ram_bytes, prg_nvram_bytes) = if has_battery {
                    (0, some_prg_ram_bytes)
                } else {
                    (some_prg_ram_bytes, 0)
                };

                (
                    console_type,
                    prg_ram_bytes,
                    prg_nvram_bytes,
                    chr_ram_bytes,
                    0,
                )
            };

        Ok(Self {
            is_nes2,
            n_prg_banks,
            n_chr_banks,
            hardwired_mirroring,
            has_battery,
            has_trainer,
            mapper_num,
            submapper_num,
            console_type,
            prg_ram_bytes,
            prg_nvram_bytes,
            chr_ram_bytes,
            chr_nvram_bytes,
            timing,
            vs_system_type,
            n_misc_roms,
            default_expansion_device,
        })
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreenUpper,
    SingleScreenLower,
    FourScreen,
}

pub enum ConsoleType {
    NESFamicom,
    VsSystem,
    Playchoice10,
    DecimalFamiclone,
    VT01Monochrome,
    VT01STN,
    VT02,
    VT03,
    VT09,
    VT32,
    VT369,
    UM6578,
    Other,
}

pub enum ClockTiming {
    NTSC,
    PAL,
    MultiRegion,
    Dendy,
}
