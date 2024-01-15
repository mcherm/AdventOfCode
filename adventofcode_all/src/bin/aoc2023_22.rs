use std::fmt::{Debug};
use anyhow;


// ======= Constants =======


// ======= Parsing =======

/// A point in 3d-space
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Point3D(usize,usize,usize);



/// A 3D cuboid.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Brick {
    lower: Point3D, // lower bound (inclusive) for each dimension
    upper: Point3D, // upper bound (exclusive) for each dimension
}

type Input = Vec<Brick>;

impl Brick {
    /// Given any 2 points that are opposite and are part of the brick, this
    /// constructs the Brick.
    fn from_inner_pts(p1: Point3D, p2: Point3D) -> Self {
        let x_min = std::cmp::min(p1.0, p2.0);
        let x_max = std::cmp::max(p1.0, p2.0) + 1;
        let y_min = std::cmp::min(p1.1, p2.1);
        let y_max = std::cmp::max(p1.1, p2.1) + 1;
        let z_min = std::cmp::min(p1.2, p2.2);
        let z_max = std::cmp::max(p1.2, p2.2) + 1;
        let lower = Point3D(x_min, y_min, z_min);
        let upper = Point3D(x_max, y_max, z_max);
        Brick{lower,upper}
    }
}


mod parse {
    use super::{Input, Point3D, Brick};
    use std::fs;
    use nom;
    use nom::IResult;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_22.txt")?;
        match Brick::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    /// Parse a usize. (I KNOW this is running on a 64-bit Mac.)
    fn nom_num(input: &str) -> IResult<&str, usize> {
        nom::character::complete::u64(input)
            .map(|(s,n): (&str,u64)| (s, n as usize))
    }

    impl Point3D {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom_num,
                    nom::bytes::complete::tag(","),
                    nom_num,
                    nom::bytes::complete::tag(","),
                    nom_num,
                )),
                |(x,_,y,_,z)| Point3D(x,y,z)
            )(input)
        }
    }

    impl Brick {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    Point3D::parse,
                    nom::bytes::complete::tag("~"),
                    Point3D::parse,
                )),
                |(p1,_,p2)| Brick::from_inner_pts(p1,p2)
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
    println!("{:?}", input);
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
