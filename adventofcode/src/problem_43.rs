use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::i32 as nom_coord;
use nom::sequence::tuple as nom_tuple;
use nom::branch::alt as nom_alt;


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
        let (rest, instruction) = Instruction::parse_nom(&text).unwrap();
        if rest != "" {
            return Err(InputError::InvalidReactorRebootLine);
        }
        instructions.push(instruction);
    }

    // --- return result ---
    Ok(instructions)
}




// ======== Types ========

type Coord = i32;

#[derive(Debug, Eq, PartialEq, Clone)]
enum PowerLevel {
    On,
    Off,
}

#[derive(Debug)]
enum Axis {X, Y, Z}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Bounds {
    low: Coord,
    high: Coord,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Cuboid {
    m_bounds: [Bounds; Axis::NUM_AXES],
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Instruction {
    power_level: PowerLevel,
    cuboid: Cuboid,
}


// ======== Implementations ========


impl PowerLevel {
    fn parse_regex(text: &str) -> Result<Self,InputError> {
        match text {
            "on" => Ok(PowerLevel::On),
            "off" => Ok(PowerLevel::Off),
            _ => Err(InputError::InvalidReactorRebootLine),
        }
    }

    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            nom_tag("on"),
            nom_tag("off"),
        ))(input).map(|(rest, res)| (rest, match res {
            "on" => PowerLevel::On,
            "off" => PowerLevel::Off,
            _ => panic!("should never happen")
        }))
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

    fn all() -> [&'static Axis; Self::NUM_AXES] {
        [&Axis::X, &Axis::Y, &Axis::Z]
    }

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
    fn new(low: Coord, high: Coord) -> Self {
        assert!(low < high);
        Bounds{low, high}
    }

    fn parse_regex(text: &str) -> Result<Self, InputError> {
        let bounds_regex = Regex::new(
            r"^(-?\d+)\.\.(-?\d+)$"
        ).unwrap();
        let capture = bounds_regex.captures(&text).ok_or(InputError::InvalidReactorRebootLine)?;
        let low: Coord = capture.get(1).unwrap().as_str().parse()?;
        let high_included: Coord = capture.get(2).unwrap().as_str().parse::<Coord>()?;
        let high: Coord = high_included - 1; // subtract 1 to switch from including both endpoints to including only one
        Ok(Bounds::new(low, high))
    }

    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_coord,
            nom_tag(".."),
            nom_coord,
        ))(input).map(|(rest, (low, _, high))| (rest, Bounds::new(low, high + 1))) // add 1 to switch from including both endpoints to including only one
    }

    /// Parse input to create a Bounds. Returns None if there was any issue with the parsing.
    #[allow(dead_code)]
    fn parse(input: &str) -> Option<Self> {
        match Self::parse_nom(input) {
            Err(_) => None,
            Ok((rest, bounds)) => {
                if rest != "" {
                    None
                } else {
                    Some(bounds)
                }
            },
        }
    }

    /// Returns true if the coord is within (NOT on the boundaries of) this Bounds
    /// and false otherwise.
    fn contains(&self, coord: Coord) -> bool {
        coord > self.low && coord < self.high
    }

    /// Returns true if the other has a boundary that falls within self (and therefore
    /// self would need to split to avoid a partial overlap situation).
    fn is_split_by(&self, other: &Self) -> bool {
        self.contains(other.low) || self.contains(other.high)
    }

    /// Given an other which splits self, this returns a Vec of Bounds which consist
    /// of self split up into pieces. The Vec will always be of length 2 or 3.
    fn split_by(&self, other: &Self) -> Vec<Self> {
        let intersects = (self.contains(other.low), self.contains(other.high));
        match intersects {
            (false, false) => panic!("Bounds::split_by() may only be called when it splits it."),
            (true, false) => vec![
                Bounds::new(self.low, other.low),
                Bounds::new(other.low, self.high),
            ],
            (false, true) => vec![
                Bounds::new(self.low, other.high),
                Bounds::new(other.high, self.high),
            ],
            (true, true) => vec![
                Bounds::new(self.low, other.low),
                Bounds::new(other.low, other.high),
                Bounds::new(other.high, self.high),
            ],
        }
    }
}

impl Display for Bounds {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.low, self.high - 1)
    }
}

