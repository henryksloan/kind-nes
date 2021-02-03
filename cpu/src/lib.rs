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

    }

    fn stack_push(&mut self, data: u8) {
        self.memory.write(0x100 + (self.s as u16), data);
        self.s = self.s.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.memory.read(0x100 + (self.s as u16))
    }

    fn get_operand_address(&self, mode: AddressingMode) -> (u16, bool) {
        // TODO: It would be nice to have a (separate?) function just to get data
        // But it would also have to get page cross?
        (0x0, false)
    }

    fn execute_op(&mut self, op_str: &str, mode: AddressingMode) {
        match op_str {
            "ADC" => self.arithmetic_op(false, mode),
            "AND" => self.bit_op(ops::BitAnd::bitand, mode),
            // "ASL" => bind_op(&CPU6502::Op_ASL),
            "BCC" => self.branch_op(StatusRegister::CARRY, false),
            "BCS" => self.branch_op(StatusRegister::CARRY, true),
            "BEQ" => self.branch_op(StatusRegister::ZERO, true),
            // "BIT" => bind_op(&CPU6502::Op_BIT),
            "BMI" => self.branch_op(StatusRegister::NEGATIVE, true),
            "BNE" => self.branch_op(StatusRegister::ZERO, false),
            "BPL" => self.branch_op(StatusRegister::NEGATIVE, false),
            // "BRK" => bind_op(&CPU6502::Op_BRK),
            "BVC" => self.branch_op(StatusRegister::OVERFLOW, false),
            "BVS" => self.branch_op(StatusRegister::OVERFLOW, true),
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
            // "JMP" => bind_op(&CPU6502::Op_JMP), // TODO: See if these two can be generated
            // "JSR" => bind_op(&CPU6502::Op_JSR),
            "LDA" => self.a = self.load_op(mode),
            "LDX" => self.x = self.load_op(mode),
            "LDY" => self.y = self.load_op(mode),
            // "LSR" => bind_op(&CPU6502::Op_LSR),
            "NOP" => {},
            "ORA" => self.bit_op(ops::BitOr::bitor, mode),
            "PHA" => self.stack_push(self.a),
            "PHP" => self.stack_push(self.p.bits()),
            "PLA" => self.pla(),
            "PLP" => { let val = self.stack_pop(); self.p.set_from_stack(val) },
            // "ROL" => bind_op(&CPU6502::Op_ROL), // TODO: Maybe these too
            // "ROR" => bind_op(&CPU6502::Op_ROR),
            // "RTI" => bind_op(&CPU6502::Op_RTI),
            // "RTS" => bind_op(&CPU6502::Op_RTS),
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
    fn bit_op(&mut self, f: impl Fn(u8, u8) -> u8, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.memory.read(addr);
        self.a = f(self.a, data);
        self.p.set(StatusRegister::ZERO, self.a == 0);
        self.p.set(StatusRegister::NEGATIVE, (self.a & 0x80) != 0);
    }

    /// Branch if the given flag has the given value
    fn branch_op(&mut self, flag: StatusRegister, value: bool) {
        // TODO
        // let (addr, _) = self.get_operand_address(mode);
        // let data = self.memory.read(addr);
        // if self.p.contains(flag) == value {
        // }
    }

    /// Compare a register to memory, then set flags
    fn compare_op(&mut self, reg: u8, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.memory.read(addr);
        let temp: i16 = reg as i16 - data as i16;
        self.p.set(StatusRegister::NEGATIVE, (temp & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, temp == 0);
        self.p.set(StatusRegister::CARRY, temp >= 0x0);
    }

    /// Either increment or decrement data
    fn step_op(&mut self, delta: i8, mode: AddressingMode) {
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
    fn load_op(&mut self, mode: AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.memory.read(addr);
        self.p.set(StatusRegister::NEGATIVE, (data & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, data == 0);
        data
    }

    /// Store data from a register into memory
    fn store_op(&mut self, reg: u8, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.memory.write(addr, reg);
    }

    /** Return a register value to be placed in another register,
        setting flags if the destination is not the stack pointer  **/
    fn transfer_op(&mut self, from: u8, to_stack_pointer: bool) -> u8 {
        if !to_stack_pointer {
            self.p.set(StatusRegister::NEGATIVE, (from & 0x80) != 0);
            self.p.set(StatusRegister::ZERO, from == 0);
        }
        from
    }

    /// Add/subtract data to/from A and set flags, possibly using decimal mode
    fn arithmetic_op(&mut self, subtract: bool, mode: AddressingMode) {
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

    /// Pop from stack, update the value of A, and set flags
    fn pla(&mut self) {
        self.a = self.stack_pop();
        self.p.set(StatusRegister::NEGATIVE, (self.a & 0x80) != 0);
        self.p.set(StatusRegister::ZERO, self.a == 0);
    }
}

/** Translates a binary integer to a "Binary Coded Decimal"
    i.e. decimal(49) => 0x49 **/
fn bin_to_bcd(x: u8) -> Result<u8, &'static str> {
    if x > 99 { Err("Invalid BCD") }
    else { Ok((x % 10) + ((x / 10) << 4)) }
}

/** Translates a "Binary Coded Decimal" to a binary integer
    i.e. 0x49 => decimal(49) **/
fn bcd_to_bin(x: u8) -> Result<u8, &'static str> {
    if x > 0x99 { Err("Invalid BCD") }
    else { Ok(10 * ((x & 0xF0) >> 4) + (x & 0x0F)) }
}
