use cpu::CPU;
use memory::mmu::MMU;

pub struct NES {
    cpu: CPU,
}

impl NES {
    pub fn new() -> Self {
        let mut cpu_mmu = MMU::new();
        cpu_mmu.map_ram_mirrored(0x0000, 0x1FFF, 0x0800);
        // cpu_mmu.map_mirrored(&ppu, 0x2000, 0x3FFF, 0x0008);
        // cpu_mmu.map(&apu_io_reg, 0x4000, 0x401F);
        // cpu_mmu.map(&cart, 0x4020, 0xFFFF);
        let mut cpu = CPU::new(Box::from(cpu_mmu));
        cpu.reset();

        Self { cpu }
    }

    pub fn tick(&mut self) {
        self.cpu.tick();
    }
}
