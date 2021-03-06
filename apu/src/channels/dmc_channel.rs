use memory::Memory;

use std::cell::RefCell;
use std::rc::Rc;

// https://wiki.nesdev.com/w/index.php/APU_DMC
pub struct DMCChannel {
    enabled: bool,
    even_latch: bool,
    timer: u16,
    timer_period: u16,

    irq_enable: bool,
    pub irq: bool,
    loop_flag: bool,
    pub dac_level: u8,
    sample_address: u16,
    sample_len: u16,

    dma_option: Option<Rc<RefCell<dyn Memory>>>,
    dma_request: bool,
    pub stall_cpu: bool,
    sample_buffer: Option<u8>,
    shift: u8,
    dma_address: u16,
    pub bytes_remaining: u16,
    bits_remaining: u16,
    silence: bool,
}

impl DMCChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            even_latch: true,
            timer: 0,
            timer_period: 0,

            irq_enable: false,
            irq: false,
            loop_flag: false,
            dac_level: 0,
            sample_address: 0,
            sample_len: 0,

            dma_option: None,
            dma_request: false,
            stall_cpu: false,
            sample_buffer: None,
            shift: 0,
            dma_address: 0,
            bytes_remaining: 0,
            bits_remaining: 8,
            silence: true,
        }
    }

    pub fn set_dma(&mut self, dma: Rc<RefCell<dyn Memory>>) {
        self.dma_option = Some(dma);
    }

    pub fn tick(&mut self) {
        if self.dma_request {
            self.dma_request = false;
            self.sample_buffer = Some(self.dma_read(self.dma_address));
        }

        if self.even_latch {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                self.timer = self.timer_period;
                if !self.silence {
                    if self.shift & 1 == 1 {
                        if self.dac_level <= 125 {
                            self.dac_level += 2;
                        }
                    } else {
                        if self.dac_level >= 2 {
                            self.dac_level -= 2;
                        }
                    }
                }

                self.shift >>= 1;
                self.bits_remaining -= 1;
                if self.bits_remaining == 0 {
                    self.bits_remaining = 8;

                    if let Some(buffer_contents) = self.sample_buffer {
                        self.silence = false;
                        self.shift = buffer_contents;
                        self.sample_buffer = None;
                        if self.bytes_remaining > 0 {
                            self.sample_buffer = Some(self.dma_read(self.dma_address));
                            if self.dma_address == 0xFFFF {
                                self.dma_address = 0;
                            } else {
                                self.dma_address += 1;
                            }

                            self.bytes_remaining -= 1;
                            if self.bytes_remaining == 0 {
                                if self.loop_flag {
                                    self.dma_address = self.sample_address;
                                    self.bytes_remaining = self.sample_len;
                                } else if self.irq_enable {
                                    self.irq = true;
                                }
                            }
                        }
                    } else {
                        self.silence = true;
                    }
                }
            }
        }
        self.even_latch = !self.even_latch;
    }

    pub fn update_enabled(&mut self, control_bit: u8) {
        self.enabled = control_bit == 1;
        if self.enabled {
            if self.bytes_remaining == 0 {
                self.dma_address = self.sample_address;
                self.bytes_remaining = self.sample_len;
                self.dma_request = true;
            }
        } else {
            self.bytes_remaining = 0;
        }
    }

    pub fn update_register(&mut self, register_offset: u16, data: u8) {
        match register_offset {
            0 => {
                self.irq_enable = data >> 7 == 1;
                self.loop_flag = (data >> 6) & 1 == 1;
                self.timer_period = NTSC_RATE_TABLE[(data & 0b1111) as usize];
            }
            1 => {
                self.dac_level = data & 0b0111_1111;
            }
            2 => {
                self.sample_address = 0xC000 + ((data as u16) << 6);
            }
            3 => {
                self.sample_len = ((data as u16) << 4) + 1;
            }
            _ => unimplemented!(),
        }
    }

    pub fn output(&self) -> u8 {
        self.dac_level
    }

    fn dma_read(&mut self, addr: u16) -> u8 {
        self.stall_cpu = true;
        if self.dma_option.is_none() {
            0
        } else {
            self.dma_option.as_ref().unwrap().borrow_mut().read(addr)
        }
    }
}

const NTSC_RATE_TABLE: [u16; 16] = [
    0x1AC, 0x17C, 0x154, 0x140, 0x11E, 0x0FE, 0x0E2, 0x0D6, 0x0BE, 0x0A0, 0x08E, 0x080, 0x06A,
    0x054, 0x048, 0x036,
];
