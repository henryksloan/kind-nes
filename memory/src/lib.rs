pub trait Memory {
    fn read(&mut self, addr: u16) -> u8;
    fn peek(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);

    fn read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_u16(&mut self, addr: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.write(addr, lo);
        self.write(addr + 1, hi);
    }
}