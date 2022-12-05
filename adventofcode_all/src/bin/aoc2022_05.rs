
extern crate anyhow;

use std::fs;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, anychar, newline, digit1},
    combinator::{value, map},
    multi::{many0, many1, separated_list1},
    sequence::{terminated, tuple, delimited},
};
use nom::character::complete::u32 as nom_u32;
use anyhow::Context;


fn input() -> Result<Puzzle, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_05.txt")?;
    match Puzzle::parse(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}




#[derive(Debug, Copy, Clone)]
struct Instruction {
    count: usize,
    from: usize,
    to: usize,
}

#[derive(Debug, Copy, Clone)]
struct Crate {
    c: char
}

#[derive(Debug, Copy, Clone)]
enum CrateOrSpace {
    Crate(Crate),
    Space,
}

#[derive(Debug, Clone)]
struct Stacks {
    crates: Vec<Vec<Crate>>,
}

#[derive(Debug)]
struct Puzzle {
    start_stacks: Stacks,
    instructions: Vec<Instruction>
}

/// Converts a u32 to a usize, panicking if it fails.
fn u32_to_usize(x: u32) -> usize {
    usize::try_from(x).unwrap()
}

impl Instruction {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("move "),
                nom_u32,
                tag(" from "),
                nom_u32,
                tag(" to "),
                nom_u32,
            )),
            |(_, c, _, f, _, t)| Instruction{count: u32_to_usize(c), from: u32_to_usize(f), to: u32_to_usize(t)}
        )(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }
}


impl Crate {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            delimited(char('['), anychar, char(']')),
            |c| Self{c}
        )(input)
    }
}

impl CrateOrSpace {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(CrateOrSpace::Space, tag("   ")),
            map(Crate::parse, |cr| CrateOrSpace::Crate(cr)),
        ))(input)
    }

    fn parse_line<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        terminated(separated_list1(tag(" "), CrateOrSpace::parse), newline)(input)
    }
}


/// Given a Vector of Vectors of CrateOrSpace, this transposes them (in a specific direction) filling
/// in any missing value with default_value.
fn transpose(orig: Vec<Vec<CrateOrSpace>>) -> Vec<Vec<Crate>> {
    let mut answer: Vec<Vec<Crate>> = Vec::new();
    let cols = orig.iter().map(|x| x.len()).max().unwrap();
    for _i in 0..cols {
        answer.push(Vec::new())
    }
    for j in (0..orig.len()).rev() {
        let orig_vec = orig.get(j).unwrap();
        for i in 0..cols {
            let orig_value: &CrateOrSpace = orig_vec.get(i).unwrap_or(&CrateOrSpace::Space);
            match orig_value {
                CrateOrSpace::Space => {},
                CrateOrSpace::Crate(cr) => {
                    answer.get_mut(i).unwrap().push(cr.clone())
                },
            }
        }
    }
    answer
}


impl Stacks {
    /// Parses (but ignores) the column numbers under the rows of CrateOrSpace.
    fn parse_col_nums<'a>(input: &'a str) -> IResult<&'a str, ()> {
        map(
            many1(alt((tag(" "), digit1))), // spaces and digits
            |_| () // which we ignore, returning a space.
        )(input)
    }

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                many1(CrateOrSpace::parse_line),
                Self::parse_col_nums,
                newline,
            )),
            |(v, _, _)| Stacks{ crates: transpose(v) }
        )(input)
    }

    /// Modifies self by applying a particular instruction.
    fn apply_instruction(&mut self, ins: &Instruction) -> Result<(), anyhow::Error> {
        assert!((ins.from - 1) < self.crates.len() && (ins.to - 1) < self.crates.len());
        for _ in 0..ins.count {
            let cr: Crate = self.crates[ins.from - 1].pop().context("Stack ran out")?;
            self.crates[ins.to - 1].push(cr);
        };
        Ok(())
    }

    /// This reads the top crate of each stack and returns the result as a string. If any
    /// stack is empty it returns an error.
    fn get_top_crates_string(&self) -> Result<String, anyhow::Error> {
        let mut answer: String = String::new();
        for stack in self.crates.iter() {
            answer.push( stack.last().context("Stack was empty")?.c );
        }
        Ok(answer)
    }
}

impl Puzzle {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                Stacks::parse,
                newline,
                Instruction::parse_list,
            )),
            |(start_stacks, _, instructions)| Puzzle{start_stacks, instructions}
        )(input)
    }

    fn apply_instructions(&self) -> Result<Stacks, anyhow::Error> {
        let mut answer = self.start_stacks.clone();
        for ins in self.instructions.iter() {
            answer.apply_instruction(ins)?;
        }
        Ok(answer)
    }
}


fn part_a(input: &Puzzle) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    let s = input.apply_instructions()?.get_top_crates_string()?;
    println!("Top crates: {}", s);
    Ok(())
}


fn part_b(_input: &Puzzle) -> Result<(), anyhow::Error> {
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
