
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fs;
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
        combinator::map,
        character::complete::line_ending,
        sequence::{tuple, terminated},
        multi::many0,
    };
    use nom::character::complete::u32 as nom_Num;
    use std::collections::HashSet;
    use std::fmt::{Display, Formatter};


    pub fn input() -> Result<Object3D, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_18.txt")?;
        match Object3D::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct Point3D {
        x: u32,
        y: u32,
        z: u32,
    }


    #[derive(Debug)]
    pub struct Object3D {
        pt_set: HashSet<Point3D>
    }

    struct NeighborsIter {
        i: u8,
        p: Point3D,
    }


    impl Point3D {
        /// Construtor. Add 1 in all directions so we won't need to worry about a neighbor being
        /// out of bounds.
        fn new_offset(x: u32, y: u32, z: u32) -> Self {
            Point3D{x: x+1, y: y+1, z: z+1}
        }

        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            map(
                tuple((
                    nom_Num,
                    tag(","),
                    nom_Num,
                    tag(","),
                    nom_Num
                )),
                |(x,_,y,_,z)| Point3D::new_offset(x,y,z)
            )(input)
        }

        /// Returns an iterator of the points neighboring this one. 6, unless it's
        /// beside 0.
        fn neighbors(&self) -> impl Iterator<Item=Self> {
            NeighborsIter{i: 0, p: *self}
        }
    }

    impl Display for Point3D {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "({:2},{:2},{:2})", self.x, self.y, self.z)
        }
    }

    impl Object3D {
        /// Create an Object
        fn new(pts: Vec<Point3D>) -> Self {
            let pt_set: HashSet<Point3D> = pts.iter().copied().collect();
            assert!(pt_set.len() == pts.len()); // if not, there was a dup
            Object3D{pt_set}
        }


        /// Parses a newline-terminated list of LineSpecs
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            map(
                many0(terminated(Point3D::parse, line_ending)),
                |pts| Object3D::new(pts)
            )(input)
        }

        /// Finds the surface area
        pub fn surface_area(&self) -> u32 {
            self.pt_set.iter()
                .map(|p| p.neighbors().filter(|n| !self.pt_set.contains(n)).count() as u32)
                .sum()
        }
    }

    impl Iterator for NeighborsIter {
        type Item = Point3D;

        fn next(&mut self) -> Option<Self::Item> {
            self.i += 1;
            match self.i {
                0 => panic!(),
                1 => Some(Point3D{x: self.p.x - 1, ..self.p}),
                2 => Some(Point3D{y: self.p.y - 1, ..self.p}),
                3 => Some(Point3D{z: self.p.z - 1, ..self.p}),
                4 => Some(Point3D{x: self.p.x + 1, ..self.p}),
                5 => Some(Point3D{y: self.p.y + 1, ..self.p}),
                6 => Some(Point3D{z: self.p.z + 1, ..self.p}),
                _ => None,
            }
        }
    }
}



// ======= Part 1 Compute =======

mod compute {

}




// ======= main() =======

use crate::parse::{Object3D, input};


fn part_a(input: &Object3D) {
    println!("\nPart a:");
    println!("The surface area is {}", input.surface_area())
}


fn part_b(_input: &Object3D) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}


// ======= Tests =======

