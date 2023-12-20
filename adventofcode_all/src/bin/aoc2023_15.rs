use std::fmt::{Display, Formatter};
use anyhow;


// ======= Constants =======


// ======= Parsing =======


#[derive(Debug)]
pub struct Step {
    s: String,
    // label: String,
    // operation: char,
}


impl Step {
    /// Construct a new Step.
    fn new(s: &str) -> Self {
        // assert!(s.len() >= 2);
        // let label: String = s[0..(s.len() - 1)].to_string();
        // let operation: char = s[s.len() - 1..].chars().next().unwrap();
        // println!("op '{}'", operation); // FIXME: Remove
        // assert!(['=', '-'].contains(&operation));
        Step{s: s.to_string(), /*label, operation*/}
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)
    }
}


type Input = Vec<Step>;



mod parse {
    use super::{Input, Step};
    use std::fs;

    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_15.txt")?;
        Ok(s.split(',').map(|s| Step::new(s)).collect())
    }

}

// #[derive(Debug)]
// struct Box {
//
// }
//
// #[derive(Debug)]
// struct BoxRow {
//     boxes: [Box; 256]
// }


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

    // fn hash_label(&self) -> u32 {
    //     hash_str(&self.label)
    // }
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let sum: u32 = input.iter().map(|step| step.hash()).sum();
    println!("The sum is {}", sum);
}


fn part_b(_input: &Input) {
    println!("\nPart b:");
    // for step in input {
    //     println!("{} has hash {} and op {}", step, step.hash_label(), step.operation);
    // }
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
