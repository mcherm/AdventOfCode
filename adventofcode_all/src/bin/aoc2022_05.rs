
extern crate anyhow;

use std::fs;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, anychar, newline, digit1},
    combinator::{value, map, rest},
    multi::{many0, many1, separated_list1},
    sequence::{terminated, tuple, delimited},
};
use nom::character::complete::u32 as nom_u32;


fn input() -> Result<Puzzle, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_05.txt")?;
    match Puzzle::parse(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



type Num = u32;

#[derive(Debug, Copy, Clone)]
struct Instruction {
    count: Num,
    from: Num,
    to: Num,
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

#[derive(Debug)]
struct RowOfStacks {
    items: Vec<CrateOrSpace>,
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
            |(_, c, _, f, _, t)| Instruction{count: c, from: f, to: t}
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
            )),
            |(v, _)| Stacks{ crates: transpose(v) }
        )(input)
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
}

fn part_a(input: &Puzzle) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    println!("Puzzle: {:?}", input);
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
