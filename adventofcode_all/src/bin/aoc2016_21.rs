
extern crate anyhow;

use std::fs;
use anyhow::Error;


use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, newline},
    combinator::{opt, map},
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u32 as nom_u32;


fn input() -> Result<Vec<Operation>, Error> {
    let s = fs::read_to_string("input/2016/input_21.txt")?;
    match Operation::parse_list(&s) {
        Ok(("", operations)) => Ok(operations),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



#[derive(Copy, Clone, Debug)]
enum Operation {
    SwapPosition(usize, usize),
    SwapLetter(char, char),
    ReverseRange(usize, usize),
    RotateLeft(usize),
    RotateRight(usize),
    RotateByLetter(char),
    MovePosition(usize, usize),
}


impl Operation {

    fn parse_swap_position(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("swap position "),
                nom_u32,
                tag(" with position "),
                nom_u32,
            )),
            |(_, p1, _, p2)| Operation::SwapPosition(usize::try_from(p1).unwrap(), usize::try_from(p2).unwrap())
        )(input)
    }

    fn parse_swap_letter(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("swap letter "),
                anychar,
                tag(" with letter "),
                anychar,
            )),
            |(_, c1, _, c2)| Operation::SwapLetter(c1, c2)
        )(input)
    }

    fn parse_reverse_range(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("reverse positions "),
                nom_u32,
                tag(" through "),
                nom_u32,
            )),
            |(_, p1, _, p2)| Operation::ReverseRange(usize::try_from(p1).unwrap(), usize::try_from(p2).unwrap())
        )(input)
    }

    fn parse_rotate_left(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("rotate left "),
                nom_u32,
                tag(" step"),
                opt(tag("s")), // could be singular or plural
            )),
            |(_, n, _, _)| Operation::RotateLeft(usize::try_from(n).unwrap())
        )(input)
    }

    fn parse_rotate_right(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("rotate right "),
                nom_u32,
                tag(" step"),
                opt(tag("s")), // could be singular or plural
            )),
            |(_, n, _, _)| Operation::RotateRight(usize::try_from(n).unwrap())
        )(input)
    }

    fn parse_rotate_by_letter(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("rotate based on position of letter "),
                anychar,
            )),
            |(_, c)| Operation::RotateByLetter(c)
        )(input)
    }

    fn parse_move_position(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("move position "),
                nom_u32,
                tag(" to position "),
                nom_u32,
            )),
            |(_, p1, _, p2)| Operation::MovePosition(usize::try_from(p1).unwrap(), usize::try_from(p2).unwrap())
        )(input)
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            Self::parse_swap_position,
            Self::parse_swap_letter,
            Self::parse_reverse_range,
            Self::parse_rotate_left,
            Self::parse_rotate_right,
            Self::parse_rotate_by_letter,
            Self::parse_move_position,
        ))(input)
    }

    fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }
}




fn part_a(operations: &Vec<Operation>) {
    println!("\nPart a:");
    for op in operations {
        println!("{:?}", op);
    }
}



fn part_b(_operations: &Vec<Operation>) {
    println!("\nPart b:");
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}



// ==========================================================================================

#[cfg(test)]
mod tests {
    use super::*;
}
