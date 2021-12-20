use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;


/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    InvalidNameLine,
    InvalidBeaconLine,
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
            InputError::InvalidNameLine => write!(f, "Invalid name line"),
            InputError::InvalidBeaconLine => write!(f, "Invalid beacon line"),
        }
    }
}



/// Read in the input file.
fn read_beacon_file() -> Result<Vec<Scanner>, InputError> {
    let filename = "data/2021/day/19/input.txt";
    let name_line_regex = Regex::new(
        r"^--- (.+) ---$"
    ).unwrap();
    let beacon_line_regex = Regex::new(
        r"^(-?\d+),(-?\d+),(-?\d+)$"
    ).unwrap();

    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();
    let mut scanners: Vec<Scanner> = Vec::new();
    while let Some(name_line) = lines.next() {
        let name_text = name_line?;
        let name_capture = name_line_regex.captures(&name_text).ok_or(InputError::InvalidNameLine)?;
        let name: String = name_capture.get(1).unwrap().as_str().to_string();
        let mut beacons: Vec<Beacon> = Vec::new();
        loop {
            if let Some(line) = lines.next() {
                let text: String = line?;
                if text.len() == 0 {
                    break; // blank lines end the beacons
                }
                let beacon_capture = beacon_line_regex.captures(&text).ok_or(InputError::InvalidBeaconLine)?;
                let x: Coord = beacon_capture.get(1).unwrap().as_str().parse()?;
                let y: Coord = beacon_capture.get(2).unwrap().as_str().parse()?;
                let z: Coord = beacon_capture.get(3).unwrap().as_str().parse()?;
                let beacon: Beacon = Beacon{x,y,z};
                beacons.push(beacon);
            } else {
                break; // Out of lines ends the file
            }
        }
        scanners.push(Scanner{name, beacons})
    }
    Ok(scanners)
}


type Coord = i32;
type LenSq = u32;

#[derive(Copy, Clone)]
struct Beacon {
    x: Coord,
    y: Coord,
    z: Coord
}

struct Scanner {
    name: String,
    beacons: Vec<Beacon>
}

#[derive(Debug)]
struct LengthSet {
    lengths: Vec<LenSq>
}

/// A direction.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Axis {X, Y, Z}

/// Substructure of Orient
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct AxisOrient {
    maps_to: Axis, // tells which axis of scanner_0 this axis of
    flip: bool,  // if true, then scanner_1 values should be multiplied by -1
    offset: Coord, // add this to a scanner_1 value to get the scanner_0 value.
}

/// Specification of how to orient one scanner relative to another.
#[derive(Debug, Copy, Clone)]
struct Orient {
    orients: [AxisOrient; 3]
}


/// Represents a possible mapping of source axis to dest axis
#[derive(Debug, Copy, Clone)]
struct AxisMapping {
    maps_to: [Axis;3],
    flip: [bool;3],
}

// FIXME: Maybe later? may be overengineered
// struct PointRef<'a> {
//     scanner: &'a Scanner,
//     pos: usize,
// }

// FIXME: Maybe later? may be overengineered
// struct PointRefPair<'a> {
//     a: PointRef<'a>,
//     b: PointRef<'a>,
//     dist: LenSq,
// }

// FIXME: Maybe later? may be overengineered
// /// A set of point pairs from one scanner
// #[derive(Debug)]
// struct PairSet<'a> {
// }


// FIXME: Maybe later? may be overengineered
// /// Represents a pair of points from two different Scanners
// /// that might match.
// struct Pair<'a> {
//     scanner_1: &'a Scanner,
//     scanner_2: &'a Scanner,
//     s1_positions: [usize;2],
//     s2_positions: [usize;2],
// }



fn squared(c: Coord) -> LenSq {
    LenSq::try_from(c * c).unwrap()
}

// Returns the length squared between the two points
fn get_length(b1: &Beacon, b2: &Beacon) -> LenSq {
    squared(b1.x - b2.x) + squared(b1.y - b2.y) + squared(b1.z - b2.z)
}



impl Beacon {
    fn get(&self, axis: Axis) -> Coord {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }
}

impl Scanner {
    // Returns a sorted list of the squares of the distances between pairs of points.
    fn get_lengths(&self) -> LengthSet {
        let mut lengths: Vec<LenSq> = Vec::new();
        for (i,b1) in self.beacons.iter().enumerate() {
            for b2 in self.beacons[(i+1)..].iter() {
                lengths.push(get_length(&b1, &b2));
            }
        }
        lengths.sort();
        LengthSet{lengths}
    }
}
impl PartialEq for Scanner {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Scanner {}


impl Axis {
    #[allow(dead_code)] // FIXME: Remove
    fn all() -> [Axis;3] {
        [Axis::X, Axis::Y, Axis::Z]
    }

