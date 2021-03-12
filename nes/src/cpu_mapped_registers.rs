use memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;

// https://wiki.nesdev.com/w/index.php/2A03
pub struct CPUMappedRegisters {
    ppu: Rc<RefCell<dyn Memory>>,
    apu: Rc<RefCell<dyn Memory>>,
    joy1: Rc<RefCell<dyn Memory>>,
    joy2: Rc<RefCell<dyn Memory>>,
}

impl CPUMappedRegisters {
    pub fn new(
        ppu: Rc<RefCell<dyn Memory>>,
        apu: Rc<RefCell<dyn Memory>>,
        joy1: Rc<RefCell<dyn Memory>>,
        joy2: Rc<RefCell<dyn Memory>>,
    ) -> Self {
        Self {
            ppu,
            apu,
            joy1,
            joy2,
        }
    }

    fn access(&self, addr: u16, write: bool) -> Result<&Rc<RefCell<dyn Memory>>, &'static str> {
        if (0x4000 <= addr && addr <= 0x4008)
            || (0x400A <= addr && addr <= 0x400C)
            || (0x400E <= addr && addr <= 0x4013)
            || (addr == 0x4015)
            || (write && addr == 0x4017)
        {
            // Ok(&self.apu)
            Err("TODO")
        } else if addr == 0x4014 {
            Ok(&self.ppu)
        } else if addr == 0x4016 {
            Ok(&self.joy1)
        } else if !write && addr == 0x4017 {
            Ok(&self.joy2)
        } else {
            Err("invalid memory access")
        }
    }
}

impl Memory for CPUMappedRegisters {
    fn read(&mut self, addr: u16) -> u8 {
        if let Ok(result) = self.access(addr, false) {
            result.borrow_mut().read(addr)
        } else {
            0
        }
    }

    fn peek(&self, addr: u16) -> u8 {
        if let Ok(result) = self.access(addr, false) {
            result.borrow_mut().peek(addr)
        } else {
            0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if let Ok(result) = self.access(addr, true) {
            result.borrow_mut().write(addr, data)
        }
    }
}
