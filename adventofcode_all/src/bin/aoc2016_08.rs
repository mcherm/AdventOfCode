
extern crate anyhow;

use std::fs;
use anyhow::Error;

use nom::IResult;
use nom::combinator::{value, map};
use nom::sequence::tuple;
use nom::bytes::complete::tag;
use nom::character::complete::u32 as nom_u32;
use nom::character::complete::newline;
use nom::branch::alt;
use nom::multi::many0;



fn input() -> Result<Vec<Command>, Error> {
    let s = fs::read_to_string("input/2016/input_08.txt")?;
    match parse_commands(&s) {
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

#[derive(Debug, Copy, Clone)]
pub enum Command {
    Rect{x: Value, y: Value},
    Rotate{dir: Direction, which: Value, dist: Value},
}

fn parse_direction<'a>(input: &'a str) -> IResult<&'a str, Direction> {
    alt((
        value(Direction::Row, tag("row y")),
        value(Direction::Column, tag("column x")),
    ))(input)
}

fn parse_rect<'a>(input: &'a str) -> IResult<&'a str, Command> {
    map(
        tuple((
            tag("rect "),
            nom_u32,
            tag("x"),
            nom_u32,
            newline
        )),
        |(_, x, _, y, _)| Command::Rect{x, y}
    )(input)
}

fn parse_rotate<'a>(input: &'a str) -> IResult<&'a str, Command> {
    map(
        tuple((
            tag("rotate "),
            parse_direction,
            tag("="),
            nom_u32,
            tag(" by "),
            nom_u32,
            newline
        )),
        |(_, dir, _, which, _, dist, _)| Command::Rotate{dir, which, dist}
    )(input)
}

fn parse_command<'a>(input: &'a str) -> IResult<&'a str, Command> {
    alt((
        parse_rect,
        parse_rotate,
    ))(input)
}

fn parse_commands<'a>(input: &'a str) -> IResult<&'a str, Vec<Command>> {
    many0(parse_command)(input)
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
