use crate::addressing_mode::AddressingMode;
use std::collections::HashMap;

pub struct Instruction {
    pub op_str: &'static str,
    pub mode: AddressingMode,
    pub cycles: u32,
}

lazy_static! {
    pub static ref INSTRUCTIONS: HashMap<u8, Instruction> = {
        let mut map = HashMap::new();
        let mut add = |op_str, ops: Vec<(u8, AddressingMode, u32)>| {
            for op in ops {
                map.insert(op.0, Instruction { op_str, mode: op.1, cycles: op.2 });
            }
        };
        use AddressingMode::*;
        add("ADC", vec![(0x69, IMM, 2), (0x65, ZER, 3), (0x75, ZEX, 4), (0x6D, ABS, 4),
                       (0x7D, ABX, 4), (0x79, ABY, 4), (0x61, INX, 6), (0x71, INY, 5)]);
        add("AND", vec![(0x29, IMM, 2), (0x25, ZER, 3), (0x35, ZEX, 4), (0x2D, ABS, 4),
                       (0x3D, ABX, 4), (0x39, ABY, 4), (0x21, INX, 6), (0x31, INY, 5)]);
        add("ASL", vec![(0x0A, ACC, 2), (0x06, ZER, 5), (0x16, ZEX, 6),
                        (0x0E, ABS, 6), (0x1E, ABX, 7)]);
        add("BCC", vec![(0x90, REL, 2)]);
        add("BCS", vec![(0xB0, REL, 2)]);
        add("BEQ", vec![(0xF0, REL, 2)]);
        add("BIT", vec![(0x24, ZER, 3), (0x2C, ABS, 4)]);
        add("BMI", vec![(0x30, REL, 2)]);
        add("BNE", vec![(0xD0, REL, 2)]);
        add("BPL", vec![(0x10, REL, 2)]);
        add("BRK", vec![(0x00, IMP, 7)]);
        add("BVC", vec![(0x50, REL, 2)]);
        add("BVS", vec![(0x70, REL, 2)]);
        add("CLC", vec![(0x18, IMP, 2)]);
        add("CLD", vec![(0xD8, IMP, 2)]);
        add("CLI", vec![(0x58, IMP, 2)]);
        add("CLV", vec![(0xB8, IMP, 2)]);
        add("CMP", vec![(0xC9, IMM, 2), (0xC5, ZER, 3), (0xD5, ZEX, 4), (0xCD, ABS, 4),
                        (0xDD, ABX, 4), (0xD9, ABY, 4), (0xC1, INX, 6), (0xD1, INY, 5)]);
        add("CPX", vec![(0xE0, IMM, 2), (0xE4, ZER, 3), (0xEC, ABS, 4)]);
        add("CPY", vec![(0xC0, IMM, 2), (0xC4, ZER, 3), (0xCC, ABS, 4)]);
        add("DEC", vec![(0xC6, ZER, 5), (0xD6, ZEX, 6), (0xCE, ABS, 6), (0xDE, ABX, 7)]);
        add("DEX", vec![(0xCA, IMP, 2)]);
        add("DEY", vec![(0x88, IMP, 2)]);
        add("EOR", vec![(0x49, IMM, 2), (0x45, ZER, 3), (0x55, ZEX, 4), (0x4D, ABS, 4),
                        (0x5D, ABX, 4), (0x59, ABY, 4), (0x41, INX, 6), (0x51, INY, 5)]);
        add("INC", vec![(0xE6, ZER, 5), (0xF6, ZEX, 6), (0xEE, ABS, 6), (0xFE, ABX, 7)]);
        add("INX", vec![(0xE8, IMP, 2)]);
        add("INY", vec![(0xC8, IMP, 2)]);
        add("JMP", vec![(0x4C, ABS, 3), (0x6C, ABI, 5)]);
        add("JSR", vec![(0x20, ABS, 6)]);
        add("LDA", vec![(0xA9, IMM, 2), (0xA5, ZER, 3), (0xB5, ZEX, 4), (0xAD, ABS, 4),
                        (0xBD, ABX, 4), (0xB9, ABY, 4), (0xA1, INX, 6), (0xB1, INY, 5)]);
        add("LDX", vec![(0xA2, IMM, 2), (0xA6, ZER, 3), (0xB6, ZEY, 4),
                        (0xAE, ABS, 4), (0xBE, ABY, 4)]);
        add("LDY", vec![(0xA0, IMM, 2), (0xA4, ZER, 3), (0xB4, ZEX, 4),
                        (0xAC, ABS, 4), (0xBC, ABX, 4)]);
        add("LSR", vec![(0x4A, ACC, 2), (0x46, ZER, 5), (0x56, ZEX, 6),
                        (0x4E, ABS, 6), (0x5E, ABX, 7)]);
        add("NOP", vec![(0xEA, IMP, 2)]);
        add("ORA", vec![(0x09, IMM, 2), (0x05, ZER, 3), (0x15, ZEX, 4), (0x0D, ABS, 4),
                        (0x1D, ABX, 4), (0x19, ABY, 4), (0x01, INX, 6), (0x11, INY, 5)]);
        add("PHA", vec![(0x48, IMP, 2)]);
        add("PHP", vec![(0x08, IMP, 2)]);
        add("PLA", vec![(0x68, IMP, 2)]);
        add("PLP", vec![(0x28, IMP, 2)]);
        add("ROL", vec![(0x2A, ACC, 2), (0x26, ZER, 5), (0x36, ZEX, 6),
                        (0x2E, ABS, 6), (0x3E, ABX, 7)]);
        add("ROR", vec![(0x6A, ACC, 2), (0x66, ZER, 5), (0x76, ZEX, 6),
                        (0x6E, ABS, 6), (0x7E, ABX, 7)]);
        add("RTI", vec![(0x40, IMP, 2)]);
        add("RTS", vec![(0x60, IMP, 2)]);
        add("SBC", vec![(0xE9, IMM, 2), (0xE5, ZER, 3), (0xF5, ZEX, 4), (0xED, ABS, 4),
                        (0xFD, ABX, 4), (0xF9, ABY, 4), (0xE1, INX, 6), (0xF1, INY, 5)]);
        add("SEC", vec![(0x38, IMP, 2)]);
        add("SED", vec![(0xF8, IMP, 2)]);
        add("SEI", vec![(0x78, IMP, 2)]);
        add("STA", vec![(0x85, ZER, 3), (0x95, ZEX, 4), (0x8D, ABS, 4), (0x9D, ABX, 5),
                        (0x99, ABY, 5), (0x81, INX, 6), (0x91, INY, 6)]);
        add("STX", vec![(0x86, ZER, 3), (0x96, ZEY, 4), (0x8E, ABS, 4)]);
        add("STY", vec![(0x84, ZER, 3), (0x94, ZEX, 4), (0x8C, ABS, 4)]);
        add("TAX", vec![(0xAA, IMP, 2)]);
        add("TAY", vec![(0xA8, IMP, 2)]);
        add("TSX", vec![(0xBA, IMP, 2)]);
        add("TXA", vec![(0x8A, IMP, 2)]);
        add("TXS", vec![(0x9A, IMP, 2)]);
        add("TYA", vec![(0x98, IMP, 2)]);

        // https://wiki.nesdev.com/w/index.php/CPU_unofficial_opcodes
        add("NOP", vec![(0x80, IMM, 2),
                        (0x82, IMM, 2), (0xC2, IMM, 2), (0xE2, IMM, 2),
                        (0x04, ZER, 3), (0x44, ZER, 3), (0x64, ZER, 3),
                        (0x89, IMM, 2),
                        (0xEA, IMP, 2),
                        (0x0C, ABS, 4),
                        (0x14, ZEX, 4), (0x34, ZEX, 4), (0x54, ZEX, 4),
                        (0x74, ZEX, 4), (0xD4, ZEX, 4), (0xF4, ZEX, 4),
                        (0x1A, IMP, 2), (0x3A, IMP, 2), (0x5A, IMP, 2),
                        (0x7A, IMP, 2), (0xDA, IMP, 2), (0xFA, IMP, 2),
                        (0x1C, ABX, 4), (0x3C, ABX, 4), (0x5C, ABX, 4),
                        (0x7C, ABX, 4), (0xDC, ABX, 4), (0xFC, ABX, 4)]);
        add("STP", vec![(0x02, IMM, 2), (0x22, IMM, 2), (0x42, IMM, 2), (0x62, IMM, 2),
                        (0x12, ZEX, 2), (0x32, ZEX, 2), (0x52, ZEX, 2), (0x72, ZEX, 2),
                        (0xD2, ZEX, 2), (0xF2, ZEX, 2)]);
        add("SLO", vec![(0x03, INX, 8), (0x07, ZER, 5), (0x0F, ABS, 6), (0x13, INY, 8),
                        (0x17, ZEX, 6), (0x1B, ABY, 7), (0x1F, ABX, 7)]);
        add("RLA", vec![(0x23, INX, 8), (0x27, ZER, 5), (0x2F, ABS, 6), (0x33, INY, 8),
                        (0x37, ZEX, 6), (0x3B, ABY, 7), (0x3F, ABX, 7)]);
        add("SRE", vec![(0x43, INX, 8), (0x47, ZER, 5), (0x4F, ABS, 6), (0x53, INY, 8),
                        (0x57, ZEX, 6), (0x5B, ABY, 7), (0x5F, ABX, 7)]);
        add("RRA", vec![(0x63, INX, 8), (0x67, ZER, 5), (0x6F, ABS, 6), (0x73, INY, 8),
                        (0x77, ZEX, 6), (0x7B, ABY, 7), (0x7F, ABX, 7)]);
        add("SAX", vec![(0x83, INX, 6), (0x87, ZER, 3), (0x8F, ABS, 4), (0x97, ZEY, 4)]);
        add("SBC", vec![(0xEB, IMM, 2)]);
        add("LAX", vec![(0xA3, INX, 6), (0xA7, ZER, 3), (0xAF, ABS, 4), (0xB3, INY, 5),
                        (0xB7, ZEY, 4), (0xBF, ABY, 4)]);
        add("LAS", vec![(0xBB, ABY, 4)]);
        add("DCP", vec![(0xC3, INX, 8), (0xC7, ZER, 5), (0xCF, ABS, 6), (0xD3, INY, 8),
                        (0xD7, ZEX, 6), (0xDB, ABY, 7), (0xDF, ABX, 7)]);
        add("ISC", vec![(0xE3, INX, 8), (0xE7, ZER, 5), (0xEF, ABS, 6), (0xF3, INY, 8),
                        (0xF7, ZEX, 6), (0xFB, ABY, 7), (0xFF, ABX, 7)]);
        map
    };
}