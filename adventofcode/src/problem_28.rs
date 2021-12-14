use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;
use itertools::Itertools;


/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    BlankFile,
    NoInsertionRules,
    InvalidInsertionRule,
    MappingMissing(char,char),
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl From<std::num::ParseIntError> for InputError {
    fn from(error: std::num::ParseIntError) -> Self {
        InputError::BadInt(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err) => write!(f, "{}", err),
            InputError::BadInt(err) => write!(f, "{}", err),
            InputError::BlankFile => write!(f, "File is blank."),
            InputError::NoInsertionRules => write!(f, "No insertion rules."),
            InputError::InvalidInsertionRule => write!(f, "Invalid insertion rule."),
            InputError::MappingMissing(leading,trailing) => write!(f, "No mapping for {} followed by {}.", leading, trailing),
        }
    }
}


/// Read in the input file.
fn read_polymer_file() -> Result<(Vec<char>, HashMap<(char,char),char>), InputError> {
    let filename = "data/2021/day/14/input.txt";
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    let template: Vec<char> = lines.next().ok_or(InputError::BlankFile)??.chars().collect();
    let empty_line: String = lines.next().ok_or(InputError::NoInsertionRules)??;
    if empty_line.len() > 0 {
        return Err(InputError::NoInsertionRules);
    }

    fn read_insertion_rule(text: String) -> Result<(char,char,char),InputError> {
        lazy_static! {
            static ref INSERTION_RULE_REGEX: Regex = Regex::new(
                r"^([A-Z])([A-Z]) -> ([A-Z])$"
            ).unwrap();
        }
        let captures = INSERTION_RULE_REGEX.captures(&text).ok_or(InputError::InvalidInsertionRule)?;

        fn get_char_field(captures: &Captures, group: usize) -> char {
            captures.get(group).unwrap().as_str().chars().next().unwrap()
        }

        Ok((
            get_char_field(&captures, 1),
            get_char_field(&captures, 2),
            get_char_field(&captures, 3)
        ))
    }

    let mut insertion_rules: HashMap<(char,char),char> = HashMap::new();
    while let Some(line) = lines.next() {
        let (leading, trailing, inserted) = read_insertion_rule(line?)?;
        insertion_rules.insert((leading,trailing), inserted);
    }
    if insertion_rules.len() == 0 {
        return Err(InputError::NoInsertionRules);
    }

    Ok((template, insertion_rules))
}



fn apply_rules(
    input: &Vec<char>,
    insertion_rules: &HashMap<(char,char), char>
) -> Result<Vec<char>,InputError> {
    if input.len() == 0 {
        return Ok(Vec::new());
    }
    let mut result: Vec<char> = Vec::new();
    let mut char_iter = input.iter();
    let mut leading: &char = char_iter.next().unwrap();
    result.push(*leading);
    while let Some(trailing) = char_iter.next() {
        let inserted: &char = insertion_rules.get(&(*leading,*trailing))
            .ok_or(InputError::MappingMissing(*leading, *trailing))?;
        result.push(*inserted);
        result.push(*trailing);
        leading = trailing;
    }
    Ok(result)
}


fn score(polymer: &Vec<char>) -> usize {
    let mut frequencies: HashMap<char,usize> = HashMap::new();
    for c in polymer {
        let new_val = match frequencies.get(c) {
            None => 1,
            Some(count) => count + 1,
        };
        frequencies.insert(*c, new_val);
    }
    assert!(!frequencies.is_empty());
    if frequencies.len() == 1 {
        return 0;
    }
    let mut sorted_counts = frequencies.values().sorted();
    let smallest: &usize = sorted_counts.next().unwrap();
    let biggest: &usize = sorted_counts.last().unwrap();
    biggest - smallest
}


fn run() -> Result<(),InputError> {
    let (mut polymer, insertion_rules) = read_polymer_file()?;
    for _i in 0..40 {
        polymer = apply_rules(&polymer, &insertion_rules)?;
    }
    let score = score(&polymer);
    println!("score: {}",score);
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
