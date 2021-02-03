bitflags! {
    /// Status register (P) http://wiki.nesdev.com/w/index.php/Status_flags
    pub struct StatusRegister: u8 {
        const CARRY       = 1 << 0;
        const ZERO        = 1 << 1;
        const IRQ_DISABLE = 1 << 2;
        const DECIMAL     = 1 << 3;
        const BREAK_LO    = 1 << 4;
        const BREAK_HI    = 1 << 5;
        const OVERFLOW    = 1 << 6;
        const NEGATIVE    = 1 << 7;

        const BREAK = Self::BREAK_HI.bits | Self::BREAK_LO.bits;
    }
}

impl StatusRegister {
    pub fn set_from_stack(&mut self, val: u8) {
        self.bits = (val & !StatusRegister::BREAK_LO.bits) | StatusRegister::BREAK_HI.bits;
    }
}