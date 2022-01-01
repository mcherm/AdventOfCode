use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt::{Display, Formatter};
use std::collections::{HashMap, HashSet};
use itertools;
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
    NoStartingInputInstruction,
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
            InputError::NoStartingInputInstruction => write!(f, "No starting input instruction"),
        }
    }
}

/// Read in the input file.
fn read_alu_file() -> Result<Vec<Segment>, InputError> {
    // --- open file ---
    let filename = "data/2021/day/24/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    // --- read instructions ---
    let mut segments: Vec<Segment> = Vec::new();
    let mut input_register: Option<Register> = None;
    let mut computes: Vec<Compute> = Vec::new();
    for line in lines {
        let text: String = line?;
        match Instruction::parse(&text) {
            Ok(("", instruction)) => {  // the parse was OK
                match instruction {
                    Instruction::Input(reg) => {
                        // -- Start a new segment --
                        if let Some(input) = input_register {
                            segments.push(Segment{input, computes: computes.clone()});
                        }
                        input_register = Some(reg);
                        computes.clear();
                    }
                    Instruction::Compute(compute) => {
                        if input_register.is_none() {
                            return Err(InputError::NoStartingInputInstruction);
                        }
                        computes.push(compute)
                    }
                }
            },
            Ok((_, _)) => return Err(InputError::InvalidInstruction), // if extra stuff on the line
            Err(_) => return Err(InputError::InvalidInstruction), // if parse failed
        };
    }
    match input_register {
        None => return Err(InputError::NoStartingInputInstruction),
        Some(input) => segments.push(Segment{input, computes: computes.clone()}),
    }

    // --- return result ---
    Ok(segments)
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

/// One segment of instructions consists of one Input instruction followed
/// by a series of Compute instructions.
#[derive(Debug, Clone)]
struct Segment {
    input: Register,
    computes: Vec<Compute>,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Alu {
    values: [Value; Register::NUM_ITEMS],
}


/// Cache for ONE particular segment.
struct SegmentCache {
    segment: Segment,
    cache: HashMap<(Alu, Value), Result<Alu,()>>, // map from (start_alu, input_value) to output Alu
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


impl Segment {
    /// Applies the segment. Either returns Ok(alu) for the Alu that results OR
    /// returns Err(()) if the computation would result in an illegal operation.
    fn apply(&self, start_alu: Alu, input: Value) -> Result<Alu,()> {
        let mut alu = start_alu;
        alu = alu.eval_input(self.input, input);
        for compute in self.computes.iter() {
            alu = alu.eval_compute(*compute)?;
        }
        Ok(alu)
    }
}


impl Alu {
    /// Given a register, tells the value stored in that register.
    fn value_in(&self, reg: Register) -> Value {
        self.values[reg.id()]
    }

    /// Given a param, tells the value of that parameter.
    fn value_of(&self, param: Parameter) -> Value {
        match param {
            Parameter::Constant(val) => val,
            Parameter::Register(reg) => self.value_in(reg),
        }
    }


    /// Returns true if the ALU is a valid final accept state.
    fn valid(&self) -> bool {
        self.values[Register::Z.id()] == 0
    }

    /// Executes any instruction OTHER than input. Either returns the Alu
    /// that results OR Err(()) if the computation hit an invalid snag.
    fn eval_compute(&self, compute: Compute) -> Result<Alu, ()> {
        let mut values: [Value; Register::NUM_ITEMS] = self.values.clone();
        match compute {
            Compute::Add(reg, param) => {
                values[reg.id()] = self.value_in(reg) + self.value_of(param);
            },
            Compute::Mul(reg, param) => {
                values[reg.id()] = self.value_in(reg) * self.value_of(param);
            },
            Compute::Div(reg, param) => {
                let p = self.value_of(param);
                if p == 0 {
                    return Err(());
                }
                values[reg.id()] = self.value_in(reg) / p;
            },
            Compute::Mod(reg, param) => {
                let r = self.value_in(reg);
                let p = self.value_of(param);
                if r < 0 || p <= 0 {
                    return Err(());
                }
                values[reg.id()] = r % p;
            },
            Compute::Eql(reg, param) => {
                values[reg.id()] = if self.value_in(reg) == self.value_of(param) {1} else {0};
            },
        }
        Ok(Alu{values})
    }

    fn eval_input(&self, input_reg: Register, input: Value) -> Alu {
        let mut values: [Value; Register::NUM_ITEMS] = self.values.clone();
        values[input_reg.id()] = input;
        Alu{values}
    }
}

impl Display for Alu {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {} {} {}]", self.values[0], self.values[1], self.values[2], self.values[3])
    }
}


impl SegmentCache {
    fn new(segment: Segment) -> Self {
        SegmentCache{segment, cache: HashMap::new()}
    }

    fn apply_segment(&mut self, start_alu: Alu, input: Value) -> Result<Alu,()> {
        match self.cache.get(&(start_alu, input)) {
            Some(alu) => *alu,
            None => {
                let answer: Result<Alu,()> = self.segment.apply(start_alu, input);
                self.cache.insert((start_alu, input), answer);
                answer
            },
        }
    }
}

// ======== Functions ========

fn prepend(v: Value, vals: Vec<Value>) -> Vec<Value> {
    itertools::chain((&[v]).iter().copied(), vals.iter().copied()).collect::<Vec<Value>>()
}


/// caches: the vector of SegmentCaches
/// pos: the position of that vector we are evaluating
/// start_alu: the starting Alu
///
/// This evaluates possible inputs for a segment. It returns a list of
/// input value sequences that will give valid results.
fn evaluate(caches: &mut Vec<SegmentCache>, pos: usize, start_alu: Alu) -> Vec<Vec<Value>> {
    let mut answer: Vec<Vec<Value>> = Vec::new();
    for input in (1..=9).rev() {
        let apply_result = caches[pos].apply_segment(start_alu, input);
        match apply_result {
            Err(()) => {}, // that failed... move on
            Ok(alu) => { // found an output
                if pos + 1 == caches.len() {
                    // -- last one; check for validity --
                    if alu.valid() {
                        answer.push(vec![input]);
                    }
                } else {
                    // -- not last one; recurse --
                    for tail in evaluate(caches, pos + 1, alu) {
                        let tail_end_of_number = prepend(input, tail);
                        answer.push(tail_end_of_number);
                    }
                }
            },
        }
    }
    answer
}


// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let segments: Vec<Segment> = read_alu_file()?;

    // let last_seg: Segment = segments[segments.len() - 1].clone();
    // let last_2_seg: Segment = segments[segments.len() - 2].clone();
    // let mut last_cache = SegmentCache::new(last_seg);
    // let mut last_2_cache = SegmentCache::new(last_2_seg);
    // let mut valid_pairs: HashSet<[Value;2]> = HashSet::new();


    let mut caches: Vec<SegmentCache> = segments.iter().map(|x| SegmentCache::new(x.clone())).collect();
    let min_val = 0;
    let max_val = 0;
    let mut valid_paths: HashSet<[Value;2]> = HashSet::new();
    for a in min_val..=max_val {
        for b in min_val..=max_val {
            for c in min_val..=max_val {
                for d in min_val..=max_val {
                    let start_alu = Alu{values: [a, b, c, d]};
                    let start_pos = caches.len() - 2;
                    let paths = evaluate(&mut caches, start_pos, start_alu);
                    for path in paths.iter() {
                        let path_as_array = [path[0],path[1]];
                        valid_paths.insert(path_as_array);
                    }
                }
            }
        }
    }
    println!();
    println!("The valid paths are:");
    for path in valid_paths {
        println!("{}{}", path[0], path[1]);
    }

    // println!();
    // println!("---------------");
    // println!("Just checking...");
    // {
    //     let alu = Alu{values: [0,0,0,0]};
    //     let mut pentultimate: SegmentCache = SegmentCache::new(segments[0].clone());
    //     let mut ultimate: SegmentCache = SegmentCache::new(segments[1].clone());
    //     println!("alu = {}", alu);
    //     let alu = pentultimate.apply_segment(alu, 2).unwrap();
    //     println!("input 7 and then alu = {}", alu);
    //     let alu = ultimate.apply_segment(alu, 9).unwrap();
    //     println!("input 9 and then alu = {}", alu);
    // }

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

/*
NOTES:
  For the last 2 digits, I tried all combinations from -10 to +20
  The ONLY input values that passed the checks were
    79
    35
    24
    13
    46
    57
    68
  Interestingly, all of those work with a starting value of [0,0,0,0].

 */