// https://wiki.nesdev.com/w/index.php/PPU_registers
pub struct PPURegisters {
    pub ppuctrl: ControlRegister,
    pub ppumask: MaskRegister,
    pub ppustatus: StatusRegister,
    pub oamaddr: u8,
    pub oamdata: u8,
    pub ppudata: u8,
    pub oamdma: u8,

    pub curr_addr: AddressRegister,
    pub temp_addr: AddressRegister,
    pub write_latch: bool,
    pub fine_x: u8,
}

// https://wiki.nesdev.com/w/index.php/PPU_power_up_state
impl PPURegisters {
    pub fn new() -> Self {
        Self {
            ppuctrl: ControlRegister::empty(),
            ppumask: MaskRegister::empty(),
            ppustatus: StatusRegister::from_bits(0b10100000).unwrap(),
            oamaddr: 0x00,
            oamdata: 0x00,
            ppudata: 0x00,
            oamdma: 0x00,

            curr_addr: AddressRegister::empty(),
            temp_addr: AddressRegister::empty(),
            write_latch: false,
            fine_x: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ppuctrl.bits = 0;
        self.ppumask.bits = 0;
        self.ppustatus.bits &= 0b1000_0000;
        // OAMADDR unchanged
        self.write_latch = false;
        // v, t, and fine_x unchanged (?)
        self.ppudata = 0x00;
        self.oamdma = 0x00;
    }
}

bitflags! {
    pub struct ControlRegister: u8 {
        const NAMETABLE_LO     = 1 << 0;
        const NAMETABLE_HI     = 1 << 1;
        const VRAM_INCR        = 1 << 2;
        const SPRITE_PATT_ADDR = 1 << 3;
        const BACK_PATT_ADDR   = 1 << 4;
        const SPRITE_HEIGHT    = 1 << 5;
        const MASTER_SLAVE     = 1 << 6;
        const NMI_ENABLE       = 1 << 7;

        const NAMETABLE = Self::NAMETABLE_HI.bits | Self::NAMETABLE_LO.bits;
    }
}

bitflags! {
    pub struct MaskRegister: u8 {
        const GRAYSCALE       = 1 << 0;
        const BACK_LEFT_COL   = 1 << 1;
        const SPRITE_LEFT_COL = 1 << 2;
        const BACK_ENABLE     = 1 << 3;
        const SPRITE_ENABLE   = 1 << 4;
        const EMPHASIZE_R     = 1 << 5;
        const EMPHASIZE_G     = 1 << 6;
        const EMPHASIZE_B     = 1 << 7;
    }
}

bitflags! {
    pub struct StatusRegister: u8 {
        const SPRITE_OVERFLOW = 1 << 5;
        const SPRITE_ZERO_HIT = 1 << 6;
        const VBLANK_STARTED  = 1 << 7;
    }
}

bitflags! {
    pub struct AddressRegister: u16 {
        const COARSE_X       = 0b000_00_00000_11111;
        const COARSE_Y       = 0b000_00_11111_00000;
        const NAMETABLE_SEL  = 0b000_11_00000_00000;
        const FINE_Y         = 0b111_00_00000_00000;
    }
}
