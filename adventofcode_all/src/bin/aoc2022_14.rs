
extern crate anyhow;

use std::fs;
use anyhow::anyhow;
use nom;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::line_ending,
};
use nom::character::complete::u32 as nom_u32;
use std::fmt::{Display, Formatter};


// ======= Parsing =======

fn input() -> Result<Vec<LineSpec>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_14.txt")?;
    match LineSpec::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Num = u32;

#[derive(Debug, Copy, Clone)]
struct Point(Num, Num);

#[derive(Debug)]
struct LineSpec {
    points: Vec<Point>,
}


impl Point {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        nom::combinator::map(
            nom::sequence::separated_pair(
                nom_u32,
                tag(","),
                nom_u32,
            ),
            |(a,b)| Point(a,b)
        )(input)
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}


impl LineSpec {
    /// Construct a new Line from Points
    fn new(points: Vec<Point>) -> Result<Self, anyhow::Error> {
        // ensure each points shares either x or y coord with the previous point
        for pair in points.windows(2) {
            let a = pair[0];
            let b = pair[1];
            if a.0 != b.0 && a.1 != b.1 {
                return Err(anyhow!("Line has diagonal segment from {} to {}.", a, b));
            }
        }
        Ok(Self{points})
    }

    /// Parses a single LineSpec
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        nom::combinator::map_res(
            nom::multi::separated_list1( tag(" -> "), Point::parse ),
            |points| LineSpec::new(points)
        )(input)
    }

    /// Parses a newline-terminated list of LineSpecs
    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        nom::multi::many0( nom::sequence::terminated(Self::parse, line_ending) )(input)
    }
}


// ======= Compute =======


// ======= main() =======

fn part_a(input: &Vec<LineSpec>) {
    println!("\nPart a:");
    println!("{:?}", input);
}


fn part_b(_input: &Vec<LineSpec>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
