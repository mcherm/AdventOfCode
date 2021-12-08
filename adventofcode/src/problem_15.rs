use std::fmt;
use std::num::ParseIntError;
use std::fs::File;
use std::io::{BufRead, BufReader};
use itertools::Itertools;
use std::convert::TryInto;



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


#[derive(Debug)]
struct SevenSegData {
    combos: [String; 10],
    digits: [String; 4],
}


/// This is given a string and a delimiter to split by and it returns an array of
/// exactly n parts (or panics if that doesn't work out). The parts are copied
/// into String objects, which are now owned by the caller..
fn split_into_n_strings<const N: usize>(s: &str, delim: &str) -> [String; N] {
    let vec: Vec<String> = s.split(delim).map(|x| x.to_string()).collect();
    let array: [String; N] = vec.try_into().unwrap();
    array
}


fn read_seven_seg_display_file() -> Result<Vec<SevenSegData>, InputError> {
    let filename = "data/2021/day/8/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    let mut results: Vec<SevenSegData> = Vec::new();
    for line in lines {
        let text = line?;
        let (combos_txt, digits_txt): (&str, &str) = text.split(" | ").collect_tuple().unwrap();
        let combos: [String; 10] = split_into_n_strings(combos_txt, " ");
        let digits: [String; 4] = split_into_n_strings(digits_txt, " ");
        results.push(SevenSegData{combos, digits});
    }

    return Ok(results);
}



pub fn main() {
    match read_seven_seg_display_file() {
        Ok(seven_seg_data_list) => {
            println!("seven_seg_data_list {:#?}", seven_seg_data_list);
            let mut count = 0;
            for seven_seg_data in seven_seg_data_list {
                for digit in seven_seg_data.digits {
                    match digit.len() {
                        2 | 3 | 4 | 7 => count += 1,
                        5 | 6 => {},
                        _ => panic!("Invalid number of characters in digit: {}", digit)
                    }
                }
            }
            println!("The count is {}", count);
        },
        Err(err) => println!("Error: {}", err),
    }
}
