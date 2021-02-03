#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

mod status_register;
mod addressing_mode;
pub mod instruction;

use status_register::StatusRegister;
use memory::Memory;
use crate::addressing_mode::AddressingMode;

use std::ops;
use crate::instruction::{Instruction, INSTRUCTIONS};

pub const NMI_VEC: u16 = 0xFFFA;
pub const RST_VEC: u16 = 0xFFFC;
pub const IRQ_VEC: u16 = 0xFFFE;

pub struct CPU {
    // Registers
    a: u8, // Accumulator
    x: u8, pub y: u8, // Index registers
    p: StatusRegister,
    s: u8, // Stack pointer

    pc: u16, // Program counter
    wait_cycles: u32,

    memory: Box<dyn Memory>,
}

impl CPU {
    fn tick(&mut self) {
        if self.wait_cycles > 0 {
            self.wait_cycles -= 1;
            return;
        }

        let opcode = self.memory.read(self.pc);
        self.pc += 1;
        let op = INSTRUCTIONS.get(&opcode).expect("Unimplemented instruction");
        self.wait_cycles = op.cycles;
        self.execute_op(op.op_str, &op.mode);
        self.pc += op.mode.operand_length();
    }

    fn stack_push(&mut self, data: u8) {
        self.memory.write(0x100 + (self.s as u16), data);
        self.s = self.s.wrapping_sub(1);
    }

    fn stack_push_u16(&mut self, data: u16) {
        self.stack_push((data >> 8) as u8);
        self.stack_push(data as u8);
    }

    fn stack_pop(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.memory.read(0x100 + (self.s as u16))
    }

    fn stack_pop_u16(&mut self) -> u16 {
        self.stack_pop() as u16 | ((self.stack_pop() as u16) << 8)
    }

