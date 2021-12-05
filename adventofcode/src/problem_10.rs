use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt;
use std::num::ParseIntError;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::cmp::{min, max};



/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(ParseIntError),
    InvalidLine,
    ZeroLengthLine,
    UnsupportedLineAngle,
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
            InputError::UnsupportedLineAngle => write!(f, "Unsupported line angle"),
        }
    }
}


#[derive(Debug)]
enum VentType {
    Horizontal,
    Vertical,
    Positive45,
    Negative45,
}

type Point = (u32,u32);

struct VentLine {
    vent_type: VentType,
    coordinates: (Point,Point),
}

impl fmt::Display for VentLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let direc: char = match self.vent_type {
            VentType::Horizontal => 'H',
            VentType::Vertical => 'V',
            VentType::Positive45 => 'P',
            VentType::Negative45 => 'N',
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

/// Absolute value of difference between 2 u32s
fn abs_diff(a: u32, b: u32) -> u32 {
    max(a,b) - min(a,b)
}


fn read_vent_file() -> Result<Vec<VentLine>, InputError>  {
    lazy_static! {
        static ref VENT_REGEX: Regex = Regex::new(
            r"^(\d*),(\d*) -> (\d*),(\d*)$"
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
        let vent_type = if x1 == x2 && y1 == y2 {
            return Err(InputError::ZeroLengthLine);
        } else if x1 == x2 {
            VentType::Vertical
        } else if y1 == y2 {
            VentType::Horizontal
        } else {
            // Some kind of angle
            if abs_diff(x1,x2) == abs_diff(y1,y2) {
                // 45 degree angle
                if (x1 > x2) == (y1 > y2) {
                    VentType::Positive45
                } else {
                    VentType::Negative45
                }
            } else {
                return Err(InputError::UnsupportedLineAngle)
            }
        };
        vent_lines.push(VentLine{vent_type, coordinates});
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

    fn count_overlaps(&self) -> u32 {
        let mut overlaps: u32 = 0;
        for y in 0..=self.max_y {
            for x in 0..=self.max_x {
                if let Some(count) = self.counts.get(&(x,y)) {
                    if count >= &2 {
                        overlaps += 1;
                    }
                }
            }
        }
        overlaps
    }
}

impl fmt::Display for CountMatrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..=self.max_y {
            for x in 0..=self.max_x {
                let digit: char = match self.counts.get(&(x,y)) {
                    Some(count) if count <= &9 => count.to_string().chars().next().unwrap(),
                    Some(_count) => '+',
                    None => '.',
                };
                write!(f, "{}", digit)?;
            }
            write!(f, "\n")?;
        };
        Ok(())
    }
}


fn signum(x: i64) -> i64 {
    match x {
        0 => 0,
        x if x < 0 => -1,
        _ => 1,
    }
}


// Generates a CountMatrix with the count of each point based on the vent_lines.
fn mark_matrix(vent_lines: &Vec<VentLine>) -> CountMatrix {
    let mut count_matrix = CountMatrix::new();
    for vent_line in vent_lines {
        let ((x1,y1),(x2,y2)) = vent_line.coordinates;
        let delta_x: i64 = (x2 as i64) - (x1 as i64);
        let delta_y: i64 = (y2 as i64) - (y1 as i64);
        assert!( delta_x == 0 || delta_y == 0 || delta_x == delta_y || delta_x == -delta_y ); // horizontal, vertical, or diagonal
        assert!( delta_x != 0 || delta_y != 0 ); // not a point
        let steps: i64 = max(delta_x.abs(), delta_y.abs());
        let dx = signum(delta_x);
        let dy = signum(delta_y);
        for step in 0..=steps {
            let x_64: i64 = (x1 as i64) + step * dx;
            let y_64: i64 = (y1 as i64) + step * dy;
            assert!( x_64 >= 0 && y_64 >= 0 );
            let x: u32 = x_64 as u32;
            let y: u32 = y_64 as u32;
            count_matrix.mark((x,y));
        }
    }
    count_matrix
}


pub fn main() {
    match read_vent_file() {
        Ok(vent_lines) => {
            let count_matrix = mark_matrix(&vent_lines);
            let overlaps = count_matrix.count_overlaps();
            println!("Counts: \n{}", count_matrix);
            println!("Overlaps: {}", overlaps);
        },
        Err(err) => println!("Error: {}", err),
    }
}
