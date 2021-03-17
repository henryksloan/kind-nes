// https://wiki.nesdev.com/w/index.php/PPU_registers
pub mod register_addrs {
    pub const PPUCTRL: u16 = 0x2000;
    pub const PPUMASK: u16 = 0x2001;
    pub const PPUSTATUS: u16 = 0x2002;
    pub const OAMADDR: u16 = 0x2003;
    pub const OAMDATA: u16 = 0x2004;
    pub const PPUSCROLL: u16 = 0x2005;
    pub const PPUADDR: u16 = 0x2006;
    pub const PPUDATA: u16 = 0x2007;
    pub const OAMDMA: u16 = 0x4014;
}

pub struct PPURegisters {
    pub ppuctrl: ControlRegister,
    pub ppumask: MaskRegister,
    pub ppustatus: StatusRegister,
    pub oamaddr: u8,
    pub oamdata: u8,
    pub ppudata: u8,

    pub curr_addr: AddressRegister,
    pub temp_addr: AddressRegister,
    pub write_latch: bool,
    pub fine_x: u8,

    pub bus_latch: u8,
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

            curr_addr: AddressRegister::new(),
            temp_addr: AddressRegister::new(),
            write_latch: false,
            fine_x: 0,

            bus_latch: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ppuctrl.bits = 0;
        self.ppumask.bits = 0;
        self.ppustatus.bits &= 0b1000_0000;
        // OAMADDR unchanged
        self.ppudata = 0x00;
        // v, t, and fine_x unchanged (?)
        self.write_latch = false;
        self.bus_latch = 0;
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

impl ControlRegister {
    pub fn write(&mut self, val: u8) {
        self.bits = val;
    }

    pub fn get_vram_increment(&self) -> u16 {
        if self.contains(Self::VRAM_INCR) {
            32
        } else {
            1
        }
    }

    pub fn get_patt_base(&self) -> u16 {
        if self.contains(Self::BACK_PATT_ADDR) {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn get_sprite_patt_base(&self) -> u16 {
        if self.contains(Self::SPRITE_PATT_ADDR) {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn get_sprite_height(&self) -> u16 {
        if self.contains(Self::SPRITE_HEIGHT) {
            16
        } else {
            8
        }
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

impl MaskRegister {
    pub fn write(&mut self, val: u8) {
        self.bits = val;
    }

    pub fn is_rendering(&self) -> bool {
        self.contains(Self::BACK_ENABLE) || self.contains(Self::SPRITE_ENABLE)
    }
}

bitflags! {
    pub struct StatusRegister: u8 {
        const SPRITE_OVERFLOW = 1 << 5;
        const SPRITE_ZERO_HIT = 1 << 6;
        const VBLANK_STARTED  = 1 << 7;
    }
}

impl StatusRegister {
    pub fn high_three(&self) -> u8 {
        self.bits & 0b111_00000
    }

    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

pub struct AddressRegister {
    pub raw: u16,
}

impl AddressRegister {
    pub fn new() -> Self {
        Self { raw: 0 }
    }

    fn bitmask(mask: (u16, u16)) -> u16 {
        ((1 << mask.1) - 1) << mask.0 // e.g. (4, 5) => 111110000
    }

    pub fn set(&mut self, mask: (u16, u16), val: u8) {
        let bitmask = Self::bitmask(mask);
        self.raw &= !bitmask;
        self.raw |= ((val as u16) << mask.0) & bitmask;
    }

    pub fn get(&self, mask: (u16, u16)) -> u16 {
        (self.raw & Self::bitmask(mask)) >> mask.0
    }

    pub fn copy_horizontal(&mut self, other: &AddressRegister) {
        self.set(vram_addr::COARSE_X, other.get(vram_addr::COARSE_X) as u8);
        self.raw &= !(1 << 10); // Low NT bit
        self.raw |= other.raw & (1 << 10);
    }

    pub fn copy_vertical(&mut self, other: &AddressRegister) {
        use vram_addr::*;
        self.set(COARSE_Y, other.get(COARSE_Y) as u8);
        self.set(FINE_Y, other.get(FINE_Y) as u8);
        self.raw &= !(1 << 11); // High NT bit
        self.raw |= other.raw & (1 << 11);
    }

    pub fn increment_horizontal(&mut self) {
        // https://wiki.nesdev.com/w/index.php/PPU_scrolling#Coarse_X_increment
        let coarse_x = self.get(vram_addr::COARSE_X) as u8;
        if coarse_x == 0b11111 {
            self.set(vram_addr::COARSE_X, 0);
            self.raw ^= 0x0400; // Switch horizontal nametable
        } else {
            self.set(vram_addr::COARSE_X, coarse_x + 1);
        }
    }

    pub fn increment_vertical(&mut self) {
        // https://wiki.nesdev.com/w/index.php/PPU_scrolling#Y_increment
        let fine_y = self.get(vram_addr::FINE_Y) as u8;
        if fine_y < 0b111 {
            self.set(vram_addr::FINE_Y, fine_y + 1);
        } else {
            self.set(vram_addr::FINE_Y, 0);
            let coarse_y = self.get(vram_addr::COARSE_Y) as u8;
            if coarse_y == 29 {
                self.set(vram_addr::COARSE_Y, 0);
                self.raw ^= 0x0800; // Switch vertical nametable
            } else if coarse_y == 31 {
                self.set(vram_addr::COARSE_Y, 0);
            } else {
                self.set(vram_addr::COARSE_Y, coarse_y + 1);
            }
        }
    }
}

pub mod vram_addr {
    pub const COARSE_X: (u16, u16) = (0, 5); // (offset, length)
    pub const COARSE_Y: (u16, u16) = (5, 5);
    pub const NAMETABLE_SEL: (u16, u16) = (10, 2);
    pub const FINE_Y: (u16, u16) = (12, 3);

    pub const LO_BYTE: (u16, u16) = (0, 8);
    pub const HI_BYTE: (u16, u16) = (8, 8);
}
