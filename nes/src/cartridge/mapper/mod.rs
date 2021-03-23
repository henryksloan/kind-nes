use crate::cartridge::Mirroring;
use memory::Memory;

mod mapper0;
mod mapper1;
mod mapper2;
mod mapper3;
mod mapper4;
mod mapper7;
mod mapper9;

pub use self::mapper0::Mapper0;
pub use self::mapper1::Mapper1;
pub use self::mapper2::Mapper2;
pub use self::mapper3::Mapper3;
pub use self::mapper4::Mapper4;
pub use self::mapper7::Mapper7;
pub use self::mapper9::Mapper9;

pub trait Mapper: Memory {
    fn get_nametable_mirroring(&self) -> Option<Mirroring> {
        None // Unless otherwise specified, mirroring is hard-wired
    }

    fn check_irq(&mut self) -> bool {
        false
    }

    fn cycle(&mut self) {}
    fn reset(&mut self) {}
}
