
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fmt::Debug;
    use std::fs;
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::tag,
        combinator::{value, map},
        character::complete::{line_ending, u8 as nom_num},
        sequence::{terminated, tuple},
        multi::{many1},
    };


    pub fn input() -> Result<InputData, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_22.txt")?;
        match InputData::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    #[derive(Debug, Copy, Clone)]
    pub enum GridElem {
        Wall,
        Open,
        Blank,
    }

    #[derive(Debug)]
    pub struct MapOfBoard {
        width: usize,
        height: usize,
        data: Vec<GridElem>,
    }


    #[derive(Debug, Copy, Clone)]
    pub enum TurnDir {
        Left, Right
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Step {
        Move(usize),
        Turn(TurnDir),
    }

    /// Stores 4-character (lowercase letter) names efficiently.
    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    pub struct Name {
        code: u32
    }

    #[derive(Debug)]
    pub struct InputData {
        map: MapOfBoard,
        steps: Vec<Step>,
    }



    impl GridElem {
        /// Parses a single GridElem
        fn parse(input: &str) -> IResult<&str, Self> {
            alt((
                value(GridElem::Wall, tag("#")),
                value(GridElem::Open, tag(".")),
                value(GridElem::Blank, tag(" ")),
            ))(input)
        }
    }

    impl MapOfBoard {
        /// Create an instance from what we read.
        fn from_rows(rows: Vec<Vec<GridElem>>) -> Self {
            let height = rows.len();
            let width = rows.iter().map(|row| row.len()).max().unwrap();
            let mut data = Vec::new();
            for y in 0..height {
                let row = &rows[y];
                for x in 0..width {
                    let grid_elem = row.get(x).unwrap_or(&GridElem::Blank);
                    data.push(*grid_elem);
                }
            }
            MapOfBoard{width, height, data}
        }

        /// Retrieve from an (x,y) location
        fn get_at(&self, x: usize, y: usize) -> GridElem {
            assert!(x < self.width);
            assert!(y < self.height);
            self.data[y * self.width + x]
        }

        /// Parses the whole MapOfBoard
        fn parse(input: &str) -> IResult<&str, Self> {
            map(
                many1(
                    terminated(
                        many1( GridElem::parse ),
                        line_ending
                    )
                ),
                |rows| MapOfBoard::from_rows(rows)
            )(input)
        }
    }

    impl Step {
        /// Parses a single Step
        fn parse(input: &str) -> IResult<&str, Self> {
            alt((
                map(
                    nom_num,
                    |x| Step::Move( x as usize )
                ),
                value(Step::Turn(TurnDir::Left), tag("L")),
                value(Step::Turn(TurnDir::Right), tag("R")),
            ))(input)
        }


        /// Parses a newline-terminated list of Steps. This does not attempt to enforce
        /// that the steps alternate between moves and turns. But I don't think that NEEDS
        /// to be enforced.
        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
            many1(Step::parse)(input)
        }
    }

    impl InputData {
        /// Parses the entire input data
        fn parse(input: &str) -> IResult<&str, Self> {
            map(
                tuple((
                    MapOfBoard::parse,
                    line_ending,
                    Step::parse_list,
                    line_ending,
                )),
                |(map, _, steps, _)| InputData{map, steps}
            )(input)
        }
    }

}



// ======= Part 1 Compute =======



// ======= Part 2 Compute =======



// ======= main() =======

use crate::parse::{input, InputData};


fn part_a(input: &InputData) {
    println!("\nPart a:");
    println!("Input: {:?}", input);
}


fn part_b(_input: &InputData) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}

