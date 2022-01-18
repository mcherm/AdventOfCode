mod eznom;

use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use nom::sequence::tuple as nom_tuple;
use nom::bytes::complete::tag as nom_tag;
use nom::multi::many0 as nom_many0;
use nom::character::complete::newline as nom_newline;
use nom::branch::alt as nom_alt;
use nom::character::complete::alpha1 as nom_alpha1;
use nom::character::complete::u16 as nom_value;
use eznom::type_builder;



fn input() -> Result<Vec<Instruction>, io::Error> {
    let s = fs::read_to_string("input/2015/07/input.txt")?;
    match parse_instructions(&s) {
        Ok(("", instructions)) => Ok(instructions),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



type Value = u16;

#[derive(Debug, Eq, PartialEq, Clone)]
enum Input {
    Wire(String),
    Const(Value),
}


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Operation {
    And,
    Or,
    Lshift,
    Rshift,
    Not,
    Nop,
}


#[derive(Debug, Eq, PartialEq, Clone)]
struct Instruction {
    op: Operation,
    args: Vec<Input>,
    output: String,
}


impl Input {
    fn parse_wire(input: &str) -> nom::IResult<&str, Self> {
        type_builder(nom_alpha1, |s| Input::Wire(s.to_string()))(input)
    }

    fn parse_const(input: &str) -> nom::IResult<&str, Self> {
        type_builder(nom_value, |x: u16| Input::Const(x))(input)
    }

    pub fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Input::parse_wire,
            Input::parse_const,
        ))(input)
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Input::Wire(s) => write!(f, "{}", s),
            Input::Const(val) => write!(f, "{}", val),
        }
    }
}

impl Operation {
    fn parse_binary_op(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            nom_tag("AND"),
            nom_tag("OR"),
            nom_tag("LSHIFT"),
            nom_tag("RSHIFT"),
        ))(input).map(|(rest, s)| (rest, match s {
            "AND" => Operation::And,
            "OR" => Operation::Or,
            "LSHIFT" => Operation::Lshift,
            "RSHIFT" => Operation::Rshift,
            _ => panic!()
        }))
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::And => write!(f, "AND"),
            Operation::Or => write!(f, "OR"),
            Operation::Lshift => write!(f, "LSHIFT"),
            Operation::Rshift => write!(f, "RSHIFT"),
            Operation::Not => write!(f, "NOT"),
            Operation::Nop => write!(f, "NOP"),
        }
    }
}

fn parse_wire_id(input: &str) -> nom::IResult<&str, String> {
    nom_alpha1(input).map(|(rest, s)| (rest, s.to_string()))
}


impl Instruction {
    fn parse_nop(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            Input::parse,
            nom_tag(" -> "),
            parse_wire_id,
            nom_newline,
        ))(input).map(|(rest, (arg, _, output, _))| (rest, Instruction{op: Operation::Nop, args: vec![arg], output}))
    }

    fn parse_not(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("NOT "),
            Input::parse,
            nom_tag(" -> "),
            parse_wire_id,
            nom_newline,
        ))(input).map(|(rest, (_, arg, _, output, _))| (rest, Instruction{op: Operation::Not, args: vec![arg], output}))
    }

    fn parse_binary(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            Input::parse,
            nom_tag(" "),
            Operation::parse_binary_op,
            nom_tag(" "),
            Input::parse,
            nom_tag(" -> "),
            parse_wire_id,
            nom_newline,
        ))(input).map(|(rest, (arg1, _, op, _, arg2, _, output, _))| (rest, Instruction{op, args: vec![arg1, arg2], output}))
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Instruction::parse_nop,
            Instruction::parse_not,
            Instruction::parse_binary,
        ))(input)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.op {
            Operation::Nop => {
                assert_eq!(1, self.args.len());
                write!(f, "{} -> {}", self.args[0], self.output)
            },
            Operation::Not => {
                assert_eq!(1, self.args.len());
                write!(f, "{} {} -> {}", self.op, self.args[0], self.output)
            },
            _ => {
                assert_eq!(2, self.args.len());
                write!(f, "{} {} {} -> {}", self.args[0], self.op, self.args[1], self.output)
            },
        }
    }
}

fn parse_instructions(input: &str) -> nom::IResult<&str, Vec<Instruction>> {
    nom_many0(Instruction::parse)(input)
}



fn part_a(instructions: &Vec<Instruction>) -> Result<(), io::Error> {
    for instruction in instructions.iter() {
        println!("{:?}", instruction);
    }
    Ok(())
}

fn part_b(_instructions: &Vec<Instruction>) -> Result<(), io::Error> {
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
