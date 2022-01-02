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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Occupant {
    Eastward,
    Southward,
    Empty,
}

#[derive(Debug, Eq, PartialEq)]
struct CucumberRegion {
    data: Vec<Vec<Occupant>>,
    height: usize,
    width: usize,
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
    fn new(data: Vec<Vec<Occupant>>) -> Self {
        let height = data.len();
        assert!(height >= 2);
        let width = data[0].len();
        assert!(data.iter().all(|x| x.len() == width));
        CucumberRegion{data, height, width}
    }


    /// Returns the occupant at location (x,y). Will wrap around if
    /// x or y is larger than the region.
    fn occupant(&self, x: usize, y: usize) -> Occupant {
        self.data[y % self.height][x % self.width]
    }

    /// Used to update an occupant.
    fn set_occupant(&mut self, x: usize, y: usize, value: Occupant) {
        self.data[y % self.height][x % self.width] = value;
    }

    /// Performs one step of motion. Returns true if anything changed; false if not.
    fn perform_step(&mut self) -> bool {
        let mut anything_moved = false;
        // -- Decide who will move east --
        let will_move: Vec<Vec<bool>> = (0..self.height).map(|y| {
            (0..self.width).map(|x| {
                self.occupant(x,y) == Occupant::Eastward && self.occupant(x+1,y) == Occupant::Empty
            }).collect()
        }).collect();
        // -- Move them --
        for y in 0..self.height {
            for x in 0..self.width {
                if will_move[y][x] {
                    self.set_occupant(x, y, Occupant::Empty);
                    self.set_occupant(x+1, y, Occupant::Eastward);
                    anything_moved = true;
                }
            }
        }
        // -- Decide who will move south --
        let will_move: Vec<Vec<bool>> = (0..self.height).map(|y| {
            (0..self.width).map(|x| {
                self.occupant(x,y) == Occupant::Southward && self.occupant(x,y+1) == Occupant::Empty
            }).collect()
        }).collect();
        // -- Move them --
        for y in 0..self.height {
            for x in 0..self.width {
                if will_move[y][x] {
                    self.set_occupant(x, y, Occupant::Empty);
                    self.set_occupant(x, y+1, Occupant::Southward);
                    anything_moved = true;
                }
            }
        }
        // -- Return result --
        anything_moved
    }

    /// This takes steps repeatedly until nothing changes. It returns the first step on
    /// which no cucumbers moved (one more than the count of the steps where things moved).
    fn run_to_completion(&mut self) -> usize {
        let mut count: usize = 0;
        while self.perform_step() {
            count += 1;
        }
        count + 1
    }

    /// Parse one line of the input
    fn parse_line(input: &str) -> nom::IResult<&str, Vec<Occupant>> {
        nom_pair(
            nom_many1(Occupant::parse),
            nom_tag("\n"),
        )(input).map(|(rest, (line, _))| (rest, line))
    }

    /// Parse the entire input
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_many1(
            CucumberRegion::parse_line
        )(input).map(|(rest, data)| (rest, CucumberRegion::new(data)))
    }
}
impl Display for CucumberRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{}", self.occupant(x,y))?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}

// ======== Functions ========


// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let mut region: CucumberRegion = read_cucumber_file()?;
    let steps = region.run_to_completion();
    println!("Region: \n{}", region);
    println!();
    println!("After just {} steps.", steps);

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

    #[test]
    fn test_single_step() {
        let (_, mut region_1) = CucumberRegion::parse("\
            ...>...\n\
            .......\n\
            ......>\n\
            v.....>\n\
            ......>\n\
            .......\n\
            ..vvv..\n\
        ").unwrap();
        let (_, region_2) = CucumberRegion::parse("\
            ..vv>..\n\
            .......\n\
            >......\n\
            v.....>\n\
            >......\n\
            .......\n\
            ....v..\n\
        ").unwrap();
        region_1.perform_step();
        assert_eq!(region_2, region_1);
    }
}
