use crate::cartridge::Mapper;
use crate::cartridge::Mirroring;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/MMC3
pub struct Mapper4 {
    n_prg_banks: u16,
    n_chr_banks: u16,
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_mem: Vec<u8>,
    chr_mem_is_ram: bool,

    // Memory mapping ($8000-$9FFF, $A000-$BFFF)
    select_bank_register: u8,
    bank_registers: [u8; 8],
    prg_bank_mode: bool,
    chr_bank_mode: bool,
    mirroring: Mirroring,
    write_protection: bool,
    prg_ram_enable: bool,

    // Scanline counting ($C000-$DFFF, and $E000-$FFFF)
    irq_counter: u8,
    irq_latch: u8,
    schedule_irq_reload: bool,
    irq_enable: bool,
    previous_a12: u16,
    trigger_irq: bool,

    // A submapper with PRG ram and write protection
    is_mmc6: bool,
    mmc6_ram_enable: bool,
    mmc6_ram_lo_write: bool,
    mmc6_ram_lo_read: bool,
    mmc6_ram_hi_write: bool,
    mmc6_ram_hi_read: bool,
}

impl Mapper for Mapper4 {
    fn get_nametable_mirroring(&self) -> Option<Mirroring> {
        Some(self.mirroring)
    }

    fn check_irq(&mut self) -> bool {
        let ret = self.trigger_irq;
        self.trigger_irq = false;
        ret
    }
}

impl Mapper4 {
    pub fn new(
        n_prg_banks: u16,
        n_chr_banks: u16,
        prg_data: Vec<u8>,
        chr_data: Vec<u8>,
        is_mmc6: bool,
    ) -> Self {
        Self {
            n_prg_banks,
            n_chr_banks: std::cmp::max(n_chr_banks, 1),
            prg_rom: prg_data,
            prg_ram: vec![0; 0x2000],
            chr_mem: if n_chr_banks == 0 {
                // TODO: RAM sizes
                vec![0; 0x2000]
            } else {
                chr_data
            },
            chr_mem_is_ram: n_chr_banks == 0,

            select_bank_register: 0,
            bank_registers: [0; 8],
            prg_bank_mode: false,
            chr_bank_mode: false,
            mirroring: Mirroring::Vertical,
            write_protection: false,
            prg_ram_enable: false,

            irq_counter: 0,
            irq_latch: 0,
            schedule_irq_reload: false,
            irq_enable: false,
            previous_a12: 0,
            trigger_irq: false,

            is_mmc6,
            mmc6_ram_enable: false,
            mmc6_ram_lo_write: false,
            mmc6_ram_lo_read: false,
            mmc6_ram_hi_write: false,
            mmc6_ram_hi_read: false,
        }
    }

    fn map_chr(&self, addr: u16) -> usize {
        if (self.chr_bank_mode && addr >= 0x1000) || (!self.chr_bank_mode && addr < 0x1000) {
            let bank = self.bank_registers[((addr % 0x1000) >= 0x800) as usize];
            let high = if (addr % 0x800) >= 0x400 { 0x400 } else { 0 };
            0x800 * ((bank as usize >> 1) % (self.n_chr_banks as usize * 4)) + high
        } else {
            let bank = self.bank_registers[((addr as usize % 0x1000) / 0x400) + 2];
            0x400 * (bank as usize % (self.n_chr_banks as usize * 8))
        }
    }

    fn map_prg(&self, addr: u16) -> usize {
        let r6 = (self.bank_registers[6] & 0b111111) as u16;
        let r7 = (self.bank_registers[7] & 0b111111) as u16;
        let last = self.n_prg_banks * 2 - 1;
        let second_last = self.n_prg_banks * 2 - 2;
        let banks = if self.prg_bank_mode {
            [second_last, r7, r6, last]
        } else {
            [r6, r7, second_last, last]
        };

        0x2000 * banks[(addr as usize - 0x8000) / 0x2000] as usize
    }
}