impl Cuboid {
    fn parse_regex(text: &str) -> Result<Self,InputError> {
        let cuboid_regex = Regex::new(
            r"^x=(.*),y=(.*),z=(.*)$"
        ).unwrap();
        let capture = cuboid_regex.captures(&text).ok_or(InputError::InvalidReactorRebootLine)?;
        let x: Bounds = Bounds::parse_regex(capture.get(1).unwrap().as_str())?;
        let y: Bounds = Bounds::parse_regex(capture.get(2).unwrap().as_str())?;
        let z: Bounds = Bounds::parse_regex(capture.get(3).unwrap().as_str())?;
        let m_bounds = [x,y,z];
        Ok(Cuboid{m_bounds})
    }

    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("x="),
            Bounds::parse_nom,
            nom_tag(",y="),
            Bounds::parse_nom,
            nom_tag(",z="),
            Bounds::parse_nom,
        ))(input).map(|(rest, (_, xb, _, yb, _, zb))| (rest, Cuboid{m_bounds: [xb, yb, zb]}))
    }

    /// Parse input to create a Cuboid. Returns None if there was any issue with the parsing.
    fn parse(input: &str) -> Option<Self> {
        match Self::parse_nom(input) {
            Err(_) => None,
            Ok((rest, cuboid)) => {
                if rest != "" {
                    None
                } else {
                    Some(cuboid)
                }
            },
        }
    }


    /// Use this to access the Bounds for a given axis.
    fn bounds(&self, axis: &Axis) -> &Bounds {
        return &self.m_bounds[axis.index()]
    }

    /// Returns a copy of the bounds for this Cuboid
    fn copy_bounds(&self) -> [Bounds; Axis::NUM_AXES] {
        Axis::all().map(|a| (*self.bounds(a)).clone())
    }

    /// Returns true if the other has a boundary along axis that falls within self (and
    /// therefore self would need to split to avoid a partial overlap situation).
    fn is_split_by_axis(&self, other: &Self, axis: &Axis) -> bool {
        self.bounds(axis).is_split_by(&other.bounds(axis))
    }

    /// Returns true if the other has any boundary that falls within self (and therefore
    /// self would need to split to avoid a partial overlap situation).
    fn is_split_by(&self, other: &Self) -> bool {
        Axis::all().iter().any(|a| self.is_split_by_axis(other,a))
    }

    /// Given an other which splits self along the given axis, this returns a Vec of Cuboids
    /// which consist of self split up into pieces that may overlap but don't intersect
    /// with other. The Vec will always be of length 2 or 3.
    fn split_by_axis(&self, other: &Self, axis: &Axis) -> Vec<Self> {
        let mut answer: Vec<Self> = Vec::new();
        for split_bound in self.bounds(axis).split_by(other.bounds(axis)) {
            let mut new_bounds: [Bounds; Axis::NUM_AXES] = self.copy_bounds();
            new_bounds[axis.index()] = split_bound;
            answer.push(Cuboid{m_bounds: new_bounds})
        }
        answer
    }

    /// Given an other which splits self, this returns a Vec of Cuboids which consist
    /// of self split up into pieces that may overlap but don't intersect with other.
    fn split_by(&self, other: &Self) -> Vec<Self> { // FIXME: I'm confident that this can be written better.
        assert!(self.is_split_by(other)); // Just verifying to help catch bugs
        let all_axes = Axis::all();
        let axes_that_split = all_axes.iter().filter(|a| self.is_split_by_axis(other, a));
        // let mut existing_splits_opt: Option<Vec<Self>> = None; // if None then work from self
        for axis in axes_that_split {
            println!("beginning axis = {}", axis); // FIXME: I'm testing this as I go
            // let what_to_iterate = match &existing_splits_opt {
            //     None => [self].iter(),
            //     Some(existing_splits) => existing_splits.iter().map(|x| &x)
            // };
            // for bound_we_have in what_to_iterate {
            //     println!("bound_we_have = {}", bound_we_have); // FIXME: I'm testing this as I go
            // }
        }

        let mut splits: Vec<Self> = vec![self.clone()]; // Unnecessary clone. But deal with it.
        for axis in Axis::all() {
            let mut next_splits: Vec<Self> = Vec::new();
            for cuboid in splits {
                if cuboid.is_split_by_axis(other, axis) {
                    next_splits.extend(cuboid.split_by_axis(other, axis));
                } else {
                    next_splits.push(cuboid);
                }
            }
            splits = next_splits;
        }
        // let mut split_2: Vec<Self> = Vec::new();
        // for cuboid in split_1 {
        //     if cuboid.is_split_by_axis(other, &Axis::Y) {
        //         split_2.extend(cuboid.split_by_axis(other, &Axis::Y));
        //     } else {
        //         split_2.push(cuboid);
        //     }
        // }
        // let mut split_3: Vec<Self> = Vec::new();
        // for cuboid in split_2 {
        //     if cuboid.is_split_by_axis(other, &Axis::Z) {
        //         split_3.extend(cuboid.split_by_axis(other, &Axis::Z));
        //     } else {
        //         split_3.push(cuboid);
        //     }
        // }

        splits
    }
}

