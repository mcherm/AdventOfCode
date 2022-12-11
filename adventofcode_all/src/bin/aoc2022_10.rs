
extern crate anyhow;
extern crate elsa;

use std::fs;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::newline,
    combinator::map,
    multi::many0,
    sequence::{terminated, pair},
};
use nom::character::complete::i32 as nom_i32;


// ======= Parsing =======

fn input() -> Result<Vec<Instruction>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_10.txt")?;
    match Instruction::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}

type Num = i32;

#[derive(Debug, Copy, Clone)]
enum Instruction {
    Noop,
    Addx(Num),
}


impl Instruction {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                tag("noop"),
                |_| Instruction::Noop
            ),
            map(
                pair(tag("addx "), nom_i32),
                |(_, num)| Instruction::Addx(num)
            ),
        ))(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }

    fn ticks_needed(&self) -> usize {
        match self {
            Instruction::Noop => 1,
            Instruction::Addx(_) => 2,
        }
    }
}

// ======= Calculations =======

fn process<'a>(instructions: &'a Vec<Instruction>) -> RegisterValueIter<'a> {
    RegisterValueIter::new(instructions)
}


struct RegisterValueIter<'a> {
    register: Num,
    next_pos: usize,
    ticks_worked: usize,
    finished: bool,
    instructions: &'a Vec<Instruction>
}

impl<'a> RegisterValueIter<'a> {
    fn new(instructions: &'a Vec<Instruction>) -> Self {
        RegisterValueIter{
            register: 1,
            next_pos: 0,
            ticks_worked: 0,
            finished: false,
            instructions
        }
    }
}

impl<'a> Iterator for RegisterValueIter<'a> {
    type Item = Num;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else if self.next_pos >= self.instructions.len() {
            self.finished = true;
            Some(self.register)
        } else {
            let answer = self.register;
            let inst = self.instructions[self.next_pos];
            self.ticks_worked += 1;
            if self.ticks_worked == inst.ticks_needed() {
                self.next_pos += 1;
                self.ticks_worked = 0;
                match inst {
                    Instruction::Noop => {},
                    Instruction::Addx(add_val) => {
                        self.register += add_val;
                    },
                }
            }
            Some(answer)
        }
    }
}


// ======= main() =======

fn part_a(input: &Vec<Instruction>) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    let mut sum: i32 = 0;
    for (i, val) in process(input).enumerate() {
        let step = i + 1;
        if step % 40 == 20 {
            let strength = i32::try_from(step)? * val;
            sum += strength;
            println!("Step {} has strength {}", step, strength);
        }
        println!("Value at step {} is {}", step, val);
    }
    println!("Sum of Signal Strengths = {}", sum);
    Ok(())
}


fn part_b(_input: &Vec<Instruction>) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
