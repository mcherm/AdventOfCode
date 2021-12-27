use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;


// ======== Reading Input ========

/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    InvalidReactorRebootLine,
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
            InputError::InvalidReactorRebootLine => write!(f, "Invalid reactor reboot line"),
        }
    }
}

/// Read in the input file.
fn read_reactor_reboot_file() -> Result<Vec<Instruction>, InputError> {
    // --- open file ---
    let filename = "data/2021/day/22/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    // --- read instructions ---
    let mut instructions: Vec<Instruction> = Vec::new();
    for line in lines {
        let text: String = line?;
        let instruction: Instruction = Instruction::parse(&text)?;
        instructions.push(instruction);
    }

    // --- return result ---
    Ok(instructions)
}


// ======== Types ========

type Coord = i32;

#[derive(Debug)]
enum PowerLevel {
    On,
    Off,
}

#[derive(Debug)]
enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug)]
struct Bounds {
    low: Coord,
    high: Coord,
}

#[derive(Debug)]
struct Cuboid {
    bounds: [Bounds; Axis::NUM_AXES],
}

#[derive(Debug)]
struct Instruction {
    power_level: PowerLevel,
    cuboid: Cuboid,
}


// ======== Implementations ========


impl PowerLevel {
    fn parse(text: &str) -> Result<Self,InputError> {
        match text {
            "on" => Ok(PowerLevel::On),
            "off" => Ok(PowerLevel::Off),
            _ => Err(InputError::InvalidReactorRebootLine)
        }
    }
}

impl Display for PowerLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            PowerLevel::On => "on",
            PowerLevel::Off => "off"
        })
    }
}

impl Axis {
    const NUM_AXES: usize = 3;

    fn index(&self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

impl Display for Axis {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        })
    }
}

impl Bounds {
    fn parse(text: &str) -> Result<Self, InputError> {
        let bounds_regex = Regex::new(
            r"^(-?\d+)\.\.(-?\d+)$"
        ).unwrap();
        let capture = bounds_regex.captures(&text).ok_or(InputError::InvalidReactorRebootLine)?;
        let low: Coord = capture.get(1).unwrap().as_str().parse()?;
        let high: Coord = capture.get(2).unwrap().as_str().parse()?;
        Ok(Bounds{low, high})
    }
}

impl Display for Bounds {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.low, self.high)
    }
}

impl Cuboid {
    fn parse(text: &str) -> Result<Self,InputError> {
        let cuboid_regex = Regex::new(
            r"^x=(.*),y=(.*),z=(.*)$"
        ).unwrap();
        let capture = cuboid_regex.captures(&text).ok_or(InputError::InvalidReactorRebootLine)?;
        let x: Bounds = Bounds::parse(capture.get(1).unwrap().as_str())?;
        let y: Bounds = Bounds::parse(capture.get(2).unwrap().as_str())?;
        let z: Bounds = Bounds::parse(capture.get(3).unwrap().as_str())?;
        let bounds = [x,y,z];
        Ok(Cuboid{bounds})
    }
}

impl Display for Cuboid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "x={},y={},z={}",
            self.bounds[Axis::X.index()],
            self.bounds[Axis::Y.index()],
            self.bounds[Axis::Z.index()]
        )
    }
}

impl Instruction {
    fn parse(text: &str) -> Result<Self,InputError> {
        let instruction_regex = Regex::new(
            r"^(.*) (.*)$"
        ).unwrap();
        let capture = instruction_regex.captures(&text).ok_or(InputError::InvalidReactorRebootLine)?;
        let power_level = PowerLevel::parse(capture.get(1).unwrap().as_str())?;
        let cuboid = Cuboid::parse(capture.get(2).unwrap().as_str())?;
        Ok(Instruction{power_level, cuboid})
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.power_level, self.cuboid)
    }
}



// ======== Functions ========


// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let instructions = read_reactor_reboot_file()?;
    println!("Instructions:");
    for instruction in instructions {
        println!("{}", instruction);
    }

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
        let _ = read_reactor_reboot_file().unwrap();
    }

}
