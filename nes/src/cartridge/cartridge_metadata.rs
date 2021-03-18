// https://wiki.nesdev.com/w/index.php/NES_2.0#Header
pub struct CartridgeMetadata {
    pub n_prg_banks: u16,
    pub n_chr_banks: u16,
    pub hardwired_mirroring: Mirroring,
    pub has_battery: bool,
    pub has_trainer: bool,
    pub mapper_num: u16,

    // Uncommon features
    pub submapper_num: u16,
    pub console_type: ConsoleType,
    pub prg_ram_shifts: u8,
    pub prg_nvram_shifts: u8,
    pub chr_ram_shifts: u8,
    pub chr_nvram_shifts: u8,
    pub timing: ClockTiming,
    pub vs_system_type: u8,
    pub extended_console_type: u8,
    pub n_misc_roms: u8,
    pub default_expansion_device: u8,
}

impl CartridgeMetadata {
    pub fn from_header(header: Vec<u8>) -> Result<Self, &'static str> {
        if header[0..=3] != [b'N', b'E', b'S', 0x1A] {
            return Err("header does not begin with NES<EOF> identifier");
        }

        let n_prg_banks = (((header[9] & 0x0F) as u16) << 8) | (header[4] as u16);
        let n_chr_banks = (((header[9] & 0xF0) as u16) << 4) | (header[5] as u16);

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

        let mapper_num = ((header[6] >> 4) as u16)
            | ((header[7] & 0xF0) as u16)
            | (((header[8] & 0x0F) as u16) << 8);
        let submapper_num = (header[8] >> 4) as u16;

        let console_type = match header[7] & 0b11 {
            0 => ConsoleType::NESFamicom,
            1 => ConsoleType::VsSystem,
            2 => ConsoleType::Playchoice10,
            _ => ConsoleType::Extended,
        };

        let prg_ram_shifts = header[10] & 0x0F;
        let prg_nvram_shifts = (header[10] & 0xF0) >> 4;
        let chr_ram_shifts = header[11] & 0x0F;
        let chr_nvram_shifts = (header[11] & 0xF0) >> 4;

        let timing = match header[12] & 0b11 {
            0 => ClockTiming::NTSC,
            1 => ClockTiming::PAL,
            2 => ClockTiming::MultiRegion,
            _ => ClockTiming::Dendy,
        };

        // TODO: These can be enums
        let vs_system_type = header[13];
        let extended_console_type = header[13] & 0x0F;

        let n_misc_roms = header[14] & 0b11;
        let default_expansion_device = header[14] & 0b11_1111;

        Ok(Self {
            n_prg_banks,
            n_chr_banks,
            hardwired_mirroring,
            has_battery,
            has_trainer,
            mapper_num,
            submapper_num,
            console_type,
            prg_ram_shifts,
            prg_nvram_shifts,
            chr_ram_shifts,
            chr_nvram_shifts,
            timing,
            vs_system_type,
            extended_console_type,
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
    Extended,
}

pub enum ClockTiming {
    NTSC,
    PAL,
    MultiRegion,
    Dendy,
}
