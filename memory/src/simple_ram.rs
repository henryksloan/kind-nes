use crate::Memory;

pub struct SimpleRAM {
    pub memory: [u8; 0xFFFF],
}

impl SimpleRAM {
    pub fn new() -> Self {
        SimpleRAM {
            memory: [0; 0xFFFF],
        }
    }
}

impl Memory for SimpleRAM {
    fn read(&mut self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}
