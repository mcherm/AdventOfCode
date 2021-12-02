use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;


#[derive(Debug)]
pub enum Movement {
    Forward(isize),
    Down(isize),
    Up(isize),
}

fn invalid_movement_error() -> Result<Movement, Error> {
    Err(Error::new(ErrorKind::InvalidData, "Invalid Movement"))
}

pub enum MovementError {
    FlyingSubError,
}

impl fmt::Display for MovementError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MovementError::FlyingSubError => write!(f, "FlyingSubError")
        }
    }
}

fn parse_movement(s: &str) -> Result<Movement, Error> {
    lazy_static! {
        static ref MOVEMENT_REGEX: Regex = Regex::new(
            r"^(forward|down|up) ([1-9][0-9]*)$"
        ).unwrap();
    }

    let maybe_captures = MOVEMENT_REGEX.captures(s);
    match maybe_captures {
        Some(captures) => {
            match captures.get(1) {
                Some(direction) => {
                    match captures.get(2) {
                        Some(distance) => {
                            match distance.as_str().parse() {
                                Ok(dist) => match direction.as_str() {
                                    "forward" => Ok(Movement::Forward(dist)),
                                    "down" => Ok(Movement::Down(dist)),
                                    "up" => Ok(Movement::Up(dist)),
                                    _ => invalid_movement_error()
                                }
                                Err(_) => invalid_movement_error(),
                            }
                        },
                        None => invalid_movement_error()
                    }
                },
                None => invalid_movement_error(),
            }
        },
        None => invalid_movement_error(),
    }
}


fn read_file_of_movements() -> Result<Vec<Movement>, Error>  {
    let filename = "data/2021/day/2/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();
    let mut movements = Vec::new();
    for line in lines {
        match line {
            Ok(text) => {
                match parse_movement(&text) {
                    Ok(movement) => movements.push(movement),
                    Err(err) => return Err(err)
                }
            }
            Err(err) => return Err(err)
        }
    }
    Ok(movements)
}


#[derive(Debug)]
struct Position {
    x: isize,
    depth: isize,
}

impl Position {
    fn move_by(&self, movement: Movement) -> Result<Position, MovementError> {
        match movement {
            Movement::Forward(x) => Ok(Position{x: self.x + x, depth: self.depth}),
            Movement::Down(x) => Ok(Position{x: self.x, depth: self.depth + x}),
            Movement::Up(x) => if self.depth - x >= 0 {
                Ok(Position{x: self.x, depth: self.depth - x})
            } else {
                Err(MovementError::FlyingSubError)
            }
        }
    }
}


fn update_position(movements: Vec<Movement>, start: Position) -> Result<Position, MovementError> {
    let mut position = start;
    for movement in movements {
        position = position.move_by(movement)?
    }
    Ok(position)
}


pub fn main() {
    match read_file_of_movements() {
        Ok(movements) => {
            let end_position = update_position(movements, Position{x: 0, depth: 0});
            match end_position {
                Ok(p) => println!("It ends up at: {:#?} which has 'area' of {}", p, p.x * p.depth),
                Err(err) => println!("Error: {}", err),
            }
        },
        Err(err) => println!("Error: {}", err)
    };
}
