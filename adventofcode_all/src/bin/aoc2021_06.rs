
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

    fn read_fish_file() -> Result<Vec<i32>, InputError> {
        let filename = "input/2021/input_06.txt";
        let text: String = fs::read_to_string(filename)?;
        let pieces_or_error: Result<Vec<i32>,ParseIntError> = text.split(",").map(|x| x.parse::<i32>()).collect();
        let pieces: Vec<i32> = pieces_or_error?;
        return Ok(pieces);
    }


    fn grow_fish(fish_counts: [i32; 9]) -> [i32; 9] {
        [
            fish_counts[1],
            fish_counts[2],
            fish_counts[3],
            fish_counts[4],
            fish_counts[5],
            fish_counts[6],
            fish_counts[7] + fish_counts[0],
            fish_counts[8],
            fish_counts[0],
        ]
    }

    pub fn main() {
        match read_fish_file() {
            Ok(fish_values) => {
                println!("fish_values: {:#?}", fish_values);
                let mut fish_counts: [i32; 9] = [0; 9];
                for value in fish_values {
                    fish_counts[value as usize] += 1;
                }
                println!("fish_counts: {:#?}", fish_counts);
                for day in 1..=80 {
                    fish_counts = grow_fish(fish_counts);
                    println!("After {} days: {:?}", day, fish_counts)
                }
                let total_fish: i32 = fish_counts.iter().sum();
                println!("Total Fish: {}", total_fish);
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

    fn read_fish_file() -> Result<Vec<i32>, InputError> {
        let filename = "input/2021/input_06.txt";
        let text: String = fs::read_to_string(filename)?;
        let pieces_or_error: Result<Vec<i32>,ParseIntError> = text.split(",").map(|x| x.parse::<i32>()).collect();
        let pieces: Vec<i32> = pieces_or_error?;
        return Ok(pieces);
    }

    type FishCount = i64;
    type FishCounts = [FishCount; 9];

    fn grow_fish(fish_counts: FishCounts) -> FishCounts {
        [
            fish_counts[1],
            fish_counts[2],
            fish_counts[3],
            fish_counts[4],
            fish_counts[5],
            fish_counts[6],
            fish_counts[7] + fish_counts[0],
            fish_counts[8],
            fish_counts[0],
        ]
    }

    pub fn main() {
        match read_fish_file() {
            Ok(fish_values) => {
                println!("fish_values: {:#?}", fish_values);
                let mut fish_counts: FishCounts = [0; 9];
                for value in fish_values {
                    fish_counts[value as usize] += 1;
                }
                println!("fish_counts: {:#?}", fish_counts);
                for day in 1..=256 {
                    fish_counts = grow_fish(fish_counts);
                    println!("After {} days: {:?}", day, fish_counts)
                }
                let total_fish: FishCount = fish_counts.iter().sum();
                println!("Total Fish: {}", total_fish);
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
