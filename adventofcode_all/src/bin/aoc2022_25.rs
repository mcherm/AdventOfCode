
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

    #[derive(Debug, Eq, PartialEq)]
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
    use itertools::EitherOrBoth::{Both, Right};
    use itertools::{EitherOrBoth, Itertools};

    #[derive(Debug, Copy, Clone)]
    enum Overflow {
        OverMinus,
        OverZero,
        OverPlus,
    }

    impl Overflow {
        fn as_num(&self) -> i8 {
            use Overflow::*;
            match self {
                OverMinus => -1,
                OverZero => 0,
                OverPlus => 1,
            }
        }
    }

    impl Display for Overflow {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", match self {
                Overflow::OverMinus => '-',
                Overflow::OverZero => '0',
                Overflow::OverPlus => '+',
            })
        }
    }

    impl Digit {

        fn as_num(&self) -> i8 {
            use Digit::*;
            match self {
                MinusTwo => -2,
                MinusOne => -1,
                Zero => 0,
                One => 1,
                Two => 2,
            }
        }
    }

    /// This sums individual digits. It takes in a pair of digits and an overflow
    /// and returns the output digit and overflow.
    fn sum_digit(d1: Digit, d2: Digit, overflow: Overflow) -> (Digit, Overflow) {
        use Digit::*;
        use Overflow::*;
        match d1.as_num() + d2.as_num() + overflow.as_num() {
            -5 => (Zero    , OverMinus),
            -4 => (One     , OverMinus),
            -3 => (Two     , OverMinus),
            -2 => (MinusTwo, OverZero),
            -1 => (MinusOne, OverZero),
            0  => (Zero    , OverZero),
            1  => (One     , OverZero),
            2  => (Two     , OverZero),
            3  => (MinusTwo, OverPlus),
            4  => (MinusOne, OverPlus),
            5  => (Zero, OverPlus),
            _ => panic!("Add out of bounds."),
        }
    }



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
            let mut overflow = Overflow::OverZero;

            // A bunch of work to loop through the pairs of digits from least significant to
            // greatest, left-padding with zeros to make them the same length.
            let left_digits = self.digits.iter().rev();
            let right_digits = rhs.digits.iter().rev();
            let mut digits = left_digits.zip_longest(right_digits)
                .map(|digit_pair| match digit_pair {
                    Both(x, y) => (x,y),
                    EitherOrBoth::Left(x) => (x, &Digit::Zero),
                    Right(y) => (&Digit::Zero, y)
                })
                .map(|(d1, d2)| {
                    let (d_sum, new_overflow) = sum_digit(*d1, *d2, overflow);
                    overflow = new_overflow;
                    d_sum
                })
                .collect_vec();

            // add a "leading" (trailing) digit if we had overflow
            match overflow {
                Overflow::OverZero => {},
                Overflow::OverMinus => {
                    digits.push(Digit::MinusOne)
                },
                Overflow::OverPlus => {
                    digits.push(Digit::One)
                },
            }
            digits.reverse(); // but we've built it backward, so fix that
            Snafu{digits}
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;
        use anyhow::anyhow;

        fn read_snafu(s: &'static str) -> Result<Snafu,anyhow::Error> {
            match Snafu::parse(s)? {
                ("", x) => Ok(x),
                (_, _) => Err(anyhow!("Did not use the whole input")),
            }
        }

        #[test]
        fn test_add() -> Result<(), anyhow::Error>{
            let data = [
                ("2", "-", "1"),
                ("-", "-" , "="),
                ("111", "1" , "112"),
                ("=2", "-" , "=1"),
                ("2222", "1", "1===="),
                ("2", "1", "1="),
                ("1=-0-2", "12111", "1-111="),
            ];
            for (s1, s2, answer) in data {
                let v1 = read_snafu(s1)?;
                let v2 = read_snafu(s2)?;
                let expect = read_snafu(answer)?;
                let sum = v1 + &v2;
                assert_eq!(sum, expect);
            }
            Ok(())
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
    println!("Nothing to do.")
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
