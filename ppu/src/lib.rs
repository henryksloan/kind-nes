#[macro_use]
extern crate bitflags;

mod background_data;
mod registers;
mod scan;
mod sprite_data;

use background_data::BackgroundData;
use memory::ram::RAM;
use memory::Memory;
use registers::*;
use scan::Scan;
use sprite_data::SpriteData;

use std::cell::RefCell;
use std::rc::Rc;

pub struct PPU {
    registers: PPURegisters,
    scan: Scan,
    bg_data: BackgroundData,
    spr_data: SpriteData,
    memory: Box<dyn Memory>,
    oam: RAM,
    oam2: RAM,
    dma_option: Option<Rc<RefCell<dyn Memory>>>,
    dma_request: Option<u8>,
    pub framebuffer: [[u8; 256]; 240],
    pub nmi: bool,
    pub frame_ready: bool,
}

impl PPU {
    pub fn new(memory: Box<dyn Memory>) -> Self {
        PPU {
            registers: PPURegisters::new(),
            scan: Scan::new(),
            bg_data: BackgroundData::new(),
            spr_data: SpriteData::new(),
            memory,
            oam: RAM::new(0x100, 0),
            oam2: RAM::new(0x20, 0),
            dma_option: None,
            dma_request: None,
            framebuffer: [[0; 256]; 240],
            nmi: false,
            frame_ready: false,
        }
    }

    pub fn set_dma(&mut self, dma: Rc<RefCell<dyn Memory>>) {
        self.dma_option = Some(dma);
    }

    pub fn reset(&mut self) {
        self.registers.reset();
        self.scan = Scan::new();
        self.bg_data = BackgroundData::new();
        self.spr_data = SpriteData::new();
        self.dma_request = None;
        self.framebuffer = [[0; 256]; 240];
        self.nmi = false;
        self.frame_ready = false;
    }

    pub fn tick(&mut self) {
        if let Some(data) = self.dma_request {
            self.dma_request = None;
            self.run_oam_dma(data);
        }

        // https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
        // Background operations happend on visible lines and pre-render line
        if self.scan.on_visible_line() || self.scan.on_prerender_line() {
            if self.scan.on_idle_cycle() {
                // Idle
            } else if self.scan.on_bg_fetch_cycle() {
                self.bg_fetch((self.scan.cycle - 1) % 8);
            } else if self.scan.cycle == 257 && self.registers.ppumask.is_rendering() {
                // https://wiki.nesdev.com/w/index.php/PPU_scrolling#At_dot_257_of_each_scanline
                self.registers
                    .curr_addr
                    .copy_horizontal(&self.registers.temp_addr);
            }

            if self.scan.on_spr_fetch_cycle() {
                self.spr_fetch((self.scan.cycle - 257) / 8, (self.scan.cycle - 1) % 8);
            }
        }

        // Sprite operations that happen only on visible lines
        if self.scan.on_visible_line() {
            if self.scan.on_oam2_clear_cycle() {
                if (self.scan.cycle - 1) % 2 == 0 {
                    self.oam2.write((self.scan.cycle - 1) / 2, 0xFF);
                }
            } else if self.scan.on_spr_eval_cycle() {
                if self.scan.cycle == 65 {
                    self.spr_data.reset();
                }
                self.spr_eval((self.scan.cycle % 2) == 1);
            }

            if 1 <= self.scan.cycle && self.scan.cycle <= 256 {
                let (mut pixel_on, mut color) = self.get_bg_pixel();
                let (spr_pixel_on, spr_color) = self.get_spr_pixel();
                if spr_pixel_on {
                    pixel_on = true;
                    color = spr_color;
                }
                let x = (self.scan.cycle - 1) as usize;
                let y = self.scan.line as usize;
                self.framebuffer[y][x] = if pixel_on {
                    color
                } else {
                    self.memory.read(0x3F00)
                }
            }
        }

        // https://wiki.nesdev.com/w/index.php/PPU_scrolling#During_dots_280_to_304_of_the_pre-render_scanline_.28end_of_vblank.29
        if self.scan.on_prerender_line()
            && self.registers.ppumask.is_rendering()
            && (280 <= self.scan.cycle && self.scan.cycle <= 304)
        {
            self.registers
                .curr_addr
                .copy_vertical(&self.registers.temp_addr);
        }

        // Set or clear VBlank and other flags
        if self.scan.cycle == 1 {
            if self.scan.line == 241 {
                self.registers
                    .ppustatus
                    .insert(StatusRegister::VBLANK_STARTED);
                if self.registers.ppuctrl.contains(ControlRegister::NMI_ENABLE) {
                    self.nmi = true;
                }
                self.frame_ready = true;
            } else if self.scan.line == 261 {
                self.spr_data.spr_num = 0;
                self.spr_data.oam2_index = 0;
                self.registers.ppustatus.clear();
            }
        }

        self.scan.increment(self.registers.ppumask.is_rendering());
    }

    pub fn cpu_cycle(&mut self) {
        for _ in 0..3 {
            self.tick();
        }
    }

