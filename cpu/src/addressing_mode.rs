#[derive(PartialEq, Debug)]
pub enum AddressingMode {
    ACC, // Accumulator
    IMM, // Immediate
    ABS, // Absolute
    ZER, // Zero page
    ZEX, // Zero page, X-indexed
    ZEY, // Zero page, Y-indexed
    ABX, // Absolute, X-indexed
    ABY, // Absolute, Y-indexed
    IMP, // Implied
    REL, // Relative
    INX, // Indirect, X-indexed
    INY, // Indirect, Y-indexed
    ABI, // Absolute indirect
}

impl AddressingMode {
    pub fn operand_length(&self) -> u16 {
        match self {
            AddressingMode::ACC => 0,
            AddressingMode::IMM => 1,
            AddressingMode::ABS => 2,
            AddressingMode::ZER => 1,
            AddressingMode::ZEX => 1,
            AddressingMode::ZEY => 1,
            AddressingMode::ABX => 2,
            AddressingMode::ABY => 2,
            AddressingMode::IMP => 0,
            AddressingMode::REL => 1,
            AddressingMode::INX => 1,
            AddressingMode::INY => 1,
            AddressingMode::ABI => 2,
        }
    }
}
