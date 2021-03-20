// https://wiki.nesdev.com/w/index.php/APU_DMC
pub struct DMCChannel {
    pub enabled: bool,
    pub even_latch: bool,
    pub timer: u16,
    pub timer_period: u16,
}

impl DMCChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            even_latch: true,
            timer: 0,
            timer_period: 0,
        }
    }

    pub fn tick(&mut self) {
        todo!()
    }

    pub fn update_enabled(&mut self, control_bit: u8) {
        todo!()
    }
}
