pub struct PulseChannel {
    pub enabled: bool,
    pub even_latch: bool,
    pub timer: u16,
    pub sequence_step: u8,
    pub length_counter: LengthCounter,
    pub envelope: Envelope,

    pub sweep_enable: bool,
    pub sweep_period: u8,
    pub sweep_negate: bool,
    pub sweep_shift: u8,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            even_latch: true,
            timer: 0,
            sequence_step: 0,
            length_counter: LengthCounter::new(),
            envelope: Envelope::new(),

            sweep_enable: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
        }
    }

    pub fn tick(&mut self) {
        todo!()
    }
}

pub struct TriangleChannel {
    pub enabled: bool,
    pub timer: u16,
    pub sequence_step: u8,
    pub length_counter: LengthCounter,
}

impl TriangleChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            timer: 0,
            sequence_step: 0,
            length_counter: LengthCounter::new(),
        }
    }

    pub fn tick(&mut self) {
        todo!()
    }
}

pub struct NoiseChannel {
    pub enabled: bool,
    pub even_latch: bool,
    pub length_counter: LengthCounter,
    pub envelope: Envelope,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            even_latch: true,
            length_counter: LengthCounter::new(),
            envelope: Envelope::new(),
        }
    }

    pub fn tick(&mut self) {
        todo!()
    }
}

pub struct DMCChannel {
    pub enabled: bool,
    pub even_latch: bool,
}

impl DMCChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            even_latch: true,
        }
    }

    pub fn tick(&mut self) {
        todo!()
    }
}

pub struct LengthCounter {
    pub length_counter: u8,
    pub length_halt: bool,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            length_counter: 0,
            length_halt: false,
        }
    }

    pub fn tick(&mut self) {
        todo!()
    }
}

pub struct Envelope {
    pub constant_volume: bool,
    pub loop_flag: bool,
    pub period: u8, // Either the constant volume or envelope period
    pub decay_counter: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            constant_volume: false,
            loop_flag: false,
            period: 0,
            decay_counter: 0,
        }
    }

    pub fn tick(&mut self) {
        todo!()
    }
}
