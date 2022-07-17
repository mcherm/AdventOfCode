
extern crate anyhow;

use std::fmt::{Display, Formatter};
use std::fs;
use anyhow::Error;


use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::newline,
    combinator::{value, map},
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::i32 as nom_i32;


fn input() -> Result<Vec<Instruction>, Error> {
    let s = fs::read_to_string("input/2016/input_12.txt")?;
    match Instruction::parse_list(&s) {
        Ok(("", instructions)) => Ok(instructions),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Value = i32;

#[derive(Debug, Copy, Clone)]
enum Register {A,B,C,D}

#[derive(Debug)]
enum Instruction {
    CpyVal(Value, Register),
    CpyReg(Register, Register),
    Inc(Register),
    Dec(Register),
    JnzVal(Value, Value),
    JnzReg(Register, Value),
}


impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                Register::A => "a",
                Register::B => "b",
                Register::C => "c",
                Register::D => "d",
            }
        )
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::CpyVal(val, reg) => write!(f, "cpy {} {}", val, reg),
            Instruction::CpyReg(reg1, reg2) => write!(f, "cpy {} {}", reg1, reg2),
            Instruction::Inc(reg) => write!(f, "inc {}", reg),
            Instruction::Dec(reg) => write!(f, "dec {}", reg),
            Instruction::JnzVal(val1, val2) => write!(f, "jnz {} {}", val1, val2),
            Instruction::JnzReg(reg, val) => write!(f, "jnz {} {}", reg, val),
        }
    }
}


impl Register {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(Register::A, tag("a")),
            value(Register::B, tag("b")),
            value(Register::C, tag("c")),
            value(Register::D, tag("d")),
        ))(input)
    }

    fn addr(&self) -> usize {
        match self {
            Register::A => 0,
            Register::B => 1,
            Register::C => 2,
            Register::D => 3,
        }
    }
}

impl Instruction {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                tuple((tag("cpy "), nom_i32, tag(" "), Register::parse)),
                |(_, val, _, reg)| Instruction::CpyVal(val, reg)
            ),
            map(
                tuple((tag("cpy "), Register::parse, tag(" "), Register::parse)),
                |(_, reg1, _, reg2)| Instruction::CpyReg(reg1, reg2)
            ),
            map(
                tuple((tag("inc "), Register::parse)),
                |(_, reg)| Instruction::Inc(reg)
            ),
            map(
                tuple((tag("dec "), Register::parse)),
                |(_, reg)| Instruction::Dec(reg)
            ),
            map(
                tuple((tag("jnz "), nom_i32, tag(" "), nom_i32)),
                |(_, val1, _, val2)| Instruction::JnzVal(val1, val2)
            ),
            map(
                tuple((tag("jnz "), Register::parse, tag(" "), nom_i32)),
                |(_, reg, _, val)| Instruction::JnzReg(reg,val)
            ),
        ))(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }
}


struct Machine {
    ip: usize,
    regs: [Value; 4],
}

impl Display for Machine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{} {} {} {}]",
               self.ip,
               self.regs[0],
               self.regs[1],
               self.regs[2],
               self.regs[3],
        )
    }
}


/// Wrap the type casting needed to add an i32 to a
/// usize. Panics if it goes below zero.
fn incr(a: &mut usize, b: i32) {
    *a = usize::try_from(i32::try_from(*a).unwrap() + b).unwrap()
}

impl Machine {
    fn new() -> Self {
        Machine{ip: 0, regs: [0;4] }
    }

    /// Executes an instruction, modifying the Machine.
    fn execute(&mut self, ins: &Instruction) {
        let (i,a,b,c,d) = (self.ip, self.regs[0], self.regs[1], self.regs[2], self.regs[3]);
        match ins {
            Instruction::CpyVal(val, reg) => {self.regs[reg.addr()] = *val; self.ip += 1},
            Instruction::CpyReg(reg1, reg2) => {self.regs[reg2.addr()] = self.regs[reg1.addr()]; self.ip += 1},
            Instruction::Inc(reg) => {self.regs[reg.addr()] += 1; self.ip += 1},
            Instruction::Dec(reg) => {self.regs[reg.addr()] -= 1; self.ip += 1},
            Instruction::JnzVal(val1, val2) => if *val1 != 0 {
                incr(&mut self.ip, *val2)
            } else {
                self.ip += 1;
            },
            Instruction::JnzReg(reg, val) => if self.regs[reg.addr()] != 0 {
                incr(&mut self.ip, *val);
            } else {
                self.ip += 1;
            },
        }
        if (i,a,b,c,d) == (self.ip, self.regs[0], self.regs[1], self.regs[2], self.regs[3]) {
            println!("{}", self);
            panic!("Did not change!");
        }
    }
}


fn simulate_machine(machine: &mut Machine, instructions: &Vec<Instruction>) {
    while machine.ip < instructions.len() {
        machine.execute(instructions.get(machine.ip).unwrap());
    }
    println!("After, register a contains {}", machine.regs[Register::A.addr()])
}


fn part_a(instructions: &Vec<Instruction>) {
    println!("\nPart a:");
    let mut machine = Machine::new();
    simulate_machine(&mut machine, instructions);
}


fn part_b(instructions: &Vec<Instruction>) {
    println!("\nPart b:");
    let mut machine = Machine::new();
    machine.regs[2] = 1;
    simulate_machine(&mut machine, instructions);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
