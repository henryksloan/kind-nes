use cpu;

fn main() {
    println!("{}", cpu::instruction::INSTRUCTIONS[&0x69].op_str);
}