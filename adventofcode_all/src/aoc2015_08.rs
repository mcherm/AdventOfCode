
use std::fs;
use std::io;



fn input() -> Result<String, io::Error> {
    fs::read_to_string("input/2015/08/input.txt")
}




fn part_a(s: &str) -> Result<(), io::Error> {
    let mut count: u32 = 0;
    let mut chars = s.chars();
    match chars.next() {
        Some('"') => count += 2, // for the very first quote, and the final end quote we'll eventually get to
        _ => panic!("file does not begin with a quote"),
    }
    'read_string: loop {
        match chars.next() { // in a string
            None => panic!("file ends inside a string"),
            Some('\n') => panic!("whitespace inside a string"),
            Some('"') => {
                'read_outside_string: loop {
                    match chars.next() { // outside a string
                        None => break 'read_string,
                        Some('\n') => {}, // ignore whitespace
                        Some('"') => {
                            count += 2; // end and begin quote added 2 extra characters
                            break 'read_outside_string
                        },
                        Some(_) => panic!("non-whitespace outside a string"),
                    }
                }
            },
            Some('\\') => {
                match chars.next() { // in an escape
                    None => panic!("file ends in the middle of an escape"),
                    Some('\\') => count += 1, // escaped backslash was an extra character
                    Some('"') => count += 1, // escaped quote was an extra character
                    Some('x') => {
                        match (chars.next(), chars.next()) {
                            (Some(_), Some(_)) => count += 3, // hex escape was an extra 3 characters
                            _ => panic!("file ends in the middle of an escape"),
                        }
                    },
                    Some(_) => panic!("invalid escape"),
                }
            },
            Some(_) => {}, // normal characters don't make a difference
        }
    }
    println!("The string literals take up {} more characters than the string in memory.", count);
    Ok(())
}

fn part_b(s: &str) -> Result<(), io::Error> {
    let mut count: u32 = 0;
    let mut chars = s.chars();
    match chars.next() {
        Some('"') => count += 4, // for the very first quote, and the final end quote we'll eventually get to
        _ => panic!("file does not begin with a quote"),
    }
    'read_string: loop {
        match chars.next() { // in a string
            None => panic!("file ends inside a string"),
            Some('\n') => panic!("whitespace inside a string"),
            Some('"') => {
                'read_outside_string: loop {
                    match chars.next() { // outside a string
                        None => break 'read_string,
                        Some('\n') => {}, // ignore whitespace
                        Some('"') => {
                            count += 4; // end and begin quote added 2 extra characters
                            break 'read_outside_string
                        },
                        Some(_) => panic!("non-whitespace outside a string"),
                    }
                }
            },
            Some('\\') => {
                match chars.next() { // in an escape
                    None => panic!("file ends in the middle of an escape"),
                    Some('\\') => count += 2, // escaped backslash was an extra character
                    Some('"') => count += 2, // escaped quote was an extra character
                    Some('x') => {
                        match (chars.next(), chars.next()) {
                            (Some(_), Some(_)) => count += 1, // hex escape was just one extra character
                            _ => panic!("file ends in the middle of an escape"),
                        }
                    },
                    Some(_) => panic!("invalid escape"),
                }
            },
            Some(_) => {}, // normal characters don't make a difference
        }
    }
    println!("The escaped string literals take up {} more characters than the original.", count);
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
