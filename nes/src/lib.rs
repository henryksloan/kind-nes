mod cartridge;
mod nametable_memory;

use crate::cartridge::Cartridge;
use crate::nametable_memory::NametableMemory;
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
}

impl NES {
    pub fn new() -> Self {
        let cart = Rc::new(RefCell::new(Cartridge::new()));

        // https://wiki.nesdev.com/w/index.php/PPU_memory_map
        let mut ppu_mmu = MMU::new();
        ppu_mmu.map(0x0000, 0x1FFF, cart.clone()); // Pattern tables
        ppu_mmu.map(
            0x2000,
            0x3EFF,
            Rc::new(RefCell::new(NametableMemory::new(cart.clone()))),
        );
        ppu_mmu.map_ram_mirrored(0x3F00, 0x3FFF, 0x0020); // Palette RAM indices
        let ppu = Rc::new(RefCell::new(PPU::new(Box::from(ppu_mmu))));

        // https://wiki.nesdev.com/w/index.php/CPU_memory_map
        let mut cpu_mmu = MMU::new();
        cpu_mmu.map_ram_mirrored(0x0000, 0x1FFF, 0x0800);
        cpu_mmu.map_mirrored(0x2000, 0x3FFF, 0x0008, ppu.clone()); // PPU registers
        cpu_mmu.map(0x4014, 0x4014, ppu.clone()); // OAMDMA
                                                  // cpu_mmu.map(&apu_io_reg, 0x4000, 0x401F);
        cpu_mmu.map(0x4020, 0xFFFF, cart.clone());
        let cpu = Rc::new(RefCell::new(CPU::new(Box::from(cpu_mmu))));
        cpu.borrow_mut().reset();

        ppu.borrow_mut().set_dma(cpu.clone());

        Self { cpu, ppu, cart }
    }

    /// Load a rom from a file and reset the system, returning whether it succeeded
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
        if self.ppu.borrow().nmi {
            self.ppu.borrow_mut().nmi = false;
            self.cpu.borrow_mut().nmi();
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
}
