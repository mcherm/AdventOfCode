mod eznom;

use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io;

use nom::branch::alt as nom_alt;
use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::i32 as nom_num;
use nom::character::complete::newline as nom_newline;
use nom::multi::many0 as nom_many0;
use nom::sequence::tuple as nom_tuple;
use eznom::type_builder;



#[derive(Debug)]
enum Error {
    Io(io::Error),
}
impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Error::Io(e) } }



fn input() -> Result<Program, Error> {
    let s = fs::read_to_string("input/2015/23/input.txt")?;
    match Program::parse(&s) {
        Ok(("", boss)) => Ok(boss),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type RegisterValue = u32;
type Offset = i32;
type InstructionPointer = usize;


/// Adjusts the instruction pointer by the given offset. Panics if it
/// is reduced below 0.
fn add_offset(ip: InstructionPointer, off: Offset) -> InstructionPointer {
    if off.is_negative() {
        let off_abs: InstructionPointer = InstructionPointer::try_from(off.abs()).unwrap();
        if off_abs > ip {
            panic!("Instruction Pointer moved to below zero.");
        }
        ip - off_abs
    } else {
        ip + InstructionPointer::try_from(off).unwrap()
    }
}




#[derive(Debug, Copy, Clone)]
enum Register {
    A,
    B,
}

impl Register {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_alt((
            nom_tag("a"),
            nom_tag("b"),
        ))(s);
        let build = |s| match s {
            "a" => Register::A,
            "b" => Register::B,
            _ => panic!()
        };
        type_builder(recognize, build)(input)
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {Register::A => "a", Register::B => "b"})
    }
}


#[derive(Debug, Copy, Clone)]
enum Instruction {
    Hlf(Register),
    Tpl(Register),
    Inc(Register),
    Jmp(Offset),
    Jie(Register, Offset),
    Jio(Register, Offset),
}

impl Instruction {
    fn parse_hlf(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("hlf "),
            Register::parse,
            nom_newline,
        ))(s);
        let build = |(_, register, _): (&str, Register, char)|
            Instruction::Hlf(register);
        type_builder(recognize, build)(input)
    }

    fn parse_tpl(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("tpl "),
            Register::parse,
            nom_newline,
        ))(s);
        let build = |(_, register, _): (&str, Register, char)|
            Instruction::Tpl(register);
        type_builder(recognize, build)(input)
    }

    fn parse_inc(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("inc "),
            Register::parse,
            nom_newline,
        ))(s);
        let build = |(_, register, _): (&str, Register, char)|
            Instruction::Inc(register);
        type_builder(recognize, build)(input)
    }

    fn parse_offset(input: &str) -> nom::IResult<&str, Offset> {
        let recognize = |s| nom_tuple((
            nom_alt((
                nom_tag("+"),
                nom_tag("-"),
            )),
            nom_num,
        ))(s);
        let build = |(sign, num): (&str, i32)| {
            match sign {
                "+" => num,
                "-" => -num,
                _ => panic!(),
            }
        };
        type_builder(recognize, build)(input)
    }

    fn parse_jmp(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("jmp "),
            Instruction::parse_offset,
            nom_newline,
        ))(s);
        let build = |(_, offset, _): (&str, Offset, char)|
            Instruction::Jmp(offset);
        type_builder(recognize, build)(input)
    }

    fn parse_jie(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("jie "),
            Register::parse,
            nom_tag(", "),
            Instruction::parse_offset,
            nom_newline,
        ))(s);
        let build = |(_, register, _, offset, _): (&str, Register, &str, Offset, char)|
            Instruction::Jie(register, offset);
        type_builder(recognize, build)(input)
    }

    fn parse_jio(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("jio "),
            Register::parse,
            nom_tag(", "),
            Instruction::parse_offset,
            nom_newline,
        ))(s);
        let build = |(_, register, _, offset, _): (&str, Register, &str, Offset, char)|
            Instruction::Jio(register, offset);
        type_builder(recognize, build)(input)
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_alt((
            Instruction::parse_hlf,
            Instruction::parse_tpl,
            Instruction::parse_inc,
            Instruction::parse_jmp,
            Instruction::parse_jie,
            Instruction::parse_jio,
        ))(s);
        let build = |ins: Instruction| ins;
        type_builder(recognize, build)(input)
    }

    pub fn parse_list(input: &str) -> nom::IResult<&str, Vec<Self>> {
        nom_many0(Self::parse)(input)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Hlf(r) => write!(f, "hlf {}", r),
            Instruction::Tpl(r) => write!(f, "tpl {}", r),
            Instruction::Inc(r) => write!(f, "inc {}", r),
            Instruction::Jmp(off) => write!(f, "jmp {}", off),
            Instruction::Jie(r, off) => write!(f, "jie {}, {}", r, off),
            Instruction::Jio(r, off) => write!(f, "jio {}, {}", r, off),
        }
    }
}


#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| Instruction::parse_list(s);
        let build = |instructions: Vec<Instruction>| Self{instructions};
        type_builder(recognize, build)(input)
    }
}


struct Machine {
    a: RegisterValue,
    b: RegisterValue,
    next: InstructionPointer,
}

impl Default for Machine {
    fn default() -> Self {
        Machine{a: 0, b: 0, next: 0}
    }
}

impl Machine {
    fn reg(&mut self, r: Register) -> &mut RegisterValue {
        match r {
            Register::A => &mut self.a,
            Register::B => &mut self.b,
        }
    }

    fn execute(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::Hlf(r) => {
                let rr = self.reg(*r);
                if *rr % 2 != 0 {
                    panic!("Calling hlf on an odd-valued register.");
                }
                *rr /= 2;
                self.next += 1;
            }
            Instruction::Tpl(r) => {
                *self.reg(*r) *= 3;
                self.next += 1;
            }
            Instruction::Inc(r) => {
                *self.reg(*r) += 1;
                self.next += 1;
            }
            Instruction::Jmp(off) => {
                self.next = add_offset(self.next, *off);
            }
            Instruction::Jie(r, off) => {
                if *self.reg(*r) % 2 == 0 {
                    self.next = add_offset(self.next, *off);
                } else {
                    self.next += 1;
                }
            }
            Instruction::Jio(r, off) => {
                if *self.reg(*r) % 2 == 1 {
                    self.next = add_offset(self.next, *off);
                } else {
                    self.next += 1;
                }
            }
        }
    }
}

impl Display for Machine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "A: {}, B: {}, IP: {}", self.a, self.b, self.next)
    }
}



/// Given a program, this runs it.
fn run_program(program: &Program) {
    let mut machine: Machine = Default::default();

    println!("Initial State: {}", machine);
    while let Some(instruction) = program.instructions.get(machine.next) {
        machine.execute(instruction);
        println!("Machine State: {} after doing {}", machine, instruction);
    }
    println!("Machine halted with {} in register b.", machine.b);
}


fn part_a(program: &Program) {
    println!("---- Part A ----");
    run_program(program);
}


fn part_b(_program: &Program) {
    println!("---- Part B ----");
}

fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
