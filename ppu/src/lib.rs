#[macro_use]
extern crate bitflags;

mod registers;

use memory::Memory;
use registers::PPURegisters;

pub struct PPU {
    registers: PPURegisters,

    memory: Box<dyn Memory>,
}

impl PPU {
    pub fn new(memory: Box<dyn Memory>) -> Self {
        PPU {
            registers: PPURegisters::new(),
            memory,
        }
    }

    pub fn reset(&mut self) {
        todo!()
    }

    pub fn tick(&mut self) {
        todo!()
    }

    pub fn cpu_cycle(&mut self) {
        for _ in 0..3 {
            self.tick();
        }
    }
}

impl Memory for PPU {
    fn read(&mut self, _: u16) -> u8 {
        todo!()
    }

    fn peek(&self, _: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, _: u16, _: u8) {
        todo!()
    }
}
