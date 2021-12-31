use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt::{Display, Formatter};
use nom::bytes::complete::tag as nom_tag;
use nom::sequence::tuple as nom_tuple;
use nom::branch::alt as nom_alt;
use nom::character::complete::i64 as nom_value;


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

type Value = i64;

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
enum Compute {
    Add(Register, Parameter),
    Mul(Register, Parameter),
    Div(Register, Parameter),
    Mod(Register, Parameter),
    Eql(Register, Parameter),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Instruction {
    Input(Register),
    Compute(Compute),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Alu {
    values: [Value; Register::NUM_ITEMS],
}


// ======== Implementations ========

impl Register {
    const NUM_ITEMS: usize = 4;

    fn id(&self) -> usize {
        match self {
            Register::W => 0,
            Register::X => 1,
            Register::Y => 2,
            Register::Z => 3,
        }
    }

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

impl Compute {
    fn parse_add(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("add "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Add(reg, param)))
    }
    fn parse_mul(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("mul "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Mul(reg, param)))
    }
    fn parse_div(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("div "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Div(reg, param)))
    }
    fn parse_mod(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("mod "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Mod(reg, param)))
    }
    fn parse_eql(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("eql "),
            Register::parse,
            nom_tag(" "),
            Parameter::parse,
        ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Eql(reg, param)))
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Compute::parse_add,
            Compute::parse_mul,
            Compute::parse_div,
            Compute::parse_mod,
            Compute::parse_eql,
        ))(input)
    }
}

impl Display for Compute {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Compute::Add(reg, param) => write!(f, "add {} {}", reg, param),
            Compute::Mul(reg, param) => write!(f, "mul {} {}", reg, param),
            Compute::Div(reg, param) => write!(f, "div {} {}", reg, param),
            Compute::Mod(reg, param) => write!(f, "mod {} {}", reg, param),
            Compute::Eql(reg, param) => write!(f, "eql {} {}", reg, param),
        }
    }
}


impl Instruction {
    fn parse_inp(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("inp "),
            Register::parse,
        ))(input).map(|(rest, (_, reg))| (rest, Instruction::Input(reg)))
    }
    fn parse_compute(input: &str) -> nom::IResult<&str, Self> {
        Compute::parse(input).map(|(rest, ci)| (rest, Instruction::Compute(ci)))
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            Instruction::parse_inp,
            Instruction::parse_compute,
        ))(input)
    }

}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Input(reg) => write!(f, "inp {}", reg),
            Instruction::Compute(ci) => write!(f, "{}", ci),
        }
    }
}


impl Alu {
    /// Given a register, tells the value stored in that register.
    fn value_in(&self, reg: Register) -> Value {
        self.values[reg.id()]
    }

    // Given a param, tells the value of that parameter.
    fn value_of(&self, param: Parameter) -> Value {
        match param {
            Parameter::Constant(val) => val,
            Parameter::Register(reg) => self.value_in(reg),
        }
    }


    /// Executes any instruction OTHER than input.
    fn eval_internal(&self, instruction: Instruction) -> Alu {
        let mut values: [Value; Register::NUM_ITEMS] = self.values.clone();
        match instruction {
            Instruction::Input(_) => panic!(),
            Instruction::Compute(Compute::Add(reg, param)) => {
                values[reg.id()] = self.value_in(reg) + self.value_of(param);
            },
            Instruction::Compute(Compute::Mul(reg, param)) => {
                values[reg.id()] = self.value_in(reg) * self.value_of(param);
            },
            Instruction::Compute(Compute::Div(reg, param)) => {
                let p = self.value_of(param);
                assert!(p != 0);
                values[reg.id()] = self.value_in(reg) / p;
            },
            Instruction::Compute(Compute::Mod(reg, param)) => {
                let r = self.value_in(reg);
                let p = self.value_of(param);
                assert!(r >= 0);
                assert!(p > 0);
                values[reg.id()] = r % p;
            },
            Instruction::Compute(Compute::Eql(reg, param)) => {
                values[reg.id()] = if self.value_in(reg) == self.value_of(param) {1} else {0};
            },
        }
        Alu{values}
    }
}

impl Display for Alu {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {} {} {}]", self.values[0], self.values[1], self.values[2], self.values[3])
    }
}

// ======== Functions ========



// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let instructions: Vec<Instruction> = read_alu_file()?;

    let mut alu = Alu{values: [0;Register::NUM_ITEMS]};
    for ins in instructions.iter() {
        if !matches!(ins, Instruction::Input(_)) {
            alu = alu.eval_internal(*ins);
            println!("alu: {}", alu);
        }
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

