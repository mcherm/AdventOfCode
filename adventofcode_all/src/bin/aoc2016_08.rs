
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
    sequence::tuple
};
use nom::character::complete::u32 as nom_u32;


const SCREEN_WIDTH: usize = 50;
const SCREEN_HEIGHT: usize = 6;


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

#[derive(Debug)]
struct Screen {
    data: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]
}

impl Screen {
    fn new() -> Self {
        Self{data: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT]}
    }

    fn get(&self, x: Value, y: Value) -> bool {
        let xx = usize::try_from(x % Value::try_from(SCREEN_WIDTH).unwrap()).unwrap();
        let yy = usize::try_from(y % Value::try_from(SCREEN_HEIGHT).unwrap()).unwrap();
        self.data[yy][xx]
    }

    fn set(&mut self, x: Value, y: Value, v: bool) {
        let xx = usize::try_from(x % Value::try_from(SCREEN_WIDTH).unwrap()).unwrap();
        let yy = usize::try_from(y % Value::try_from(SCREEN_HEIGHT).unwrap()).unwrap();
        self.data[yy][xx] = v;
    }

    fn rect_on(&mut self, x: Value, y: Value) {
        for yv in 0..y {
            for xv in 0..x {
                self.set(xv, yv, true);
            }
        }
    }

    fn rotate(&mut self, dir: Direction, which: Value, dist: Value) {
        match dir {
            Direction::Row => {
                let mut old_row: [bool; SCREEN_WIDTH] = [false; SCREEN_WIDTH];
                for x in 0..SCREEN_WIDTH {
                    old_row[x] = self.get(Value::try_from(x).unwrap(), which);
                }
                for x in 0..SCREEN_WIDTH {
                    self.set(Value::try_from(x).unwrap() + dist, which, old_row[x]);
                }
            },
            Direction::Column => {
                let mut old_col: [bool; SCREEN_HEIGHT] = [false; SCREEN_HEIGHT];
                for y in 0..SCREEN_HEIGHT {
                    old_col[y] = self.get(which, Value::try_from(y).unwrap());
                }
                for y in 0..SCREEN_HEIGHT {
                    self.set(which, Value::try_from(y).unwrap() + dist, old_col[y]);
                }
            },
        }
    }

    fn perform(&mut self, command: &Command) {
        match command {
            Command::Rect{x,y} => self.rect_on(*x,*y),
            Command::Rotate{dir, which, dist} => self.rotate(*dir, *which, *dist),
        }
    }

    fn pixel_count(&self) -> usize {
        let mut sum = 0;
        for y in 0..Value::try_from(SCREEN_HEIGHT).unwrap() {
            for x in 0..Value::try_from(SCREEN_WIDTH).unwrap() {
                if self.get(x,y) {
                    sum += 1;
                }
            }
        }
        sum
    }
}

impl Display for Screen {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..Value::try_from(SCREEN_HEIGHT).unwrap() {
            for x in 0..Value::try_from(SCREEN_WIDTH).unwrap() {
                write!(f, "{}", if self.get(x,y) {"###"} else {" . "})?
            }
            writeln!(f, "")?
        }
        Ok(())
    }
}


fn part_a(commands: &Vec<Command>) {
    println!("\nPart a:");
    let mut screen = Screen::new();
    for command in commands {
        screen.perform(command);
    }
    println!("There are a total of {} pixels lit.", screen.pixel_count())
}


fn part_b(commands: &Vec<Command>) {
    println!("\nPart b:");
    let mut screen = Screen::new();
    for command in commands {
        screen.perform(command);
    }
    println!("{}", screen);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
