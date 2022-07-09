use advent_lib::eznom;

use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::collections::HashMap;
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

/// Represents a fully wired system.
struct WiredSystem {
    connections: HashMap<String, Instruction>,
    cache: HashMap<String, Option<Value>>,
    overrides: HashMap<String, Value>,
}



impl Input {
    pub fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            type_builder(nom_alpha1, |s| Input::Wire(s.to_string())),
            type_builder(nom_value, |x| Input::Const(x)),
        ))(input)
    }

    pub fn eval(&self, system: &mut WiredSystem) -> Option<Value>{
        match self {
            Input::Const(value) => Some(*value),
            Input::Wire(wire_id) => system.eval(wire_id),
        }
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
        type_builder(
            |input| nom_alt((
                nom_tag("AND"),
                nom_tag("OR"),
                nom_tag("LSHIFT"),
                nom_tag("RSHIFT"),
            ))(input),
            |s| match s {
                "AND" => Operation::And,
                "OR" => Operation::Or,
                "LSHIFT" => Operation::Lshift,
                "RSHIFT" => Operation::Rshift,
                _ => panic!()
            }
        )(input)
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
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize_nop = |s| nom_tuple((
            Input::parse,
            nom_tag(" -> "),
            parse_wire_id,
            nom_newline,
        ))(s);
        let recognize_not = |s| nom_tuple((
            nom_tag("NOT "),
            Input::parse,
            nom_tag(" -> "),
            parse_wire_id,
            nom_newline,
        ))(s);
        let recognize_binary_op = |s| nom_tuple((
            Input::parse,
            nom_tag(" "),
            Operation::parse_binary_op,
            nom_tag(" "),
            Input::parse,
            nom_tag(" -> "),
            parse_wire_id,
            nom_newline,
        ))(s);

        let build_nop = |(arg, _, output, _)| Instruction{op: Operation::Nop, args: vec![arg], output};
        let build_not = |(_, arg, _, output, _)| Instruction{op: Operation::Not, args: vec![arg], output};
        let build_binary_op = |(arg1, _, op, _, arg2, _, output, _)| Instruction{op, args: vec![arg1, arg2], output};

        nom_alt((
            type_builder(recognize_nop, build_nop),
            type_builder(recognize_not, build_not),
            type_builder(recognize_binary_op, build_binary_op),
        ))(input)
    }

    /// Find the output from this instruction; system is used to provide the
    /// inputs. Returns None if the value can't be evaluated due to some missing
    /// input.
    fn eval(&self, system: &mut WiredSystem) -> Option<Value> {
        match self.op {
            Operation::And => {
                assert!(self.args.len() == 2);
                let arg0: Value = self.args[0].eval(system)?;
                let arg1: Value = self.args[1].eval(system)?;
                Some(arg0 & arg1)
            },
            Operation::Or => {
                assert!(self.args.len() == 2);
                let arg0: Value = self.args[0].eval(system)?;
                let arg1: Value = self.args[1].eval(system)?;
                Some(arg0 | arg1)
            },
            Operation::Lshift => {
                assert!(self.args.len() == 2);
                let arg0: Value = self.args[0].eval(system)?;
                let arg1: Value = self.args[1].eval(system)?;
                Some(arg0 << arg1)
            },
            Operation::Rshift => {
                assert!(self.args.len() == 2);
                let arg0: Value = self.args[0].eval(system)?;
                let arg1: Value = self.args[1].eval(system)?;
                Some(arg0 >> arg1)
            },
            Operation::Not => {
                assert!(self.args.len() == 1);
                let arg0: Value = self.args[0].eval(system)?;
                Some(!arg0)
            },
            Operation::Nop => {
                assert!(self.args.len() == 1);
                let arg0: Value = self.args[0].eval(system)?;
                Some(arg0)
            },
        }
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

impl WiredSystem {
    fn new(instructions: &Vec<Instruction>) -> Self {
        let mut connections = HashMap::new();
        for instruction in instructions {
            connections.insert(instruction.output.clone(), instruction.clone());
        }
        let cache = HashMap::new();
        let overrides = HashMap::new();
        WiredSystem{connections, cache, overrides}
    }

    fn eval(&mut self, wire_id: &str) -> Option<Value> {
        match self.overrides.get(wire_id) {
            Some(v) => Some(*v),
            None => match self.cache.get(wire_id) {
                Some(v) => *v,
                None => {
                    // have to clone the instruction since it's borrowed from the WiredSystem
                    let answer = match self.connections.get(wire_id).cloned() {
                        None => None,
                        Some(instruction) => instruction.eval(self),
                    };
                    self.cache.insert(wire_id.to_owned(), answer);
                    answer
                },
            },
        }
    }

    /// Replaces the given wire_id with the given value, regardless of what
    /// value it used to have. This also clears the cache since who knows
    /// what other changes might have been made
    fn override_value(&mut self, wire_id: &str, value: Value) {
        self.overrides.insert(wire_id.to_owned(), value);
        self.cache.clear();
    }
}



fn part_a(instructions: &Vec<Instruction>) -> Result<(), io::Error> {
    let mut system = WiredSystem::new(instructions);
    let wire_id = "a";
    match system.eval(&wire_id) {
        Some(v) => println!("{} = {}", wire_id, v),
        None => println!("{} is invalid", wire_id),
    }
    Ok(())
}

fn part_b(instructions: &Vec<Instruction>) -> Result<(), io::Error> {
    let mut system = WiredSystem::new(instructions);
    let wire_id = "a";
    let second_wire_id = "b";
    let orig_a_value = system.eval(wire_id).unwrap();
    system.override_value(second_wire_id, orig_a_value);
    match system.eval(wire_id) {
        Some(v) => println!("Plugging {} back in for {}, {} = {}", orig_a_value, second_wire_id, wire_id, v),
        None => println!("Plugging {} back in for {}, {} is invalid", orig_a_value, second_wire_id, wire_id),
    }
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
