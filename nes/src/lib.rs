mod cartridge;
mod controllers;
mod cpu_mapped_registers;
mod nametable_memory;
mod palette_ram;

use crate::cartridge::Cartridge;
use crate::controllers::Controller;
use crate::controllers::NoController;
use crate::controllers::StandardController;
use crate::cpu_mapped_registers::CPUMappedRegisters;
use crate::nametable_memory::NametableMemory;
use crate::palette_ram::PaletteRAM;

use apu::APU;
use cpu::CPU;
use memory::mmu::MMU;
use ppu::PPU;
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

pub struct NES {
    cpu: Rc<RefCell<CPU>>,
    ppu: Rc<RefCell<PPU>>,
    apu: Rc<RefCell<APU>>,
    cart: Rc<RefCell<Cartridge>>,
    joy1: Rc<RefCell<dyn Controller>>,
    joy2: Rc<RefCell<dyn Controller>>,

    pub paused: bool,
}

impl NES {
    pub fn new() -> Self {
        let cart = Rc::new(RefCell::new(Cartridge::new()));
        let joy1 = Rc::new(RefCell::new(StandardController::new(true)));
        let joy2 = Rc::new(RefCell::new(NoController::new()));

        // https://wiki.nesdev.com/w/index.php/PPU_memory_map
        let mut ppu_mmu = MMU::new();
        ppu_mmu.map(0x0000, 0x1FFF, cart.clone()); // Pattern tables
        ppu_mmu.map(
            0x2000,
            0x3EFF,
            Rc::new(RefCell::new(NametableMemory::new(cart.clone()))),
        );
        ppu_mmu.map(0x3F00, 0x3FFF, Rc::new(RefCell::new(PaletteRAM::new()))); // Palette RAM indices
        let ppu = Rc::new(RefCell::new(PPU::new(Box::from(ppu_mmu))));

        let apu = Rc::new(RefCell::new(APU::new()));

        let cpu_mapped_registers = Rc::new(RefCell::new(CPUMappedRegisters::new(
            ppu.clone(),
            apu.clone(),
            joy1.clone(),
            joy2.clone(),
        )));

        // https://wiki.nesdev.com/w/index.php/CPU_memory_map
        let mut cpu_mmu = MMU::new();
        cpu_mmu.map_ram_mirrored(0x0000, 0x1FFF, 0x0800);
        cpu_mmu.map_mirrored(0x2000, 0x3FFF, 0x0008, ppu.clone()); // PPU registers
        cpu_mmu.map(0x4000, 0x401F, cpu_mapped_registers.clone()); // NES APU and I/O registers
        cpu_mmu.map(0x4020, 0xFFFF, cart.clone());
        let cpu = Rc::new(RefCell::new(CPU::new(Box::from(cpu_mmu))));
        cpu.borrow_mut().reset();

        ppu.borrow_mut().set_dma(cpu.clone());
        apu.borrow_mut().set_dma(cpu.clone());

        Self {
            cpu,
            ppu,
            apu,
            cart,
            joy1,
            joy2,
            paused: false,
        }
    }

    pub fn reset(&mut self) {
        self.cpu.borrow_mut().reset();
        self.ppu.borrow_mut().reset();
        self.apu.borrow_mut().reset();
        self.cart.borrow_mut().reset();
    }

    /// Load a ROM from a file and reset the system, returning whether it succeeded
    pub fn load_rom(&mut self, file: File) -> Result<(), &'static str> {
        match Cartridge::from_file(file) {
            Ok(new_cart) => {
                self.cart.replace(new_cart);
                self.cpu.borrow_mut().reset();
                self.ppu.borrow_mut().reset();
                self.apu.borrow_mut().reset();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn unload_rom(&mut self) {
        self.cart.replace(Cartridge::new());
    }

    pub fn tick(&mut self) {
        if self.paused {
            return;
        }

        self.ppu.borrow_mut().frame_ready = false;
        if let Some(log) = self.cpu.borrow_mut().tick() {
            println!("{}", log);
        }

        self.apu.borrow_mut().tick();
        // https://wiki.nesdev.com/w/index.php/APU_DMC#Memory_reader
        // TODO: Cover the cases that sleep for less time
        if self.apu.borrow_mut().check_stall_cpu() {
            self.cpu.borrow_mut().stall(4);
        }

        self.ppu.borrow_mut().cpu_cycle();
        self.cart.borrow_mut().cycle(); // TODO: Probably per-ppu tick for some mappers
        if self.ppu.borrow().nmi {
            self.ppu.borrow_mut().nmi = false;
            self.cpu.borrow_mut().nmi();
        } else if self.cart.borrow_mut().check_irq() || self.apu.borrow_mut().check_irq() {
            self.cpu.borrow_mut().irq();
        }
    }

    pub fn has_cartridge(&self) -> bool {
        !self.cart.borrow().is_empty()
    }

    pub fn get_new_frame(&self) -> Option<[[u8; 256]; 240]> {
        let ppu = self.ppu.borrow();
        if ppu.frame_ready {
            Some(ppu.framebuffer)
        } else {
            None
        }
    }

    pub fn take_audio_buff(&mut self) -> Vec<f32> {
        self.apu.borrow_mut().take_audio_buff()
    }

    pub fn get_shift_strobe(&self) -> bool {
        self.joy1.borrow().get_shift_strobe()
    }

    pub fn try_fill_controller_shift(&mut self, val: u8) {
        let mut joy1 = self.joy1.borrow_mut();
        if joy1.get_shift_strobe() {
            joy1.set_state_shift(val);
        }
    }
}

impl Default for NES {
    fn default() -> Self {
        NES::new()
    }
}
