
extern crate anyhow;

use std::fmt::{Display, Formatter};
use std::fs;
use anyhow::Error;
use itertools::Itertools;
use nom::{
    IResult,
    character::complete::{space0, space1, newline},
    combinator::map,
    multi::many0,
    sequence::tuple
};
use nom::character::complete::u16 as parse_u16;



fn input() -> Result<Triangles, Error> {
    let s = fs::read_to_string("input/2016/input_03.txt")?;
    match Triangles::parse(&s) {
        Ok(("", triangles)) => Ok(triangles),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type SideLength = u16;

#[derive(Debug, Copy, Clone)]
struct Triangle {
    lengths: [SideLength; 3],
}

impl Triangle {
    fn is_valid(&self) -> bool {
        let largest: SideLength = self.lengths.into_iter().max().unwrap();
        let smaller_sum = if self.lengths[0] == largest {
            self.lengths[1] + self.lengths[2]
        } else if self.lengths[1] == largest {
            self.lengths[0] + self.lengths[2]
        } else {
            self.lengths[0] + self.lengths[1]
        };
        largest < smaller_sum
    }

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                space0,
                parse_u16,
                space1,
                parse_u16,
                space1,
                parse_u16,
                space0,
                newline
            )),
            |(_, a, _, b, _, c, _, _)| Self{lengths: [a,b,c]}
        )(input)
    }
}

impl Display for Triangle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}\n", self.lengths[0], self.lengths[1], self.lengths[2])
    }
}




#[derive(Debug)]
struct Triangles(Vec<Triangle>);

impl Triangles {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(many0(Triangle::parse), |x| Self(x))(input)
    }
}


impl Display for Triangles {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for triangle in &self.0 {
            write!(f, "{}", triangle)?;
        }
        Ok(())
    }
}




fn part_a(triangles: &Triangles) {
    println!("\nPart a:");
    let possible_count = triangles.0.iter().filter(|t| t.is_valid()).count();
    println!("{} of the triangles are possible.", possible_count);
}


fn part_b(triangles: &Triangles) {
    println!("\nPart b:");
    let mut transposed_triangles = Vec::new();
    for chunk in &triangles.0.iter().chunks(3) {
        let mut iterator = chunk.into_iter();
        let a: [SideLength;3] = iterator.next().unwrap().lengths;
        let b: [SideLength;3] = iterator.next().unwrap().lengths;
        let c: [SideLength;3] = iterator.next().unwrap().lengths;
        assert!(iterator.next().is_none());
        transposed_triangles.push(Triangle{lengths: [a[0], b[0], c[0]]});
        transposed_triangles.push(Triangle{lengths: [a[1], b[1], c[1]]});
        transposed_triangles.push(Triangle{lengths: [a[2], b[2], c[2]]});
    }
    let possible_count = transposed_triangles.iter().filter(|t| t.is_valid()).count();
    println!("{} of the triangles are possible.", possible_count);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
