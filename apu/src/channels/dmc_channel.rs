// https://wiki.nesdev.com/w/index.php/APU_DMC
// TODO: Make fewer fields pub in all these structs
pub struct DMCChannel {
    pub enabled: bool,
    pub even_latch: bool,
    pub timer: u16,
    pub timer_period: u16,

    pub irq_enable: bool,
    pub irq: bool,
    pub loop_flag: bool,
    pub dac_level: u8,
    pub sample_address: u16,
    pub sample_len: u16,

    pub sample_buffer: Option<u8>,
    pub shift: u8,
    pub dma_address: u16,
    pub bytes_remaining: u16,
    pub bits_remaining: u16,
    pub silence: bool,
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

            sample_buffer: None,
            shift: 0,
            dma_address: 0,
            bytes_remaining: 0,
            bits_remaining: 8,
            silence: true,
        }
    }

    pub fn tick(&mut self) {
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
                            self.sample_buffer = Some(todo!("DMA read"));
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
                self.sample_buffer = Some(todo!("DMA read"));
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
}

const NTSC_RATE_TABLE: [u16; 16] = [
    0x1AC, 0x17C, 0x154, 0x140, 0x11E, 0x0FE, 0x0E2, 0x0D6, 0x0BE, 0x0A0, 0x08E, 0x080, 0x06A,
    0x054, 0x048, 0x036,
];
