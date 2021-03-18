use crate::cartridge::Mapper;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/INES_Mapper_003
pub struct Mapper3 {
    n_chr_banks: u16,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    chr_bank: u8,
}

impl Mapper for Mapper3 {}

impl Mapper3 {
    pub fn new(n_chr_banks: u16, prg_data: Vec<u8>, chr_data: Vec<u8>) -> Self {
        Self {
            n_chr_banks: n_chr_banks,
            prg_rom: prg_data,
            chr_rom: chr_data,
            chr_bank: 0,
        }
    }
}

impl Memory for Mapper3 {
    fn read(&mut self, addr: u16) -> u8 {
        self.peek(addr)
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr <= 0x1FFF {
            self.chr_rom[(self.chr_bank as usize * 0x2000) + addr as usize]
        } else if 0x8000 <= addr {
            self.prg_rom[addr as usize - 0x8000]
        } else {
            0x0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr >= 0x8000 {
            self.chr_bank = data % (self.n_chr_banks as u8);
        }
    }
}
