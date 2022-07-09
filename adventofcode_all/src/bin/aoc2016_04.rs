
extern crate anyhow;

use lazy_static::lazy_static;
use std;
use std::{fs, io};
use std::cmp::Reverse;
use std::fmt::{Display, Formatter};
use std::io::BufRead;
use anyhow::Error;
use regex::Regex;
use itertools::Itertools;



fn input() -> Result<Vec<String>, Error> {
    let mut res: Vec<String> = Vec::new();
    let file = fs::File::open("input/2016/input_04.txt")?;
    for line in io::BufReader::new(file).lines() {
        res.push(line?);
    }
    Ok(res)
}


#[derive(Debug, Clone)]
struct InputError {
    msg: String,
}

impl InputError {
    fn new(msg: &str) -> Self {
        Self{msg: msg.to_string()}
    }
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for InputError {
}


fn get_checksum(letters: &str) -> String {
    let counts = letters.chars().filter(|x| *x != '-').counts();
    counts.iter().sorted_by_key(|x| (Reverse(x.1), x.0)).map(|x| x.0).take(5).collect()
}


enum Action { PartA, PartB }

fn shared_code(lines: &Vec<String>, action: Action) -> Result<(), Error> {
    let mut sector_sum = 0;
    for line in lines {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r#"^([-a-z]+)-([0-9]+)\[([a-z]{5})]$"#).unwrap();
        }
        let caps = REGEX.captures(line).ok_or(InputError::new("Invalid line"))?; // FIXME: An error might be nicer
        let letters: &str = caps.get(1).unwrap().as_str();
        let sector_id: u32 = caps.get(2).unwrap().as_str().parse()?;
        let checksum: &str = caps.get(3).unwrap().as_str();
        if checksum == get_checksum(letters) {
            match action {
                Action::PartA => sector_sum += sector_id,
                Action::PartB => {
                    let decrypted = decrypt(letters, sector_id);
                    if decrypted.contains("northpole") {
                        println!("{}: {}", decrypted, sector_id);
                    }
                }
            }
        }
    }
    match action {
        Action::PartA => println!("The sum of the sector IDs of real rooms is {}", sector_sum),
        _ => {}
    }
    Ok(())
}

fn part_a(lines: &Vec<String>) -> Result<(), Error> {
    println!("\nPart a:");
    shared_code(lines, Action::PartA)
}

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";

fn decrypt(letters: &str, sector_id: u32) -> String {
    letters.chars().map(|c| {match c {
        '-' => ' ',
        c => {
            ALPHABET.chars().nth(
                (
                    ALPHABET.find(c).unwrap() + usize::try_from(sector_id).unwrap()
                ) % ALPHABET.len()
            ).unwrap()
        }
    }}).collect()
}

fn part_b(lines: &Vec<String>) -> Result<(), Error> {
    println!("\nPart b:");
    shared_code(lines, Action::PartB)
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
