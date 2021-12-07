use std::fs;
use std::fmt;
use std::num::ParseIntError;



/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(ParseIntError),
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
        }
    }
}


fn read_crab_file() -> Result<Vec<i32>, InputError> {
    let filename = "data/2021/day/7/input.txt";
    let text: String = fs::read_to_string(filename)?;
    let pieces_or_error: Result<Vec<i32>,ParseIntError> = text.split(",").map(|x| x.parse::<i32>()).collect();
    let pieces: Vec<i32> = pieces_or_error?;
    return Ok(pieces);
}



pub fn main() {
    match read_crab_file() {
        Ok(crab_positions) => {
            if crab_positions.len() == 0 {
                println!("Error: no crabs");
            } else {
                let count: i32 = crab_positions.len() as i32;
                let total: i32 = crab_positions.iter().sum();
                println!("There are {} crabs with a total of {}.", count, total);
                let divides_evenly = total % count == 0;
                println!("It {} evenly.", match divides_evenly {true=>"divides", false=>"does not divide"});
                let lower_value = total / count;
                let upper_value = lower_value + 1;
                println!("Consider positions {} and {}.", lower_value, upper_value);
                fn get_fuel(crab_positions: &Vec<i32>, destination: i32) -> i32 {
                    let mut fuel = 0;
                    for val in crab_positions {
                        let distance = (val - destination).abs();
                        fuel += distance * distance;
                    }
                    fuel
                }
                let lower_fuel = get_fuel(&crab_positions, lower_value);
                let upper_fuel = get_fuel(&crab_positions, upper_value);
                println!("They would cost fuel {} and {}.", lower_fuel, upper_fuel);
                println!("So the answer is {}", std::cmp::min(lower_fuel, upper_fuel));
            }
        },
        Err(err) => println!("Error: {}", err),
    }
}
