mod eznom;

use std::fmt::{Debug};
use std::fs;
use std::io;

use nom::branch::alt as nom_alt;
use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::i32 as nom_num;
use nom::character::complete::newline as nom_newline;
use nom::multi::many0 as nom_many0;
use nom::sequence::tuple as nom_tuple;
use eznom::type_builder;



#[derive(Debug)]
enum Error {
    Io(io::Error),
}
impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Error::Io(e) } }



fn input() -> Result<Program, Error> {
    let s = fs::read_to_string("input/2015/23/input.txt")?;
    match Program::parse(&s) {
        Ok(("", boss)) => Ok(boss),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Offset = i32;

#[derive(Debug)]
enum Register {
    A,
    B,
}

impl Register {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_alt((
            nom_tag("a"),
            nom_tag("b"),
        ))(s);
        let build = |s| match s {
            "a" => Register::A,
            "b" => Register::B,
            _ => panic!()
        };
        type_builder(recognize, build)(input)
    }
}


#[derive(Debug)]
enum Instruction {
    Hlf(Register),
    Tpl(Register),
    Inc(Register),
    Jmp(Offset),
    Jie(Register, Offset),
    Jio(Register, Offset),
}

impl Instruction {
    fn parse_hlf(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("hlf "),
            Register::parse,
            nom_newline,
        ))(s);
        let build = |(_, register, _): (&str, Register, char)|
            Instruction::Hlf(register);
        type_builder(recognize, build)(input)
    }

    fn parse_tpl(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("tpl "),
            Register::parse,
            nom_newline,
        ))(s);
        let build = |(_, register, _): (&str, Register, char)|
            Instruction::Tpl(register);
        type_builder(recognize, build)(input)
    }

    fn parse_inc(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("inc "),
            Register::parse,
            nom_newline,
        ))(s);
        let build = |(_, register, _): (&str, Register, char)|
            Instruction::Inc(register);
        type_builder(recognize, build)(input)
    }

    fn parse_offset(input: &str) -> nom::IResult<&str, Offset> {
        let recognize = |s| nom_tuple((
            nom_alt((
                nom_tag("+"),
                nom_tag("-"),
            )),
            nom_num,
        ))(s);
        let build = |(sign, num): (&str, i32)| {
            match sign {
                "+" => num,
                "-" => -num,
                _ => panic!(),
            }
        };
        type_builder(recognize, build)(input)
    }

    fn parse_jmp(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("jmp "),
            Instruction::parse_offset,
            nom_newline,
        ))(s);
        let build = |(_, offset, _): (&str, Offset, char)|
            Instruction::Jmp(offset);
        type_builder(recognize, build)(input)
    }

    fn parse_jie(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("jie "),
            Register::parse,
            nom_tag(", "),
            Instruction::parse_offset,
            nom_newline,
        ))(s);
        let build = |(_, register, _, offset, _): (&str, Register, &str, Offset, char)|
            Instruction::Jie(register, offset);
        type_builder(recognize, build)(input)
    }

    fn parse_jio(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("jio "),
            Register::parse,
            nom_tag(", "),
            Instruction::parse_offset,
            nom_newline,
        ))(s);
        let build = |(_, register, _, offset, _): (&str, Register, &str, Offset, char)|
            Instruction::Jio(register, offset);
        type_builder(recognize, build)(input)
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_alt((
            Instruction::parse_hlf,
            Instruction::parse_tpl,
            Instruction::parse_inc,
            Instruction::parse_jmp,
            Instruction::parse_jie,
            Instruction::parse_jio,
        ))(s);
        let build = |ins: Instruction| ins;
        type_builder(recognize, build)(input)
    }

    pub fn parse_list(input: &str) -> nom::IResult<&str, Vec<Self>> {
        nom_many0(Self::parse)(input)
    }
}


#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| Instruction::parse_list(s);
        let build = |instructions: Vec<Instruction>| Self{instructions};
        type_builder(recognize, build)(input)
    }
}


fn part_a(program: &Program) {
    println!("---- Part A ----");
    println!("{:?}", program);
}


fn part_b(_program: &Program) {
    println!("---- Part B ----");
}

fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
