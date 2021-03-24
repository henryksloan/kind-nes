use crate::cartridge::Mapper;
use crate::cartridge::Mirroring;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/MMC1
pub struct Mapper1 {
    n_prg_banks: u16,
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_mem: Vec<u8>,
    chr_mem_is_ram: bool,

    last_write_timer: u8,
    shift_register: u8,
    shift_write_count: u8,
    control_register: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
}

impl Mapper1 {
    pub fn new(n_prg_banks: u16, n_chr_banks: u16, prg_data: Vec<u8>, chr_data: Vec<u8>) -> Self {
        Self {
            n_prg_banks,
            prg_rom: prg_data,
            prg_ram: vec![0; 0x2000],
            chr_mem: if n_chr_banks == 0 {
                // TODO: RAM sizes
                vec![0; 0x2000]
            } else {
                chr_data
            },
            chr_mem_is_ram: n_chr_banks == 0,

            last_write_timer: 0,
            shift_register: 0b10000,
            shift_write_count: 0,
            control_register: 0b01100, // "MMC1 seems to reliably power on in the last bank"
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }
}

impl Mapper for Mapper1 {
    fn get_nametable_mirroring(&self) -> Option<Mirroring> {
        Some(match self.control_register & 0b11 {
            0 => Mirroring::SingleScreenLower,
            1 => Mirroring::SingleScreenUpper,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        })
    }

    fn cycle(&mut self) {
        if self.last_write_timer > 0 {
            self.last_write_timer -= 1;
        }
    }

    fn reset(&mut self) {
        self.last_write_timer = 0;
        self.shift_register = 0b10000;
        self.shift_write_count = 0;
        self.control_register = 0b01100;
        self.chr_bank_0 = 0;
        self.chr_bank_1 = 0;
        self.prg_bank = 0;

        // TODO: Different RAM sizes
        if self.chr_mem_is_ram {
            self.chr_mem = vec![0; 0x2000];
            self.prg_ram = vec![0; 0x2000];
        }
    }
}

impl Memory for Mapper1 {
    fn read(&mut self, addr: u16) -> u8 {
        self.peek(addr)
    }

    fn peek(&self, addr: u16) -> u8 {
        if 0x6000 <= addr && addr <= 0x7FFF {
            return if self.prg_bank >> 4 == 0 {
                self.prg_ram[(addr as usize) - 0x6000]
            } else {
                0
            };
        }

        if addr <= 0x0FFF {
            let chr_bank_mode = self.control_register >> 4;
            let base = if chr_bank_mode == 0 {
                0x2000 * ((self.chr_bank_0 as usize) >> 1)
            } else {
                0x1000 * (self.chr_bank_0 as usize)
            };

            let len = self.chr_mem.len();
            self.chr_mem[(base + (addr as usize)) % len]
        } else if 0x1000 <= addr && addr <= 0x1FFF {
            let chr_bank_mode = self.control_register >> 4;
            let base = if chr_bank_mode == 0 {
                0x1000 + 0x2000 * ((self.chr_bank_0 as usize) >> 1)
            } else {
                0x1000 * (self.chr_bank_1 as usize)
            };

            let len = self.chr_mem.len();
            self.chr_mem[(base + ((addr as usize) - 0x1000)) % len]
        } else if 0x8000 <= addr && addr <= 0xBFFF {
            let prg_bank_mode = (self.control_register & 0b1100) >> 2;
            let base = if prg_bank_mode == 0 || prg_bank_mode == 1 {
                0x8000 * (((self.prg_bank >> 1) as usize) & 0b111)
            } else if prg_bank_mode == 2 {
                0x0000
            } else {
                0x4000 * ((self.prg_bank as usize) & 0b1111)
            };

            self.prg_rom[base + ((addr as usize) - 0x8000)]
        } else {
            let prg_bank_mode = (self.control_register & 0b1100) >> 2;
            let base = if prg_bank_mode == 0 || prg_bank_mode == 1 {
                0x4000 + 0x8000 * (((self.prg_bank >> 1) as usize) & 0b111)
            } else if prg_bank_mode == 2 {
                0x4000 * ((self.prg_bank as usize) & 0b1111)
            } else {
                0x4000 * ((self.n_prg_banks as usize) - 1)
            };

            self.prg_rom[base + ((addr as usize) - 0xC000)]
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr <= 0x0FFF && self.chr_mem_is_ram {
            let chr_bank_mode = self.control_register >> 4;
            let base = if chr_bank_mode == 0 {
                0x2000 * ((self.chr_bank_0 as usize) >> 1)
            } else {
                0x1000 * (self.chr_bank_0 as usize)
            };

            let len = self.chr_mem.len();
            self.chr_mem[(base + (addr as usize)) % len] = data;
            return;
        } else if 0x1000 <= addr && addr <= 0x1FFF && self.chr_mem_is_ram {
            let chr_bank_mode = self.control_register >> 4;
            let base = if chr_bank_mode == 0 {
                0x1000 + 0x2000 * ((self.chr_bank_0 as usize) >> 1)
            } else {
                0x1000 * (self.chr_bank_1 as usize)
            };

            let len = self.chr_mem.len();
            self.chr_mem[(base + ((addr as usize) - 0x1000)) % len] = data;
            return;
        } else if 0x4020 <= addr && addr <= 0x5FFF {
            return;
        } else if 0x6000 <= addr && addr <= 0x7FFF {
            if self.prg_bank >> 4 == 0 {
                self.prg_ram[(addr as usize) - 0x6000] = data;
            }
            return;
        }

        if self.last_write_timer > 0 {
            return;
        }
        self.last_write_timer = 2;

        if data >> 7 == 1 {
            // "Reset shift register and write Control with (Control OR $0C),
            // locking PRG ROM at $C000-$FFFF to the last bank."
            self.shift_register = 0b10000;
            self.shift_write_count = 0;
            self.control_register |= 0b01100;
            return;
        }

        self.shift_register = ((self.shift_register >> 1) | ((data & 1) << 4)) & 0b11111;
        self.shift_write_count += 1;
        if self.shift_write_count == 5 {
            if 0x8000 <= addr && addr <= 0x9FFF {
                self.control_register = self.shift_register;
            } else if 0xA000 <= addr && addr <= 0xBFFF {
                self.chr_bank_0 = self.shift_register;
            } else if 0xC000 <= addr && addr <= 0xDFFF {
                self.chr_bank_1 = self.shift_register;
            } else {
                self.prg_bank = self.shift_register;
            }
            self.shift_register = 0b10000;
            self.shift_write_count = 0;
        }
    }
}
