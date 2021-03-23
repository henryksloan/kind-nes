// https://wiki.nesdev.com/w/index.php/APU_Envelope
pub struct Envelope {
    pub constant_volume: bool,
    pub loop_flag: bool,
    pub start: bool,
    pub period: u8, // Either the constant volume or envelope period
    timer: u8,
    decay_counter: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            constant_volume: false,
            loop_flag: false,
            start: false,
            period: 0,
            timer: 0,
            decay_counter: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.start {
            self.start = false;
            self.decay_counter = 15;
            self.timer = self.period;
        } else {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                self.timer = self.period;
                if self.decay_counter > 0 {
                    self.decay_counter -= 1;
                } else if self.loop_flag {
                    self.decay_counter = 15;
                }
            }
        }
    }

    pub fn get_volume(&self) -> u8 {
        if self.constant_volume {
            self.period
        } else {
            self.decay_counter
        }
    }
}
