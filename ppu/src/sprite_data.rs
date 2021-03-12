// https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
pub struct SpriteData {
    pub registers: [SpriteRegisters; 8],
    pub spr_num: u8,  // "sprite n (0-63)"
    pub byte_num: u8, // "byte m (0-3)"
    pub oam_byte: u8, // Filled on odd cycles
    // TODO: Can I just use spr_num % 8? I think not?
    pub oam2_index: u8, // Number of sprites found on this line.
}

impl SpriteData {
    pub fn new() -> Self {
        Self {
            registers: [SpriteRegisters::new(); 8],
            spr_num: 0,
            byte_num: 0,
            oam_byte: 0,
            oam2_index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.byte_num = 0;
        self.oam_byte = 0;
    }
}

#[derive(Clone, Copy)]
pub struct SpriteRegisters {
    pub patt_shift: [u8; 2],
    pub attr_latch: u8,
    pub x_counter: u8,
}

impl SpriteRegisters {
    pub fn new() -> Self {
        Self {
            patt_shift: [0; 2],
            attr_latch: 0,
            x_counter: 0,
        }
    }
}
