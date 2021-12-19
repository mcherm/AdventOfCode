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



fn squared(c: Coord) -> LenSq {
    LenSq::try_from(c * c).unwrap()
}

// Returns the length squared between the two points
fn get_length(b1: &Beacon, b2: &Beacon) -> LenSq {
    squared(b1.x - b2.x) + squared(b1.y - b2.y) + squared(b1.z - b2.z)
}



impl Scanner {
    // Returns a sorted list of the squares of the distances between pairs of points.
    fn get_lengths(&self) -> LengthSet {
        let mut lengths: Vec<LenSq> = Vec::new();
        for b1 in &self.beacons {
            for b2 in &self.beacons {
                let length = get_length(&b1, &b2);
                if length != 0 {
                    lengths.push(length);
                }
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


impl LengthSet {
    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.lengths.len()
    }

    #[allow(dead_code)]
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







fn run() -> Result<(),InputError> {
    let scanners = read_beacon_file()?;
    for (i, scanner1) in scanners.iter().enumerate() {
        for scanner2 in &scanners[(i+1)..] {
            let overlaps = scanner1.get_lengths().overlaps(&scanner2.get_lengths());
            println!("{} to {}: {}", scanner1.name, scanner2.name, overlaps);
        }
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



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file() {
        let _ = read_beacon_file();
    }
}
