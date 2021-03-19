// https://wiki.nesdev.com/w/index.php/APU_Length_Counter
pub struct LengthCounter {
    pub counter: u8,
    pub halt: bool,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            halt: false,
        }
    }

    pub fn tick(&mut self) {
        if !self.halt && self.counter > 0 {
            self.counter -= 1;
        }
    }
}
