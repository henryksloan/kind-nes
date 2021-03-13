pub struct BackgroundData {
    pub latch: BackgroundLatches,
    pub shift: BackgroundShifts,
}

impl BackgroundData {
    pub fn new() -> Self {
        Self {
            latch: BackgroundLatches {
                nt_byte: 0,
                attr_byte: 0,
                patt_lo: 0,
                patt_hi: 0,
            },
            shift: BackgroundShifts {
                attr_shift: [0; 2],
                attr_latch: [false; 2],
                patt_shift: [0; 2],
            },
        }
    }
}

pub struct BackgroundLatches {
    pub nt_byte: u8,
    pub attr_byte: u8,
    pub patt_lo: u8,
    pub patt_hi: u8,
}

pub struct BackgroundShifts {
    pub attr_shift: [u8; 2],
    pub attr_latch: [bool; 2],
    pub patt_shift: [u16; 2],
}
