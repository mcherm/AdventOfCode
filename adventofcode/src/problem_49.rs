use std::fs;
use std::fmt;
use std::fmt::{Display, Formatter};
use nom::bytes::complete::tag as nom_tag;
use nom::sequence::pair as nom_pair;
use nom::branch::alt as nom_alt;
use nom::multi::many1 as nom_many1;


// ======== Reading Input ========

/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    BadCucumber,
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err) => write!(f, "{}", err),
            InputError::BadCucumber => write!(f, "Bad cucumber"),
        }
    }
}

/// Read in the input file.
fn read_cucumber_file() -> Result<CucumberRegion, InputError> {
    // --- open file ---
    let filename = "data/2021/day/25/input.txt";
    let contents = fs::read_to_string(filename)?;

    // --- read instructions ---
    match CucumberRegion::parse(&contents) {
        Ok(("", region)) => Ok(region),
        Ok((_, _)) => return Err(InputError::BadCucumber), // if extra stuff on the line
        Err(_) => return Err(InputError::BadCucumber), // if parse failed
    }
}



// ======== Types ========

#[derive(Debug, Copy, Clone)]
enum Occupant {
    Eastward,
    Southward,
    Empty,
}

#[derive(Debug)]
struct CucumberRegion {
    data: Vec<Vec<Occupant>>,
}

// ======== Implementations ========

impl Occupant {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            nom_tag("v"),
            nom_tag(">"),
            nom_tag("."),
        ))(input).map(|(rest, res)| (rest, match res {
            ">" => Occupant::Eastward,
            "v" => Occupant::Southward,
            "." => Occupant::Empty,
            _ => panic!("should never happen")
        }))
    }
}
impl Display for Occupant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Occupant::Eastward => ">",
            Occupant::Southward => "v",
            Occupant::Empty => ".",
        })
    }
}



impl CucumberRegion {
    fn parse_line(input: &str) -> nom::IResult<&str, Vec<Occupant>> {
        nom_pair(
            nom_many1(Occupant::parse),
            nom_tag("\n"),
        )(input).map(|(rest, (line, _))| (rest, line))
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_many1(
            CucumberRegion::parse_line
        )(input).map(|(rest, data)| (rest, CucumberRegion{data}))
    }
}
impl Display for CucumberRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for line in self.data.iter() {
            for occ in line {
                write!(f, "{}", occ)?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}

// ======== Functions ========


// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let region: CucumberRegion = read_cucumber_file()?;
    println!("Region: \n{}", region);

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

// ======== Tests ========

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file() {
        let _ = read_cucumber_file().unwrap();
    }

}