    fn bg_fetch(&mut self, cycles_into_tile: u16) {
        // https://wiki.nesdev.com/w/index.php/PPU_rendering#Cycles_1-256
        match cycles_into_tile {
            0 => {
                self.bg_data.shift.patt_shift[0] &= 0xFF00;
                self.bg_data.shift.patt_shift[0] |= self.bg_data.latch.patt_lo as u16;
                self.bg_data.shift.patt_shift[1] &= 0xFF00;
                self.bg_data.shift.patt_shift[1] |= self.bg_data.latch.patt_hi as u16;

                self.bg_data.shift.attr_latch[0] = (self.bg_data.latch.attr_byte & 0b01) == 0b01;
                self.bg_data.shift.attr_latch[1] = (self.bg_data.latch.attr_byte & 0b10) == 0b10;

                // https://wiki.nesdev.com/w/index.php/PPU_scrolling#Tile_and_attribute_fetching
                // Read tile data from a nametable
                let nt_addr = 0x2000 | (self.registers.curr_addr.raw & 0x0FFF);
                self.bg_data.latch.nt_byte = self.memory.read(nt_addr);
            }
            2 => {
                // https://wiki.nesdev.com/w/index.php/PPU_scrolling#Tile_and_attribute_fetching
                // Read attribute data from the nametable's attribute table
                let attr_addr = 0x23C0
                    | (self.registers.curr_addr.raw & 0x0C00) // Nametable select
                    | ((self.registers.curr_addr.raw >> 4) & 0x38) // High 3 coarse Y => attr table row
                    | ((self.registers.curr_addr.raw >> 2) & 0x07); // High 3 coarse X => attr table col
                self.bg_data.latch.attr_byte = self.memory.read(attr_addr);

                // https://wiki.nesdev.com/w/index.php/PPU_attribute_tables
                // Move the correct bit pair to the low end of the latch
                if self.registers.curr_addr.get(vram_addr::COARSE_Y) & 0b10 == 0b10 {
                    self.bg_data.latch.attr_byte >>= 4;
                }
                if self.registers.curr_addr.get(vram_addr::COARSE_X) & 0b10 == 0b10 {
                    self.bg_data.latch.attr_byte >>= 2;
                }
            }
            4 => {
                // https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
                // Read pattern data from the lower bit plane of the pattern table
                let patt_addr = self.registers.ppuctrl.get_patt_base() // PPUCTRL selects left or right half of table
                    | ((self.bg_data.latch.nt_byte as u16) << 4) // NT byte is 4 bits of row, 4 bits of col
                    | self.registers.curr_addr.get(vram_addr::FINE_Y); // "the row number within a tile"

                self.bg_data.latch.patt_lo = self.memory.read(patt_addr + 0);
            }
            6 => {
                // Read pattern data from the upper bit plane of the pattern table
                let patt_addr = self.registers.ppuctrl.get_patt_base()
                    | ((self.bg_data.latch.nt_byte as u16) << 4)
                    | self.registers.curr_addr.get(vram_addr::FINE_Y);

                // Same as lower bit, but adding 0b1000 selects the upper table plane
                self.bg_data.latch.patt_hi = self.memory.read(patt_addr + 8);
            }
            7 => {
                if self.registers.ppumask.is_rendering() {
                    // This is redundant at the end of a row of pixels, but hardware still does it
                    self.registers.curr_addr.increment_horizontal();

                    // Increment y position at the end of the row
                    if self.scan.cycle == 256 {
                        self.registers.curr_addr.increment_vertical();
                    }
                }
            }
            _ => {} // Reads take two cycles, so we just skip the odd ones
        }
    }

    fn spr_fetch(&mut self, spr_num: u16, cycles_into_tile: u16) {
        // https://wiki.nesdev.com/w/index.php/PPU_rendering#Cycles_257-320
        match cycles_into_tile {
            0 => {
                // Garbage nametable byte
            }
            2 => {
                self.spr_data.registers[spr_num as usize].attr_latch =
                    self.oam2.read(4 * spr_num + 2);
            }
            3 => {
                self.spr_data.registers[spr_num as usize].x_counter =
                    self.oam2.read(4 * spr_num + 3);
            }
            4 => {
                // Pattern table tile low
                // TODO: This
            }
            6 => {
                // Pattern table tile high
                // TODO: This
            }
            _ => {} // Reads take two cycles, so we just skip the odd ones
        }
    }

    fn spr_eval(&mut self, odd_cycle: bool) {
        // TODO: This doesn't emulate hardware at all
        // TODO: It should be like a state machine, like https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
        let mut n_found = 0;
        for oam_spr in 0..64 {
            if n_found == 8 {
                break;
            }
            let y = self.oam.read(4 * oam_spr) as u16;
            if y <= self.scan.line && self.scan.line <= (y + 7) {
                for j in 0..4 {
                    self.oam2.write(n_found * 4 + j, self.oam.read(oam_spr * 4 + j));
                }
                n_found += 1;
            }
        }
    }

