use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
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
type Volume = u64;

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
    ContainedBy, // second has all points of first, plus some others
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
                (Ordering::Equal, Ordering::Less) => Comparison::ContainedBy,
                (Ordering::Equal, Ordering::Equal) => Comparison::Equal,
                (Ordering::Equal, Ordering::Greater) => Comparison::Surrounds,
                (Ordering::Greater, Ordering::Less) => Comparison::ContainedBy,
                (Ordering::Greater, Ordering::Equal) => Comparison::ContainedBy,
                (Ordering::Greater, Ordering::Greater) => Comparison::Intersects,
            }
        }
    }

    /// Returns a count of the size of this bound (as a volume).
    fn length(&self) -> Volume {
        // It's safe to unwrap because we know high > low
        Volume::try_from(self.high - self.low).unwrap()
    }

    /// Returns true if other overlaps at least some with self.
    fn overlaps(&self, other: &Self) -> bool {
        other.low < self.high && other.high > self.low
    }

    /// Given an other which splits self, this returns a Vec of Bounds which consist
    /// of self split up into pieces. The Vec will always be of length 2 or 3.
    fn split_by(&self, other: &Self) -> Vec<Self> {
        assert!(matches!(self.compare_with(other), Comparison::Intersects | Comparison::Surrounds));
        let intersects = (other.low > self.low, other.high < self.high);
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
        } else if compares.iter().all(|c| matches!(c, Comparison::ContainedBy | Comparison::Equal)) {
            Comparison::ContainedBy
        } else if compares.iter().all(|c| matches!(c, Comparison::Surrounds | Comparison::Equal)) {
            Comparison::Surrounds
        } else {
            Comparison::Intersects
        }
    }

    /// Returns a count of the number of points that are in this cuboid.
    fn volume(&self) -> Volume {
        Axis::all().iter().map(|a| self.bounds(a).length()).product()
    }

    /// Returns true if the other overlaps with self and also has a boundary along axis
    /// that falls within self (and therefore self would need to split along axis to avoid a
    /// partial overlap situation).
    fn is_split_by_axis(&self, other: &Self, axis: &Axis) -> bool {
        match self.bounds(axis).compare_with(other.bounds(axis)) {
            Comparison::ContainedBy | Comparison::Separate | Comparison::Equal => false,
            Comparison::Intersects | Comparison::Surrounds => {
                // if other has a boundary within self along axis, then we still need
                // to make sure that we overlap along the other axes.
                axis.others().iter().all(|oa| self.bounds(oa).overlaps(other.bounds(oa)))
            },
        }
    }

    /// Returns true if the other has any boundary that falls within self (and therefore
    /// self would need to split to avoid a partial overlap situation).
    fn is_split_by(&self, other: &Self) -> bool {
        match self.compare_with(other) {
            Comparison::Equal | Comparison::Separate | Comparison::ContainedBy => false,
            Comparison::Intersects | Comparison::Surrounds => true,
        }
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
            for piece in splits {
                if piece.is_split_by_axis(other, axis) {
                    next_splits.extend(piece.split_by_axis(other, axis));
                } else {
                    next_splits.push(piece);
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

    /// Returns a count of the number of points that are on.
    fn volume_on(&self) -> Volume {
        self.on_blocks.iter().map(|c| c.volume()).sum()
    }

    /// Modifies this core by performing the given instruction.
    fn perform(&mut self, instruction: &Instruction) -> Self {
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
                        match instruction.power_level {
                            PowerLevel::On => new_on_blocks.push(on_block.clone()),
                            PowerLevel::Off => {},
                        }
                    },
                    Comparison::ContainedBy => {
                        println!("Instruction {} contained in {}", instruction_cuboid, on_block);
                        match instruction.power_level {
                            PowerLevel::On => new_on_blocks.push(on_block.clone()),
                            PowerLevel::Off => new_instruction_cuboids.push(instruction_cuboid.clone()),
                        }
                    },
                    Comparison::Surrounds => {
                        println!("Instruction {} surrounds {}", instruction_cuboid, on_block);
                        new_instruction_cuboids.push(instruction_cuboid.clone());
                    },
                    Comparison::Intersects => {
                        println!("Instruction {} intersects {}", instruction_cuboid, on_block);
                        match instruction.power_level {
                            PowerLevel::On => {
                                // -- keep the existing block --
                                new_on_blocks.push(on_block.clone());
                                // -- keep all pieces of the instruction except the bit already covered
                                let pieces = instruction_cuboid.split_by(on_block);
                                for piece in pieces.iter() {
                                    match piece.compare_with(on_block) {
                                        Comparison::Separate => new_instruction_cuboids.push(piece.clone()),
                                        Comparison::Equal | Comparison::ContainedBy => {},
                                        _ => panic!("Split pieces shouldn't Intersect or Surround.")
                                    }
                                }
                            },
                            PowerLevel::Off => {
                                // -- keep the existing instruction --
                                new_instruction_cuboids.push(instruction_cuboid.clone());
                                // -- keep all pieces of the existing on_block except the bit that was covered
                                let pieces = on_block.split_by(instruction_cuboid);
                                for piece in pieces.iter() {
                                    match piece.compare_with(instruction_cuboid) {
                                        Comparison::Separate => new_on_blocks.push(piece.clone()),
                                        Comparison::Equal | Comparison::ContainedBy => {},
                                        _ => panic!("Split pieces shouldn't Intersect or Surround.")
                                    }
                                }
                            },
                        }
                    },
                }
            }
            instruction_cuboids = new_instruction_cuboids;
        }
        match instruction.power_level {
            PowerLevel::On => new_on_blocks.extend(instruction_cuboids),
            PowerLevel::Off => {},
        }
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


// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let instructions = read_reactor_reboot_file()?;
    println!("Instructions:");
    for instruction in instructions.iter() {
        println!("{}", instruction);
    }

    let mut reactor_core = ReactorCore::new();
    println!("Reactor Core before has {} on: {}", reactor_core.volume_on(), reactor_core);
    for instruction in instructions.iter() {
        reactor_core = reactor_core.perform(instruction);
        println!("Reactor Core: has {} on: {}", reactor_core.volume_on(), reactor_core);
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
        assert_eq!(Comparison::Separate,    b.compare_with(&Bounds::new(0, 5)));
        assert_eq!(Comparison::Separate,    b.compare_with(&Bounds::new(0, 10)));
        assert_eq!(Comparison::Intersects,  b.compare_with(&Bounds::new(0, 15)));
        assert_eq!(Comparison::ContainedBy, b.compare_with(&Bounds::new(0, 20)));
        assert_eq!(Comparison::ContainedBy, b.compare_with(&Bounds::new(0, 25)));
        assert_eq!(Comparison::Surrounds,   b.compare_with(&Bounds::new(10, 15)));
        assert_eq!(Comparison::Equal,       b.compare_with(&Bounds::new(10, 20)));
        assert_eq!(Comparison::ContainedBy, b.compare_with(&Bounds::new(10, 25)));
        assert_eq!(Comparison::Surrounds,   b.compare_with(&Bounds::new(13, 18)));
        assert_eq!(Comparison::Surrounds,   b.compare_with(&Bounds::new(15, 20)));
        assert_eq!(Comparison::Intersects,  b.compare_with(&Bounds::new(15, 25)));
        assert_eq!(Comparison::Separate,    b.compare_with(&Bounds::new(20, 25)));
        assert_eq!(Comparison::Separate,    b.compare_with(&Bounds::new(25, 30)));
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
        assert_eq!(
            vec![Bounds::parse("11..12").unwrap(), Bounds::parse("13..13").unwrap()],
            Bounds::parse("11..13").unwrap().split_by(&Bounds::parse("10..12").unwrap())
        );
    }

    #[test]
    fn test_cuboid_compare_with() {
        let c = Cuboid::parse("x=0..5,y=0..5,z=0..5").unwrap();
        assert_eq!(Comparison::Separate, c.compare_with(&Cuboid::parse("x=0..5,y=6..8,z=0..5").unwrap()));
        assert_eq!(Comparison::Equal, c.compare_with(&Cuboid::parse("x=0..5,y=0..5,z=0..5").unwrap()));
        assert_eq!(Comparison::Surrounds, c.compare_with(&Cuboid::parse("x=0..3,y=0..5,z=0..5").unwrap()));
        assert_eq!(Comparison::Surrounds, c.compare_with(&Cuboid::parse("x=0..3,y=0..5,z=2..5").unwrap()));
        assert_eq!(Comparison::ContainedBy, c.compare_with(&Cuboid::parse("x=-5..10,y=-5..10,z=-5..10").unwrap()));
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
    fn test_cuboid_is_split_by() {
        let c0 = Cuboid::parse("x=11..13,y=11..13,z=11..13").unwrap();
        let c1 = Cuboid::parse("x=10..12,y=10..12,z=10..12").unwrap();
        assert_eq!(
            vec![Bounds::parse("11..12").unwrap(), Bounds::parse("13..13").unwrap()],
            c0.bounds(&Axis::X).split_by(c1.bounds(&Axis::X)),
        );
        assert!(c0.is_split_by_axis(&c1, &Axis::X));
        assert!(c0.is_split_by(&c1));
        assert_eq!(
            vec![
                Cuboid::parse("x=11..12,y=11..12,z=11..12").unwrap(),
                Cuboid::parse("x=11..12,y=11..12,z=13..13").unwrap(),
                Cuboid::parse("x=11..12,y=13..13,z=11..13").unwrap(),
                Cuboid::parse("x=13..13,y=11..13,z=11..13").unwrap(),
            ],
            c0.split_by(&c1)
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

    #[test]
    fn test_splitting_small_thing() {
        // I added this test because I had a bug with this behavior.
        let b1 = Bounds::parse("0..1").unwrap();
        let b2 = &Bounds::parse("1..1").unwrap();
        assert_eq!(
            vec![Bounds::parse("0..0").unwrap(), Bounds::parse("1..1").unwrap()],
            b1.split_by(b2)
        );

        let piece = Cuboid::parse("x=0..0,y=0..1,z=0..0").unwrap();
        let other = &Cuboid::parse("x=0..0,y=1..1,z=0..0").unwrap();
        let axis = &Axis::Y;
        assert_eq!(true, piece.is_split_by_axis(other, axis));
        assert_eq!(
            vec![
                Cuboid::parse("x=0..0,y=0..0,z=0..0").unwrap(),
                Cuboid::parse("x=0..0,y=1..1,z=0..0").unwrap(),
            ],
            piece.split_by_axis(other, axis)
        );
    }
}
