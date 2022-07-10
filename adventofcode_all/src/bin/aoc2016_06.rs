
extern crate anyhow;

use std::{fs, io};
use std::io::BufRead;
use anyhow::{anyhow, Error};
use itertools::Itertools;
use std::collections::HashMap;


fn input() -> Result<Vec<String>, Error> {
    let mut line_len: Option<usize> = None;
    let mut res: Vec<String> = Vec::new();
    let file = fs::File::open("input/2016/input_06.txt")?;
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        match line_len {
            None => {
                if line.len() == 0 {
                    return Err(anyhow!("First line of length zero."));
                }
                line_len = Some(line.len());
            },
            Some(len) => if len != line.len() {
                return Err(anyhow!("Lines not the same length."));
            }
        }
        res.push(line);
    }
    Ok(res)
}




fn part_a(input: &Vec<String>) {
    println!("\nPart a:");
    assert!(input.len() >= 1);
    let length = input[0].len();
    let mut output_chars: Vec<char> = Vec::with_capacity(length);
    for pos in 0..length {
        let counts: HashMap<char, usize> = input.iter().map(|x| x.chars().nth(pos).unwrap()).counts();
        let c: char = *counts.iter().max_by_key(|x| x.1).unwrap().0;
        output_chars.push(c);
    }
    let message: String = output_chars.into_iter().collect();
    println!("The message is {:?}", message);
}


fn part_b(input: &Vec<String>) {
    println!("\nPart b:");
    assert!(input.len() >= 1);
    let length = input[0].len();
    let mut output_chars: Vec<char> = Vec::with_capacity(length);
    for pos in 0..length {
        let counts: HashMap<char, usize> = input.iter().map(|x| x.chars().nth(pos).unwrap()).counts();
        let c: char = *counts.iter().min_by_key(|x| x.1).unwrap().0;
        output_chars.push(c);
    }
    let message: String = output_chars.into_iter().collect();
    println!("The message is {:?}", message);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
