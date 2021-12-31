use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt::{Display, Formatter};
use nom::bytes::complete::tag as nom_tag;
use nom::sequence::tuple as nom_tuple;
use nom::branch::alt as nom_alt;
use nom::character::complete::i32 as nom_value;


// ======== Reading Input ========

/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    InvalidInstruction,
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err) => write!(f, "{}", err),
            InputError::InvalidInstruction => write!(f, "Invalid Instruction"),
        }
    }
}

/// Read in the input file.
fn read_alu_file() -> Result<Vec<Instruction>, InputError> {
    // --- open file ---
    let filename = "data/2021/day/24/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    // --- read instructions ---
    let mut instructions: Vec<Instruction> = Vec::new();
    for line in lines {
        let text: String = line?;
        match Instruction::parse(&text) {
            Ok(("", instruction)) => instructions.push(instruction), // the parse was OK
            Ok((_, _)) => return Err(InputError::InvalidInstruction), // if extra stuff on the line
            Err(_) => return Err(InputError::InvalidInstruction), // if parse failed
        };
    }

    // --- return result ---
    Ok(instructions)
}



// ======== Types ========

type Value = i32;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Register {
    W, X, Y, Z
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Parameter {
    Constant(Value),
    Register(Register),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Instruction {
    Inp(Register),
    Add(Register, Parameter),
    Mul(Register, Parameter),
    Div(Register, Parameter),
    Mod(Register, Parameter),
    Eql(Register, Parameter),
}


// ======== Implementations ========

impl Register {

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            nom_tag("w"),
            nom_tag("x"),
            nom_tag("y"),
            nom_tag("z"),
        ))(input).map(|(rest, res)| (rest, match res {
            "w" => Register::W,
            "x" => Register::X,
            "y" => Register::Y,
            "z" => Register::Z,
            _ => panic!("should never happen")
        }))
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Register::W => "w",
            Register::X => "x",
            Register::Y => "y",
            Register::Z => "z",
        })
    }
}



impl Parameter {
    fn parse_constant(input: &str) -> nom::IResult<&str, Self> {
        nom_value(input).map(|(rest, x)| (rest, Parameter::Constant(x)))
    }

    fn parse_register(input: &str) -> nom::IResult<&str, Self> {
        Register::parse(input).map(|(rest, x)| (rest, Parameter::Register(x)))
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Parameter::parse_constant,
            Parameter::parse_register,
        ))(input)
    }
}
impl Display for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Parameter::Constant(val) => write!(f, "{}", val),
            Parameter::Register(reg) => write!(f, "{}", reg),
        }
    }
}


impl Instruction {
    fn parse_inp(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("inp "),
            Register::parse,
        ))(input).map(|(rest, (_, reg))| (rest, Instruction::Inp(reg)))
    }
    fn parse_add(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("add "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Add(reg, param)))
    }
    fn parse_mul(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("mul "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Mul(reg, param)))
    }
    fn parse_div(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("div "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Div(reg, param)))
    }
    fn parse_mod(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("mod "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Mod(reg, param)))
    }
    fn parse_eql(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("eql "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Eql(reg, param)))
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Instruction::parse_inp,
            Instruction::parse_add,
            Instruction::parse_mul,
            Instruction::parse_div,
            Instruction::parse_mod,
            Instruction::parse_eql,
        ))(input)
    }

}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Inp(reg) => write!(f, "inp {}", reg),
            Instruction::Add(reg, param) => write!(f, "add {} {}", reg, param),
            Instruction::Mul(reg, param) => write!(f, "mul {} {}", reg, param),
            Instruction::Div(reg, param) => write!(f, "div {} {}", reg, param),
            Instruction::Mod(reg, param) => write!(f, "mod {} {}", reg, param),
            Instruction::Eql(reg, param) => write!(f, "eql {} {}", reg, param),
        }
    }
}


// ======== Functions ========



// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let instructions: Vec<Instruction> = read_alu_file()?;
    for instruction in instructions {
        println!("{}", instruction);
    }

    Ok(())
}


pub fn main() {
    match run() {
        Ok(()) => {
            println!("Done");
        },
        Err(err) => println!("Error: {}", err),
    }
}

// ======== Tests ========

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file() {
        let _ = read_alu_file().unwrap();
    }

}

