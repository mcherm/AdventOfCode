use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use nom::sequence::tuple as nom_tuple;
use nom::character::complete::u32 as nom_value;
use nom::bytes::complete::tag as nom_tag;
use nom::multi::many0 as nom_many0;
use nom::character::complete::newline as nom_newline;


type Dim = u32;

struct Box {
    dims: [Dim; 3]
}

impl Box {
    fn new(x: Dim, y: Dim, z: Dim) -> Self {
        Box{dims: [x,y,z]}
    }

    fn smallest_side_area(&self) -> Dim {
        self.dims.iter().product::<Dim>() / self.dims.iter().max().unwrap()
    }

    fn surface_area(&self) -> Dim {
        let [l,w,h] = self.dims;
        2*l*w + 2*w*h + 2*h*l
    }

    fn paper_needed(&self) -> Dim {
        self.surface_area() + self.smallest_side_area()
    }

    fn ribbon_needed(&self) -> Dim {
        let small_side_perimeter: Dim = 2 * (self.dims.iter().sum::<Dim>() - self.dims.iter().max().unwrap());
        let bow: Dim = self.dims.iter().product();
        small_side_perimeter + bow
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_value,
            nom_tag("x"),
            nom_value,
            nom_tag("x"),
            nom_value,
            nom_newline,
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
    let paper_to_order: Dim = boxes.iter().map(|x| x.paper_needed()).sum();
    println!("Need to order {} square feet of paper.", paper_to_order);
    Ok(())
}

fn part_b(boxes: &Vec<Box>) -> Result<(), io::Error> {
    let ribbon: Dim = boxes.iter().map(|x| x.ribbon_needed()).sum();
    println!("Need to order {} feet of ribbon.", ribbon);
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let s = input()?;
    part_a(&s)?;
    part_b(&s)?;
    Ok(())
}
