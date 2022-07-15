
extern crate anyhow;

use std::fs;
use anyhow::Error;
use regex::Regex;
use lazy_static::lazy_static;


fn input() -> Result<String, Error> {
    let s = fs::read_to_string("input/2016/input_09.txt")?;
    Ok(s)
}



/// Given an input, this decompresses it and returns the result.
fn decompress(data: &str) -> String {
    lazy_static! {
        static ref MARKER_RE: Regex = Regex::new("\\(([1-9][0-9]*)x([1-9][0-9]*)\\)").unwrap();
    }
    let source: Vec<char> = data.chars().collect();
    let mut answer: Vec<char> = Vec::with_capacity(source.len());
    let mut pos = 0; // position in source
    for captures in MARKER_RE.captures_iter(data) {
        let match_pos = captures.get(0).unwrap().start();
        println!("{}: {:?}", match_pos, captures);
        if match_pos >= pos {
            // --- copy over everything before the match ---
            answer.extend_from_slice( &source[pos..match_pos] );
            pos = captures.get(0).unwrap().end();

            // --- parse the numbers ---
            let repeat_len = captures.get(1).unwrap().as_str().parse::<usize>().unwrap();
            let repeat_times = captures.get(2).unwrap().as_str().parse::<usize>().unwrap();

            // --- get the slice to be copied ---
            let repeated = &source[pos..(pos + repeat_len)];
            for _ in 0..repeat_times {
                answer.extend_from_slice(repeated);
            }
            pos += repeat_len;
        }
    }
    // -- copy anything remaining ---
    answer.extend_from_slice( &source[pos..source.len()] );
    answer.iter().collect()
}


fn part_a(data: &String) {
    println!("\nPart a:");
    let expanded = decompress(data);
    // println!("expanded = '{}'", expanded);
    println!("The length is {}", expanded.len());
}


fn part_b(_data: &String) {
    println!("\nPart b:");
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
