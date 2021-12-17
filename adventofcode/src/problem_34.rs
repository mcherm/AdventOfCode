use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;


/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    NoData,
    MultipleLines,
    InvalidLine,
    BadXRange,
    BadYRange,
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
            InputError::NoData => write!(f, "File is empty."),
            InputError::MultipleLines => write!(f, "File contains multiple lines."),
            InputError::InvalidLine => write!(f, "Invalid line."),
            InputError::BadXRange => write!(f, "Bad x range."),
            InputError::BadYRange => write!(f, "Bad y range."),
        }
    }
}



/// Read in the input file.
fn read_probe_file() -> Result<Target, InputError> {
    lazy_static! {
        static ref TARGET_RANGE_REGEX: Regex = Regex::new(
            r"^target area: x=(-?\d+)..(-?\d+), y=(-?\d+)..(-?\d+)$"
        ).unwrap();
    }

    let filename = "data/2021/day/17/input.txt";
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    let first_line: String = lines.next().ok_or(InputError::NoData)??;
    if lines.next().is_some() {
        return Err(InputError::MultipleLines);
    }

    let captures = TARGET_RANGE_REGEX.captures(&first_line).ok_or(InputError::InvalidLine)?;
    let x_min: i32 = captures.get(1).unwrap().as_str().parse()?;
    let x_max: i32 = captures.get(2).unwrap().as_str().parse()?;
    let y_min: i32 = captures.get(3).unwrap().as_str().parse()?;
    let y_max: i32 = captures.get(4).unwrap().as_str().parse()?;

    if x_min <= 0 || x_max <= 0 || x_max <= x_min {
        return Err(InputError::BadXRange);
    }
    if y_max <= y_min {
        return Err(InputError::BadYRange);
    }


    Ok(Target{x_min, x_max, y_min, y_max})
}


struct Target {
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
}
impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}..{}, {}..{}]", self.x_min, self.x_max, self.y_min, self.y_max)
    }
}


#[derive(Debug)]
enum PossibleX {
    Exactly(u32, i32), // steps, start_xv
    AtOrAbove(u32, i32), // steps, start_xv
}
impl PossibleX {
    // Returns true if the given step would be in range
    fn in_range(&self, step:u32) -> bool {
        match self {
            PossibleX::Exactly(s, _) => step == *s,
            PossibleX::AtOrAbove(s, _) => step >= *s,
        }
    }

    fn start_xv(&self) -> i32 {
        match self {
            PossibleX::Exactly(_, start_xv) => *start_xv,
            PossibleX::AtOrAbove(_, start_xv) => *start_xv,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct PossibleSolution {
    start_xv: i32,
    start_yv: i32,
    max_y: i32,
}


/// By examining the x coordinate, this finds possible numbers of steps that the
/// whole thing could take.
fn find_possible_steps(target: &Target) -> Vec<PossibleX> {
    let mut result = Vec::new();
    for start_xv in 1..(target.x_max + 1) {
        let mut steps = 0;
        let mut xv = start_xv;
        let mut x = 0;
        while xv > 0 && x <= target.x_max {
            x += xv;
            xv = xv - 1;
            steps += 1;
            if target.x_min <= x && x <= target.x_max && xv > 0 {
                result.push(PossibleX::Exactly(steps, start_xv));
            }
        }
        if target.x_min <= x && x <= target.x_max && xv == 0 {
            result.push(PossibleX::AtOrAbove(steps, start_xv));
        }
    }
    result
}


/// Adds to a set of starting positions
fn find_starting_positions(target: &Target, possible_step: &PossibleX, start_yv: i32, positions: &mut HashSet<[i32;2]>) {
    let start_xv = possible_step.start_xv();
    let mut steps: u32 = 0;
    let mut yv = start_yv;
    let mut y = 0;
    let mut max_y = y;
    loop {
        y += yv;
        yv = yv - 1;
        steps += 1;
        if y > max_y {
            max_y = y;
        }
        if y < target.y_min {
            break; // can no longer hit the target
        }
        if possible_step.in_range(steps) && target.y_min <= y && y <= target.y_max {
            // We hit the target!
            positions.insert([start_xv, start_yv]);
        }
    }
}


fn run() -> Result<(),InputError> {
    let target = read_probe_file()?;
    println!("target range: {}", target);
    let possible_steps = find_possible_steps(&target);
    println!("possible_steps: {:?}", possible_steps);
    let mut positions: HashSet<[i32;2]> = HashSet::new();
    let mut solutions_found = positions.len();
    for abs_y in 0..50000 {
        for start_yv in [-abs_y, abs_y] {
            for possible_step in &possible_steps {
                find_starting_positions(&target, &possible_step, start_yv, &mut positions);
                if positions.len() > solutions_found {
                    solutions_found = positions.len();
                    println!("improved to {}", solutions_found);
                }
            }
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
        read_probe_file();
    }
}
