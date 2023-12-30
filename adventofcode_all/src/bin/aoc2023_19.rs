use std::fmt::{Debug, Display, Formatter};
use anyhow;


// ======= Constants =======


// ======= Parsing =======

type Num = u32;

// FIXME: Maybe it's time to build the library for "AsciiEnum".

#[derive(Copy, Clone)]
enum Rating {
    X, M, A, S
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Part {
    x: Num,
    m: Num,
    a: Num,
    s: Num,
}

#[derive(Copy, Clone)]
enum CompareOp {
    Less, More
}

#[derive(Debug)]
pub struct Rule {
    rating: Rating,
    compare_op: CompareOp,
    value: Num,
    target: String,
}

#[derive(Debug)]
pub struct Workflow {
    name: String,
    rules: Vec<Rule>,
    default_target: String,
}

#[derive(Debug)]
pub struct ProblemSet {
    workflows: Vec<Workflow>,
    parts: Vec<Part>,
}

type Input = ProblemSet;

#[derive(Debug)]
struct InvalidRatingError(char);



impl TryFrom<char> for Rating {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Rating::*;
        Ok(match value {
            'x' => X,
            'm' => M,
            'a' => A,
            's' => S,
            _ => return Err(anyhow::anyhow!("invalid rating character '{}'", value))
        })
    }
}

impl From<Rating> for char {
    fn from(value: Rating) -> Self {
        use Rating::*;
        match value {
            X => 'x',
            M => 'm',
            A => 'a',
            S => 's',
        }
    }
}

impl Debug for Rating {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let c: char = (*self).into();
        write!(f, "{}", c)
    }
}

impl Display for Rating {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<char> for CompareOp {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use CompareOp::*;
        Ok(match value {
            '<' => Less,
            '>' => More,
            _ => return Err(anyhow::anyhow!("invalid character '{}'", value))
        })
    }
}

impl From<CompareOp> for char {
    fn from(value: CompareOp) -> Self {
        use CompareOp::*;
        match value {
            Less => '<',
            More => '>',
        }
    }
}

impl Debug for CompareOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {CompareOp::Less => '<', CompareOp::More => '>'})
    }
}

impl Display for CompareOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for Part {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Part[{},{},{},{}]", self.x, self.m, self.a, self.s)
    }
}

impl Rule {
    fn new<T: ToString>(rating: Rating, compare_op: CompareOp, value: Num, target: T) -> Self {
        Self{rating, compare_op, value, target: target.to_string()}
    }
}

impl Workflow {
    fn new<T1: ToString, T2: ToString>(name: T1, rules: Vec<Rule>, default_target: T2) -> Self {
        Self{name: name.to_string(), rules, default_target: default_target.to_string()}
    }
}


mod parse {
    use super::{Input, Part, Rating, CompareOp, Rule, Workflow, ProblemSet};
    use std::fs;
    use nom;
    use nom::IResult;
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_19.txt")?;
        match Input::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    impl Rating {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::character::complete::one_of("xmas"),
                |c: char| Self::try_from(c).expect("should already be a valid character")
            )(input)
        }
    }

    impl CompareOp {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::character::complete::one_of("<>"),
                |c: char| Self::try_from(c).expect("should already be a valid character")
            )(input)
        }
    }

    impl Rule {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    Rating::parse,
                    CompareOp::parse,
                    nom_num,
                    nom::bytes::complete::tag(":"),
                    nom::character::complete::alpha1,
                )),
                |(rating, compare_op, value, _, target)| Rule::new(rating, compare_op, value, target)
            )(input)
        }
    }

    impl Workflow {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom::character::complete::alpha1,
                    nom::bytes::complete::tag("{"),
                    nom::multi::separated_list1(
                        nom::bytes::complete::tag(","),
                        Rule::parse,
                    ),
                    nom::bytes::complete::tag(","),
                    nom::character::complete::alpha1,
                    nom::bytes::complete::tag("}"),
                )),
                |(name, _, rules, _, default_target, _)| {
                    Self::new(name, rules, default_target)
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

    impl Part {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom::bytes::complete::tag("{x="),
                    nom_num,
                    nom::bytes::complete::tag(",m="),
                    nom_num,
                    nom::bytes::complete::tag(",a="),
                    nom_num,
                    nom::bytes::complete::tag(",s="),
                    nom_num,
                    nom::bytes::complete::tag("}"),
                )),
                |(_, x, _, m, _, a, _, s, _)| {
                    Part{x, m, a, s}
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

    impl ProblemSet {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    Workflow::parse_list,
                    nom::character::complete::line_ending,
                    Part::parse_list,
                )),
                |(workflows, _, parts)| {
                    Self{workflows, parts}
                }
            )(input)
        }
    }

}


// ======= Compute =======


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    println!("The input is {:?}", input);
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