    /// Return the address where data can be found
    /// Branches (relative mode) returns
    /// The boolean represents a page boundary cross
    fn get_operand_address(&mut self, mode: &AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::IMM => (self.pc, false),
            AddressingMode::ABS => (self.memory.read_u16(self.pc), false),
            AddressingMode::ZER => (self.memory.read(self.pc) as u16, false),
            AddressingMode::ZEX => (self.memory.read(self.pc).wrapping_add(self.x) as u16, false),
            AddressingMode::ZEY => (self.memory.read(self.pc).wrapping_add(self.y) as u16, false),
            AddressingMode::ABX => {
                let base = self.memory.read_u16(self.pc);
                let addr = base.wrapping_add(self.x as u16);
                (addr, pages_differ(base, addr))
            },
            AddressingMode::ABY => {
                let base = self.memory.read_u16(self.pc);
                let addr = base.wrapping_add(self.y as u16);
                (addr, pages_differ(base, addr))
            },
            AddressingMode::REL => {
                let offset = self.memory.read(self.pc) as i8;
                let dest = self.pc.wrapping_add(1).wrapping_add(offset as u16);
                (dest, pages_differ(self.pc.wrapping_add(1) & 0xFF00, dest & 0xFF00))
            },
            AddressingMode::INX => {
                let index = self.memory.read(self.pc).wrapping_add(self.x);
                let lo = self.memory.read(index as u16) as u16;
                let hi = self.memory.read(index.wrapping_add(1) as u16) as u16;
                ((hi << 8) | lo, false)
            },
            AddressingMode::INY => {
                let index = self.memory.read(self.pc);
                let lo = self.memory.read(index as u16) as u16;
                let hi = self.memory.read(index.wrapping_add(1) as u16) as u16;
                let addr_base = (hi << 8) | lo;
                let addr = addr_base.wrapping_add(self.y as u16);
                (addr, pages_differ(addr_base, addr))
            },
            AddressingMode::ABI => {
                let addr = self.memory.read_u16(self.pc);
                // 6502 indirect addressing bug at page boundaries
                let hi = if addr & 0x00FF == 0x00FF { addr & 0xFF00 } else { addr + 1 };
                ((hi as u16) << 8 | (self.memory.read(addr) as u16), false)
            },
            _ => panic!("cannot get operand address of mode {:?}", mode),
        }
    }

    fn execute_op(&mut self, op_str: &str, mode: &AddressingMode) {
        match op_str {
            "ADC" => self.arithmetic_op(false, mode),
            "AND" => self.bit_op(ops::BitAnd::bitand, mode),
            "ASL" => self.shift_op(true, mode),
            "BCC" => self.branch_op(StatusRegister::CARRY, false, mode),
            "BCS" => self.branch_op(StatusRegister::CARRY, true, mode),
            "BEQ" => self.branch_op(StatusRegister::ZERO, true, mode),
            "BIT" => self.bit(mode),
            "BMI" => self.branch_op(StatusRegister::NEGATIVE, true, mode),
            "BNE" => self.branch_op(StatusRegister::ZERO, false, mode),
            "BPL" => self.branch_op(StatusRegister::NEGATIVE, false, mode),
            "BRK" => self.brk(),
            "BVC" => self.branch_op(StatusRegister::OVERFLOW, false, mode),
            "BVS" => self.branch_op(StatusRegister::OVERFLOW, true, mode),
            "CLC" => self.p.remove(StatusRegister::CARRY),
            "CLD" => self.p.remove(StatusRegister::DECIMAL),
            "CLI" => self.p.remove(StatusRegister::IRQ_DISABLE),
            "CLV" => self.p.remove(StatusRegister::OVERFLOW),
            "CMP" => self.compare_op(self.a, mode),
            "CPX" => self.compare_op(self.x, mode),
            "CPY" => self.compare_op(self.y, mode),
            "DEC" => self.step_op(-1, mode),
            "DEX" => self.x = self.step_reg_op(-1, self.x),
            "DEY" => self.y = self.step_reg_op(-1, self.y),
            "EOR" => self.bit_op(ops::BitXor::bitxor, mode),
            "INC" => self.step_op(1, mode),
            "INX" => self.x = self.step_reg_op(1, self.x),
            "INY" => self.y = self.step_reg_op(1, self.y),
            "JMP" => self.jump_op(true, mode),
            "JSR" => self.jump_op(false, mode),
            "LDA" => self.a = self.load_op(mode),
            "LDX" => self.x = self.load_op(mode),
            "LDY" => self.y = self.load_op(mode),
            "LSR" => self.shift_op(false, mode),
            "NOP" => {},
            "ORA" => self.bit_op(ops::BitOr::bitor, mode),
            "PHA" => self.stack_push(self.a),
            "PHP" => self.stack_push(self.p.bits()),
            "PLA" => self.pla(),
            "PLP" => { let val = self.stack_pop(); self.p.set_from_stack(val) },
            "ROL" => self.rotate_op(true, mode),
            "ROR" => self.rotate_op(false, mode),
            "RTI" => self.rti(),
            "RTS" => self.pc = self.stack_pop_u16(), // TODO: Add or sub something?
            "SBC" => self.arithmetic_op(true, mode),
            "SEC" => self.p.insert(StatusRegister::CARRY),
            "SED" => self.p.insert(StatusRegister::DECIMAL),
            "SEI" => self.p.insert(StatusRegister::IRQ_DISABLE),
            "STA" => self.store_op(self.a, mode),
            "STX" => self.store_op(self.x, mode),
            "STY" => self.store_op(self.y, mode),
            "TAX" => self.x = self.transfer_op(self.a, false),
            "TAY" => self.y = self.transfer_op(self.a, false),
            "TSX" => self.x = self.transfer_op(self.s, false),
            "TXA" => self.a = self.transfer_op(self.x, false),
            "TXS" => self.s = self.transfer_op(self.x, true),
            "TYA" => self.a = self.transfer_op(self.y, false),
            _ => {},
        };
    }

    /// Perform a binary logic operation (e.g. AND) between A and memory
    fn bit_op(&mut self, f: impl Fn(u8, u8) -> u8, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.memory.read(addr);
        self.a = f(self.a, data);
        self.p.set(StatusRegister::ZERO, self.a == 0);
        self.p.set(StatusRegister::NEGATIVE, (self.a & 0x80) != 0);
    }

    /// Branch if the given flag has the given value
    fn branch_op(&mut self, flag: StatusRegister, value: bool, mode: &AddressingMode) {
        if self.p.contains(flag) == value {
            let (dest, page_cross) = self.get_operand_address(mode);
            self.wait_cycles += if page_cross { 2 } else { 1 };
            self.pc = dest;
        }
    }

    /// Compare a register to memory, then set flags
    fn compare_op(&mut self, reg: u8, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.memory.read(addr);
        let temp: i16 = reg as i16 - data as i16;
        self.p.set(StatusRegister::NEGATIVE, (temp & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, temp == 0);
        self.p.set(StatusRegister::CARRY, temp >= 0x0);
    }

    /// Either increment or decrement data
    fn step_op(&mut self, delta: i8, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.memory.read(addr);
        data = if delta > 0 { data.wrapping_add(1) } else { data.wrapping_sub(1)};
        self.memory.write(addr, data);
        self.p.set(StatusRegister::NEGATIVE, (data & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, data == 0);
    }

    /// Either increment or decrement a register, returning the new value
    fn step_reg_op(&mut self, delta: i8, reg: u8) -> u8 {
        let new_val = if delta > 0 { reg.wrapping_add(1) } else { reg.wrapping_sub(1) };
        self.p.set(StatusRegister::NEGATIVE, (new_val & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, new_val == 0);
        new_val
    }

    /// Load memory and return it to be put into a register
    fn load_op(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.memory.read(addr);
        self.p.set(StatusRegister::NEGATIVE, (data & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, data == 0);
        data
    }

    /// Store data from a register into memory
    fn store_op(&mut self, reg: u8, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.memory.write(addr, reg);
    }

    /// Return a register value to be placed in another register,
    /// setting flags if the destination is not the stack pointer
    fn transfer_op(&mut self, from: u8, to_stack_pointer: bool) -> u8 {
        if !to_stack_pointer {
            self.p.set(StatusRegister::NEGATIVE, (from & 0x80) != 0);
            self.p.set(StatusRegister::ZERO, from == 0);
        }
        from
    }

    /// Add/subtract data to/from A and set flags, possibly using decimal mode
    fn arithmetic_op(&mut self, subtract: bool, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.memory.read(addr);
        if subtract {
            data = ((data as i8).wrapping_neg().wrapping_sub(1)) as u8
        }

        let carry = self.p.contains(StatusRegister::CARRY);
        let sum =  if self.p.contains(StatusRegister::DECIMAL) {
            let temp = bcd_to_bin(self.a).unwrap()
                + bcd_to_bin(data).unwrap() + carry as u8;
            self.p.set(StatusRegister::CARRY, temp > 99);
            bin_to_bcd(temp % 100).unwrap()
        } else {
            let temp = self.a as i16 + data as i16 + carry as i16;
            self.p.set(StatusRegister::CARRY, temp > 0xFF);
            temp as u8
        };

        self.p.set(StatusRegister::OVERFLOW, ((self.a ^ sum) & (data ^ sum) & 0x80) != 0);
        self.a = (sum & 0xFF) as u8;
        self.p.set(StatusRegister::NEGATIVE, (self.a & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, self.a == 0);
    }

    /// Perform a left or right shift, setting flags
    fn shift_op(&mut self, left: bool, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let mut data =
            if *mode == AddressingMode::ACC { self.a } else { self.memory.read(addr) };

        let check_bit = if left { 0x80 } else { 0x01 };
        self.p.set(StatusRegister::CARRY, (data & check_bit) != 0);
        if left { data <<= 1; } else { data >>= 1; };
        if *mode == AddressingMode::ACC {
            self.a = data;
        } else {
            self.memory.write(addr, data);
        };
        self.p.set(StatusRegister::NEGATIVE, (data & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, data == 0);
    }

    /// Perform a left or right rotate, setting flags
    fn rotate_op(&mut self, left: bool, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let mut data =
            if *mode == AddressingMode::ACC { self.a } else { self.memory.read(addr) };

        let old_carry = self.p.contains(StatusRegister::CARRY);
        let (check_bit, carry_bit) = if left { (0x80, 0x01) } else { (0x01, 0x80) };
        self.p.set(StatusRegister::CARRY, (data & check_bit) != 0);
        if left { data <<= 1; } else { data >>= 1; };
        data = if old_carry { data | carry_bit } else { data & !carry_bit };

        if *mode == AddressingMode::ACC {
            self.a = data;
        } else {
            self.memory.write(addr, data);
        };
        self.p.set(StatusRegister::NEGATIVE, (data & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, data == 0);
    }

    fn jump_op(&mut self, save_return: bool, mode: &AddressingMode) {
        if save_return { self.stack_push_u16(self.pc - 1 + 2); } // 2 ahead of opcode
        self.pc = self.get_operand_address(mode).0;
    }

    /// Test bits in memory with accumulator
    /// 2 most significant bits are transferred from data to P [Flags N and V]
    /// Then Flag Z is set according to data & A
    fn bit(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.memory.read(addr);
        self.p.set(StatusRegister::NEGATIVE, (data & 0x80) != 0);
        self.p.set(StatusRegister::OVERFLOW, (data & 0x40) != 0);
        self.p.set(StatusRegister::ZERO, (data & self.a) == 0)
    }

    /// Force a system interrupt
    fn brk(&mut self) {
        self.stack_push_u16(self.pc + 2);
        self.stack_push((self.p | StatusRegister::BREAK).bits());
        self.p.insert(StatusRegister::IRQ_DISABLE);
        self.pc = self.memory.read_u16(IRQ_VEC);
    }

    /// Pop from stack, update the value of A, and set flags
    fn pla(&mut self) {
        self.a = self.stack_pop();
        self.p.set(StatusRegister::NEGATIVE, (self.a & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, self.a == 0);
    }

    /// Return from interrupt
    fn rti(&mut self) {
        let val = self.stack_pop();
        self.p.set_from_stack(val);
        self.pc = self.stack_pop_u16();
    }
}

/// Translates a binary integer to a "Binary Coded Decimal"
/// i.e. decimal(49) => 0x49
fn bin_to_bcd(x: u8) -> Result<u8, &'static str> {
    if x > 99 { Err("Invalid BCD") }
    else { Ok((x % 10) + ((x / 10) << 4)) }
}

/// Translates a "Binary Coded Decimal" to a binary integer
/// i.e. 0x49 => decimal(49)
fn bcd_to_bin(x: u8) -> Result<u8, &'static str> {
    if x > 0x99 { Err("Invalid BCD") }
    else { Ok(10 * ((x & 0xF0) >> 4) + (x & 0x0F)) }
}

fn pages_differ(addr1: u16, addr2: u16) -> bool {
    addr1 & 0xFF00 != addr2 & 0xFF00
}