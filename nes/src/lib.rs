use cpu::CPU;
use memory::mmu::MMU;
use memory::ram::RAM;
use memory::Memory;

use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;

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
        let file = File::open("nestest.nes").unwrap();
        let cart_data = file
            .bytes()
            .skip(0x0010)
            .take(0x4000)
            .map(|x| x.unwrap())
            .collect::<Vec<u8>>();
        cpu_mmu.map_mirrored(
            0x8000,
            0xFFFF,
            0x4000,
            Rc::new(RefCell::new(RAM {
                memory: cart_data,
                start: 0x8000,
            })),
        );
        cpu_mmu.write_u16(0xFFFC, 0xC000);
        let mut cpu = CPU::new(Box::from(cpu_mmu));
        cpu.reset();

        Self { cpu }
    }

    pub fn tick(&mut self) {
        self.cpu.tick();
    }
}
