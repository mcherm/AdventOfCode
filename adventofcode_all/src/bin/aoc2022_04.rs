
extern crate anyhow;

use std::fs;
use nom::{
    IResult,
    character::complete::{char, newline},
    combinator::map,
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u32 as parse_num;



fn input() -> Result<Vec<Assignments>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_04.txt")?;
    match Assignments::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Num = u32;

#[derive(Debug, Copy, Clone)]
struct Range {
    min: Num,
    max: Num,
}

#[derive(Debug)]
struct Assignments {
    ranges: [Range; 2],
}


impl Range {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((parse_num, char('-'), parse_num)),
            |(min, _, max)| Range{min, max}
        )(input)
    }

}

impl Assignments {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((Range::parse, char(','), Range::parse)),
            |(a, _, b)| Assignments{ranges: [a,b]}
        )(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }

    /// Returns true if one range fully contains the other (or both fully contain each other).
    fn fully_overlaps(&self) -> bool {
        let [a,b] = self.ranges;
        (a.min <= b.min && a.max >= b.max) ||
            (b.min <= a.min && b.max >= a.max)
    }

    /// Returns true if one range at all overlaps the other.
    fn partially_overlaps(&self) -> bool {
        let [a,b] = self.ranges;
        a.min <= b.max && a.max >= b.min
    }
}


fn part_a(input: &Vec<Assignments>) {
    println!("\nPart a:");
    let count = input.iter().filter(|x| x.fully_overlaps()).count();
    println!("There are {} assignment pairs that fully overlap.", count);
}


fn part_b(input: &Vec<Assignments>) {
    println!("\nPart b:");
    let count = input.iter().filter(|x| x.partially_overlaps()).count();
    println!("There are {} assignment pairs that partially overlap.", count);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