    #[allow(dead_code)] // FIXME: Remove
    fn index(&self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

impl AxisMapping {
    fn all() -> [AxisMapping;6] {
        [
            AxisMapping{maps_to:[Axis::X, Axis::Y, Axis::Z], flip: [false, false, false]},
            AxisMapping{maps_to:[Axis::Y, Axis::X, Axis::Z], flip: [false, false, false]},
            AxisMapping{maps_to:[Axis::X, Axis::Z, Axis::Y], flip: [false, false, false]},
            AxisMapping{maps_to:[Axis::Y, Axis::Z, Axis::X], flip: [false, false, false]},
            AxisMapping{maps_to:[Axis::Z, Axis::X, Axis::Y], flip: [false, false, false]},
            AxisMapping{maps_to:[Axis::Z, Axis::Y, Axis::X], flip: [false, false, false]},
        ]
    }
}

impl AxisOrient {
}


impl Orient {
    // FIXME: May be overengineered. Add if needed.
    // /// Construct an Orient from a vector with exactly 3 values
    // fn new(orient_vec: Vec<AxisOrient>) -> Self {
    //     assert_eq!(orient_vec.len(), 3);
    //     Orient{orients: [orient_vec[0], orient_vec[1], orient_vec[2]]}
    // }
}


// FIXME: Maybe later? may be overengineered
// impl<'a> Pair<'a> {
//     fn new(
//         scanner_1: &'a Scanner,
//         s1_positions: [usize;2],
//         scanner_2: &'a Scanner,
//         s2_positions: [usize;2],
//     ) -> Self {
//         assert_eq!(
//             get_length(&scanner_1.beacons[s1_positions[0]], &scanner_1.beacons[s1_positions[1]]),
//             get_length(&scanner_2.beacons[s2_positions[0]], &scanner_2.beacons[s2_positions[1]])
//         );
//         Pair{scanner_1, scanner_2, s1_positions, s2_positions}
//     }
// }


// FIXME: Maybe later? may be overengineered
// impl<'a> PointRef<'a> {
//     fn get_beacon(&self) -> &Beacon {
//         &self.scanner.beacons[self.pos]
//     }
// }

// FIXME: Maybe later? may be overengineered
// impl<'a> PointRefPair {
//     fn new(a: PointRef<'a>, b: PointRef<'a>) -> Self {
//         let dist = get_length(a.get_beacon(), b.get_beacon());
//         PointRefPair(a, b, dist)
//     }
// }


impl LengthSet {
    #[allow(dead_code)] // FIXME: Remove
    fn len(&self) -> usize {
        self.lengths.len()
    }

    #[allow(dead_code)] // FIXME: Remove
    fn has_dupes(&self) -> bool {
        for p in 1..self.lengths.len() {
            if self.lengths[p] == self.lengths[p-1] {
                return true;
            }
        }
        false
    }

    // finds number of matches between 2 lengthsets
    fn overlaps(&self, other: &Self) -> u32 {
        // FIXME: Could exploit sorting to be more efficient if needed.
        let mut count: u32 = 0;
        for len_1 in &self.lengths {
            for len_2 in &other.lengths {
                if len_1 == len_2 {
                    count += 1;
                }
            }
        }
        count
    }
}


impl fmt::Display for Beacon {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}
impl fmt::Display for Scanner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "--- {} ---", self.name)?;
        for beacon in &self.beacons {
            writeln!(f, "{}", beacon)?;
        }
        writeln!(f)
    }
}
impl fmt::Display for Axis {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        })
    }
}


/// Given 2 points in s0 and two points in s1 that we believe may be the same points, this
/// returns a vector of Orients that make this work.
fn orient(source_scanner: &Scanner, source_positions: [usize;2], dest_scanner: &Scanner, dest_positions: [usize;2]) -> Vec<Orient> {
    let s0 = source_scanner.beacons[source_positions[0]];
    let s1 = source_scanner.beacons[source_positions[1]];
    let d0 = dest_scanner.beacons[dest_positions[0]];
    let d1 = dest_scanner.beacons[dest_positions[1]];
    assert_eq!(get_length(&s0, &s1), get_length(&d0, &d1));


    // Assuming b0 -> g0 and b1 -> g1
    // Assuming x -> x, y -> y, z -> z
    // Assuming xflip -> false, yflip -> false, z-flip -> false
    // Assuming yoffset -> 0, zoffset -> 0


    // Find offset that works for both with these constraints OR None if there isn't one
    fn find_offset(source_axis: Axis, axis_mapping: AxisMapping, [s0, s1, d0, d1]: [Beacon;4]) -> Option<Coord> {
        println!("find_offset({}, {:?}, ...)", source_axis, axis_mapping); // FIXME: Remove
        let dest_axis = axis_mapping.maps_to[source_axis.index()];
        let _flip = axis_mapping.flip[source_axis.index()]; // FIXME: Incorporate this later
        let offset = d0.get(dest_axis) - s0.get(source_axis);
        println!("  try {} as offset because {} - {}", offset, d0.get(dest_axis), s0.get(source_axis)); // FIXME: Remove
        println!("  compare to {} because {} - {}", (d1.get(dest_axis) - s1.get(source_axis)), d1.get(dest_axis), s1.get(source_axis)); // FIXME: Remove
        if (d1.get(dest_axis) - s1.get(source_axis)) == offset {
            println!("  Found offset {}", offset); // FIXME: Remove
            Some(offset)
        } else {
            println!("  No offset",); // FIXME: Remove
            None
        }
    }


    let mut ret_val: Vec<Orient> = Vec::new();
    for ax_map in AxisMapping::all() {
        println!("Trying axis mapping {:?}", ax_map);
        let x_offset_opt = find_offset(Axis::X, ax_map, [s0, s1, d0, d1]);
        let y_offset_opt = find_offset(Axis::Y, ax_map, [s0, s1, d0, d1]);
        let z_offset_opt = find_offset(Axis::Z, ax_map, [s0, s1, d0, d1]);
        if x_offset_opt.is_some() && y_offset_opt.is_some() && z_offset_opt.is_some() {
            // Found a possible Orient!
            let offsets = [x_offset_opt.unwrap(), y_offset_opt.unwrap(), z_offset_opt.unwrap()];
            println!("Found a possible mapping with offsets {} / {} / {}", offsets[0], offsets[1], offsets[2]);
            let orients: [AxisOrient;3] = [
                AxisOrient{maps_to: ax_map.maps_to[0], flip: ax_map.flip[0], offset: offsets[0]},
                AxisOrient{maps_to: ax_map.maps_to[1], flip: ax_map.flip[1], offset: offsets[1]},
                AxisOrient{maps_to: ax_map.maps_to[2], flip: ax_map.flip[2], offset: offsets[2]},
            ];
            ret_val.push(Orient{orients});
        }
    }

    ret_val
}





