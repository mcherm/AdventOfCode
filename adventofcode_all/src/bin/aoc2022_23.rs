
extern crate anyhow;


// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fmt::Debug;
    use std::fs;
    use anyhow::anyhow;
    use itertools::Itertools;
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::tag,
        combinator::{value, map},
        character::complete::line_ending,
        sequence::terminated,
        multi::many1,
    };


    pub fn input() -> Result<ElfPlaces, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_23.txt")?;
        match ElfPlaces::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    #[derive(Debug)]
    pub struct ElfPlaces {
        data: Vec<Vec<bool>>,
    }



    impl ElfPlaces {
        /// Create an instance from what we read.
        fn from_rows(data: Vec<Vec<bool>>) -> Result<Self, anyhow::Error> {
            let height = data.len();
            if height == 0 {
                return Err(anyhow!("ElfGrid must have at least one row."));
            }
            let width = data[0].len();
            if width == 0 {
                return Err(anyhow!("ElfGrid must be at least 1 column wide."));
            }
            if ! data.iter().map(|row| row.len()).all_equal() {
                return Err(anyhow!("ElfGrid must be rectangular."));
            }
            Ok(ElfPlaces{data})
        }

        /// Accessor for width
        pub fn width(&self) -> usize {
            self.data.first().unwrap().len()
        }

        /// Accessor for height
        pub fn height(&self) -> usize {
            self.data.len()
        }

        /// Retrieve from an (x,y) location
        pub fn get_at(&self, x: usize, y: usize) -> bool {
            self.data[y][x]
        }

        /// Parses the whole MapOfBoard
        pub fn parse(input: &str) -> IResult<&str, Self> {
            map(
                many1(
                    terminated(
                        many1(
                            alt((
                                value(true, tag("#")),
                                value(false, tag(".")),
                            ))
                        ),
                        line_ending
                    )
                ),
                |rows| {
                    // try it, but panic on errors because I don't know enough to return a parse error
                    match ElfPlaces::from_rows(rows) {
                        Err(e) => panic!("{}", e),
                        Ok(elf_places) => elf_places
                    }
                }
            )(input)
        }
    }

}



// ======= Part 1 Compute =======


// ======= Part 2 Compute =======


// ======= main() =======

use crate::parse::{input, ElfPlaces};


fn part_a(input: &ElfPlaces) {
    println!("\nPart a:");
    println!("input: {:?}", input);
}


fn part_b(_input: &ElfPlaces) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
