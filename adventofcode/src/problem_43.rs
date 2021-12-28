use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use std::cmp::Ordering;
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

// Possible outcome of comparing two Bounds OR two Cuboids.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Comparison {
    Separate, // share no points in common
    Intersects, // some points shared, but each has some points the other lacks
    Contained, // second has all points of first, plus some others
    Surrounds, // first has all points of second, plus some others
    Equal, // it's the same Bound or Cuboid: all points are common
}

/// This is an immutable range of coordinates.
#[derive(Debug, Eq, PartialEq, Clone)]
struct Bounds {
    low: Coord,
    high: Coord,
}

/// This is an immutable rectangular parallelpiped.
#[derive(Debug, Eq, PartialEq, Clone)]
struct Cuboid {
    m_bounds: [Bounds; Axis::NUM_AXES],
}

/// This is an immutable instruction to be followed.
#[derive(Debug, Eq, PartialEq, Clone)]
struct Instruction {
    power_level: PowerLevel,
    cuboid: Cuboid,
}

/// This is an immutable reactor core which can follow instructions.
#[derive(Debug)]
struct ReactorCore {
    on_blocks: Vec<Cuboid>
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

    fn all() -> [&'static Axis; Axis::NUM_AXES] {
        [&Axis::X, &Axis::Y, &Axis::Z]
    }

    fn index(&self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }

