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

use cpu::CPU;
use memory::mmu::MMU;
use ppu::PPU;
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

pub struct NES {
    cpu: Rc<RefCell<CPU>>,
    ppu: Rc<RefCell<PPU>>,
    cart: Rc<RefCell<Cartridge>>,
    joy1: Rc<RefCell<dyn Controller>>,
    joy2: Rc<RefCell<dyn Controller>>,
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

        let cpu_mapped_registers = Rc::new(RefCell::new(CPUMappedRegisters::new(
            ppu.clone(),
            ppu.clone(), // TODO: This should be APU
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

        Self {
            cpu,
            ppu,
            cart,
            joy1,
            joy2,
        }
    }

    /// Load a ROM from a file and reset the system, returning whether it succeeded
    pub fn load_rom(&mut self, file: File) -> Result<(), &'static str> {
        match Cartridge::from_file(file) {
            Ok(new_cart) => {
                self.cart.replace(new_cart);
                self.cpu.borrow_mut().reset();
                self.ppu.borrow_mut().reset();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn unload_rom(&mut self) {
        self.cart.replace(Cartridge::new());
    }

    pub fn tick(&mut self) {
        self.ppu.borrow_mut().frame_ready = false;
        if let Some(log) = self.cpu.borrow_mut().tick() {
            println!("{}", log);
        }
        self.ppu.borrow_mut().cpu_cycle();
        self.cart.borrow_mut().cycle(); // TODO:
        if self.ppu.borrow().nmi {
            self.ppu.borrow_mut().nmi = false;
            self.cpu.borrow_mut().nmi();
        } else if self.cart.borrow_mut().check_irq() {
            self.cpu.borrow_mut().irq();
        }
    }

    pub fn get_new_frame(&self) -> Option<[[u8; 256]; 240]> {
        let ppu = self.ppu.borrow();
        if ppu.frame_ready {
            Some(ppu.framebuffer)
        } else {
            None
        }
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
