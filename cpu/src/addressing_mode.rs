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