    fn others(&self) -> [&'static Axis; Axis::NUM_AXES - 1] {
        match self {
            Axis::X => [&Axis::Y, &Axis::Z],
            Axis::Y => [&Axis::Z, &Axis::X],
            Axis::Z => [&Axis::X, &Axis::Y],
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
    /// Passed a low (inclusive) and high (exclusive) boundary, this creates a
    /// Coord. We must have low < high... they can't be equal or swapped.
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


    /// Returns a value indicating how these two Bounds compare.
    fn compare_with(&self, other: &Self) -> Comparison {
        if self.high <= other.low || self.low >= other.high {
            Comparison::Separate
        } else {
            match (self.low.cmp(&other.low), self.high.cmp(&other.high)) {
                (Ordering::Less, Ordering::Less) => Comparison::Intersects,
                (Ordering::Less, Ordering::Equal) => Comparison::Surrounds,
                (Ordering::Less, Ordering::Greater) => Comparison::Surrounds,
                (Ordering::Equal, Ordering::Less) => Comparison::Contained,
                (Ordering::Equal, Ordering::Equal) => Comparison::Equal,
                (Ordering::Equal, Ordering::Greater) => Comparison::Surrounds,
                (Ordering::Greater, Ordering::Less) => Comparison::Contained,
                (Ordering::Greater, Ordering::Equal) => Comparison::Contained,
                (Ordering::Greater, Ordering::Greater) => Comparison::Intersects,
            }
        }
    }

    /// Returns true if the coord is within (NOT on the boundaries of) this Bounds
    /// and false otherwise.
    // FIXME: There's a bug here with boundary conditions: upper allows being on the boundary; lower doesn't.
    fn surrounds(&self, coord: Coord) -> bool {
        coord > self.low && coord < self.high
    }

    /// Returns true if the other has a boundary that falls within self (and therefore
    /// self would need to split to avoid a partial overlap situation).
    fn is_split_by(&self, other: &Self) -> bool {
        self.surrounds(other.low) || self.surrounds(other.high)
    }

    /// Returns true if other overlaps at least some with self.
    fn overlaps(&self, other: &Self) -> bool {
        other.low < self.high && other.high > self.low
    }

    /// Given an other which splits self, this returns a Vec of Bounds which consist
    /// of self split up into pieces. The Vec will always be of length 2 or 3.
    fn split_by(&self, other: &Self) -> Vec<Self> {
        let intersects = (self.surrounds(other.low), self.surrounds(other.high));
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
    #[allow(dead_code)]
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

    /// Returns a value indicating how these two Cuboids compare.
    fn compare_with(&self, other: &Self) -> Comparison {
        let compares: Vec<Comparison> = Axis::all().iter().map(|a| self.bounds(a).compare_with(other.bounds(a))).collect();
        if compares.iter().any(|c| matches!(c, Comparison::Separate)) {
            Comparison::Separate
        } else if compares.iter().all(|c| matches!(c, Comparison::Equal)) {
            Comparison::Equal
        } else if compares.iter().all(|c| matches!(c, Comparison::Contained | Comparison::Equal)) {
            Comparison::Contained
        } else if compares.iter().all(|c| matches!(c, Comparison::Surrounds | Comparison::Equal)) {
            Comparison::Surrounds
        } else {
            Comparison::Intersects
        }
    }

    /// Returns true if the other has a boundary along axis that falls within self (and
    /// therefore self would need to split to avoid a partial overlap situation).
    fn is_split_by_axis(&self, other: &Self, axis: &Axis) -> bool {
        // we are split by this axis our bounds are split by theirs on this axis AND
        // the other two axes overlap.
        self.bounds(axis).is_split_by(&other.bounds(axis)) &&
            axis.others().iter().all(|oa| self.bounds(oa).overlaps(other.bounds(oa)))
    }

    /// Returns true if the other has any boundary that falls within self (and therefore
    /// self would need to split to avoid a partial overlap situation).
    fn is_split_by(&self, other: &Self) -> bool {
        // If any axis is split by other while the other 2 axes overlap with other then
        // we are split by it
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
    fn split_by(&self, other: &Self) -> Vec<Self> {
        assert!(self.is_split_by(other)); // Just verifying to help catch bugs
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


impl ReactorCore {
    fn new() -> Self {
        ReactorCore{on_blocks: Vec::new()}
    }

    /// Modifies this core by performing the given instruction.
    fn perform(&mut self, instruction: &Instruction) -> Self {
        assert!(instruction.power_level == PowerLevel::On); // For now, only on instructions are supported
        let mut new_on_blocks: Vec<Cuboid> = Vec::with_capacity(self.on_blocks.capacity() + 8);
        let mut instruction_cuboids: Vec<Cuboid> = vec![instruction.cuboid.clone()];
        for on_block in self.on_blocks.iter() {
            let mut new_instruction_cuboids: Vec<Cuboid> = Vec::with_capacity(instruction_cuboids.capacity() + 8);
            for instruction_cuboid in instruction_cuboids.iter() {
                match instruction_cuboid.compare_with(on_block) {
                    Comparison::Separate => {
                        println!("Instruction {} doesn't overlap {}", instruction_cuboid, on_block);
                        new_on_blocks.push(on_block.clone());
                        new_instruction_cuboids.push(instruction_cuboid.clone());
                    },
                    Comparison::Equal => {
                        println!("Instruction {} equals {}", instruction_cuboid, on_block);
                        new_on_blocks.push(on_block.clone());
                    },
                    Comparison::Contained => {
                        println!("Instruction {} contained in {}", instruction_cuboid, on_block);
                        new_on_blocks.push(on_block.clone());
                    },
                    Comparison::Surrounds => {
                        println!("Instruction {} surrounds {}", instruction_cuboid, on_block);
                        new_instruction_cuboids.push(instruction_cuboid.clone());
                    },
                    Comparison::Intersects => {
                        println!("Instruction {} intersects {}", instruction_cuboid, on_block);
                        new_on_blocks.push(on_block.clone());
                        let pieces = instruction_cuboid.split_by(on_block);
                        for piece in pieces.iter() {
                            match piece.compare_with(on_block) {
                                Comparison::Separate => new_instruction_cuboids.push(piece.clone()),
                                Comparison::Equal | Comparison::Contained => {},
                                _ => panic!("Split pieces shouldn't Intersect or Surround.")
                            }
                        }
                    },
                }
            }
            instruction_cuboids = new_instruction_cuboids;
        }
        new_on_blocks.extend(instruction_cuboids);
        ReactorCore{on_blocks: new_on_blocks}
    }

}

impl Display for ReactorCore {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Reactor:")?;
        for cuboid in &self.on_blocks {
            writeln!(f, "  Cubes are on at {}", cuboid)?;
        }
        writeln!(f, "  All others are off.")
    }
}


// ======== Functions ========


// FIXME: Remove
// /// Modify splitting to be a vector of cuboids that covers the exact same set of
// /// points but in which no cuboid is split by one of the cuboids in splitters.
// // FIXME: There is a totally unnecessary amount of copying going on here, because I don't know what I'm doing
// fn split_all(splitting: &Vec<Cuboid>, splitters: &Vec<Cuboid>) -> Vec<Cuboid> {
//     println!("Starting split_all");
//     let mut old_cuboids = splitting.clone();
//     let mut new_cuboids: Vec<Cuboid> = Vec::new();
//     for splitter in splitters {
//         for cuboid in old_cuboids.iter() {
//             if cuboid.is_split_by(splitter) {
//                 new_cuboids.extend(cuboid.split_by(splitter));
//             } else {
//                 new_cuboids.push(cuboid.clone());
//             }
//         }
//         old_cuboids = new_cuboids.clone();
//         new_cuboids.clear();
//     }
//     old_cuboids
// }

// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let instructions = read_reactor_reboot_file()?;
    println!("Instructions:");
    for instruction in instructions.iter() {
        println!("{}", instruction);
    }

    let mut reactor_core = ReactorCore::new();
    println!("Reactor Core before: {}", reactor_core);
    for instruction in instructions.iter() {
        reactor_core = reactor_core.perform(instruction);
        println!("Reactor Core: {}", reactor_core);
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
    fn test_bounds_compare_with() {
        let b = Bounds::new(10, 20);
        assert_eq!(Comparison::Separate,   b.compare_with(&Bounds::new(0, 5)));
        assert_eq!(Comparison::Separate,   b.compare_with(&Bounds::new(0, 10)));
        assert_eq!(Comparison::Intersects, b.compare_with(&Bounds::new(0, 15)));
        assert_eq!(Comparison::Contained,  b.compare_with(&Bounds::new(0, 20)));
        assert_eq!(Comparison::Contained,  b.compare_with(&Bounds::new(0, 25)));
        assert_eq!(Comparison::Surrounds,  b.compare_with(&Bounds::new(10, 15)));
        assert_eq!(Comparison::Equal,      b.compare_with(&Bounds::new(10, 20)));
        assert_eq!(Comparison::Contained,  b.compare_with(&Bounds::new(10, 25)));
        assert_eq!(Comparison::Surrounds,  b.compare_with(&Bounds::new(13, 18)));
        assert_eq!(Comparison::Surrounds,  b.compare_with(&Bounds::new(15, 20)));
        assert_eq!(Comparison::Intersects, b.compare_with(&Bounds::new(15, 25)));
        assert_eq!(Comparison::Separate,   b.compare_with(&Bounds::new(20, 25)));
        assert_eq!(Comparison::Separate,   b.compare_with(&Bounds::new(25, 30)));
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
    fn test_cuboid_compare_with() {
        let c = Cuboid::parse("x=0..5,y=0..5,z=0..5").unwrap();
        assert_eq!(Comparison::Separate, c.compare_with(&Cuboid::parse("x=0..5,y=6..8,z=0..5").unwrap()));
        assert_eq!(Comparison::Equal, c.compare_with(&Cuboid::parse("x=0..5,y=0..5,z=0..5").unwrap()));
        assert_eq!(Comparison::Surrounds, c.compare_with(&Cuboid::parse("x=0..3,y=0..5,z=0..5").unwrap()));
        assert_eq!(Comparison::Surrounds, c.compare_with(&Cuboid::parse("x=0..3,y=0..5,z=2..5").unwrap()));
        assert_eq!(Comparison::Contained, c.compare_with(&Cuboid::parse("x=-5..10,y=-5..10,z=-5..10").unwrap()));
        assert_eq!(Comparison::Intersects, c.compare_with(&Cuboid::parse("x=0..5,y=2..3,z=3..6").unwrap()));
        assert_eq!(Comparison::Intersects, c.compare_with(&Cuboid::parse("x=-1..8,y=2..4,z=2..4").unwrap()));
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
            Cuboid::parse("x=3..5,y=12..16,z=-200..0").unwrap(),
        ]);
    }

    // FIXME: Remove
    // #[test]
    // fn test_split_all() {
    //     let splitting = vec![
    //         Cuboid::parse("x=0..99,y=0..99,z=0..99").unwrap(),
    //     ];
    //     let splitters = vec![
    //         Cuboid::parse("x=50..120,y=30..99,z=0..99").unwrap(),
    //         Cuboid::parse("x=10..29,y=50..74,z=0..99").unwrap(),
    //         Cuboid::parse("x=0..3,y=0..3,z=0..3").unwrap(),
    //     ];
    //     let new_splitting = split_all(&splitting, &splitters);
    //     // NOTE: This test is slightly fragile in that it assumes we prefer axis X to Y to Z
    //     //   in that order. There are other ways to divide it (and other orders even if
    //     //   we had the same division) that would also be valid answers.
    //     assert_eq!(
    //         new_splitting,
    //         vec![
    //             Cuboid::parse("x=0..3,y=0..3,z=0..3").unwrap(),
    //             Cuboid::parse("x=0..3,y=0..3,z=4..99").unwrap(),
    //             Cuboid::parse("x=0..3,y=4..99,z=0..99").unwrap(),
    //             Cuboid::parse("x=4..9,y=0..99,z=0..99").unwrap(),
    //             Cuboid::parse("x=10..29,y=0..49,z=0..99").unwrap(),
    //             Cuboid::parse("x=10..29,y=50..74,z=0..99").unwrap(),
    //             Cuboid::parse("x=10..29,y=75..99,z=0..99").unwrap(),
    //             Cuboid::parse("x=30..49,y=0..99,z=0..99").unwrap(),
    //             Cuboid::parse("x=50..99,y=0..29,z=0..99").unwrap(),
    //             Cuboid::parse("x=50..99,y=30..99,z=0..99").unwrap(),
    //         ]
    //     );
    // }
}
