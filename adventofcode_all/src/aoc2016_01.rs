mod eznom;

extern crate anyhow;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fmt::{Display, Formatter};
use std::fs;
use itertools::Itertools;

use anyhow::Error;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use nom::branch::alt as nom_alt;
use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::i32 as nom_i32;
use nom::multi::separated_list0 as nom_separated_list0;
use nom::sequence::tuple as nom_tuple;
use eznom::type_builder;
use eznom::Parseable;


#[derive(Parser)]
#[grammar = "pest/aoc2016_01.pest"]
pub struct InputParser;


#[allow(dead_code)]
enum ParserType { Pest, Handwritten, Eznom01, Eznom02 }
const PARSER: ParserType = ParserType::Eznom02;


fn input() -> Result<Vec<Instruction>, Error> {
    let s = fs::read_to_string("input/2016/input_01.txt")?;
    match PARSER {
        ParserType::Pest => {
            let file_pairs: Pairs<_> = InputParser::parse(Rule::file, &s)?;
            println!("file_pairs: '{:?}'", file_pairs);
            for file_pair in file_pairs {
                let file_pair: Pair<_> = file_pair;
                println!("file_pair Rule:    {:?}", file_pair.as_rule());
                println!("file_pair Span:    {:?}", file_pair.as_span());
                println!("file_pair Text:    {}", file_pair.as_str());
                let file_inner_pairs: Pairs<_> = file_pair.into_inner();
                let (instructions_pair, _): (Pair<_>, _) = file_inner_pairs.collect_tuple().unwrap();
                println!("instructions_pair: {:?}", instructions_pair);
                println!("instructions_pair Rule:    {:?}", instructions_pair.as_rule());
                println!("instructions_pair Span:    {:?}", instructions_pair.as_span());
                println!("instructions_pair Text:    {}", instructions_pair.as_str());
                for instruction_pair in instructions_pair.into_inner() {
                    let instruction_pair: Pair<_> = instruction_pair;
                    println!("instruction_pair: {:?}", instruction_pair);
                    let file_instruction_pairs: Pairs<_> = instruction_pair.into_inner();
                    println!("file_instruction_pairs {:?}", file_instruction_pairs);
                    // println!("turn {:?}", turn);
                    // println!("dist {:?}", dist);
                }
            }
            Ok(Vec::new())
        },
        ParserType::Handwritten => {
            let newline_stripped = if &s[s.len() - 2..s.len() - 1] == "\n" {&s[0..s.len() - 1]} else {&s};
            let pieces = newline_stripped.split(", ");
            let mut answer = Vec::new();
            for piece in pieces {
                let turn: TurnDirection = match &piece[0..1] {
                    "L" => TurnDirection::L,
                    "R" => TurnDirection::R,
                    _ => panic!("Invalid turn direction"),
                };
                let dist: i32 = piece[1..piece.len()].parse::<i32>().unwrap();
                let instruction = Instruction{turn, dist};
                answer.push(instruction);
            }
            Ok(answer)
        },
        ParserType::Eznom01 => {
            match parse_input_01(&s) {
                Ok(("", instructions)) => Ok(instructions),
                Ok((s, _)) => panic!("Extra input starting at {}", s),
                Err(_) => panic!("Invalid input"),
            }
        },
        ParserType::Eznom02 => {
            match Instructions::parse(&s) {
                Ok(("", Instructions(instructions))) => Ok(instructions),
                Ok((s, _)) => panic!("Extra input starting at {}", s),
                Err(_) => panic!("Invalid input"),
            }
        },
    }
}


fn parse_instruction_01(input: &str) -> nom::IResult<&str, Instruction> {
    let recognize = |s| nom_tuple((
        nom_alt((
            nom_tag("L"),
            nom_tag("R"),
        )),
        nom_i32,
    ))(s);
    let build = |(turn, num): (&str, i32)| Instruction{
        turn: match turn {
            "L" => TurnDirection::L,
            "R" => TurnDirection::R,
            _ => unreachable ! (),
        },
        dist: num,
    };
    type_builder(recognize, build)(input)
}

fn parse_input_01(input: &str) -> nom::IResult<&str, Vec<Instruction>> {
    nom_separated_list0(nom_tag(", "), parse_instruction_01)(input)
}




#[allow(dead_code)]
const DATA: &'static str = r#"[
    Pair {
        rule: file,
        span: Span {str: "L, L, R\n", start: 0, end: 8 },
        inner: [
            Pair {
                rule: instructions,
                span: Span { str: "L, L, R", start: 0, end: 7 },
                inner: [
                    Pair {
                        rule: instruction,
                        span: Span { str: "L", start: 0, end: 1 },
                        inner: []
                    },
                    Pair {
                        rule: instruction,
                        span: Span { str: "L", start: 3, end: 4 },
                        inner: []
                    },
                    Pair {
                        rule: instruction,
                        span: Span { str: "R", start: 6, end: 7 },
                        inner: []
                    }
                ]
            },
            Pair {
                rule: EOI,
                span: Span { str: "", start: 8, end: 8 },
                inner: []
            }
        ]
    }
]
"#;


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


fn part_a(data: &Vec<Instruction>) {
    let mut x_pos: i32 = 0;
    let mut y_pos: i32 = 0;
    let mut facing: i8 = 0;
    for instruction in data {
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

fn part_b(_data: &Vec<Instruction>) {
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
