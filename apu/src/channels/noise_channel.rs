use crate::channels::{envelope::Envelope, length_counter::LengthCounter};

// https://wiki.nesdev.com/w/index.php/APU_Noise
pub struct NoiseChannel {
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
                let feedback = (self.shift & 1) ^ feedback_xor_bit;
                self.shift = (self.shift >> 1) | (feedback << 14);
            }
        }
        self.even_latch = !self.even_latch;
    }

    pub fn update_register(&mut self, register_offset: u16, data: u8) {
        match register_offset {
            0 => {
                self.length_counter.halt = (data >> 5) & 1 == 1; // These two share a bit
                self.envelope.loop_flag = (data >> 5) & 1 == 1;
                self.envelope.constant_volume = (data >> 4) & 1 == 1;
                self.envelope.period = data & 0b1111;
            }
            1 => unimplemented!(),
            2 => {
                self.timer_period = NTSC_PERIOD_TABLE[data as usize & 0b1111];
                self.shift_mode = (data >> 7) & 1 == 1;
            }
            3 => {
                if self.length_counter.enabled {
                    self.length_counter.load(data >> 3);
                }

                self.envelope.start = true;
            }
            _ => unimplemented!(),
        }
    }

    pub fn output(&self) -> u8 {
        if self.shift & 1 == 1 || self.length_counter.counter == 0 {
            0
        } else {
            self.envelope.get_volume()
        }
    }
}

const NTSC_PERIOD_TABLE: [u16; 16] = [
    0x004, 0x008, 0x010, 0x020, 0x040, 0x060, 0x080, 0x0A0, 0x0CA, 0x0FE, 0x17C, 0x1FC, 0x2FA,
    0x3F8, 0x7F2, 0xFE4,
];
