use std::fmt::{Display, Formatter};
use anyhow;
use itertools::Itertools;


// ======= Constants =======

const PRINT: bool = false;


// ======= Parsing =======

#[derive(Debug, Copy, Clone)]
enum Operation {
    Equals, Dash
}

#[derive(Debug, Clone)]
pub struct Step {
    s: String,
    label: String,
    operation: Operation,
    focal_length: Option<u8>,
}


impl Step {
    /// Construct a new Step.
    fn new(s: &str) -> Self {
        assert!(s.len() >= 2);
        let mut chars = s.chars();
        let label: String = chars.take_while_ref(|c| *c != '-' && *c != '=').collect();
        let operation = Operation::parse(chars.next().unwrap());
        let focal_length = match operation  {
            Operation::Equals => Some(chars.collect::<String>().parse::<u8>().expect("not a number after = sign")),
            Operation::Dash => None,
        };
        Step{s: s.to_string(), label, operation, focal_length}
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)
    }
}


type Input = Vec<Step>;



mod parse {
    use super::{Input, Operation, Step};
    use std::fs;

    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_15.txt")?;
        Ok(s.split(',').map(|s| Step::new(s)).collect())
    }

    impl Operation {
        pub fn parse(c: char) -> Self {
            match c {
                '=' => Self::Equals,
                '-' => Self::Dash,
                _ => panic!("invalid operation, '{}'", c),
            }
        }
    }

}

#[derive(Debug, Default, Clone)]
struct Boxx(Vec<Step>);

#[derive(Debug)]
struct BoxRow {
    boxes: [Boxx; 256]
}

impl Display for Boxx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(" "))
    }
}

impl Default for BoxRow {
    fn default() -> Self {
        BoxRow{boxes: std::array::from_fn(|_| Boxx::default()) }
    }
}

impl Display for BoxRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, boxx) in self.boxes.iter().enumerate() {
            if !boxx.0.is_empty() {
                writeln!(f, "Box {}: {}", i, boxx)?;
            }
        }
        Ok(())
    }
}

impl Boxx {
    /// Returns the first (and should be only) location where a label matches or None if
    /// there isn't one.
    fn first_match(&self, step: &Step) -> Option<usize> {
        for (i,st) in self.0.iter().enumerate() {
            if st.label == step.label {
                return Some(i);
            }
        }
        None
    }

    fn process_step(&mut self, step: &Step) {
        match step.operation {
            Operation::Equals => {
                match self.first_match(step) {
                    Some(i) => {
                        // Already a lens. Replace it.
                        self.0[i] = step.clone();
                    }
                    None => {
                        // not already there. Append it.
                        self.0.push(step.clone());
                    }
                }
            }
            Operation::Dash => {
                // Dash, so delete matching lens if it exists
                self.0.retain(|st| st.label != step.label);
            }
        }
    }
}

impl BoxRow {
    fn process_step(&mut self, step: &Step) {
        let idx = step.hash_label();
        assert!(idx < 256);
        let boxx = &mut self.boxes[idx as usize];
        boxx.process_step(step);
    }

    fn process_steps(&mut self, steps: &Vec<Step>) {
        for step in steps {
            self.process_step(step);
            if PRINT {
                println!("After \"{}\"", step);
                println!("{}", self);
                println!();
            }
        }
    }

    fn focusing_power(&self) -> usize {
        let mut answer = 0;
        for (box_num, boxx) in self.boxes.iter().enumerate() {
            for (lens_num, step) in boxx.0.iter().enumerate() {
                answer += (box_num + 1) * (lens_num + 1) * (step.focal_length.unwrap() as usize);
            }
        }
        answer
    }
}

// ======= Compute =======

/// Performs the "HASH" function from this problem.
fn hash_str(s: &str) -> u32 {
    let mut value: u32 = 0;
    for c in s.chars() {
        value += c as u32;
        value *= 17;
        value %= 256;
    }
    value
}

impl Step {
    /// This is NOT an actual hash function, it's what the problem calls a "hash" function.
    fn hash(&self) -> u32 {
        hash_str(&self.s)
    }

    fn hash_label(&self) -> u32 {
        hash_str(&self.label)
    }
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let sum: u32 = input.iter().map(|step| step.hash()).sum();
    println!("The sum is {}", sum);
}


fn part_b(input: &Input) {
    println!("\nPart b:");
    let mut box_row: BoxRow = Default::default();
    box_row.process_steps(input);
    let focusing_power = box_row.focusing_power();
    println!("The focusing power is {}", focusing_power);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
