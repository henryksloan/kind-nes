use memory::ram::RAM;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/PPU_palettes
pub struct PaletteRAM {
    memory: RAM,
}

impl PaletteRAM {
    pub fn new() -> Self {
        Self {
            memory: RAM::new(0x20, 0x00),
        }
    }

    fn mirror(&self, addr: u16) -> u16 {
        // "Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C"
        let mut mirrored = addr;
        mirrored %= 0x20;
        if (mirrored % 4 == 0) && mirrored >= 0x10 {
            mirrored -= 0x10;
        }
        mirrored
    }
}

impl Memory for PaletteRAM {
    fn read(&mut self, addr: u16) -> u8 {
        self.memory.read(self.mirror(addr))
    }

    fn peek(&self, addr: u16) -> u8 {
        self.memory.peek(self.mirror(addr))
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.memory.write(self.mirror(addr), data);
    }
}
