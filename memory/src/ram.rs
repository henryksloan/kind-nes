use crate::Memory;

pub struct RAM {
    pub memory: Vec<u8>,
    pub start: u16,
}

impl RAM {
    pub fn new(size: u16, start: u16) -> Self {
        RAM {
            memory: vec![0; size as usize],
            start,
        }
    }
}

impl Memory for RAM {
    fn read(&mut self, addr: u16) -> u8 {
        self.memory[(addr - self.start) as usize]
    }

    fn peek(&self, addr: u16) -> u8 {
        self.memory[(addr - self.start) as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.memory[(addr - self.start) as usize] = data;
    }
}
