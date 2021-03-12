use memory::Memory;

pub struct NoController {}

impl NoController {
    pub fn new() -> Self {
        Self {}
    }
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
