use cpu::CPU;
use memory::mmu::MMU;
use memory::ram::RAM;
use memory::Memory;

use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter;
use std::path::PathBuf;
use std::rc::Rc;

#[test]
fn nestest_cpu() {
    let resource_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources"].iter().collect();
    let mut nestest_path = resource_path.clone();
    nestest_path.push("nestest.nes");
    let mut log_path = resource_path.clone();
    log_path.push("nestest_cpu.log");

    let mut cpu_mmu = MMU::new();
    cpu_mmu.map_ram_mirrored(0x0000, 0x1FFF, 0x0800);
    let file = File::open(nestest_path).unwrap();
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
    cpu_mmu.map(
        0x4000,
        0x4017,
        Rc::new(RefCell::new(RAM {
            memory: vec![0xFF; 0x18],
            start: 0x4000,
        })),
    );
    cpu_mmu.write_u16(0xFFFC, 0xC000);
    let mut cpu = CPU::new(Box::from(cpu_mmu));
    cpu.log = true;
    cpu.reset();

    let log_file = File::open(log_path).unwrap();
    let reader = BufReader::new(log_file);
    for line in reader.lines() {
        let log = iter::repeat_with(|| cpu.tick())
            .skip_while(|x| x.is_none())
            .next()
            .unwrap()
            .unwrap();
        assert_eq!(log, line.unwrap());
    }
}
