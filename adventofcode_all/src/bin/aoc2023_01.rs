
use anyhow;


// ======= Parsing =======


mod parse {
    use std::fs;

    pub fn input() -> Result<Vec<String>, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_01.txt")?;
        Ok(s.lines().map(|s| s.to_string()).collect())
    }

}


// ======= Compute =======


fn find_digit(chars: impl Iterator<Item = char>) -> u32 {
    chars
        .filter(|c| c.is_digit(10))
        .map(|c| c.to_digit(10).unwrap() as u32)
        .next()
        .unwrap()
}

fn first_digit(s: &str) -> u32 {
    find_digit(s.chars())
}

fn last_digit(s: &str) -> u32 {
    find_digit(s.chars().rev())
}

fn part_a_sum(line: &str) -> u32 {
    first_digit(line) * 10 + last_digit(line)
}

static NUMBER_MAP: [(&'static str, u32); 20] = [
    ("0", 0),
    ("1", 1),
    ("2", 2),
    ("3", 3),
    ("4", 4),
    ("5", 5),
    ("6", 6),
    ("7", 7),
    ("8", 8),
    ("9", 9),
    ("zero", 0),
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
];

/// Returns the number, either digit or spelled out, starting at the beginning of this
/// str OR returns None if there isn't one.
fn recognize_number(s: &str) -> Option<u32> {
    for (name, val) in NUMBER_MAP {
        if s.starts_with(name) {
            return Some(val);
        }
    }
    None
}

/// Given the REVERSE version of a string, it looks for a reversed spelled-out-or-digit
/// number at the start and returns that if it exists.
fn recognize_number_backwards(s: &str) -> Option<u32> {
    for (name, val) in NUMBER_MAP {
        if s.starts_with(&reverse(name)) {
            return Some(val)
        }
    }
    None
}

fn first_number(s: &str) -> u32 {
    for pos in 0..s.len() {
        let substring = &s[pos..];
        if let Some(x) = recognize_number(substring) {
            return x;
        }
    }
    panic!("No number found")
}

/// Returns the reverse of a string slice as a string
fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

fn last_number(s: &str) -> u32 {
    let rev: String = reverse(s);
    for pos in 0..rev.len() {
        let substring = &rev[pos..];
        if let Some(x) = recognize_number_backwards(substring) {
            return x;
        }
    }
    panic!("No number found")
}

fn part_b_sum(line: &str) -> u32 {
    first_number(line) * 10 + last_number(line)
}


// ======= main() =======


fn part_a(data: &Vec<String>) {
    println!("\nPart a:");
    let sum: u32 = data.iter()
        .map(|x| x.as_str())
        .map(part_a_sum)
        .sum();
    println!("The sum is {}", sum);
}


fn part_b(data: &Vec<String>) {
    println!("\nPart b:");
    let sum: u32 = data.iter()
        .map(|x| x.as_str())
        .map(part_b_sum)
        .sum();
    println!("The sum is {}", sum);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
