use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};



/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    NoData,
    MultipleLines,
    InvalidCharacter,
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
            InputError::NoData => write!(f, "No data."),
            InputError::MultipleLines => write!(f, "Multiple lines."),
            InputError::InvalidCharacter => write!(f, "Invalid character."),
        }
    }
}


/// Read in the input file.
fn read_packet_file() -> Result<Vec<char>, InputError> {
    let filename = "data/2021/day/16/input.txt";
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    let first_line: String = lines.next().ok_or(InputError::NoData)??;
    if lines.next().is_some() {
        return Err(InputError::MultipleLines);
    }

    let mut result = Vec::new();
    for char in first_line.chars() {
        match char {
            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'A'|'B'|'C'|'D'|'E'|'F' => result.push(char),
            _ => return Err(InputError::InvalidCharacter),
        }
    }
    Ok(result)
}



fn run() -> Result<(),InputError> {
    let hex_chars: Vec<char> = read_packet_file()?;
    println!("Read: {:?}", hex_chars);
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



#[cfg(test)]
mod test {
}
