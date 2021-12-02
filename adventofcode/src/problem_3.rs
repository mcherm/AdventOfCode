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

fn invalid_movement_error() -> Result<Movement, Error> {
    Err(Error::new(ErrorKind::InvalidData, "Invalid Movement"))
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
