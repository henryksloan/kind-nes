// https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
pub struct SpriteData {
    pub registers: [SpriteRegisters; 8],
    pub eval_state: SpriteEvalState,
    pub spr_num: u8,    // "sprite n (0-63)"
    pub byte_num: u8,   // "byte m (0-3)"
    pub oam_byte: u8,   // Filled on odd cycles
    pub oam2_index: u8, // Number of sprites found on this line.
}

impl SpriteData {
    pub fn new() -> Self {
        Self {
            registers: [SpriteRegisters::new(); 8],
            eval_state: SpriteEvalState::CopyY,
            spr_num: 0,
            byte_num: 0,
            oam_byte: 0,
            oam2_index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.eval_state = SpriteEvalState::CopyY;
        self.spr_num = 0;
        self.byte_num = 0;
        self.oam_byte = 0;
        self.oam2_index = 0;
    }
}

#[derive(Clone, Copy)]
pub struct SpriteRegisters {
    pub num: u16,
    pub patt_shift: [u8; 2],
    pub attr_latch: u8,
    pub x_counter: u8,
}

impl SpriteRegisters {
    pub fn new() -> Self {
        Self {
            num: 0,
            patt_shift: [0; 2],
            attr_latch: 0,
            x_counter: 0,
        }
    }
}

// https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation#Details Cycles 65-256: Sprite evaluation
pub enum SpriteEvalState {
    CopyY,                // Step 1
    CopyRemaining(usize), // Step 1a (with index m to be copied)
    IncrementN,           // Step 2
    EvaluateAsY,          // Step 3
    Overflow(usize),      // Step 3a (with number of bytes read so far (0-2))
    Done,                 // Step 4
}
