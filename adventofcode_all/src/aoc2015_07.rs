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



fn input() -> Result<Vec<Instruction>, io::Error> {
    let s = fs::read_to_string("input/2015/07/input.txt")?;
    match parse_instructions(&s) {
        Ok(("", instructions)) => Ok(instructions),
        Ok((_, _)) => panic!("Extra input"),
        Err(_) => panic!("Invalid input"),
    }
}


#[derive(Debug, Eq, PartialEq, Clone)]
struct WireId {
    name: String
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum Input {
    Wire(WireId),
    Const(Value),
}

type Value = u16;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum BinaryOperation {
    And,
    Or,
    Lshift,
    Rshift,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum UnaryOperation {
    Not,
    Nop,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct BinaryInstruction {
    output: WireId,
    arg1: Input,
    arg2: Input,
    op: BinaryOperation,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct UnaryInstruction {
    output: WireId,
    arg: Input,
    op: UnaryOperation,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum Instruction {
    Binary(BinaryInstruction),
    Unary(UnaryInstruction),
}



impl WireId {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alpha1(input).map(|(rest, s)| (rest, WireId{name: s.to_string()}))
    }
}

impl Input {
    fn parse_wire(input: &str) -> nom::IResult<&str, Self> {
        WireId::parse(input).map(|(rest, wire_id)| (rest, Input::Wire(wire_id)))
    }

    fn parse_const(input: &str) -> nom::IResult<&str, Self> {
        nom_value(input).map(|(rest, val)| (rest, Input::Const(val)))
    }

    pub fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Input::parse_wire,
            Input::parse_const,
        ))(input)
    }
}

impl BinaryOperation {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            nom_tag("AND"),
            nom_tag("OR"),
            nom_tag("LSHIFT"),
            nom_tag("RSHIFT"),
        ))(input).map(|(rest, s)| (rest, match s {
            "AND" => BinaryOperation::And,
            "OR" => BinaryOperation::Or,
            "LSHIFT" => BinaryOperation::Lshift,
            "RSHIFT" => BinaryOperation::Rshift,
            _ => panic!()
        }))
    }
}

impl Display for BinaryOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperation::And => write!(f, "AND"),
            BinaryOperation::Or => write!(f, "OR"),
            BinaryOperation::Lshift => write!(f, "LSHIFT"),
            BinaryOperation::Rshift => write!(f, "RSHIFT"),
        }
    }
}

impl UnaryOperation {
    fn parse_not(input: &str) -> nom::IResult<&str, Self> {
        nom_tag("NOT")(input).map(|(rest, _)| (rest, UnaryOperation::Not))
    }
}


impl UnaryInstruction {
    fn parse_not(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            Input::parse,
            nom_tag(" -> "),
            WireId::parse,
            nom_newline,
        ))(input).map(|(rest, (arg, _, output, _))| (rest, UnaryInstruction{output, arg, op: UnaryOperation::Nop}))
    }

    fn parse_nop(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            UnaryOperation::parse_not,
            nom_tag(" "),
            Input::parse,
            nom_tag(" -> "),
            WireId::parse,
            nom_newline,
        ))(input).map(|(rest, (op, _, arg, _, output, _))| (rest, UnaryInstruction{output, arg, op}))
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            UnaryInstruction::parse_not,
            UnaryInstruction::parse_nop,
        ))(input)
    }
}

impl BinaryInstruction {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            Input::parse,
            nom_tag(" "),
            BinaryOperation::parse,
            nom_tag(" "),
            Input::parse,
            nom_tag(" -> "),
            WireId::parse,
            nom_newline,
        ))(input).map(|(rest, (arg1, _, op, _, arg2, _, output, _))| (rest, BinaryInstruction{output, arg1, arg2, op}))
    }
}


impl Instruction {
    fn parse_binary(input: &str) -> nom::IResult<&str, Self> {
        BinaryInstruction::parse(input).map(|(rest, bi)| (rest, Instruction::Binary(bi)))
    }

    fn parse_unary(input: &str) -> nom::IResult<&str, Self> {
        UnaryInstruction::parse(input).map(|(rest, ui)| (rest, Instruction::Unary(ui)))
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Instruction::parse_binary,
            Instruction::parse_unary,
        ))(input)
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
