use memory::Memory;

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

    pub fn format(&self, pc: u16, operands: u16) -> String {
        let fmt = match self {
            AddressingMode::ACC => "A",
            AddressingMode::IMM => "#$b",
            AddressingMode::ABS => "$w",
            AddressingMode::ZER => "$b",
            AddressingMode::ZEX => "$b,X",
            AddressingMode::ZEY => "$b,Y",
            AddressingMode::ABX => "$w,X",
            AddressingMode::ABY => "$w,Y",
            AddressingMode::IMP => "",
            AddressingMode::REL => "$w",
            AddressingMode::INX => "($b,X)",
            AddressingMode::INY => "($b),Y",
            AddressingMode::ABI => "($w)",
        };

        let mut fixed_operands = operands;
        if *self == AddressingMode::REL {
            fixed_operands = pc.wrapping_add(2).wrapping_add(operands as i8 as u16);
        }

        let mut out = String::from("");
        for fmt_char in fmt.chars() {
            out = match fmt_char {
                'b' => format!("{}{:02X}", out, fixed_operands),
                'w' => format!("{}{:04X}", out, fixed_operands),
                _ => format!("{}{}", out, fmt_char),
            };
        }
        out
    }

    pub fn format_data(&self, pc: u16, x: u8, y: u8, memory: &dyn Memory) -> String {
        if [0x20, 0x4C].contains(&memory.peek(pc)) {
            // JSR and ABS jump shouldn't print data
            return String::new();
        }

        match self {
            AddressingMode::ABS => format!(" = {:02X}", memory.peek(memory.peek_u16(pc + 1))),
            AddressingMode::ZER => format!(" = {:02X}", memory.peek(memory.peek(pc + 1) as u16)),
            AddressingMode::ZEX => {
                let target = memory.peek(pc + 1).wrapping_add(x);
                format!(" @ {:02X} = {:02X}", target, memory.peek(target as u16))
            }
            AddressingMode::ZEY => {
                let target = memory.peek(pc + 1).wrapping_add(y);
                format!(" @ {:02X} = {:02X}", target, memory.peek(target as u16))
            }
            AddressingMode::ABX => {
                let target = memory.peek_u16(pc + 1).wrapping_add(x as u16);
                format!(" @ {:04X} = {:02X}", target, memory.peek(target))
            }
            AddressingMode::ABY => {
                let target = memory.peek_u16(pc + 1).wrapping_add(y as u16);
                format!(" @ {:04X} = {:02X}", target, memory.peek(target))
            }
            AddressingMode::INX => {
                let index = memory.peek(pc + 1).wrapping_add(x);
                let lo = memory.peek(index as u16) as u16;
                let hi = memory.peek(index.wrapping_add(1) as u16) as u16;
                let target = (hi << 8) | lo;
                format!(
                    " @ {:02X} = {:04X} = {:02X}",
                    index,
                    target,
                    memory.peek(target)
                )
            }
            AddressingMode::INY => {
                let index = memory.peek(pc + 1);
                let lo = memory.peek(index as u16) as u16;
                let hi = memory.peek(index.wrapping_add(1) as u16) as u16;
                let addr_base = (hi << 8) | lo;
                let target = addr_base.wrapping_add(y as u16);
                format!(
                    " = {:04X} @ {:04X} = {:02X}",
                    addr_base,
                    target,
                    memory.peek(target)
                )
            }
            AddressingMode::ABI => {
                let addr = memory.peek_u16(pc + 1);
                let hi = memory.peek(if addr & 0x00FF == 0x00FF {
                    addr & 0xFF00
                } else {
                    addr.wrapping_add(1)
                });
                format!(" = {:04X}", (hi as u16) << 8 | (memory.peek(addr) as u16))
            }
            _ => String::new(),
        }
    }
}
