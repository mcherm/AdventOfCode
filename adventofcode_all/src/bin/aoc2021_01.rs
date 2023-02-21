
// ======= part_a =======

mod part_a {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::fmt::Formatter;

    #[derive(Debug)]
    pub enum PossibleError {
        Io(std::io::Error),
        ParseInt(std::num::ParseIntError),
    }

    impl From<std::io::Error> for PossibleError {
        fn from(e: std::io::Error) -> PossibleError {
            PossibleError::Io(e)
        }
    }

    impl From<std::num::ParseIntError> for PossibleError {
        fn from(e: std::num::ParseIntError) -> PossibleError {
            PossibleError::ParseInt(e)
        }
    }

    impl std::fmt::Display for PossibleError {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            match self {
                PossibleError::Io(err) => write!(f, "Error Message {}", err),
                PossibleError::ParseInt(err) => write!(f, "Error Message {}", err),
            }
        }
    }



    fn count_increases_in_file() -> Result<i32, PossibleError>  {
        let filename = "input/2021/input_01.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();
        let mut count: i32 = 0;
        let mut previous: i32 = i32::MAX;
        for line in lines {
            let text = line?;
            let value: i32 = text.parse()?;
            if value > previous {
                count += 1;
            }
            previous = value;
        }
        return Ok(count);
    }

    pub fn main() {
        match count_increases_in_file() {
            Ok(value) => println!("Result is {}", value),
            Err(err) => println!("Error: {}", err),
        }
    }
}

// ======= part_b =======

mod part_b {

    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use itertools::Itertools;


    fn read_file_of_numbers() -> Result<Vec<i32>, std::io::Error>  {
        let filename = "input/2021/input_01.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();
        lines.map(|line| line.and_then(
            |v| v.parse().map_err(
                |e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)
            )
        )).collect()
    }


    pub fn main() {
        match read_file_of_numbers() {
            Ok(number_vec) => {
                let number_iter = number_vec.into_iter();
                let runs = number_iter.tuple_windows();
                let mut count: i32 = 0;
                let mut previous: i32 = i32::MAX;
                for run in runs {
                    let (a, b, c) = run;
                    let sum = a + b + c;
                    if sum > previous {
                        count += 1;
                    }
                    previous = sum;
                }
                println!("Total of {} increases.", count);
            },
            Err(err) => println!("Error: {:#?}", err),
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
