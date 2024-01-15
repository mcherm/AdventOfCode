use std::fmt::{Debug, Display, Formatter};
use std::collections::HashSet;
use anyhow;


// ======= Constants =======


// ======= Parsing =======

/// A point in 3d-space
#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct Point3D(usize,usize,usize);



/// A 3D cuboid.
#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct Brick {
    lower: Point3D, // lower bound (inclusive) for each dimension
    upper: Point3D, // upper bound (exclusive) for each dimension
}

/// A bunch of bricks.
#[derive(Debug, Clone)]
pub struct Pile {
    bricks: Vec<Brick>,
}

type Input = Pile;


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

impl Pile {
    /// Construct from a list of bricks.
    fn new(bricks: Vec<Brick>) -> Self {
        Self{bricks}
    }
}


mod parse {
    use super::{Input, Point3D, Brick, Pile};
    use std::fs;
    use nom;
    use nom::IResult;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_22.txt")?;
        match Pile::parse(&s) {
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
    }

    impl Pile {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::multi::many1(
                    nom::sequence::terminated(
                        Brick::parse,
                        nom::character::complete::line_ending,
                    )
                ),
                |bricks| Pile::new(bricks)
            )(input)
        }
    }

}


// ======= Compute =======

impl Debug for Point3D {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Point3D {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.0, self.1, self.2)
    }
}


impl Brick {
    /// Return true if p is in this brick.
    fn contains(&self, p: Point3D) -> bool {
        p.0 >= self.lower.0 && p.0 < self.upper.0 &&
        p.1 >= self.lower.1 && p.1 < self.upper.1 &&
        p.2 >= self.lower.2 && p.2 < self.upper.2
    }

    /// Returns an iterator over the points of this brick.
    fn points(&self) -> BrickPointIterator {
        BrickPointIterator::new(self)
    }

    /// Moves this Brick down dist spaces. It will panic if that brings it below
    /// zero.
    fn fall(&mut self, dist: usize) {
        // println!("Brick {:?} falling by {}", self, dist); // FIXME: Remove
        assert!(dist <= self.lower.2);
        self.lower.2 = self.lower.2 - dist;
        self.upper.2 = self.upper.2 - dist;
    }
}

impl Debug for Brick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Brick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Brick[{}:{}]", self.lower, self.upper)
    }
}

struct BrickPointIterator<'a> {
    brick: &'a Brick,
    next: Option<Point3D>
}

impl<'a> BrickPointIterator<'a> {
    fn new(brick: &'a Brick) -> Self {
        let next = if brick.lower == brick.upper {
            None
        } else {
            Some(brick.lower)
        };
        Self{brick, next}
    }
}

impl<'a> Iterator for BrickPointIterator<'a> {
    type Item = Point3D;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.next;
        if let Some(p) = result {
            self.next = if p.0 + 1 < self.brick.upper.0 {
                Some(Point3D(p.0 + 1, p.1, p.2))
            } else if p.1 + 1 < self.brick.upper.1 {
                Some(Point3D(self.brick.lower.0, p.1 + 1, p.2))
            } else if p.2 + 1 < self.brick.upper.2 {
                Some(Point3D(self.brick.lower.0, self.brick.lower.1, p.2 + 1))
            } else {
                None
            };
        }
        result
    }
}


impl Pile {

    /// Given a brick (presumably one that is in this Pile!) this returns a Vec of references
    /// to the bricks that hold is up. If it's not being held, it will return an empty Vec.
    fn supported_by(&self, brick: &Brick) -> Vec<Brick> {
        let mut result: Vec<Brick> = Vec::new();
        for p in brick.points() {
            if p.2 > 0 {
                let p_down = Point3D(p.0, p.1, p.2 - 1);
                for b in self.bricks.iter() {
                    if *b == *brick {
                        continue; // skip itself
                    }
                    if *b != *brick && b.contains(p_down) {
                        result.push(*b);
                    }
                }
            }
        }
        result
    }

    /// Given a brick (presumably one that is in this Pile!) this returns true if it
    /// can fall at least one space. Bricks can't fall below level 1 onto level 0 because
    /// that's how the problem is written.
    fn can_fall(&self, brick: &Brick) -> bool {
        brick.lower.2 > 1 && self.supported_by(brick).is_empty()
    }

    /// Makes all the bricks fall as far as they can.
    ///
    /// NOTE: There are surely faster ways. But I'm doing this the simplest way until that
    /// proves too slow.
    fn collapse(&mut self) {
        loop {
            let mut idx_of_brick_that_can_fall: Option<usize> = None;
            for (i, brick) in self.bricks.iter().enumerate() {
                if self.can_fall(brick) {
                    idx_of_brick_that_can_fall = Some(i);
                    break
                }
            }
            match idx_of_brick_that_can_fall {
                None => return, // nothing could move, so the whole function
                Some(i) => {
                    self.bricks.get_mut(i).unwrap().fall(1);
                }
            }
        }
    }

    /// Returns the list of bricks that are not currently the ONLY thing holding up some
    /// other brick.
    fn bricks_eligible_for_disintegration(&self) -> HashSet<Brick> {
        let mut result: HashSet<Brick> = self.bricks.iter().copied().collect();
        for (i, brick) in self.bricks.iter().enumerate() {
            let supports = self.supported_by(brick);
            println!("Brick {} {} is supported by {:?}", i, brick, supports); // FIXME: Remove
            if supports.len() == 1 {
                result.remove(&supports[0]);
            }
        }
        result
    }
}

// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    println!("{:?}", input);
    let mut pile = input.clone();
    pile.collapse();
    println!("Collapsed, that is:");
    println!("{:?}", pile);
    let eligible = pile.bricks_eligible_for_disintegration();
    println!("Eligible to disintegrate: {:?}", eligible);
    println!("There are {} eligible to disintegrate.", eligible.len());
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
