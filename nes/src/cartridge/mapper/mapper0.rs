use crate::cartridge::Mapper;
use memory::rom::ROM;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/NROM
pub struct Mapper0 {
    n_prg_banks: u16,
    n_chr_banks: u16,
    prg_rom: ROM,
    chr_rom: ROM,
}

impl Mapper for Mapper0 {}

impl Mapper0 {
    pub fn new(n_prg_banks: u16, n_chr_banks: u16, prg_data: Vec<u8>, chr_data: Vec<u8>) -> Self {
        Self {
            n_prg_banks,
            n_chr_banks,
            prg_rom: ROM {
                memory: prg_data,
                start: 0x8000,
            },
            chr_rom: ROM {
                memory: chr_data,
                start: 0x0000,
            },
        }
    }
}

impl Memory for Mapper0 {
    fn read(&mut self, addr: u16) -> u8 {
        self.peek(addr)
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr <= 0x1FFF {
            self.chr_rom.peek(addr % (self.n_chr_banks * 0x2000))
        } else {
            self.prg_rom
                .peek(((addr - 0x8000) % (self.n_prg_banks * 0x4000)) + 0x8000)
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr <= 0x1FFF {
            self.chr_rom.write(addr % (self.n_chr_banks * 0x2000), data)
        } else {
            self.prg_rom.write(
                ((addr - 0x8000) % (self.n_prg_banks * 0x4000)) + 0x8000,
                data,
            )
        }
    }
}
