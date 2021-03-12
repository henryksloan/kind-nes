use memory::Memory;

// https://wiki.nesdev.com/w/index.php/Standard_controller
pub struct StandardController {
    is_controller_1: bool,
    pub shift_strobe: bool,
    pub state_shift: u8,
}

impl StandardController {
    pub fn new(is_controller_1: bool) -> Self {
        Self {
            is_controller_1,
            shift_strobe: false,
            state_shift: 0,
        }
    }
}

impl Memory for StandardController {
    fn read(&mut self, addr: u16) -> u8 {
        if (addr == 0x4016 && self.is_controller_1) || (addr == 0x4017 && !self.is_controller_1) {
            let out = self.state_shift & 1;
            if !self.shift_strobe {
                self.state_shift >>= 1;
            }
            out
        } else {
            unimplemented!()
        }
    }

    fn peek(&self, addr: u16) -> u8 {
        if (addr == 0x4016 && self.is_controller_1) || (addr == 0x4017 && !self.is_controller_1) {
            self.state_shift & 1
        } else {
            unimplemented!()
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr == 0x4016 && self.is_controller_1 {
            self.shift_strobe = (data & 1) == 1;
        } else {
            unimplemented!()
        }
    }
}
