
extern crate anyhow;


// ======= Constants =======


// ======= Parsing =======

mod parse {

    use std::fmt::Debug;
    use std::fs;
    // use std::ops::Add;
    use nom::{
        IResult,
        branch::alt,
        combinator::{value, map},
        character::complete::{char, line_ending},
        sequence::terminated,
        multi::many1,
    };


    pub fn input() -> Result<Vec<Snafu>, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_25.txt")?;
        match Snafu::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum Digit {
        MinusTwo, MinusOne, Zero, One, Two
    }

    #[derive(Debug)]
    pub struct Snafu {
        pub digits: Vec<Digit>
    }


    impl Digit {
        pub fn parse(input: &str) -> IResult<&str, Self> {
            alt((
                value(Digit::MinusTwo, char('=')),
                value(Digit::MinusOne, char('-')),
                value(Digit::Zero, char('0')),
                value(Digit::One, char('1')),
                value(Digit::Two, char('2')),
            ))(input)
        }
    }

    impl Snafu {
        pub fn parse(input: &str) -> IResult<&str, Self> {
            map(many1(Digit::parse), |digits| Snafu{digits})(input)
        }

        /// Parses a newline-terminated list of Snafus
        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
            many1(terminated(Snafu::parse, line_ending))(input)
        }

    }

}


// ======= Part 1 Compute =======

mod compute {
    use std::fmt::{Display, Formatter};
    use std::ops::Add;
    use crate::parse::{Digit, Snafu};

    impl Snafu {
        pub fn zero() -> Self {
            Snafu{digits: vec![Digit::Zero]}
        }
    }

    impl Display for Digit {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", match self {
                Digit::MinusTwo => '=',
                Digit::MinusOne => '-',
                Digit::Zero => '0',
                Digit::One => '1',
                Digit::Two => '2',
            })
        }
    }

    impl Display for Snafu {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            for d in self.digits.iter() {
                write!(f, "{}", d)?;
            }
            Ok(())
        }
    }

    impl Add<&Snafu> for Snafu {
        type Output = Self;

        fn add(self, rhs: &Snafu) -> Self::Output {
            for d in self.digits.iter().rev() {
                println!("{}", d);
            }
            Snafu::zero() // FIXME: Just a placeholder until it works.
        }
    }

}

// ======= Part 2 Compute =======


// ======= main() =======

use parse::{input, Snafu};


fn part_a(snafus: &Vec<Snafu>) {
    println!("\nPart a:");
    let sum: Snafu = snafus.iter().fold(Snafu::zero(), |x,y| x + y);
    println!("The sum is {}", sum);
}


fn part_b(_input: &Vec<Snafu>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
