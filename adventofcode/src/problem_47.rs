use std::fmt;
// use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::convert::TryFrom;
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
        match Instruction::parse_nom(&text) {
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

    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
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

impl TryFrom<&str> for Register {
    type Error = (); // FIXME: Maybe a real error type here?

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "w" => Ok(Register::W),
            "x" => Ok(Register::X),
            "y" => Ok(Register::Y),
            "z" => Ok(Register::Z),
            _ => Err(())
        }
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
    fn parse_constant_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_value(input).map(|(rest, x)| (rest, Parameter::Constant(x)))
    }

    fn parse_register_nom(input: &str) -> nom::IResult<&str, Self> {
        Register::parse_nom(input).map(|(rest, x)| (rest, Parameter::Register(x)))
    }

    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Parameter::parse_constant_nom,
            Parameter::parse_register_nom,
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
    fn parse_inp_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("inp "),
            Register::parse_nom,
        ))(input).map(|(rest, (_, reg))| (rest, Instruction::Inp(reg)))
    }
    fn parse_add_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("add "),
            Register::parse_nom,
            nom_tag(" "),
            Parameter::parse_nom,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Add(reg, param)))
    }
    fn parse_mul_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("mul "),
            Register::parse_nom,
            nom_tag(" "),
            Parameter::parse_nom,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Mul(reg, param)))
    }
    fn parse_div_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("div "),
            Register::parse_nom,
            nom_tag(" "),
            Parameter::parse_nom,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Div(reg, param)))
    }
    fn parse_mod_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("mod "),
            Register::parse_nom,
            nom_tag(" "),
            Parameter::parse_nom,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Mod(reg, param)))
    }
    fn parse_eql_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("eql "),
            Register::parse_nom,
            nom_tag(" "),
            Parameter::parse_nom,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Instruction::Eql(reg, param)))
    }

    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Instruction::parse_inp_nom,
            Instruction::parse_add_nom,
            Instruction::parse_mul_nom,
            Instruction::parse_div_nom,
            Instruction::parse_mod_nom,
            Instruction::parse_eql_nom,
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

    #[test]
    fn test_into_register() {
        let s: &str = &"w";
        assert_eq!(Ok(Register::W), s.try_into());
    }
}

