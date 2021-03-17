use crate::cartridge::Mirroring;
use memory::Memory;

mod mapper0;
mod mapper1;
mod mapper2;
mod mapper3;

pub use self::mapper0::Mapper0;
pub use self::mapper1::Mapper1;
pub use self::mapper2::Mapper2;
pub use self::mapper3::Mapper3;

pub trait Mapper: Memory {
    fn get_nametable_mirroring(&self) -> Option<Mirroring> {
        None // Unless otherwise specified, mirroring is hard-wired
    }

    fn cycle(&mut self) {}
    fn reset(&mut self) {}
}
