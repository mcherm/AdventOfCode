use std::fmt;
use std::num::ParseIntError;
use std::fs::File;
use std::io::{BufRead, BufReader};


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
struct HeightMap {
    data: Vec<Vec<u8>>,
    width: usize,
    height: usize,
}

impl HeightMap {
    fn new(data: Vec<Vec<u8>>) -> HeightMap {
        let height = data.len();
        assert!(height > 0);
        let width = data[0].len();
        assert!(width > 0);
        for row in &data {
            assert!(row.len() == width);
        }
        HeightMap{data, width, height}
    }

    fn get(&self, x: usize, y: usize) -> u8 {
        self.data[y][x]
    }

    fn find_local_mins(&self) -> Vec<u8> {
        let mut mins: Vec<u8> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let this_val = self.get(x,y);
                let mut is_local_min = true; // assumed, so far
                if x > 0 && self.get(x-1,y) <= this_val {
                    is_local_min = false;
                }
                if x < self.width-1 && self.get(x+1,y) <= this_val {
                    is_local_min = false;
                }
                if y > 0 && self.get(x,y-1) <= this_val {
                    is_local_min = false;
                }
                if y < self.height-1 && self.get(x,y+1) <= this_val {
                    is_local_min = false;
                }
                if is_local_min {
                    mins.push(this_val);
                }
            }
        }
        mins
    }
}


/// Read in the input file.
fn read_height_map_file() -> Result<HeightMap, InputError> {
    let filename = "data/2021/day/9/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    let mut data: Vec<Vec<u8>> = Vec::new();
    for line in lines {
        let text = line?;
        let mut row: Vec<u8> = Vec::new();
        let chars = text.chars();
        for c in chars {
            let val: u8 = c.to_string().parse::<u8>()?;
            row.push(val);
        }
        data.push(row);
    }

    return Ok(HeightMap::new(data));
}


pub fn main() {
    match read_height_map_file() {
        Ok(height_map) => {
            let local_mins = height_map.find_local_mins();
            let risk_level: u32 = local_mins.iter().map(|x| (x+1) as u32).sum();
            println!("risk_level: {:#?}", risk_level);
        },
        Err(err) => println!("Error: {}", err),
    }
}
