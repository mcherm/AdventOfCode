use advent_lib::eznom;

extern crate anyhow;

use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs;
use anyhow::Error;
use crate::eznom::Parseable;



fn input() -> Result<Instructions, Error> {
    let s = fs::read_to_string("input/2016/input_01.txt")?;
    match Instructions::parse(&s) {
        Ok(("", instructions)) => Ok(instructions),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


#[derive(Debug)]
enum TurnDirection { L, R }

impl Display for TurnDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self { TurnDirection::L => "L", TurnDirection::R => "R"})
    }
}

impl Parseable<String> for TurnDirection {
    fn recognize(input: &str) -> eznom::Result<String> { //nom::IResult<&str, String> {
        eznom::alt((
            eznom::fixed("L"),
            eznom::fixed("R"),
        ))(input)
    }

    fn build(turn: String) -> Self {
        match turn.as_str() {
            "L" => TurnDirection::L,
            "R" => TurnDirection::R,
            _ => unreachable!(),
        }
    }
}



#[derive(Debug)]
struct Instruction {
    turn: TurnDirection,
    dist: i32,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.turn, self.dist)
    }
}

impl Parseable<(TurnDirection, i32)> for Instruction {
    fn recognize(input: &str) -> nom::IResult<&str, (TurnDirection, i32)> {
        eznom::tuple((
            TurnDirection::parse,
            eznom::parse_i32,
        ))(input)
    }

    fn build((turn, dist): (TurnDirection, i32)) -> Self {
        Instruction{turn, dist}
    }
}



struct Instructions(Vec<Instruction>);

impl Parseable<Vec<Instruction>> for Instructions {
    fn recognize(input: &str) -> nom::IResult<&str, Vec<Instruction>> {
        eznom::separated_list0(eznom::fixed(", "), Instruction::parse)(input)
    }

    fn build(instructions: Vec<Instruction>) -> Self {
        Self(instructions)
    }
}


fn part_a(data: &Instructions) {
    println!("\nPart a:");
    let mut x_pos: i32 = 0;
    let mut y_pos: i32 = 0;
    let mut facing: i8 = 0;
    for instruction in data.0.iter() {
        match instruction.turn {
            TurnDirection::L => facing = (facing + 3) % 4,
            TurnDirection::R => facing = (facing + 1) % 4,
        }
        match facing {
            0 => y_pos += instruction.dist,
            1 => x_pos += instruction.dist,
            2 => y_pos -= instruction.dist,
            3 => x_pos -= instruction.dist,
            _ => panic!("Bad facing ({})", facing),
        }
        println!("At {},{} facing {}", x_pos, y_pos, facing);
    }

    let taxi_dist = x_pos.abs() + y_pos.abs();
    println!("Taxi dist = {}", taxi_dist);
}

fn part_b(data: &Instructions) {
    println!("\nPart b:");
    let mut x_pos: i32 = 0;
    let mut y_pos: i32 = 0;
    let mut facing: i8 = 0;
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    visited.insert((x_pos, y_pos));
    for instruction in data.0.iter() {
        match instruction.turn {
            TurnDirection::L => facing = (facing + 3) % 4,
            TurnDirection::R => facing = (facing + 1) % 4,
        }
        for _ in 1..=instruction.dist {
            match facing {
                0 => y_pos += 1,
                1 => x_pos += 1,
                2 => y_pos -= 1,
                3 => x_pos -= 1,
                _ => panic!("Bad facing ({})", facing),
            }
            if visited.contains(&(x_pos, y_pos)) {
                let taxi_dist = x_pos.abs() + y_pos.abs();
                println!("Taxi dist = {}", taxi_dist);
                return
            }
            visited.insert((x_pos, y_pos));
        }
    }
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
