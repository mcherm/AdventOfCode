use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use lazy_static::lazy_static;
use regex::Regex;


#[derive(Debug)]
pub enum Movement {
    Forward(isize),
    Down(isize),
    Up(isize),
}

fn invalid_movement_error() -> Error {
    Error::new(ErrorKind::InvalidData, "Invalid Movement")
}


fn parse_movement(s: &str) -> Option<Movement> {
    lazy_static! {
        static ref MOVEMENT_REGEX: Regex = Regex::new(
            r"^(forward|down|up) ([1-9][0-9]*)$"
        ).unwrap();
    }
    let maybe_captures = MOVEMENT_REGEX.captures(s);
    maybe_captures.and_then(|captures| {
        let maybe_direction = captures.get(1);
        match maybe_direction {
            Some(direction) => {
                let maybe_distance = captures.get(2);
                match maybe_distance {
                    Some(distance) => {
                        let dist: isize = match distance.as_str().parse() {
                            Ok(value) => value,
                            Err(_) => return None,
                        };
                        match direction.as_str() {
                            "forward" => Some(Movement::Forward(dist)),
                            "down" => Some(Movement::Down(dist)),
                            "up" => Some(Movement::Up(dist)),
                            _ => None
                        }
                    },
                    None => return None
                }
            },
            None => return None,
        }
    })
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
                    Some(movement) => movements.push(movement),
                    None => return Err(invalid_movement_error())
                }
            }
            Err(err) => return Err(err)
        }
    }
    Ok(movements)
}



pub fn main() {
    match read_file_of_movements() {
        Ok(movements) => {
            for movement in movements {
                println!("Movement: {:#?}", movement);
            }
        },
        Err(err) => println!("Error: {}", err)
    };
}
