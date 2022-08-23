
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
    let s = fs::read_to_string("input/2016/input_23.txt")?;
    match Instruction::parse_list(&s) {
        Ok(("", instructions)) => Ok(instructions),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Value = i32;

#[derive(Debug, Copy, Clone)]
enum Register {A,B,C,D}

#[derive(Debug, Copy, Clone)]
enum Argument {
    Register(Register),
    Value(Value),
}

#[derive(Debug, Copy, Clone)]
enum Instruction {
    Cpy(Argument, Argument),
    Jnz(Argument, Argument),
    Inc(Register),
    Dec(Register),
    Tgl(Register),
}


struct InvalidRegisterError;


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

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Argument::Register(r) => write!(f, "{}", r),
            Argument::Value(v) => write!(f, "{}", v),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Cpy(arg1, arg2) => write!(f, "cop {} {}", arg1, arg2),
            Instruction::Inc(reg) => write!(f, "inc {}", reg),
            Instruction::Dec(reg) => write!(f, "dec {}", reg),
            Instruction::Jnz(arg1, arg2) => write!(f, "jnz {} {}", arg1, arg2),
            Instruction::Tgl(reg) => write!(f, "tgl {}", reg),
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

impl Argument {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(nom_i32, |val| Argument::Value(val)),
            map(Register::parse, |reg| Argument::Register(reg)),
        ))(input)
    }

    /// Whether this is a register or a value, this returns the value it has.
    fn get_val<'a>(&'a self, machine: &'a Machine) -> &'a Value {
        match self {
            Argument::Value(v) => v,
            Argument::Register(r) => &machine.regs[r.addr()],
        }
    }

    /// This gets the argument as a register. If it isn't a register then this
    /// returns InvalidRegisterError
    fn get_reg(&self) -> Result<&Register, InvalidRegisterError> {
        match self {
            Argument::Register(r) => Ok(r),
            Argument::Value(_) => Err(InvalidRegisterError),
        }
    }
}

impl Instruction {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                tuple((tag("cpy "), Argument::parse, tag(" "), Argument::parse)),
                |(_, arg1, _, arg2)| Instruction::Cpy(arg1, arg2)
            ),
            map(
                tuple((tag("jnz "), Argument::parse, tag(" "), Argument::parse)),
                |(_, arg1, _, arg2)| Instruction::Jnz(arg1, arg2)
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
                tuple((tag("tgl "), Register::parse)),
                |(_, reg)| Instruction::Tgl(reg)
            )
        ))(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }

    /// Returns this instruction toggled.
    fn toggle(&self) -> Self {
        match self {
            Instruction::Inc(r) => Instruction::Dec(*r),
            Instruction::Dec(r) => Instruction::Inc(*r),
            Instruction::Tgl(r) => Instruction::Inc(*r),
            Instruction::Cpy(a1, a2) => Instruction::Jnz(*a1, *a2),
            Instruction::Jnz(a1, a2) => Instruction::Cpy(*a1, *a2),
        }
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

    /// Executes a single instruction, modifying the Machine (and maybe the Instructions).
    fn execute_one(&mut self, instructions: &mut Vec<Instruction>) {
        let ins: &Instruction = instructions.get(self.ip).unwrap();
        match ins {
            Instruction::Cpy(arg1, arg2) => {
                match arg2.get_reg() {
                    Err(_) => {}, // invalid command, nothing happens
                    Ok(reg) => {
                        self.regs[reg.addr()] = *arg1.get_val(self);
                    },
                };
                self.ip += 1;
            },

            Instruction::Jnz(arg1, arg2) => {
                if *arg1.get_val(self) != 0 {
                    let val2 = *arg2.get_val(self);
                    incr(&mut self.ip, val2);
                } else {
                    self.ip += 1;
                }
            },

            Instruction::Inc(reg) => {
                self.regs[reg.addr()] += 1;
                self.ip += 1
            },

            Instruction::Dec(reg) => {
                self.regs[reg.addr()] -= 1;
                self.ip += 1
            },

            Instruction::Tgl(reg) => {
                let mut toggle_ins = self.ip; // start with this instruction
                incr(&mut toggle_ins, self.regs[reg.addr()]); // move by contents of that register
                if toggle_ins < instructions.len() {
                    instructions[toggle_ins] = instructions.get(toggle_ins).unwrap().toggle(); // and toggle that one.
                } else {
                    // out of bounds. Nothing happens
                }
                self.ip += 1;
            }
        }
    }
}


fn simulate_machine(machine: &mut Machine, instructions: &mut Vec<Instruction>) {
    while machine.ip < instructions.len() {
        machine.execute_one(instructions); // executes one instruction
    }
    println!("After running, register 'a' contains {}", machine.regs[Register::A.addr()])
}


fn part_a(instructions: &Vec<Instruction>) {
    println!("\nPart a:");
    let mut instructions: Vec<Instruction> = (*instructions).clone(); // make a copy since we'll change it
    let mut machine = Machine::new();
    machine.regs[0] = 7; // set the initial input
    simulate_machine(&mut machine, &mut instructions);
}


fn part_b(instructions: &Vec<Instruction>) {
    println!("\nPart b:");
    let mut instructions: Vec<Instruction> = (*instructions).clone(); // make a copy since we'll change it
    let mut machine = Machine::new();
    machine.regs[0] = 12; // set the initial input
    simulate_machine(&mut machine, &mut instructions);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
