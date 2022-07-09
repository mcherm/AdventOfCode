use advent_lib::eznom;

extern crate anyhow;

use std::fmt::{Display, Formatter};
use std::fs;
use anyhow::Error;
use crate::eznom::Parseable;
use std::cmp::{min, max};


const PRINT_WORK: bool = false;

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


fn char_at(grid: &Vec<Vec<char>>, pos: &(i8, i8)) -> char {
    grid[usize::try_from(pos.1).unwrap()][usize::try_from(pos.0).unwrap()]
}


fn navigate_grid(lines: &Lines, grid: &Vec<Vec<char>>, start: (i8, i8)) -> String {
    assert!(grid.iter().all(|row| row.len() == grid.len())); // square grid
    let grid_max: i8 = i8::try_from(grid.len() - 1).unwrap();
    let mut code = String::new();
    let mut pos: (i8, i8) = start;
    for line in &lines.0 {
        for direction in &line.directions {
            let next_pos: (i8, i8) = match direction {
                Direction::L => (max(0, pos.0 - 1), pos.1),
                Direction::R => (min(grid_max, pos.0 + 1), pos.1),
                Direction::U => (pos.0, max(0, pos.1 - 1)),
                Direction::D => (pos.0, min(grid_max, pos.1 + 1)),
            };
            if char_at(grid, &next_pos) != '\0' {
                pos = next_pos;
            }
            if PRINT_WORK { println!(".....on {}", char_at(grid, &pos)); }
        }
        code.push(char_at(grid, &pos));
        if PRINT_WORK { println!("...code: {}", code); }
    }
    code
}


fn part_a(lines: &Lines) {
    println!("\nPart a:");
    let grid = vec![
        vec!['1', '2', '3'],
        vec!['4', '5', '6'],
        vec!['7', '8', '9'],
    ];
    let code = navigate_grid(lines, &grid, (1,1));
    println!("Code: {}", code);
}


fn part_b(lines: &Lines) {
    println!("\nPart b:");
    let grid = vec![
        vec!['\0', '\0', '1',  '\0', '\0'],
        vec!['\0', '2',  '3',  '4',  '\0'],
        vec!['5',  '6',  '7',  '8',  '9' ],
        vec!['\0', 'A',  'B',  'C',  '\0'],
        vec!['\0', '\0', 'D',  '\0', '\0'],
    ];
    let code = navigate_grid(lines, &grid, (0,3));
    println!("Code: {}", code);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
