use crate::cartridge::Mirroring;
use memory::Memory;

pub mod mapper0;

pub trait Mapper: Memory {
    fn get_nametable_mirroring(&self) -> Option<Mirroring> {
        None // Unless otherwise specified, mirroring is hard-wired
    }
}
