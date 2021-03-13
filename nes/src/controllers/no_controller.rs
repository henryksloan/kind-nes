use crate::controllers::Controller;
use memory::Memory;

pub struct NoController {}

impl NoController {
    pub fn new() -> Self {
        Self {}
    }
}

impl Controller for NoController {
    fn is_controller_1(&self) -> bool {
        false
    }

    fn get_shift_strobe(&self) -> bool {
        false
    }

    fn set_state_shift(&mut self, _: u8) {}
}

impl Memory for NoController {
    fn read(&mut self, _: u16) -> u8 {
        0
    }

    fn peek(&self, _: u16) -> u8 {
        0
    }

    fn write(&mut self, _: u16, _: u8) {}
}
