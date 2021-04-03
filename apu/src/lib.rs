#[macro_use]
extern crate lazy_static;

mod channels;
mod filters;

use channels::*;
use filters::{Filter, HighPassFilter, LowPassFilter};
use memory::Memory;

use std::cell::RefCell;
use std::rc::Rc;

// http://www.slack.net/~ant/nes-emu/apu_ref.txt
pub struct APU {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    dmc: DMCChannel,

    clock_rate: u64,
    sample_rate: u64,
    frame_counter_cycle: u64,
    frame_sequence_len: u8,
    frame_sequence_step: u8,
    bus_latch: u8,

    irq_disable: bool,
    frame_irq: bool,

    audio_buff: Vec<f32>,
    filters: Vec<Box<dyn Filter>>,
}

impl APU {
    pub fn new() -> Self {
        let clock_rate = 1789773;
        let sample_rate = clock_rate / 96000;
        Self {
            pulse1: PulseChannel::new(false),
            pulse2: PulseChannel::new(true),
            triangle: TriangleChannel::new(),
            noise: NoiseChannel::new(),
            dmc: DMCChannel::new(),

            clock_rate: clock_rate,
            sample_rate: sample_rate,
            frame_counter_cycle: 0,
            frame_sequence_len: 4,
            frame_sequence_step: 0,
            bus_latch: 0,

            irq_disable: false,
            frame_irq: false,

            audio_buff: Vec::new(),
            filters: vec![
                Box::from(HighPassFilter::new(90.0, sample_rate as u32)),
                Box::from(HighPassFilter::new(440.0, sample_rate as u32)),
                Box::from(LowPassFilter::new(14000.0, sample_rate as u32)),
            ],
        }
    }

    pub fn set_dma(&mut self, dma: Rc<RefCell<dyn Memory>>) {
        self.dmc.set_dma(dma);
    }

    pub fn check_stall_cpu(&mut self) -> bool {
        let stall = self.dmc.stall_cpu;
        self.dmc.stall_cpu = false;
        stall
    }

    pub fn check_irq(&mut self) -> bool {
        let out = self.dmc.irq || self.frame_irq;
        out
    }

    pub fn reset(&mut self) {
        // https://wiki.nesdev.com/w/index.php/CPU_power_up_state
        self.pulse1.length_counter.update_enabled(0);
        self.pulse2.length_counter.update_enabled(0);
        self.triangle.length_counter.update_enabled(0);
        self.triangle.sequence_step = 0;
        self.noise.length_counter.update_enabled(0);
        self.dmc.update_enabled(0);
        self.dmc.dac_level &= 1;

        self.frame_counter_cycle = 0;
        self.frame_sequence_len = 4;
        self.frame_sequence_step = 0;
        self.bus_latch = 0;

        self.dmc.irq = false;
        self.irq_disable = false;
        self.frame_irq = false;
    }

    pub fn tick(&mut self) {
        // The APU is effectively clocked once per CPU cycle
        // The triangle is clocked by this signal directly
        // The pulse, noise, and DMC channels are clocked on even cycles
        self.pulse1.tick();
        self.pulse2.tick();
        self.triangle.tick();
        self.noise.tick();
        self.dmc.tick();

        // The frame counter divides the clock to ~240 Hz
        // which feeds a variable-step sequencer, which controls
        // length counters, sweep units, envelopes, the linear counter, and interrupts
        if self.frame_counter_cycle % (self.clock_rate / 240) == 0 {
            // Do nothing on the last step of the 5-step sequence
            if self.frame_sequence_step != 4 {
                // Clock envelopes and linear counter on every frame step
                self.pulse1.envelope.tick();
                self.pulse2.envelope.tick();
                self.noise.envelope.tick();
                self.noise.envelope.tick();
                self.triangle.tick_linear();

                // Clock length counters on steps 1 and 3 in 4-step sequence, 0 and 2 in 5-step
                if self.frame_sequence_step % 2 == (self.frame_sequence_len == 4) as u8 {
                    self.pulse1.length_counter.tick();
                    self.pulse2.length_counter.tick();
                    self.triangle.length_counter.tick();
                    self.noise.length_counter.tick();

                    self.pulse1.tick_sweep();
                    self.pulse2.tick_sweep();
                }

                if self.frame_sequence_step == 3
                    && self.frame_sequence_len == 4
                    && !self.irq_disable
                {
                    self.frame_irq = true;
                }
            }

            self.frame_sequence_step = (self.frame_sequence_step + 1) % self.frame_sequence_len;
        }

        if self.frame_counter_cycle % self.sample_rate == 0 && self.audio_buff.len() < 4096 {
            let pulse_out =
                PULSE_TABLE[self.pulse1.output() as usize + self.pulse2.output() as usize];
            let tnd_out = TND_TABLE[3 * self.triangle.output() as usize
                + 2 * self.noise.output() as usize
                + self.dmc.output() as usize];
            let signal = self
                .filters
                .iter_mut()
                .fold(pulse_out + tnd_out, |acc, filter| filter.process(acc));
            self.audio_buff.push(signal);
        }

        self.frame_counter_cycle += 1;
    }

