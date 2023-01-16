
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fmt::{Debug, Display, Formatter};
    use std::fs;
    use itertools::Itertools;
    use nom;
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::tag,
        combinator::map,
        character::complete::{line_ending, satisfy, i32 as nom_Num},
        sequence::tuple,
        multi::{count, many0},
    };
    use nom::sequence::terminated;


    pub fn input() -> Result<Vec<Monkey>, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_21.txt")?;
        match Monkey::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    pub type Num = i32;

    /// Stores 4-character (lowercase letter) names efficiently.
    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    pub struct Name {
        code: u32
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Job {
        Const(Num),
        Plus(Name, Name),
        Minus(Name, Name),
        Times(Name, Name),
        Divide(Name, Name),
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Monkey {
        name: Name,
        job: Job,
    }


    /// Convert between lowercase ascii letters and u32 in the range 0..26 with panics if out of range.
    fn letter_to_num(c: char) -> u32 {
        (c as u32) - ('a' as u32)
    }

    /// Convert between lowercase ascii letters and u32 in the range 0..26 with panics if out of range.
    fn num_to_letter(n: u32) -> char {
        char::from_u32(n + ('a' as u32)).unwrap()
    }


    impl Name {
        /// Construct Self or panic if given bad data.
        pub fn new(chars: Vec<char>) -> Self {
            assert!(chars.len() == 4);
            assert!(chars.iter().all(|c| c.is_ascii_lowercase()));
            let mut code = 0;
            for c in chars.iter() {
                code *= 26;
                code += letter_to_num(*c);
            }
            Self{code}
        }

        fn parse(input: &str) -> IResult<&str, Self> {
            map(
                count( satisfy(|c| c >= 'a' && c <= 'z'), 4 ),
                Name::new
            )(input)
        }

        fn to_chars(&self) -> [char;4] {
            let mut code = self.code;
            let d = num_to_letter(code % 26);
            code = code / 26;
            let c = num_to_letter(code % 26);
            code = code / 26;
            let b = num_to_letter(code % 26);
            code = code / 26;
            let a = num_to_letter(code % 26);
            [a,b,c,d]
        }
    }

    impl Debug for Name {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self)
        }
    }

    impl Display for Name {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.to_chars().iter().join(""))
        }
    }

    impl Job {
        fn parse(input: &str) -> IResult<&str, Self> {
            alt((
                map(
                    nom_Num,
                    |x| Job::Const(x)
                ),
                map(
                    tuple((
                        Name::parse,
                        alt((
                            tag(" + "),
                            tag(" - "),
                            tag(" * "),
                            tag(" / "),
                        )),
                        Name::parse,
                    )),
                    |(name_1, operation, name_2)| {
                        match operation {
                            " + " => Job::Plus(name_1, name_2),
                            " - " => Job::Minus(name_1, name_2),
                            " * " => Job::Times(name_1, name_2),
                            " / " => Job::Divide(name_1, name_2),
                            _ => panic!(),
                        }
                    }
                )
            ))(input)
        }
    }

    impl Monkey {
        fn parse(input: &str) -> IResult<&str, Self> {
            map(
                tuple((
                    Name::parse,
                    tag(": "),
                    Job::parse,
                )),
                |(name, _, job)| Monkey{name, job}
            )(input)
        }

        /// Parses a newline-terminated list of Blueprints
        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
            many0(terminated(Monkey::parse, line_ending))(input)
        }
    }

}



// ======= Part 1 Compute =======

mod compute {
    // use crate::parse::{Num, Name, Job, Monkey};

    // #[derive(Debug)]
    // pub struct MonkeyTroop {
    //     monkeys: HashMap<Name, Monkey>,
    // }

}


// ======= main() =======

use crate::parse::{input, Monkey};



fn part_a(input: &Vec<Monkey>) {
    println!("\nPart a:");
    for monkey in input {
        println!("{:?}", monkey);
    }
}


fn part_b(_input: &Vec<Monkey>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}

