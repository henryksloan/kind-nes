use crate::ram::RAM;
use crate::Memory;

use std::cell::RefCell;
use std::rc::Rc;

struct MapRange {
    start: u16, // Both inclusive
    end: u16,
    size: u16,
    memory: Rc<RefCell<dyn Memory>>,
}

impl MapRange {
    pub fn map(&self, addr: u16) -> Option<(u16, &Rc<RefCell<dyn Memory>>)> {
        if self.start <= addr && addr <= self.end {
            Some((((addr - self.start) % self.size) + self.start, &self.memory))
        } else {
            None
        }
    }
}

pub struct MMU {
    ranges: Vec<MapRange>,
}

impl MMU {
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    pub fn map_mirrored(
        &mut self,
        start: u16,
        end: u16,
        size: u16,
        memory: Rc<RefCell<dyn Memory>>,
    ) {
        self.ranges.push(MapRange {
            start,
            end,
            size,
            memory,
        });
    }

    pub fn map(&mut self, start: u16, end: u16, memory: Rc<RefCell<dyn Memory>>) {
        self.map_mirrored(start, end, end - start, memory);
    }

    pub fn map_ram_mirrored(&mut self, start: u16, end: u16, size: u16) {
        self.map_mirrored(
            start,
            end,
            size,
            Rc::new(RefCell::new(RAM::new(size, start))),
        );
    }

    pub fn map_ram(&mut self, start: u16, end: u16) {
        self.map_ram_mirrored(start, end, end - start);
    }

    fn access(&self, addr: u16) -> Result<(u16, &Rc<RefCell<dyn Memory>>), &'static str> {
        self.ranges
            .iter()
            .find_map(|range| range.map(addr))
            .ok_or("invalid memory access")
    }
}

impl Memory for MMU {
    fn read(&mut self, addr: u16) -> u8 {
        if let Ok(result) = self.access(addr) {
            result.1.borrow_mut().read(result.0)
        } else {
            0
        }
    }

    fn peek(&self, addr: u16) -> u8 {
        if let Ok(result) = self.access(addr) {
            result.1.borrow().peek(result.0)
        } else {
            0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if let Ok(result) = self.access(addr) {
            result.1.borrow_mut().write(result.0, data)
        }
    }
}
