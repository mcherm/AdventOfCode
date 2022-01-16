use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use nom::sequence::tuple as nom_tuple;
use nom::character::complete::u32 as nom_value;
use nom::bytes::complete::tag as nom_tag;
use nom::multi::many0 as nom_many0;


type Dim = u32;

struct Box {
    dims: [Dim; 3]
}

impl Box {
    fn new(x: Dim, y: Dim, z: Dim) -> Self {
        Box{dims: [x,y,z]}
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_value,
            nom_tag("x"),
            nom_value,
            nom_tag("x"),
            nom_value,
            nom_tag("\n"),
        ))(input).map(|(rest, (x, _, y, _, z, _))| (rest, Box::new(x,y,z)))
    }
}

impl Display for Box {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}x{}", self.dims[0], self.dims[1], self.dims[2])
    }
}

fn parse_boxes(input: &str) -> nom::IResult<&str, Vec<Box>> {
    nom_many0(Box::parse)(input)
}


fn input() -> Result<Vec<Box>, io::Error> {
    let s = fs::read_to_string("input/2015/02/input.txt")?;
    match parse_boxes(&s) {
        Ok(("", boxes)) => Ok(boxes),
        Ok((_, _)) => panic!("Extra input"),
        Err(_) => panic!("Invalid input"),
    }
}

fn part_a(boxes: &Vec<Box>) -> Result<(), io::Error> {
    for b in boxes {
        println!("{}", b);
    }
    Ok(())
}

fn part_b(boxes: &Vec<Box>) -> Result<(), io::Error> {
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let s = input()?;
    part_a(&s)?;
    part_b(&s)?;
    Ok(())
}
