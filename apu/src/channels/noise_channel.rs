use crate::channels::{envelope::Envelope, length_counter::LengthCounter};

// https://wiki.nesdev.com/w/index.php/APU_Noise
pub struct NoiseChannel {
    pub enabled: bool,
    pub even_latch: bool,
    pub timer: u16,
    pub timer_period: u16,
    pub length_counter: LengthCounter,
    pub envelope: Envelope,

    pub shift: u16,
    pub shift_mode: bool,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            even_latch: true,
            timer: 0,
            timer_period: 0,
            length_counter: LengthCounter::new(),
            envelope: Envelope::new(),

            shift: 1,
            shift_mode: false,
        }
    }

    pub fn tick(&mut self) {
        if self.even_latch {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                self.timer = self.timer_period;

                // 1. Feedback is calculated as the exclusive-OR of bit 0 and one other bit: bit 6 if Mode flag is set, otherwise bit 1.
                // 2. The shift register is shifted right by one bit.
                // 3. Bit 14, the leftmost bit, is set to the feedback calculated earlier.
                let feedback_xor_bit_index = if self.shift_mode { 6 } else { 1 };
                let feedback_xor_bit =
                    (self.shift & (1 << feedback_xor_bit_index)) >> feedback_xor_bit_index;
                let feedback = (self.shift & 1) | feedback_xor_bit;
                self.shift >>= (self.shift >> 1) | (feedback << 14);
            }
        }
        self.even_latch = !self.even_latch;
    }
}
