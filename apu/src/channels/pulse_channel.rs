use crate::channels::{envelope::Envelope, length_counter::LengthCounter};

// https://wiki.nesdev.com/w/index.php/APU_Pulse
pub struct PulseChannel {
    is_pulse2: bool,
    even_latch: bool,
    timer: u16,
    timer_period: u16,
    sequence_step: u8,
    pub length_counter: LengthCounter,
    pub envelope: Envelope,

    duty_cycle_select: u8,
    sweep_enable: bool,
    sweep_timer: u8,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_target_period: u16,
    sweep_reset: bool,
}

impl PulseChannel {
    pub fn new(is_pulse2: bool) -> Self {
        Self {
            is_pulse2,
            even_latch: true,
            timer: 0,
            timer_period: 0,
            sequence_step: 0,
            length_counter: LengthCounter::new(),
            envelope: Envelope::new(),

            duty_cycle_select: 0,
            sweep_enable: false,
            sweep_timer: 0,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_target_period: 0,
            sweep_reset: false,
        }
    }

    pub fn tick(&mut self) {
        if self.even_latch {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                self.timer = self.timer_period;
                self.sequence_step = (self.sequence_step + 1) % 8;
            }
        }
        self.even_latch = !self.even_latch;
    }

    pub fn tick_sweep(&mut self) {
        // https://wiki.nesdev.com/w/index.php/APU_Sweep#Calculating_the_target_period
        self.sweep_target_period = {
            let mut change = self.timer_period >> self.sweep_shift;
            if self.sweep_negate {
                // "Pulse 1 adds the ones' complement (−c − 1)
                // Pulse 2 adds the two's complement (−c)"
                change = !change;
                change = change.wrapping_add(self.is_pulse2 as u16);
            }
            self.timer_period.wrapping_add(change)
        };

        // http://wiki.nesdev.com/w/index.php/APU_Sweep#Updating_the_period
        let muting = self.sweep_target_period > 0x7FF || self.timer_period < 8;
        if self.sweep_timer == 0 && self.sweep_enable && !muting {
            self.timer_period = self.sweep_target_period;
        }

        if self.sweep_timer == 0 || self.sweep_reset {
            self.sweep_timer = self.sweep_period + 1;
            self.sweep_reset = false;
        } else {
            self.sweep_timer -= 1;
        }
    }

    pub fn update_register(&mut self, register_offset: u16, data: u8) {
        match register_offset {
            0 => {
                self.duty_cycle_select = data >> 6;
                self.length_counter.halt = (data >> 5) & 1 == 1; // These two share a bit
                self.envelope.loop_flag = (data >> 5) & 1 == 1;
                self.envelope.constant_volume = (data >> 4) & 1 == 1;
                self.envelope.period = data & 0b1111;
            }
            1 => {
                self.sweep_reset = true;
                self.sweep_enable = data >> 7 == 1;
                self.sweep_period = (data >> 4) & 0b111;
                self.sweep_negate = (data >> 3) & 1 == 1;
                self.sweep_shift = data & 0b111;
            }
            2 => {
                self.timer_period &= 0xFF00;
                self.timer_period |= data as u16;
            }
            3 => {
                self.timer_period &= 0x00FF;
                self.timer_period |= (data as u16 & 0b111) << 8;

                if self.length_counter.enabled {
                    self.length_counter.load(data >> 3);
                }

                self.sequence_step = 0;
                self.envelope.start = true;
            }
            _ => unimplemented!(),
        }
    }

    pub fn output(&self) -> u8 {
        if PULSE_DUTY_TABLE[self.duty_cycle_select as usize][self.sequence_step as usize] == 0
            || (self.sweep_target_period > 0x7FF || self.timer_period < 8)
            || self.length_counter.counter == 0
            || self.timer < 8
        {
            0
        } else {
            self.envelope.get_volume()
        }
    }
}

// https://wiki.nesdev.com/w/index.php/APU_Pulse#Implementation_details
// The real APU counts *downwards* through simpler sequences
// But this skips a step, counting upwards through these sequences
const PULSE_DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];
