#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

mod status_register;
mod addressing_mode;
pub mod instruction;

use status_register::StatusRegister;

pub struct CPU {
    // Registers
    a: u8, // Accumulator
    x: u8, pub y: u8, // Index registers
    p: StatusRegister,
    s: u8, // Stack pointer

    pc: u16, // Program counter

    wait_cycles: u32,
}

impl CPU {
}
