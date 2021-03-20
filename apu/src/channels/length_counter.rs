// https://wiki.nesdev.com/w/index.php/APU_Length_Counter
pub struct LengthCounter {
    pub enabled: bool,
    pub counter: u8,
    pub halt: bool,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            enabled: false,
            counter: 0,
            halt: false,
        }
    }

    pub fn tick(&mut self) {
        if !self.halt && self.counter > 0 {
            self.counter -= 1;
        }
    }

    pub fn update_enabled(&mut self, control_bit: u8) {
        if control_bit == 1 {
            self.enabled = true;
        } else {
            self.enabled = false;
            self.counter = 0;
        }
    }

    pub fn load(&mut self, length_table_index: u8) {
        self.counter = if (length_table_index >> 4) & 1 == 1 {
            LENGTH_TABLE_HI[length_table_index as usize & 0b1111]
        } else {
            LENGTH_TABLE_LO[length_table_index as usize & 0b1111]
        }
    }
}

// https://wiki.nesdev.com/w/index.php/APU_Length_Counter#Table_structure
// Odd indices select linear length values, even indices select note values
// Low half contains note values in 4/4 at 90 bpm
const LENGTH_TABLE_LO: [u8; 0x10] = [
    0x0A, 0x14, 0x28, 0x50, 0xA0, 0x3C, 0x0E, 0x1A, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x48, 0x10, 0x20,
];

// High half contains note values in 4/4 at 75 bpm
const LENGTH_TABLE_HI: [u8; 0x10] = [
    0xFE, 0x02, 0x04, 0x06, 0x08, 0x0A, 0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A, 0x1C, 0x1E,
];
