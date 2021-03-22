// Represents the time constant and sampling interval
// For a low- or high-pass filter with a given frequency (f)
pub struct FilterTiming {
    rc: f32, // Time constant
    dt: f32, // Sampling interval
}

impl FilterTiming {
    pub fn new(f: f32, sample_rate: u32) -> Self {
        Self {
            // "The break frequency, also called the turnover frequency, corner frequency,
            // or cutoff frequency (in hertz), is determined by the time constant RC"
            // Here, f is given, and we need to calculate RC
            // rc: 1.0 / (2.0 * std::f32::consts::PI * f),
            rc: 1.0 / (2.0 * std::f32::consts::PI * f),
            dt: 1.0 / sample_rate as f32,
        }
    }
}

pub trait Filter {
    fn process(&mut self, in_curr: f32) -> f32;
}

// https://en.wikipedia.org/wiki/Low-pass_filter
pub struct LowPassFilter {
    alpha: f32,
    out_prev: f32,
}

impl LowPassFilter {
    pub fn new(f: f32, sample_rate: u32) -> Self {
        let timing = FilterTiming::new(f, sample_rate);
        Self {
            alpha: timing.dt / (timing.rc + timing.dt),
            out_prev: 0.0,
        }
    }
}

impl Filter for LowPassFilter {
    fn process(&mut self, in_curr: f32) -> f32 {
        self.out_prev = self.alpha * in_curr + (1.0 - self.alpha) * self.out_prev;
        self.out_prev
    }
}

// https://en.wikipedia.org/wiki/High-pass_filter
pub struct HighPassFilter {
    alpha: f32,
    in_prev: f32,
    out_prev: f32,
}

impl HighPassFilter {
    pub fn new(f: f32, sample_rate: u32) -> Self {
        let timing = FilterTiming::new(f, sample_rate);
        Self {
            // TODO: Wikipedia says the numerator should be RC, but only this works for some reason
            alpha: timing.dt / (timing.rc + timing.dt),
            in_prev: 0.0,
            out_prev: 0.0,
        }
    }
}

impl Filter for HighPassFilter {
    fn process(&mut self, in_curr: f32) -> f32 {
        self.out_prev = self.alpha * (self.out_prev + in_curr - self.in_prev);
        self.in_prev = in_curr;
        self.out_prev
    }
}
