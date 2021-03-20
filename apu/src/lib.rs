mod channels;

use channels::*;
use memory::Memory;

// http://www.slack.net/~ant/nes-emu/apu_ref.txt
pub struct APU {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    dmc: DMCChannel,

    clock_rate: u64,
    frame_counter_cycle: u64,
    frame_sequence_len: u8,
    frame_sequence_step: u8,

    irq_disable: bool,
    frame_irq: bool,
    dmc_irq: bool,
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
            frame_irq: false,
            dmc_irq: false,
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
                    self.frame_irq = true;
                }
            }
        }
    }
}

// https://wiki.nesdev.com/w/index.php/APU_registers
impl Memory for APU {
    fn read(&mut self, addr: u16) -> u8 {
        todo!()
    }

    fn peek(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, data: u8) {
        assert!((0x4000 <= addr && addr <= 0x4017) && addr != 0x4014);

        if 0x4000 <= addr && addr <= 0x4003 {
            self.pulse1.update_register(addr - 0x4000, data);
        } else if 0x4004 <= addr && addr <= 0x4007 {
            self.pulse2.update_register(addr - 0x4004, data);
        } else if 0x4008 <= addr && addr <= 0x400B {
            self.triangle.update_register(addr - 0x4008, data);
        } else if 0x400C <= addr && addr <= 0x400F {
            self.noise.update_register(addr - 0x400C, data);
        } else if 0x4010 <= addr && addr <= 0x4013 {
            // TODO
            // self.dmc.update_register(addr - 0x4010, data);
        } else if addr == 0x4015 {
            self.dmc_irq = false;
            self.pulse1.length_counter.update_enabled(data >> 0 & 1);
            self.pulse2.length_counter.update_enabled(data >> 1 & 1);
            self.triangle.length_counter.update_enabled(data >> 2 & 1);
            self.noise.length_counter.update_enabled(data >> 3 & 1);
            self.dmc.update_enabled(data >> 4 & 1);
        } else if addr == 0x4017 {
            // "If the mode flag is clear, the 4-step sequence is selected, otherwise the
            // 5-step sequence is selected and the sequencer is immediately clocked once."
            if data >> 7 == 1 {
                self.frame_sequence_len = 5;
                self.frame_sequence_step = 1;

                self.pulse1.envelope.tick();
                self.pulse2.envelope.tick();
                self.noise.envelope.tick();
                self.noise.envelope.tick();
                self.triangle.tick_linear();

                self.pulse1.length_counter.tick();
                self.pulse2.length_counter.tick();
                self.triangle.length_counter.tick();
                self.noise.length_counter.tick();

                self.pulse1.tick_sweep();
                self.pulse2.tick_sweep();
            } else {
                self.frame_sequence_len = 4;
            };
            self.irq_disable = (data >> 6) & 1 == 1;
        }
    }
}
