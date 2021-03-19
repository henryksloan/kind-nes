mod channels;

use channels::*;
use memory::Memory;

// https://wiki.nesdev.com/w/index.php/APU_registers
pub const PULSE1_BASE: u16 = 0x4000;
pub const PULSE2_BASE: u16 = 0x4004;
pub const TRIANGLE_BASE: u16 = 0x4008;
pub const NOISE_BASE: u16 = 0x400C;
pub const DMC_BASE: u16 = 0x4010;
pub const STATUS: u16 = 0x4015; // Writing exposes controls
pub const FRAME_COUNTER: u16 = 0x4017;

// http://nesdev.com/apu_ref.txt
pub struct APU {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    dmc: DMCChannel,

    clock_rate: u32,
    frame_counter_cycle: u32,
    frame_sequence_len: u8,
    frame_sequence_step: u8,
    irq_disable: bool,
    pub irq: bool,
}

impl APU {
    pub fn new() -> Self {
        Self {
            pulse1: PulseChannel::new(false),
            pulse2: PulseChannel::new(true),
            triangle: TriangleChannel::new(),
            noise: NoiseChannel::new(),
            dmc: DMCChannel::new(),

            clock_rate: 1789773,
            frame_counter_cycle: 0,
            frame_sequence_len: 4,
            frame_sequence_step: 0,
            irq_disable: false,
            irq: false,
        }
    }

    pub fn reset(&mut self) {
        todo!()
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
        self.frame_counter_cycle += 1;
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
                    self.irq = true;
                }
            }
        }
    }
}

impl Memory for APU {
    fn read(&mut self, addr: u16) -> u8 {
        todo!()
    }

    fn peek(&self, addr: u16) -> u8 {
        todo!()
    }

    // TODO: Can probably use functions like pulse1.envelope.update(data)
    fn write(&mut self, addr: u16, data: u8) {
        todo!()
    }
}
