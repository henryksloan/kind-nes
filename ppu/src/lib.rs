#[macro_use]
extern crate bitflags;

mod registers;
mod scan;

use memory::ram::RAM;
use memory::Memory;
use registers::*;
use scan::Scan;

pub struct PPU {
    registers: PPURegisters,
    scan: Scan,
    memory: Box<dyn Memory>,
    oam: RAM,
}

impl PPU {
    pub fn new(memory: Box<dyn Memory>) -> Self {
        PPU {
            registers: PPURegisters::new(),
            scan: Scan::new(),
            memory,
            oam: RAM::new(0xF0, 0),
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
    fn read(&mut self, addr: u16) -> u8 {
        assert!((addr >= 0x2000 && addr <= 0x2007) || addr == 0x4014);

        match addr {
            register_addrs::PPUSTATUS => {
                let mut high_three = self.registers.ppustatus.high_three();

                // "Race condition: Reading PPUSTATUS within two cycles of the start of
                // vertical blank will return 0 in bit 7 but clear the latch anyway,
                // causing NMI to not occur that frame."
                // (https://wiki.nesdev.com/w/index.php/PPU_programmer_reference#Status_.28.242002.29_.3C_read)
                if self.scan.line == 241 && self.scan.cycle == 0 {
                    high_three &= !0x80; // Set V to 0
                }

                self.registers.write_latch = false;
                self.registers
                    .ppustatus
                    .remove(StatusRegister::VBLANK_STARTED);
                // TODO: NMI

                high_three | (self.registers.bus_latch & 0b000_11111)
            }
            register_addrs::OAMDATA => {
                // https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
                // During secondary OAM clear, the secondary OAM actually still functions as usual;
                // however, a signal activates that pulls reads of OAMDATA to $FF
                if self.scan.is_clearing_oam2() {
                    0xFF
                } else {
                    self.oam.read(self.registers.oamaddr as u16)
                }
            }
            register_addrs::PPUDATA => {
                let old_ppudata = self.registers.ppudata;
                self.registers.ppudata = self.memory.read(self.registers.curr_addr.raw);

                let old_addr = self.registers.curr_addr.raw;
                if self.registers.ppuctrl.contains(ControlRegister::VRAM_INCR) {
                    self.registers.curr_addr.raw += 32;
                } else {
                    self.registers.curr_addr.raw += 1;
                };

                // Usually, reading PPUDATA updates the register but returns the old value
                // Reading palette data ($3F00-$3FFF), however, places the new data directly on the bus
                if old_addr <= 0x3EFF {
                    old_ppudata
                } else {
                    self.registers.ppudata
                }
            }
            _ => self.registers.bus_latch, // Whatever latent data is on the data bus
        }
    }

    fn peek(&self, addr: u16) -> u8 {
        assert!((addr >= 0x2000 && addr <= 0x2007) || addr == 0x4014);

        match addr {
            register_addrs::PPUSTATUS => {
                let mut high_three = self.registers.ppustatus.high_three();

                if self.scan.line == 241 && self.scan.cycle == 0 {
                    high_three &= !0x80;
                }

                high_three | (self.registers.bus_latch & 0b000_11111)
            }
            register_addrs::OAMDATA => {
                if self.scan.is_clearing_oam2() {
                    0xFF
                } else {
                    self.oam.peek(self.registers.oamaddr as u16)
                }
            }
            register_addrs::PPUDATA => {
                if self.registers.curr_addr.raw <= 0x3EFF {
                    self.registers.ppudata
                } else {
                    self.memory.peek(self.registers.curr_addr.raw)
                }
            }
            register_addrs::OAMDMA => 0x00,
            _ => self.registers.bus_latch,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        assert!((addr >= 0x2000 && addr <= 0x2007) || addr == 0x4014);

        match addr {
            register_addrs::PPUCTRL => {
                let old_nmi = self.registers.ppuctrl.contains(ControlRegister::NMI_ENABLE);
                self.registers.ppuctrl.write(data);
                let nmi_rising_edge =
                    !old_nmi && self.registers.ppuctrl.contains(ControlRegister::NMI_ENABLE);
                let vblank_set = self
                    .registers
                    .ppustatus
                    .contains(StatusRegister::VBLANK_STARTED);
                if vblank_set && nmi_rising_edge {
                    // TODO: NMI
                }
                self.registers
                    .temp_addr
                    .set(vram_addr::NAMETABLE_SEL, data & 0b11);
            }
            register_addrs::PPUMASK => self.registers.ppumask.write(data),
            register_addrs::OAMADDR => self.registers.oamaddr = data,
            register_addrs::OAMDATA => {
                self.oam.write(self.registers.oamaddr as u16, data);
                self.registers.oamaddr += 1;
            }
            register_addrs::PPUSCROLL => {
                use vram_addr::*;
                if !self.registers.write_latch {
                    self.registers.temp_addr.set(COARSE_X, data >> 3);
                    self.registers.fine_x = data & 0b111;
                } else {
                    self.registers.temp_addr.set(COARSE_Y, data >> 3);
                    self.registers.temp_addr.set(FINE_Y, data & 0b111);
                }
                self.registers.write_latch = !self.registers.write_latch;
            }
            register_addrs::PPUADDR => {
                use vram_addr::*;
                if !self.registers.write_latch {
                    self.registers.temp_addr.set(HI_BYTE, data & 0b00111111);
                } else {
                    self.registers.temp_addr.set(LO_BYTE, data);
                    self.registers.curr_addr.raw = self.registers.temp_addr.raw;
                }
                self.registers.write_latch = !self.registers.write_latch;
            }
            register_addrs::PPUDATA => {
                todo!()
            }
            register_addrs::OAMDMA => {
                todo!()
            }
            _ => unimplemented!(),
        }
    }
}
