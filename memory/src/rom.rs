use crate::Memory;

pub struct ROM {
  pub memory: Vec<u8>,
  pub start: u16,
}

impl ROM {
  pub fn new(size: u16, start: u16) -> Self {
    ROM {
      memory: vec![0; size as usize],
      start,
    }
  }
}

impl Memory for ROM {
  fn read(&mut self, addr: u16) -> u8 {
    self.memory[(addr - self.start) as usize]
  }

  fn peek(&self, addr: u16) -> u8 {
    self.memory[(addr - self.start) as usize]
  }

  fn write(&mut self, addr: u16, data: u8) {
    panic!(format!(
      "attempted to write {:X} to address {:X} in ROM",
      data, addr
    ))
  }
}
