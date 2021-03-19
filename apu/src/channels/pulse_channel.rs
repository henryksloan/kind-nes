use crate::channels::{envelope::Envelope, length_counter::LengthCounter};

// https://wiki.nesdev.com/w/index.php/APU_Pulse
pub struct PulseChannel {
    pub is_pulse2: bool,
    pub enabled: bool,
    pub even_latch: bool,
    pub timer: u16,
    pub timer_period: u16,
    pub sequence_step: u8,
    pub length_counter: LengthCounter,
    pub envelope: Envelope,

    pub duty_cycle_select: u8,
    pub sweep_enable: bool,
    pub sweep_timer: u8,
    pub sweep_period: u8,
    pub sweep_negate: bool,
    pub sweep_shift: u8,
    pub sweep_reset: bool,
    pub sweep_muting: bool,
}

impl PulseChannel {
    pub fn new(is_pulse2: bool) -> Self {
        Self {
            is_pulse2,
            enabled: false,
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
            sweep_reset: false,
            sweep_muting: false,
        }
    }

    pub fn tick(&mut self) {
        if self.even_latch {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                // TODO: Should this be plus 1?
                // http://nesdev.com/apu_ref.txt:
                // "For the square and triangle channels, the third and fourth registers form an 11-bit
                // value and the divider's period is set to this value *plus one*."
                self.timer = self.timer_period;
                self.sequence_step = (self.sequence_step + 1) % 8;
            }
        }
        self.even_latch = !self.even_latch;
    }

    pub fn tick_sweep(&mut self) {
        // https://wiki.nesdev.com/w/index.php/APU_Sweep#Calculating_the_target_period
        let target_period = {
            let mut change = self.timer_period >> self.sweep_shift;
            if self.sweep_negate {
                // "Pulse 1 adds the ones' complement (−c − 1)
                // Pulse 2 adds the two's complement (−c)"
                change = !change;
                change += self.is_pulse2 as u16;
            }
            self.timer_period + change
        };

        // http://wiki.nesdev.com/w/index.php/APU_Sweep#Updating_the_period
        self.sweep_muting = target_period > 0x7FF || self.timer_period < 8;
        if self.sweep_timer == 0 && self.sweep_enable && !self.sweep_muting {
            self.timer_period = target_period;
        }

        if self.sweep_timer == 0 || self.sweep_reset {
            self.sweep_timer = self.sweep_period;
            self.sweep_reset = false;
        } else {
            self.sweep_timer -= 1;
        }
    }
}
