use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};


/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
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
        }
    }
}



/// Read in the input file.
fn read_beacon_file() -> Result<Vec<String>, InputError> {
    let filename = "data/2021/day/19/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    let mut output: Vec<String> = Vec::new();
    for line in lines {
        let text: String = line?;
        output.push(text);
    }
    Ok(output)
}



fn run() -> Result<(),InputError> {
    let data = read_beacon_file()?;

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
    use super::*;

    #[test]
    fn test_read_file() {
        let _ = read_beacon_file();
    }
}
