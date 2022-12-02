
extern crate anyhow;

use std::fs;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::newline,
    combinator::{value, map},
    multi::many0,
    sequence::{terminated, tuple},
};



fn input() -> Result<Vec<Round>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_02.txt")?;
    match Round::parse_list(&s) {
        Ok(("", rounds)) => Ok(rounds),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



#[derive(Debug, Copy, Clone)]
enum TheirMove {
    A, B, C
}

#[derive(Debug, Copy, Clone)]
enum MyMove {
    X, Y, Z
}

#[derive(Debug, Copy, Clone)]
struct Round {
    their_move: TheirMove,
    my_move: MyMove,
}


impl TheirMove {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(Self::A, tag("A")),
            value(Self::B, tag("B")),
            value(Self::C, tag("C")),
        ))(input)
    }
}

impl MyMove {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(Self::X, tag("X")),
            value(Self::Y, tag("Y")),
            value(Self::Z, tag("Z")),
        ))(input)
    }
}

impl Round {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((TheirMove::parse, tag(" "), MyMove::parse)),
            |(their_move, _, my_move)| Round{their_move, my_move}
        )(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }

    /// Returns the score for this round, using rules from part 1.
    fn score_part_1(&self) -> u32 {
        let selection_score = match self.my_move {
            MyMove::X => 1,
            MyMove::Y => 2,
            MyMove::Z => 3,
        };
        let victory_score = match self {
            Round{their_move: TheirMove::A, my_move: MyMove::X} => 3,
            Round{their_move: TheirMove::A, my_move: MyMove::Y} => 6,
            Round{their_move: TheirMove::A, my_move: MyMove::Z} => 0,
            Round{their_move: TheirMove::B, my_move: MyMove::X} => 0,
            Round{their_move: TheirMove::B, my_move: MyMove::Y} => 3,
            Round{their_move: TheirMove::B, my_move: MyMove::Z} => 6,
            Round{their_move: TheirMove::C, my_move: MyMove::X} => 6,
            Round{their_move: TheirMove::C, my_move: MyMove::Y} => 0,
            Round{their_move: TheirMove::C, my_move: MyMove::Z} => 3,
        };
        selection_score + victory_score
    }

    /// Returns the score for this round, using rules from part 2.
    fn score_part_2(&self) -> u32 {
        let selection_score = match self {
            Round{their_move: TheirMove::A, my_move: MyMove::X} => 3,
            Round{their_move: TheirMove::A, my_move: MyMove::Y} => 1,
            Round{their_move: TheirMove::A, my_move: MyMove::Z} => 2,
            Round{their_move: TheirMove::B, my_move: MyMove::X} => 1,
            Round{their_move: TheirMove::B, my_move: MyMove::Y} => 2,
            Round{their_move: TheirMove::B, my_move: MyMove::Z} => 3,
            Round{their_move: TheirMove::C, my_move: MyMove::X} => 2,
            Round{their_move: TheirMove::C, my_move: MyMove::Y} => 3,
            Round{their_move: TheirMove::C, my_move: MyMove::Z} => 1,
        };
        let victory_score = match self.my_move {
            MyMove::X => 0,
            MyMove::Y => 3,
            MyMove::Z => 6,
        };
        selection_score + victory_score
    }
}





fn part_a(input: &Vec<Round>) {
    println!("\nPart a:");
    let total_score: u32 = input.iter().map(|x| x.score_part_1()).sum();
    println!("My total score is {}.", total_score);
}


fn part_b(input: &Vec<Round>) {
    println!("\nPart b:");
    let total_score: u32 = input.iter().map(|x| x.score_part_2()).sum();
    println!("My total score is {}.", total_score);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
