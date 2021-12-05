use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt;
use std::num::ParseIntError;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;



/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(ParseIntError),
    InvalidLine,
    ZeroLengthLine,
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl From<ParseIntError> for InputError {
    fn from(error: ParseIntError) -> Self {
        InputError::BadInt(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err)   => write!(f, "{}", err),
            InputError::BadInt(err)    => write!(f, "{}", err),
            InputError::InvalidLine    => write!(f, "Invalid line"),
            InputError::ZeroLengthLine => write!(f, "Zero length line"),
        }
    }
}


#[derive(Debug)]
enum VentType {
    HORIZONTAL,
    VERTICAL,
}

type Point = (u32,u32);

struct VentLine {
    vent_type: VentType,
    coordinates: (Point,Point),
}

impl fmt::Display for VentLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let direc: char = match self.vent_type {
            VentType::HORIZONTAL => 'H',
            VentType::VERTICAL => 'V',
        };
        let c = self.coordinates;
        write!(f, "{}: ({},{}) -> ({},{})", direc, c.0.0, c.0.1, c.1.0, c.1.1)
    }
}

impl fmt::Debug for VentLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}



fn read_vent_file() -> Result<Vec<VentLine>, InputError>  {
    lazy_static! {
        static ref VENT_REGEX: Regex = Regex::new(
            r"^([1-9][0-9]*),([1-9][0-9]*) -> ([1-9][0-9]*),([1-9][0-9]*)$"
        ).unwrap();
    }

    let filename = "data/2021/day/5/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    let mut vent_lines: Vec<VentLine> = Vec::new();
    for line in lines {
        let text = line?;
        let captures: Captures = VENT_REGEX.captures(&text).ok_or(InputError::InvalidLine)?;
        let x1: u32 = captures.get(1).unwrap().as_str().parse::<u32>()?;
        let y1: u32 = captures.get(2).unwrap().as_str().parse::<u32>()?;
        let x2: u32 = captures.get(3).unwrap().as_str().parse::<u32>()?;
        let y2: u32 = captures.get(4).unwrap().as_str().parse::<u32>()?;
        let coordinates = ((x1,y1), (x2,y2));
        if x1 == x2 && y1 == y2 {
            return Err(InputError::ZeroLengthLine);
        } else if x1 == x2 {
            vent_lines.push(VentLine{vent_type: VentType::HORIZONTAL, coordinates});
        } else if y1 == y2 {
            vent_lines.push(VentLine{vent_type: VentType::VERTICAL, coordinates});
        } else {
            // Left blank because we're ignoring diagonal lines
        }
    }

    // --- Return Result ---
    Ok(vent_lines)
}


// A type for tracking how many cells are filled in.
#[derive(Debug)]
struct CountMatrix {
    counts: HashMap<Point, u32>,
    max_x: u32,
    max_y: u32,
}

impl CountMatrix {

    fn new() -> CountMatrix {
        CountMatrix{
            counts: HashMap::new(),
            max_x: 0,
            max_y: 0,
        }
    }

    fn mark(&mut self, point: Point) {
        let new_count = match self.counts.get(&point) {
            Some(old_count) => old_count + 1,
            None => 1,
        };
        self.counts.insert(point, new_count);
        if point.0 > self.max_x {
            self.max_x = point.0;
        }
        if point.1 > self.max_y {
            self.max_y = point.1;
        }
    }
}


// Generates a CountMatrix with the count of each point based on the vent_lines.
fn mark_matrix(vent_lines: &Vec<VentLine>) -> CountMatrix {
    let mut count_matrix = CountMatrix::new();
    for vent_line in vent_lines {
        count_matrix.mark(vent_line.coordinates.0); // FIXME: Mark start point
    }
    count_matrix
}


pub fn main() {
    match read_vent_file() {
        Ok(vent_lines) => {
            let count_matrix = mark_matrix(&vent_lines);
            println!("Counts: {:#?}", count_matrix);
        },
        Err(err) => println!("Error: {}", err),
    }
}