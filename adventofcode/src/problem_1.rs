use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt::Formatter;

#[derive(Debug)]
pub enum PossibleError {
    Io(std::io::Error),
    ParseInt(std::num::ParseIntError),
}

impl From<std::io::Error> for PossibleError {
    fn from(e: std::io::Error) -> PossibleError {
        PossibleError::Io(e)
    }
}

impl From<std::num::ParseIntError> for PossibleError {
    fn from(e: std::num::ParseIntError) -> PossibleError {
        PossibleError::ParseInt(e)
    }
}

impl std::fmt::Display for PossibleError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PossibleError::Io(err) => write!(f, "Error Message {}", err),
            PossibleError::ParseInt(err) => write!(f, "Error Message {}", err),
        }
    }
}



fn count_increases_in_file() -> Result<i32, PossibleError>  {
    let filename = "data/2021/day/1/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();
    let mut count: i32 = 0;
    let mut previous: i32 = i32::MAX;
    for line in lines {
        let text = line?;
        let value: i32 = text.parse()?;
        if value > previous {
            count += 1;
        }
        previous = value;
    }
    return Ok(count);
}

pub fn main() {
    match count_increases_in_file() {
        Ok(value) => println!("Result is {}", value),
        Err(err) => println!("Error: {}", err),
    }
}
