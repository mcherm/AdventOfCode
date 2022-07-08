mod eznom;

extern crate anyhow;

use std::fmt::{Display, Formatter};
use std::fs;
use anyhow::Error;
use crate::eznom::Parseable;
use std::cmp::{min, max};


fn input() -> Result<Lines, Error> {
    let s = fs::read_to_string("input/2016/input_02.txt")?;
    match Lines::parse(&s) {
        Ok(("", instructions)) => Ok(instructions),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


#[derive(Debug)]
enum Direction { L, R, U, D }

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Direction::L => "L",
            Direction::R => "R",
            Direction::U => "U",
            Direction::D => "D",
        })
    }
}

impl Parseable<String> for Direction {
    fn recognize(input: &str) -> eznom::Result<String> {
        eznom::alt((
            eznom::fixed("L"),
            eznom::fixed("R"),
            eznom::fixed("U"),
            eznom::fixed("D"),
        ))(input)
    }

    fn build(turn: String) -> Self {
        match turn.as_str() {
            "L" => Direction::L,
            "R" => Direction::R,
            "U" => Direction::U,
            "D" => Direction::D,
            _ => unreachable!(),
        }
    }
}



#[derive(Debug)]
struct Line {
    directions: Vec<Direction>,
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for direction in &self.directions {
            write!(f, "{direction}")?;
        }
        write!(f, "\n")
    }
}

impl Parseable<(Vec<Direction>, char)> for Line {
    fn recognize(input: &str) -> nom::IResult<&str, (Vec<Direction>, char)> {
        eznom::tuple((
            eznom::many0(Direction::parse),
            eznom::newline,
        ))(input)
    }

    fn build((directions, _): (Vec<Direction>, char)) -> Self {
        Line{directions}
    }
}



struct Lines(Vec<Line>);

impl Display for Lines {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for line in &self.0 {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl Parseable<Vec<Line>> for Lines {
    fn recognize(input: &str) -> nom::IResult<&str, Vec<Line>> {
        eznom::many0(Line::parse)(input)
    }

    fn build(lines: Vec<Line>) -> Self {
        Self(lines)
    }
}


fn part_a(data: &Lines) {
    println!("\nPart a:");
    let mut code = String::new();
    let mut x_pos: i8 = 1;
    let mut y_pos: i8 = 1;
    let grid = [
        ['1', '2', '3'],
        ['4', '5', '6'],
        ['7', '8', '9'],
    ];
    for line in &data.0 {
        for direction in &line.directions {
            match direction {
                Direction::L => x_pos = max(0, x_pos - 1),
                Direction::R => x_pos = min(2, x_pos + 1),
                Direction::U => y_pos = max(0, y_pos - 1),
                Direction::D => y_pos = min(2, y_pos + 1),
            }
            println!(".....on {}", grid[y_pos as usize][x_pos as usize]);
        }
        code.push(grid[y_pos as usize][x_pos as usize]);
        println!("...code: {}", code);
    }
    println!("Code: {}", code);
}

fn part_b(_data: &Lines) {
    println!("\nPart b:");
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
