
// ======= part_a =======

mod part_a {
    use std::fmt;
    use std::num::ParseIntError;
    use std::fs::File;
    use std::io::{BufRead, BufReader};


    /// An error that we can encounter when reading the input.
    enum InputError {
        IoError(std::io::Error),
        BadInt(ParseIntError),
        InvalidEnergyLevel,
        MixedLengthLines,
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
                InputError::InvalidEnergyLevel => write!(f, "Invalid energy level"),
                InputError::MixedLengthLines => write!(f, "Mixed length lines"),
            }
        }
    }


    type OctopusValue = u8;


    #[derive(Debug)]
    struct OctopusMap {
        data: Vec<Vec<OctopusValue>>,
        width: usize,
        height: usize,
    }

    impl OctopusMap {
        fn incr(&mut self, x: usize, y: usize) {
            self.data[y][x] += 1;
            if self.data[y][x] == 10 {
                let x_min = if x==0 {0} else {x-1};
                let y_min = if y==0 {0} else {y-1};
                let x_max = if x+1 == self.width {x} else {x+1};
                let y_max = if y+1 == self.height {y} else {y+1};
                for y_n in y_min..=y_max {
                    for x_n in x_min..=x_max {
                        self.incr(x_n,y_n);
                    }
                }
            }
        }

        /// Causes each overcharged octopus to flash, resets their energy level,
        /// and returns a count of the flashes
        fn flash(&mut self) -> u32 {
            let mut flash_count: u32 = 0;
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.data[y][x] > 9 {
                        self.data[y][x] = 0;
                        flash_count += 1
                    }
                }
            }
            flash_count
        }

        /// Performs one step and returns the count of flashes
        fn step(&mut self) -> u32 {
            for y in 0..self.height {
                for x in 0..self.width {
                    self.incr(x,y);
                }
            }
            self.flash()
        }
    }


    /// Read in the input file.
    fn read_octopus_file() -> Result<OctopusMap, InputError> {
        let filename = "input/2021/input_11.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        let mut data: Vec<Vec<u8>> = Vec::new();
        let mut row_len: Option<usize> = None;
        for line in lines {
            let text = line?;
            let mut row: Vec<u8> = Vec::new();
            let chars = text.chars();
            for c in chars {
                let val: u8 = c.to_string().parse::<u8>()?;
                if val > 9 {
                    return Err(InputError::InvalidEnergyLevel);
                }
                row.push(val);
            }
            match row_len {
                None => {
                    row_len = Some(row.len());
                },
                Some(width) => {
                    if width != row.len() {
                        return Err(InputError::MixedLengthLines)
                    }
                }
            }
            data.push(row);
        }
        let width = row_len.unwrap();
        let height = data.len();
        return Ok(OctopusMap{data, width, height});
    }


    pub fn main() {
        match read_octopus_file() {
            Ok(mut octopus_map) => {
                let mut total_flashes = 0;
                for _i in 0..100 {
                    total_flashes += octopus_map.step();
                }
                println!("Total flashes: {}", total_flashes);
            },
            Err(err) => println!("Error: {}", err),
        }
    }
}

// ======= part_b =======

mod part_b {
    use std::fmt;
    use std::num::ParseIntError;
    use std::fs::File;
    use std::io::{BufRead, BufReader};


    /// An error that we can encounter when reading the input.
    enum InputError {
        IoError(std::io::Error),
        BadInt(ParseIntError),
        InvalidEnergyLevel,
        MixedLengthLines,
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
                InputError::InvalidEnergyLevel => write!(f, "Invalid energy level"),
                InputError::MixedLengthLines => write!(f, "Mixed length lines"),
            }
        }
    }


    type OctopusValue = u8;


    #[derive(Debug)]
    struct OctopusMap {
        data: Vec<Vec<OctopusValue>>,
        width: usize,
        height: usize,
    }

    impl OctopusMap {
        fn incr(&mut self, x: usize, y: usize) {
            self.data[y][x] += 1;
            if self.data[y][x] == 10 {
                let x_min = if x==0 {0} else {x-1};
                let y_min = if y==0 {0} else {y-1};
                let x_max = if x+1 == self.width {x} else {x+1};
                let y_max = if y+1 == self.height {y} else {y+1};
                for y_n in y_min..=y_max {
                    for x_n in x_min..=x_max {
                        self.incr(x_n,y_n);
                    }
                }
            }
        }

        /// Causes each overcharged octopus to flash, resets their energy level,
        /// and returns a count of the flashes
        fn flash(&mut self) -> u32 {
            let mut flash_count: u32 = 0;
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.data[y][x] > 9 {
                        self.data[y][x] = 0;
                        flash_count += 1
                    }
                }
            }
            flash_count
        }

        /// Performs one step and returns whether ALL flashed.
        fn step(&mut self) -> bool {
            for y in 0..self.height {
                for x in 0..self.width {
                    self.incr(x,y);
                }
            }
            let flash_count = self.flash();
            return flash_count == ((self.width as u32) * (self.height as u32));
        }
    }


    /// Read in the input file.
    fn read_octopus_file() -> Result<OctopusMap, InputError> {
        let filename = "input/2021/input_11.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        let mut data: Vec<Vec<u8>> = Vec::new();
        let mut row_len: Option<usize> = None;
        for line in lines {
            let text = line?;
            let mut row: Vec<u8> = Vec::new();
            let chars = text.chars();
            for c in chars {
                let val: u8 = c.to_string().parse::<u8>()?;
                if val > 9 {
                    return Err(InputError::InvalidEnergyLevel);
                }
                row.push(val);
            }
            match row_len {
                None => {
                    row_len = Some(row.len());
                },
                Some(width) => {
                    if width != row.len() {
                        return Err(InputError::MixedLengthLines)
                    }
                }
            }
            data.push(row);
        }
        let width = row_len.unwrap();
        let height = data.len();
        return Ok(OctopusMap{data, width, height});
    }


    pub fn main() {
        match read_octopus_file() {
            Ok(mut octopus_map) => {
                let mut steps = 0;
                loop {
                    let all_flashed = octopus_map.step();
                    steps += 1;
                    if all_flashed {
                        break;
                    }
                }
                println!("Total steps until all synchronize: {}", steps);
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
