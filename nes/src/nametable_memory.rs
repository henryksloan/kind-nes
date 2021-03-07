use crate::cartridge::Cartridge;
use crate::cartridge::Mirroring;
use memory::ram::RAM;
use memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;

pub struct NametableMemory {
  cart: Rc<RefCell<Cartridge>>,
  memory: RAM,
}

impl NametableMemory {
  pub fn new(cart: Rc<RefCell<Cartridge>>) -> Self {
    Self {
      cart,
      memory: RAM::new(0x400 * 4, 0x0000),
    }
  }

  fn mirror(&self, addr: u16) -> u16 {
    // Adapted from a clever approach by daniel5151
    let mut _addr = addr;
    if _addr > 0x3000 {
      _addr -= 0x1000;
    }
    let mut fix_4s = 0;
    let nt_mirroring = match self.cart.borrow().get_nametable_mirroring() {
      Mirroring::Vertical => [0, 1, 0, 1],
      Mirroring::Horizontal => [0, 0, 1, 1],
      Mirroring::FourScreen => {
        fix_4s = 0x2000;
        [0, 1, 2, 3]
      }
      Mirroring::SingleScreen => [0, 0, 0, 0],
    };

    nt_mirroring[((_addr - 0x2000) / 0x400) as usize] * 0x400 + (_addr % 0x400) + fix_4s
  }
}

impl Memory for NametableMemory {
  fn read(&mut self, addr: u16) -> u8 {
    self.memory.read(self.mirror(addr))
  }

  fn peek(&self, addr: u16) -> u8 {
    self.memory.peek(self.mirror(addr))
  }

  fn write(&mut self, addr: u16, data: u8) {
    self.memory.write(self.mirror(addr), data);
  }
}
