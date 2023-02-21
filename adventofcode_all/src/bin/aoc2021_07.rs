
// ======= part_a =======

mod part_a {
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
        let filename = "input/2021/input_07.txt";
        let text: String = fs::read_to_string(filename)?;
        let pieces_or_error: Result<Vec<i32>,ParseIntError> = text.split(",").map(|x| x.parse::<i32>()).collect();
        let pieces: Vec<i32> = pieces_or_error?;
        return Ok(pieces);
    }



    pub fn main() {
        match read_crab_file() {
            Ok(mut crab_positions) => {
                if crab_positions.len() == 0 {
                    println!("Error: no crabs");
                } else {
                    crab_positions.sort();
                    println!("sorted {} items", crab_positions.len());
                    let odd_length = (crab_positions.len() % 2) == 1;
                    match odd_length {
                        true => {
                            let middle_pos = crab_positions.len() / 2;
                            println!("Odd number of values, and optimal one is {}", crab_positions.get(middle_pos).unwrap());
                        },
                        false => {
                            let right_middle_pos = crab_positions.len() / 2;
                            let left_middle_pos = right_middle_pos - 1;
                            let left_val = crab_positions.get(left_middle_pos).unwrap();
                            let right_val = crab_positions.get(right_middle_pos).unwrap();
                            let optimal_val: i32 = *(if left_val == right_val {
                                println!("Even number of values, the optimal one is {}", left_val);
                                left_val
                            } else {
                                println!("Even number of values, either {} or {} is optimal; we'll use {}", left_val, right_val, left_val);
                                left_val
                            });
                            let mut fuel = 0;
                            for val in crab_positions {
                                fuel += (val - optimal_val).abs();
                            }
                            println!("Total fuel needed: {}", fuel);
                        },
                    }
                }
            },
            Err(err) => println!("Error: {}", err),
        }
    }
}

// ======= part_b =======

mod part_b {
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
        let filename = "input/2021/input_07.txt";
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
                    fn get_fuel(crab_positions: &Vec<i32>, destination: i32) -> i32 {
                        let mut fuel = 0;
                        for val in crab_positions {
                            let distance = (val - destination).abs();
                            fuel += (distance * (distance + 1)) / 2;
                        }
                        fuel
                    }
                    let min_position: i32 = *crab_positions.iter().min().unwrap();
                    let max_position: i32 = *crab_positions.iter().max().unwrap();
                    let mut least_fuel = i32::MAX;
                    for position in min_position..=max_position {
                        let fuel = get_fuel(&crab_positions, position);
                        println!("At position {} it costs {} fuel.", position, fuel);
                        least_fuel = std::cmp::min(least_fuel, fuel);
                    }
                    println!("Least fuel: {}", least_fuel);
                }
            },
            Err(err) => println!("Error: {}", err),
        }
    }
}


// ======= main() =======


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    println!("\nPart a:");
    part_a::main();
    println!("\nPart b:");
    part_b::main();
    Ok(())
}
