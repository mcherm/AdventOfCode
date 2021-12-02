
use std::fs::File;
use std::io::{BufRead, BufReader};
use itertools::Itertools;


fn read_file_of_numbers() -> Result<Vec<i32>, std::io::Error>  {
    let filename = "data/2021/day/1/input.txt";
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
