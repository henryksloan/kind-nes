use crate::cartridge::Mapper;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/UxROM
pub struct Mapper2 {
    n_prg_banks: u16,
    prg_rom: Vec<u8>,
    chr_mem: Vec<u8>,
    prg_bank: u8,
}

impl Mapper for Mapper2 {}

impl Mapper2 {
    pub fn new(n_prg_banks: u16, prg_data: Vec<u8>) -> Self {
        Self {
            n_prg_banks,
            chr_mem: vec![0; 0x2000],
            prg_rom: prg_data,
            prg_bank: 0,
        }
    }
}

impl Memory for Mapper2 {
    fn read(&mut self, addr: u16) -> u8 {
        self.peek(addr)
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr <= 0x1FFF {
            self.chr_mem[addr as usize % self.chr_mem.len()]
        } else if 0x8000 <= addr && addr <= 0xBFFF {
            self.prg_rom[self.prg_bank as usize * 0x4000 + (addr as usize - 0x8000)]
        } else if 0xC000 <= addr {
            self.prg_rom[(self.n_prg_banks as usize - 1) * 0x4000 + (addr as usize - 0xC000)]
        } else {
            0x0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr <= 0x1FFF {
            let len = self.chr_mem.len();
            self.chr_mem[addr as usize % len] = data;
        } else if addr >= 0x8000 {
            self.prg_bank = data;
        }
    }
}
