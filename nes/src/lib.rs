use cpu::CPU;
use memory::mmu::MMU;
use ppu::PPU;
use std::cell::RefCell;
use std::rc::Rc;

pub struct NES {
    cpu: Rc<RefCell<CPU>>,
    ppu: Rc<RefCell<PPU>>,
}

impl NES {
    pub fn new() -> Self {
        // https://wiki.nesdev.com/w/index.php/PPU_memory_map
        let mut ppu_mmu = MMU::new();
        // ppu_mmu.map(&cart, 0x0000, 0x1FFF); // Pattern tables
        ppu_mmu.map_ram_mirrored(0x2000, 0x3EFF, 0x0400); // Nametables
        ppu_mmu.map_ram_mirrored(0x3F00, 0x3FFF, 0x0020); // Palette RAM indices
        let ppu = Rc::new(RefCell::new(PPU::new(Box::from(ppu_mmu))));
        ppu.borrow_mut().reset();

        // https://wiki.nesdev.com/w/index.php/CPU_memory_map
        let mut cpu_mmu = MMU::new();
        cpu_mmu.map_ram_mirrored(0x0000, 0x1FFF, 0x0800);
        cpu_mmu.map_mirrored(0x2000, 0x3FFF, 0x0008, ppu.clone());
        // cpu_mmu.map(&apu_io_reg, 0x4000, 0x401F);
        // cpu_mmu.map(&cart, 0x4020, 0xFFFF);
        let cpu = Rc::new(RefCell::new(CPU::new(Box::from(cpu_mmu))));
        cpu.borrow_mut().reset();

        ppu.borrow_mut().set_dma(cpu.clone());

        Self { cpu, ppu }
    }

    pub fn tick(&mut self) {
        self.cpu.borrow_mut().tick();
        self.ppu.borrow_mut().cpu_cycle();
    }
}