    fn get_bg_pixel(&mut self) -> (bool, u8) {
        // https://wiki.nesdev.com/w/index.php/PPU_rendering#Preface
        let nth_bit = |val: u16, n: u8| (val & (1 << n)) >> n;

        let offset = self.registers.fine_x;
        let patt_pair = (nth_bit(self.bg_data.shift.patt_shift[1], 15 - offset) << 1)
            | nth_bit(self.bg_data.shift.patt_shift[0], 15 - offset);
        let attr_pair = (nth_bit(self.bg_data.shift.attr_shift[1] as u16, 7 - offset) << 1)
            | nth_bit(self.bg_data.shift.attr_shift[0] as u16, 7 - offset);

        self.bg_data.shift.patt_shift[0] <<= 1;
        self.bg_data.shift.patt_shift[1] <<= 1;

        self.bg_data.shift.attr_shift[0] <<= 1;
        self.bg_data.shift.attr_shift[0] |= self.bg_data.shift.attr_latch[0] as u8;
        self.bg_data.shift.attr_shift[1] <<= 1;
        self.bg_data.shift.attr_shift[1] |= self.bg_data.shift.attr_latch[1] as u8;

        if !self.registers.ppumask.contains(MaskRegister::BACK_ENABLE)
            || (!self.registers.ppumask.contains(MaskRegister::BACK_LEFT_COL)
                && (self.scan.cycle - 1 < 8))
        {
            return (false, 0x00);
        }

        // https://wiki.nesdev.com/w/index.php/PPU_palettes#Memory_Map
        let color_index = 0x3F00 // Palette RAM base = universal background color
            | (attr_pair << 2) // "Palette number from attribute table"
            | patt_pair; // "Pixel value from tile data"

        // TODO: Make a struct for this
        // (pixel_on, color)
        (patt_pair != 0, self.memory.read(color_index))
    }

    fn get_spr_pixel(&mut self) -> (bool, u8) {
        // https://wiki.nesdev.com/w/index.php/PPU_rendering#Preface
        // TODO: Implement the actual pattern behavior
        if self.scan.cycle == 1 {
            return (false, 0x00);
        }

        for sprite_registers in &self.spr_data.registers {
            if sprite_registers.x_counter as u16 <= (self.scan.cycle - 2)
                && (self.scan.cycle - 2) <= (sprite_registers.x_counter as u16 + 7) {
                return (true, 0x55);
            }
        }

        (false, 0x00)
    }

    fn run_oam_dma(&mut self, data: u8) {
        if self.dma_option.is_none() {
            return;
        }

        // https://wiki.nesdev.com/w/index.php/PPU_programmer_reference#OAM_DMA_.28.244014.29_.3E_write
        // "1 wait state cycle while waiting for writes to complete,
        self.cpu_cycle();

        // // +1 if on an odd CPU cycle,
        if ((self.scan.total_cycles / 3) % 2) == 1 {
            self.cpu_cycle();
        }

        // then 256 alternating read/write cycles"
        let base = (data as u16) << 8;
        for i in 0..256 {
            let dma_val = self
                .dma_option
                .as_ref()
                .unwrap()
                .borrow_mut()
                .read(base + i);
            self.cpu_cycle(); // Read takes 1 CPU cycle

            // TODO: I think this should be oamaddr, but it has to be reset at some point
            self.oam.write(i, dma_val);
            // self.oam.write(self.registers.oamaddr as u16, dma_val);
            // self.registers.oamaddr = self.registers.oamaddr.wrapping_add(1);
            self.cpu_cycle(); // Write takes another
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

                high_three | (self.registers.bus_latch & 0b000_11111)
            }
            register_addrs::OAMDATA => {
                // https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
                // During secondary OAM clear, the secondary OAM actually still functions as usual;
                // however, a signal activates that pulls reads of OAMDATA to $FF
                if self.scan.on_visible_line() && self.scan.on_oam2_clear_cycle() {
                    0xFF
                } else {
                    self.oam.read(self.registers.oamaddr as u16)
                }
            }
            register_addrs::PPUDATA => {
                let old_ppudata = self.registers.ppudata;
                self.registers.ppudata = self.memory.read(self.registers.curr_addr.raw);

                let old_addr = self.registers.curr_addr.raw;
                self.registers.curr_addr.raw += self.registers.ppuctrl.get_vram_increment();

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
                if self.scan.on_visible_line() && self.scan.on_oam2_clear_cycle() {
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
                    self.nmi = true;
                }
                self.registers
                    .temp_addr
                    .set(vram_addr::NAMETABLE_SEL, data & 0b11);
            }
            register_addrs::PPUMASK => self.registers.ppumask.write(data),
            register_addrs::OAMADDR => self.registers.oamaddr = data,
            register_addrs::OAMDATA => {
                self.oam.write(self.registers.oamaddr as u16, data);
                self.registers.oamaddr += self.registers.oamaddr.wrapping_add(1);
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
                self.memory.write(self.registers.curr_addr.raw, data);
                self.registers.curr_addr.raw += self.registers.ppuctrl.get_vram_increment();
            }
            register_addrs::OAMDMA => {
                self.dma_request = Some(data);
            }
            _ => unimplemented!(),
        }
    }
}
