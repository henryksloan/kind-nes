pub struct Scan {
    pub line: u16,
    pub cycle: u16,
    pub total_cycles: u32,
    pub total_frames: u32,
    odd_frame: bool,
}

impl Scan {
    pub fn new() -> Self {
        Self {
            line: 0,
            cycle: 0,
            total_cycles: 0,
            total_frames: 0,
            odd_frame: false,
        }
    }

    pub fn increment(&mut self, is_rendering: bool) {
        self.cycle += 1;
        self.total_cycles += 1;

        // https://wiki.nesdev.com/w/index.php/PPU_frame_timing
        // If rendering is enabled, the pre-render line is one cycle shorter on odd frames.
        // In the real PPU, the scan jumps from (339,261) to (0,0),
        // "doing the last cycle of the last dummy nametable fetch there instead",
        // but in emulation, we can just skip (340,261) entirely and continue as usual
        if self.odd_frame && self.line == 261 && self.cycle == 340 && is_rendering {
            self.cycle += 1;
        }

        if self.cycle > 340 {
            self.cycle = 0;

            self.line += 1;
            if self.line > 261 {
                self.line = 0;
                self.odd_frame = !self.odd_frame;
                self.total_frames += 1;
            }
        }
    }

    pub fn on_visible_line(&self) -> bool {
        self.line <= 239
    }

    pub fn is_clearing_oam2(&self) -> bool {
        self.on_visible_line() && (1 <= self.cycle && self.cycle <= 64)
    }
}