impl Display for Cuboid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "x={},y={},z={}",
            self.bounds(&Axis::X),
            self.bounds(&Axis::Y),
            self.bounds(&Axis::Z)
        )
    }
}

impl Instruction {
    #[allow(dead_code)]
    fn parse_regex(text: &str) -> Result<Self,InputError> {
        let instruction_regex = Regex::new(
            r"^(.*) (.*)$"
        ).unwrap();
        let capture = instruction_regex.captures(&text).ok_or(InputError::InvalidReactorRebootLine)?;
        let power_level = PowerLevel::parse_regex(capture.get(1).unwrap().as_str())?;
        let cuboid = Cuboid::parse_regex(capture.get(2).unwrap().as_str())?;
        Ok(Instruction{power_level, cuboid})
    }

    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            PowerLevel::parse_nom,
            nom_tag(" "),
            Cuboid::parse_nom,
        ))(input).map(|(rest, (power_level, _, cuboid))| (rest, Instruction{power_level, cuboid}))
    }

    /// Parse input to create an Instruction. Returns None if there was any issue with the parsing.
    #[allow(dead_code)]
    fn parse(input: &str) -> Option<Self> {
        match Self::parse_nom(input) {
            Err(_) => None,
            Ok((rest, instruction)) => {
                if rest != "" {
                    None
                } else {
                    Some(instruction)
                }
            },
        }
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

    #[test]
    fn test_parse_serialize() {
        let input = "on x=-13..-11,y=-55..8,z=0..0";
        let (rest, instruction): (&str, Instruction) = Instruction::parse_nom(&input).unwrap();
        assert_eq!("", rest);
        assert_eq!(
            instruction,
            Instruction{power_level: PowerLevel::On, cuboid: Cuboid{m_bounds: [
                Bounds{low: -13, high: -10},
                Bounds{low: -55, high: 9},
                Bounds{low: 0, high: 1},
            ]}}
        );
        let output = format!("{}", instruction);
        assert_eq!(input, output);
    }

    #[test]
    fn test_bounds_split_by() {
        let b = Bounds::new(5, 15);
        assert_eq!(
            b.split_by(&Bounds::new(0,10)),
            vec![Bounds::new(5,10), Bounds::new(10,15)]
        );
        assert_eq!(
            b.split_by(&Bounds::new(10,20)),
            vec![Bounds::new(5,10), Bounds::new(10,15)]
        );
        assert_eq!(
            b.split_by(&Bounds::new(8,13)),
            vec![Bounds::new(5,8), Bounds::new(8,13), Bounds::new(13,15)]
        );
    }

    #[test]
    fn test_cuboid_split_by_axis() {
        let c = Cuboid::parse("x=3..5,y=5..16,z=-200..-99").unwrap();
        assert_eq!(
            c.split_by_axis(&Cuboid::parse("x=3..5,y=0..11,z=-200..-99").unwrap(), &Axis::Y),
            vec![
                Cuboid::parse("x=3..5,y=5..11,z=-200..-99").unwrap(),
                Cuboid::parse("x=3..5,y=12..16,z=-200..-99").unwrap(),
            ]
        );
    }

    #[test]
    fn test_cuboid_split() {
        let c0 = Cuboid::parse("x=3..5,y=5..16,z=-200..0").unwrap();
        let c1 = Cuboid::parse("x=3..5,y=0..11,z=-200..-99").unwrap();
        let split = c0.split_by(&c1);
        assert_eq!(split, vec![
            Cuboid::parse("x=3..5,y=5..11,z=-200..-99").unwrap(),
            Cuboid::parse("x=3..5,y=5..11,z=-98..0").unwrap(),
            Cuboid::parse("x=3..5,y=12..16,z=-200..-99").unwrap(),
            Cuboid::parse("x=3..5,y=12..16,z=-98..0").unwrap(),
        ]);
    }
}