impl Memory for Mapper4 {
    fn read(&mut self, addr: u16) -> u8 {
        // IRQ counter is a side-effect of reading
        if addr <= 0x1FFF {
            let a12 = (addr >> 12) & 1;
            if (self.previous_a12 == 0) && (a12 == 1) {
                if self.schedule_irq_reload {
                    self.irq_counter = self.irq_latch;
                    self.schedule_irq_reload = false;
                } else if self.irq_counter == 0 {
                    self.irq_counter = self.irq_latch;
                } else {
                    self.irq_counter -= 1;
                }

                if self.irq_counter == 0 && self.irq_enable {
                    self.trigger_irq = true;
                }
            }
            self.previous_a12 = a12;
        }

        self.peek(addr)
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr <= 0x1FFF {
            self.chr_mem[self.map_chr(addr) + (addr as usize % 0x400)]
        } else if 0x6000 <= addr && addr <= 0x7FFF {
            if self.is_mmc6 {
                if self.mmc6_ram_enable && addr >= 0x7000 {
                    let mirrored = (addr - 0x7000) % 0x400;
                    if (mirrored < 0x200 && self.mmc6_ram_lo_read)
                        || (mirrored >= 0x200 && self.mmc6_ram_hi_read)
                    {
                        self.prg_ram[mirrored as usize]
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                if self.prg_ram_enable {
                    self.prg_ram[addr as usize - 0x6000]
                } else {
                    0
                }
            }
        } else if addr >= 0x8000 {
            self.prg_rom[self.map_prg(addr) + (addr as usize % 0x2000)]
        } else {
            0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        let even = (addr % 2) == 0;
        if addr <= 0x1FFF {
            if self.chr_mem_is_ram {
                let mapped_addr = self.map_chr(addr) + (addr as usize % 0x400);
                self.chr_mem[mapped_addr] = data;
            }
        } else if 0x6000 <= addr && addr <= 0x7FFF {
            if self.is_mmc6 {
                if self.mmc6_ram_enable && addr >= 0x7000 {
                    let mirrored = (addr - 0x7000) % 0x400;
                    if (mirrored < 0x200 && self.mmc6_ram_lo_write)
                        || (mirrored >= 0x200 && self.mmc6_ram_hi_write)
                    {
                        self.prg_ram[mirrored as usize] = data;
                    }
                }
            } else {
                if self.prg_ram_enable && !self.write_protection {
                    self.prg_ram[addr as usize - 0x6000] = data;
                }
            }
        } else if 0x8000 <= addr && addr <= 0x9FFF {
            if even {
                // Bank select
                self.select_bank_register = data & 0b111;

                if self.is_mmc6 {
                    self.mmc6_ram_enable = ((data >> 5) & 1) == 1;
                }

                self.prg_bank_mode = ((data >> 6) & 1) == 1;
                self.chr_bank_mode = ((data >> 7) & 1) == 1;
            } else {
                // Bank data
                self.bank_registers[self.select_bank_register as usize] = data;
            }
        } else if 0xA000 <= addr && addr <= 0xBFFF {
            if even {
                // Mirroring
                self.mirroring = if data & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                };
            } else {
                // PRG RAM protect
                if self.is_mmc6 {
                    if self.mmc6_ram_enable {
                        self.mmc6_ram_lo_write = ((data >> 4) & 1) == 1;
                        self.mmc6_ram_lo_read = ((data >> 5) & 1) == 1;
                        self.mmc6_ram_hi_write = ((data >> 6) & 1) == 1;
                        self.mmc6_ram_hi_read = ((data >> 7) & 1) == 1;
                    }
                } else {
                    self.write_protection = ((data >> 6) & 1) == 1;
                    self.prg_ram_enable = ((data >> 7) & 1) == 1;
                }
            }
        } else if 0xC000 <= addr && addr <= 0xDFFF {
            if even {
                // IRQ latch
                self.irq_latch = data;
            } else {
                // IRQ reload
                self.schedule_irq_reload = true;
            }
        } else if 0xE000 <= addr {
            self.irq_enable = !even;
        }
    }
}
