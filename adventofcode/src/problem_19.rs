use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err)   => write!(f, "{}", err),
        }
    }
}


/// Read in the input file.
fn read_chunk_file() -> Result<Vec<String>, InputError> {
    let filename = "data/2021/day/10/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    let mut chunk_lines: Vec<String> = Vec::new();
    for line in lines {
        let text = line?;
        chunk_lines.push(text);
    }

    return Ok(chunk_lines);
}


/// An error that we can encounter when processing chunk lines.
enum ChunkError {
    ExtraCloseBracket(char), // there was an extra close bracket of type c
    WrongCloseBracket(char, char), // a chunk starting with opener (first) ended with closer (second)
    UnclosedBracket(char), // there was a bracket of type c that wasn't closed
}

impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChunkError::ExtraCloseBracket(c) => write!(f, "Extra close bracket: {}", c),
            ChunkError::WrongCloseBracket(o, c) => write!(f, "Opened with {} but closed with {}.", o, c),
            ChunkError::UnclosedBracket(c) => write!(f, "Unclosed bracket: {}", c),
        }
    }
}


fn check_chunk_line(line: &str) -> Result<(),ChunkError> {
    fn expect_close(stack: &mut Vec<char>, closer: char, expected: char) -> Result<(),ChunkError> {
        match stack.pop() {
            Some(opener) => {
                if opener != expected {
                    Err(ChunkError::WrongCloseBracket(opener, closer))
                } else {
                    Ok(())
                }
            },
            None => {
                Err(ChunkError::ExtraCloseBracket(closer))
            }
        }
    }

    let mut stack: Vec<char> = Vec::new();
    for c in line.chars() {
        match c {
            '(' => stack.push(c),
            '[' => stack.push(c),
            '{' => stack.push(c),
            '<' => stack.push(c),
            ')' => expect_close(&mut stack, c, '(')?,
            ']' => expect_close(&mut stack, c, '[')?,
            '}' => expect_close(&mut stack, c, '{')?,
            '>' => expect_close(&mut stack, c, '<')?,
            _ => panic!("Invalid character {}", c),
        }
    }
    match stack.pop() {
        Some(c) => return Err(ChunkError::UnclosedBracket(c)),
        None => return Ok(()),
    }
}


pub fn main() {
    match read_chunk_file() {
        Ok(chunk_lines) => {
            let mut score = 0;
            for chunk_line in &chunk_lines {
                match check_chunk_line(chunk_line) {
                    Err(ChunkError::WrongCloseBracket(op, cl)) => {
                        println!("Expected {} but found {} instead.", op, cl);
                        score += match cl {
                            ')' => 3,
                            ']' => 57,
                            '}' => 1197,
                            '>' => 25137,
                            _ => panic!("Invalid closer {}", cl),
                        }
                    }
                    _ => {},
                }
            }
            println!("The total syntax score is {}", score);
        },
        Err(err) => println!("Error: {}", err),
    }
}
