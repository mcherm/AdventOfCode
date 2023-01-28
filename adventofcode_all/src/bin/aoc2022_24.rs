
extern crate anyhow;


// ======= Constants =======


// ======= Parsing =======

mod parse {
    #![allow(dead_code)] // FIXME: Remove later


    use std::fmt::Debug;
    use std::fs;
    use itertools::Itertools;
    use itertools::iproduct;
    use nom::{
        IResult,
        branch::alt,
        combinator::{value, map},
        character::complete::{char, line_ending},
        sequence::{delimited, tuple},
        multi::{many0, many1},
    };


    pub fn input() -> Result<Grove, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_24.txt")?;
        match Grove::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(e) => panic!("Invalid input: {:?}", e),
        }
    }


    type Num = u16;

    #[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
    struct Coord(Num, Num);

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Direction {
        N, S, E, W,
    }

    #[derive(Debug)]
    pub struct Blizzard {
        dir: Direction,
        loc: Coord,
    }

    #[derive(Debug)]
    pub struct Grove {
        size: Coord, // width and height of inside. (0,0) is the lower left
        start_x: Num,
        goal_x: Num,
        blizzards: Vec<Blizzard>,
    }



    impl Grove {
        /// Create an instance from what we read.
        fn from_data(data: (Num, Vec<Vec<Option<Direction>>>, Num)) -> Self {
            let (start_x, rows, goal_x) = data;
            let height = rows.len();
            if height == 0 {
                panic!("ElfPlaces must have at least one row.");
            }
            let width = rows[0].len();
            if width == 0 {
                panic!("ElfPlaces must be at least 1 column wide.");
            }
            if ! rows.iter().map(|row| row.len()).all_equal() {
                panic!("ElfGrid must be rectangular.");
            }
            let size = Coord(height.try_into().unwrap(), width.try_into().unwrap());
            let xs = 0..width;
            let ys = 0..height;
            let blizzards: Vec<Blizzard> = iproduct!(xs, ys).filter_map(|(x,y)| {
                match rows.get(y).unwrap().get(x).unwrap() {
                    None => None,
                    Some(dir) => {
                        let x = x.try_into().unwrap();
                        let y = (height - y - 1).try_into().unwrap(); // reverse so (0,0) is bottom left
                        let dir = (*dir).clone();
                        Some(Blizzard{dir, loc: Coord(x,y)})
                    },
                }
            }).collect();
            Grove{size, start_x, goal_x, blizzards}
        }


        /// Parses the top or bottom row of a blizzard basin and returns the x coordinate
        /// of the gap.
        pub fn parse_exit_row(input: &str) -> IResult<&str, Num> {
            map(
                tuple((
                    char('#'), // left wall
                    many0(char('#')), // leading wall
                    char('.'), // open space
                    many0(char('#')), // trailing wall
                    line_ending, // end of line
                )),
                |(_, leading, _, _, _)| leading.len().try_into().unwrap()
            )(input)
        }

        pub fn parse_body_row(input: &str) -> IResult<&str, Vec<Option<Direction>>> {
            delimited(
                char('#'),
                many1(
                    alt((
                        value(None, char('.')),
                        value(Some(Direction::N), char('^')),
                        value(Some(Direction::S), char('v')),
                        value(Some(Direction::E), char('>')),
                        value(Some(Direction::W), char('<')),
                    ))
                ),
                tuple((char('#'), line_ending)),
            )(input)
        }

        /// Parses the whole ElfPlaces
        pub fn parse(input: &str) -> IResult<&str, Self> {
            map(
                tuple((
                    Self::parse_exit_row,
                    many1( Self::parse_body_row ),
                    Self::parse_exit_row,
                )),
                |data| Self::from_data(data)
            )(input)
        }
    }

}


// ======= Part 1 Compute =======



// ======= Part 2 Compute =======


// ======= main() =======

use parse::{input, Grove};



fn part_a(input: &Grove) {
    println!("\nPart a:");
    println!("{:?}", input);
}


fn part_b(_input: &Grove) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