    pub fn take_audio_buff(&mut self) -> Vec<f32> {
        let out = self.audio_buff.clone();
        self.audio_buff.clear();
        out
    }
}

// https://wiki.nesdev.com/w/index.php/APU_registers
impl Memory for APU {
    fn read(&mut self, addr: u16) -> u8 {
        if addr != 0x4015 {
            return 0;
        }

        let data = self.peek(addr);
        self.frame_irq = false;
        self.bus_latch = data;
        data
    }

    fn peek(&self, addr: u16) -> u8 {
        if addr != 0x4015 {
            return 0;
        }

        ((self.dmc.irq as u8) << 7)
            | ((self.frame_irq as u8) << 6)
            | (((self.dmc.bytes_remaining > 0) as u8) << 4)
            | (((self.noise.length_counter.counter > 0) as u8) << 3)
            | (((self.triangle.length_counter.counter > 0) as u8) << 2)
            | (((self.pulse2.length_counter.counter > 0) as u8) << 1)
            | (((self.pulse1.length_counter.counter > 0) as u8) << 0)
    }

    fn write(&mut self, addr: u16, data: u8) {
        assert!((0x4000 <= addr && addr <= 0x4017) && addr != 0x4014);

        self.bus_latch = data;
        if 0x4000 <= addr && addr <= 0x4003 {
            self.pulse1.update_register(addr - 0x4000, data);
        } else if 0x4004 <= addr && addr <= 0x4007 {
            self.pulse2.update_register(addr - 0x4004, data);
        } else if 0x4008 <= addr && addr <= 0x400B {
            self.triangle.update_register(addr - 0x4008, data);
        } else if 0x400C <= addr && addr <= 0x400F {
            self.noise.update_register(addr - 0x400C, data);
        } else if 0x4010 <= addr && addr <= 0x4013 {
            self.dmc.update_register(addr - 0x4010, data);
        } else if addr == 0x4015 {
            self.dmc.irq = false;
            self.pulse1.length_counter.update_enabled((data >> 0) & 1);
            self.pulse2.length_counter.update_enabled((data >> 1) & 1);
            self.triangle.length_counter.update_enabled((data >> 2) & 1);
            self.noise.length_counter.update_enabled((data >> 3) & 1);
            self.dmc.update_enabled((data >> 4) & 1);
        } else if addr == 0x4017 {
            // "If the mode flag is clear, the 4-step sequence is selected, otherwise the
            // 5-step sequence is selected and the sequencer is immediately clocked once."
            if data >> 7 == 1 {
                self.frame_sequence_len = 5;
                self.frame_sequence_step = 1;

                self.pulse1.tick();
                self.pulse2.tick();
                self.triangle.tick();
                self.noise.tick();

                self.pulse1.length_counter.tick();
                self.pulse2.length_counter.tick();
                self.triangle.length_counter.tick();
                self.noise.length_counter.tick();

                self.pulse1.tick_sweep();
                self.pulse2.tick_sweep();
            } else {
                self.frame_sequence_len = 4;
                self.frame_sequence_step = 0;
            };
            self.irq_disable = (data >> 6) & 1 == 1;
            if self.irq_disable {
                self.frame_irq = false;
            }
        }
    }
}

// https://wiki.nesdev.com/w/index.php/APU_Mixer#Lookup_Table
lazy_static! {
    pub static ref PULSE_TABLE: [f32; 31] = {
        let mut table = [0.0; 31];
        for i in 0..31 {
            table[i] = 95.52 / ((8128.0 / i as f32) + 100.0);
        }
        table
    };
    pub static ref TND_TABLE: [f32; 203] = {
        let mut table = [0.0; 203];
        for i in 0..203 {
            table[i] = 163.67 / ((24329.0 / i as f32) + 100.0);
        }
        table
    };
}
