use crate::channels::length_counter::LengthCounter;

// https://wiki.nesdev.com/w/index.php/APU_Triangle
pub struct TriangleChannel {
    pub timer: u16,
    pub timer_period: u16,
    pub sequence_step: u8,
    pub length_counter: LengthCounter,

    pub linear_enable: bool,
    pub linear_control: bool,
    pub linear_counter: u8,
    pub linear_period: u8,
    pub linear_reset: bool,
}

impl TriangleChannel {
    pub fn new() -> Self {
        Self {
            timer: 0,
            timer_period: 0,
            sequence_step: 0,
            length_counter: LengthCounter::new(),

            linear_enable: false,
            linear_control: false,
            linear_counter: 0,
            linear_period: 0,
            linear_reset: false,
        }
    }

    pub fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.timer = self.timer_period;
            self.sequence_step = (self.sequence_step + 1) % 32;
        }
    }

    pub fn tick_linear(&mut self) {
        if self.linear_reset {
            self.linear_counter = self.linear_period;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.linear_control {
            self.linear_reset = false;
        }
    }

    pub fn update_register(&mut self, register_offset: u16, data: u8) {
        match register_offset {
            0 => {
                self.length_counter.halt = (data >> 7) & 1 == 1; // These two share a bit
                self.linear_control = (data >> 7) & 1 == 1;
                self.linear_period = data & 0b0111_1111;
            }
            1 => unimplemented!(),
            2 => {
                self.timer_period &= 0xFF00;
                self.timer_period |= data as u16;
            }
            3 => {
                self.timer_period &= 0x00FF;
                self.timer_period |= (data as u16 & 0b111) << 8;

                if self.length_counter.enabled {
                    self.length_counter.load(data >> 3);
                }

                self.linear_reset = true;
            }
            _ => unimplemented!(),
        }
    }

    pub fn output(&self) -> u8 {
        if self.length_counter.counter == 0 || self.linear_counter == 0 {
            0
        } else {
            TRIANGLE_TABLE[self.sequence_step as usize]
        }
    }
}

const TRIANGLE_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, // Both 0 and 15 are repeated
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];