fn run() -> Result<(),InputError> {
    let scanners = read_beacon_file()?;
    for (i, scanner1) in scanners.iter().enumerate() {
        for scanner2 in &scanners[(i+1)..] {
            let overlaps = scanner1.get_lengths().overlaps(&scanner2.get_lengths());
            println!("{} to {}: {}", scanner1.name, scanner2.name, overlaps);
        }
    }
    println!("----------");
    let s0: &Scanner = &scanners[0];
    let s1: &Scanner = &scanners[1];
    println!("s0 lengths: {:?}", s0.get_lengths());
    println!("s1 lengths: {:?}", s1.get_lengths());
    let orients: Vec<Orient> = orient(s0, [0,1], s1, [0,1]);
    assert!(orients.len() == 1);
    println!("Orient: {:?}", orients[0]);
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

    #[test]
    fn test_orient_1() {
        fn newb(x: Coord, y: Coord, z: Coord) -> Beacon {
            Beacon{x, y, z}
        }
        let s0 = Scanner{name: "Zero".to_string(), beacons: vec![newb(100,101,102), newb(130,133,134)]};
        let s1 = Scanner{name: "One".to_string(),  beacons: vec![newb(120,101,102), newb(150,133,134)]};
        let orients: Vec<Orient> = orient(&s0, [0,1], &s1, [0,1]);
        assert!(orients.len() == 1);
        assert_eq!(orients[0].orients[0].offset, 20);
        println!("Orient: {:?}", orients[0]);
    }

    #[test]
    fn test_orient_2() {
        fn newb(x: Coord, y: Coord, z: Coord) -> Beacon {
            Beacon{x, y, z}
        }
        let s0 = Scanner{name: "Zero".to_string(), beacons: vec![newb(2,3,0), newb(3,0,0)]};
        let s1 = Scanner{name: "One".to_string(),  beacons: vec![newb(0,1,0), newb(1,-2,0)]};
        let orients: Vec<Orient> = orient(&s0, [0,1], &s1, [0,1]);
        assert!(orients.len() == 1);
        let or = orients[0];
        assert_eq!(or.orients[0].offset, -2); assert_eq!(or.orients[0].maps_to, Axis::X);
        assert_eq!(or.orients[1].offset, -2); assert_eq!(or.orients[1].maps_to, Axis::Y);
        assert_eq!(or.orients[2].offset,  0); assert_eq!(or.orients[2].maps_to, Axis::Z);
        println!("Orient: {:?}", orients[0]);
    }

    #[test]
    fn test_orient_3() {
        fn newb(x: Coord, y: Coord, z: Coord) -> Beacon {
            Beacon{x, y, z}
        }
        let s0 = Scanner{name: "Zero".to_string(), beacons: vec![newb(2,3,0), newb(3,0,0)]};
        let s1 = Scanner{name: "One".to_string(),  beacons: vec![newb(1,0,0), newb(-2,1,0)]};
        let orients: Vec<Orient> = orient(&s0, [0,1], &s1, [0,1]);
        assert_eq!(orients.len(), 1);
        let or = orients[0];
        assert_eq!(or.orients[0].offset, -2); assert_eq!(or.orients[0].maps_to, Axis::Y);
        assert_eq!(or.orients[1].offset, -2); assert_eq!(or.orients[1].maps_to, Axis::X);
        assert_eq!(or.orients[2].offset,  0); assert_eq!(or.orients[2].maps_to, Axis::Z);
        println!("Orient: {:?}", orients[0]);
    }
}
