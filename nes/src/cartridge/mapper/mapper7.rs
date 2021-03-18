use crate::cartridge::Mapper;
use crate::cartridge::Mirroring;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/INES_Mapper_003
pub struct Mapper7 {
    n_prg_banks: u16,
    prg_rom: Vec<u8>,
    chr_ram: Vec<u8>,
    prg_bank: u8,
    mirroring: Mirroring,
}

impl Mapper for Mapper7 {}

impl Mapper7 {
    pub fn new(n_prg_banks: u16, prg_data: Vec<u8>) -> Self {
        Self {
            n_prg_banks: n_prg_banks,
            prg_rom: prg_data,
            chr_ram: vec![0; 0x2000],
            prg_bank: 0,
            mirroring: Mirroring::SingleScreenLower,
        }
    }
}

impl Memory for Mapper7 {
    fn read(&mut self, addr: u16) -> u8 {
        self.peek(addr)
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr <= 0x1FFF {
            self.chr_ram[addr as usize]
        } else if 0x8000 <= addr {
            self.prg_rom[0x8000 * self.prg_bank as usize + (addr as usize - 0x8000)]
        } else {
            0x0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr <= 0x1FFF {
            self.chr_ram[addr as usize] = data;
        } else if addr >= 0x8000 {
            self.mirroring = if (data >> 4) & 1 == 1 {
                Mirroring::SingleScreenUpper
            } else {
                Mirroring::SingleScreenLower
            };
            self.prg_bank = (data & 0b111) % (self.n_prg_banks as u8 / 2);
        }
    }
}
