mod no_controller;
mod standard_controller;

use memory::Memory;

pub trait Controller: Memory {
    fn is_controller_1(&self) -> bool;
    fn get_shift_strobe(&self) -> bool;
    fn set_state_shift(&mut self, val: u8);
}

pub use self::no_controller::NoController;
pub use self::standard_controller::StandardController;
