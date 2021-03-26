use crate::cartridge::Mapper;
use crate::cartridge::Mirroring;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/INES_Mapper_003
pub struct Mapper9 {
    n_prg_banks: u16,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,

    prg_bank: u8,
    chr_fd_bank_lo: u8,
    chr_fe_bank_lo: u8,
    chr_fd_bank_hi: u8,
    chr_fe_bank_hi: u8,
    chr_latch_0: bool,
    chr_latch_1: bool,

    mirroring: Mirroring,
}

impl Mapper for Mapper9 {
    fn get_nametable_mirroring(&self) -> Option<Mirroring> {
        Some(self.mirroring)
    }
}

impl Mapper9 {
    pub fn new(n_prg_banks: u16, prg_data: Vec<u8>, chr_data: Vec<u8>) -> Self {
        Self {
            n_prg_banks,
            prg_rom: prg_data,
            chr_rom: chr_data,

            prg_bank: 0,
            chr_fd_bank_lo: 0,
            chr_fe_bank_lo: 0,
            chr_fd_bank_hi: 0,
            chr_fe_bank_hi: 0,
            chr_latch_0: false,
            chr_latch_1: false,

            mirroring: Mirroring::Vertical,
        }
    }
}

impl Memory for Mapper9 {
    fn read(&mut self, addr: u16) -> u8 {
        // Latch changes go into effect only on the next read
        let data = self.peek(addr);

        if addr == 0x0FD8 {
            self.chr_latch_0 = false;
        } else if addr == 0x0FE8 {
            self.chr_latch_0 = true;
        } else if 0x1FD8 <= addr && addr <= 0x1FDF {
            self.chr_latch_1 = false;
        } else if 0x1FE8 <= addr && addr <= 0x1FEF {
            self.chr_latch_1 = true;
        }

        data
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr <= 0x0FFF {
            if self.chr_latch_0 {
                self.chr_rom[(self.chr_fe_bank_lo as usize * 0x1000) + addr as usize]
            } else {
                self.chr_rom[(self.chr_fd_bank_lo as usize * 0x1000) + addr as usize]
            }
        } else if 0x1000 <= addr && addr <= 0x1FFF {
            if self.chr_latch_1 {
                self.chr_rom[(self.chr_fe_bank_hi as usize * 0x1000) + (addr as usize - 0x1000)]
            } else {
                self.chr_rom[(self.chr_fd_bank_hi as usize * 0x1000) + (addr as usize - 0x1000)]
            }
        } else if 0x8000 <= addr && addr <= 0x9FFF {
            self.prg_rom[(self.prg_bank as usize * 0x2000) + (addr as usize - 0x8000)]
        } else if 0xA000 <= addr {
            self.prg_rom[((self.n_prg_banks as usize * 2 - 3) * 0x2000) + (addr as usize - 0xA000)]
        } else {
            0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if 0xA000 <= addr && addr <= 0xAFFF {
            self.prg_bank = (data & 0b1111) % (self.n_prg_banks as u8 * 2);
        } else if 0xB000 <= addr && addr <= 0xBFFF {
            self.chr_fd_bank_lo = data & 0b11111;
        } else if 0xC000 <= addr && addr <= 0xCFFF {
            self.chr_fe_bank_lo = data & 0b11111;
        } else if 0xD000 <= addr && addr <= 0xDFFF {
            self.chr_fd_bank_hi = data & 0b11111;
        } else if 0xE000 <= addr && addr <= 0xEFFF {
            self.chr_fe_bank_hi = data & 0b11111;
        } else if 0xF000 <= addr {
            self.mirroring = if data & 1 == 1 {
                Mirroring::Horizontal
            } else {
                Mirroring::Vertical
            };
        }
    }
}
