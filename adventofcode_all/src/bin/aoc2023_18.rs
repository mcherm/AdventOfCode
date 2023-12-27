use std::fmt::{Display, Formatter};
use anyhow;
use advent_lib::grid::Direction;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct DigDir(Direction);

#[derive(Debug, Copy, Clone)]
pub struct DigStep {
    dig_dir: DigDir,
    dist: u32,
}


impl DigDir {
    /// Creates a DigDir from the given character, or panics if it's an invalid character.
    fn from_char(c: char) -> Self {
        use Direction::*;
        DigDir(match c {
            'R' => East,
            'D' => South,
            'L' => West,
            'U' => North,
            _ => panic!("invalid DigDir '{}'", c)
        })
    }
}

impl Display for DigDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for DigStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.dig_dir, self.dist)
    }
}


type Input = Vec<DigStep>;



mod parse {
    use super::{Input, DigDir, DigStep};
    use std::fs;
    use nom;
    use nom::IResult;
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_18.txt")?;
        match DigStep::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    impl DigDir {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::character::complete::one_of("UDLR"),
                |c: char| DigDir::from_char(c)
            )(input)
        }
    }

    impl DigStep {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    DigDir::parse,
                    nom::bytes::complete::tag(" "),
                    nom_num,
                    nom::bytes::complete::tag(" ("),
                    nom::bytes::complete::is_not(")"),
                    nom::bytes::complete::tag(")"),
                )),
                |(dig_dir, _, dist, _, _, _)| {
                    DigStep{dig_dir, dist}
                }
            )(input)
        }

        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
            nom::multi::many1(
                nom::sequence::terminated(
                    Self::parse,
                    nom::character::complete::line_ending,
                )
            )(input)
        }

    }

}


// ======= Compute =======


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    println!("The input is:");
    for step in input {
        println!("    {}", step);
    }
}


fn part_b(_input: &Input) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
