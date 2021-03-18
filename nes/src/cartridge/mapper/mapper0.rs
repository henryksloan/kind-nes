use crate::cartridge::Mapper;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/NROM
pub struct Mapper0 {
    n_prg_banks: u16,
    prg_rom: Vec<u8>,
    chr_mem: Vec<u8>,
    chr_mem_is_ram: bool,
}

impl Mapper for Mapper0 {}

impl Mapper0 {
    pub fn new(n_prg_banks: u16, n_chr_banks: u16, prg_data: Vec<u8>, chr_data: Vec<u8>) -> Self {
        Self {
            n_prg_banks,
            chr_mem: if n_chr_banks == 0 {
                // TODO: RAM sizes
                vec![0; 0x2000]
            } else {
                chr_data
            },
            chr_mem_is_ram: n_chr_banks == 0,
            prg_rom: prg_data,
        }
    }
}

impl Memory for Mapper0 {
    fn read(&mut self, addr: u16) -> u8 {
        self.peek(addr)
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr <= 0x1FFF {
            self.chr_mem[addr as usize % self.chr_mem.len()]
        } else {
            self.prg_rom[((addr as usize - 0x8000) % (self.n_prg_banks as usize * 0x4000))]
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr <= 0x1FFF {
            if self.chr_mem_is_ram {
                let len = self.chr_mem.len();
                self.chr_mem[addr as usize % len] = data;
            }
        } else {
            self.prg_rom[((addr as usize - 0x8000) % (self.n_prg_banks as usize * 0x4000))] = data;
        }
    }
}
