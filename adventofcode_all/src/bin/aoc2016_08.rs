
extern crate anyhow;

use std::fs;
use anyhow::Error;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::newline,
    combinator::{value, map},
    multi::many0,
    sequence::tuple
};
use nom::character::complete::u32 as nom_u32;



fn input() -> Result<Vec<Command>, Error> {
    let s = fs::read_to_string("input/2016/input_08.txt")?;
    match Command::parse_vec(&s) {
        Ok(("", commands)) => Ok(commands),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Value = u32;

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Row,
    Column,
}

impl Direction {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(Self::Row, tag("row y")),
            value(Self::Column, tag("column x")),
        ))(input)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Command {
    Rect{x: Value, y: Value},
    Rotate{dir: Direction, which: Value, dist: Value},
}

impl Command {
    fn parse_rect<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("rect "),
                nom_u32,
                tag("x"),
                nom_u32,
                newline
            )),
            |(_, x, _, y, _)| Self::Rect{x, y}
        )(input)
    }

    fn parse_rotate<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("rotate "),
                Direction::parse,
                tag("="),
                nom_u32,
                tag(" by "),
                nom_u32,
                newline
            )),
            |(_, dir, _, which, _, dist, _)| Self::Rotate{dir, which, dist}
        )(input)
    }

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            Self::parse_rect,
            Self::parse_rotate,
        ))(input)
    }

    fn parse_vec<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0(Self::parse)(input)
    }
}


fn part_a(commands: &Vec<Command>) {
    println!("\nPart a:");
    for command in commands {
        println!("Command: {:?}", command);
    }
}


fn part_b(_commands: &Vec<Command>) {
    println!("\nPart b:");
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
