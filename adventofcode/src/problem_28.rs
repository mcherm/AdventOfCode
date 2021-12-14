use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::collections::hash_map;
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


#[derive(Debug)]
struct PairCounts {
    counts: HashMap<(char,char),usize>,
}

impl PairCounts {
    fn new() -> Self {
        PairCounts{counts: HashMap::new()}
    }

    fn get_count(&self, pair: &(char,char)) -> usize {
        match self.counts.get(pair) {
            None => 0,
            Some(old) => *old,
        }
    }

    fn add_counts(&mut self, pair: (char,char), count: usize) {
        self.counts.insert(pair, self.get_count(&pair) + count);
    }
}

impl<'a> IntoIterator for &'a PairCounts {
    type Item = (&'a (char,char), &'a usize);
    type IntoIter = hash_map::Iter<'a,(char,char),usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.counts.iter()
    }
}

impl fmt::Display for PairCounts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pairs<")?;
        for stuff in self {
            write!(f, "{:?}", stuff)?;
        }
        write!(f, ">")
    }
}



fn apply_rules(
    old_pair_counts: &PairCounts,
    insertion_rules: &HashMap<(char,char), char>
) -> Result<PairCounts,InputError> {
    let mut new_pair_counts: PairCounts = PairCounts::new();
    for ((leading,trailing), count) in old_pair_counts {
        let inserted: &char = insertion_rules.get(&(*leading,*trailing))
            .ok_or(InputError::MappingMissing(*leading, *trailing))?;
        new_pair_counts.add_counts((*leading, *inserted), *count);
        new_pair_counts.add_counts((*inserted, *trailing), *count);
    }
    Ok(new_pair_counts)
}



fn score(template: &Vec<char>, pair_counts: &PairCounts) -> usize {
    let mut letter_counts: HashMap<char,usize> = HashMap::new();
    letter_counts.insert(template[0], 1); // count the first item in the string
    for ((_, trailing), count) in pair_counts {
        // for each pair-count, add to the count for the trailing character
        let old_count = *letter_counts.get(trailing).unwrap_or(&0);
        letter_counts.insert(*trailing, old_count + count);
    }

    let mut sorted_counts = letter_counts.values().sorted();
    let smallest = sorted_counts.next().unwrap(); // Safe as there is at least one letter
    let biggest = sorted_counts.last().unwrap_or(smallest); // if there are none left, then biggest==smallest
    biggest - smallest
}


fn run() -> Result<(),InputError> {
    let (template, insertion_rules) = read_polymer_file()?;
    assert!(template.len() > 1);

    // --- initalize pair_counts ---
    let mut pair_counts: PairCounts = PairCounts::new();
    let mut char_iter = template.iter();
    let mut leading: &char = char_iter.next().unwrap();
    while let Some(trailing) = char_iter.next() {
        pair_counts.add_counts((*leading, *trailing), 1);
        leading = trailing;
    }

    // --- apply rules ---
    for _i in 0..40 {
        pair_counts = apply_rules(&pair_counts, &insertion_rules)?;
    }
    let score = score(&template, &pair_counts);
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
