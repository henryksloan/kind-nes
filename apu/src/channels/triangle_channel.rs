use crate::channels::length_counter::LengthCounter;

// https://wiki.nesdev.com/w/index.php/APU_Triangle
pub struct TriangleChannel {
    pub enabled: bool,
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
            enabled: false,
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
}
