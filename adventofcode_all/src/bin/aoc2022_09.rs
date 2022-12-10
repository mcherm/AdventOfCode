
extern crate anyhow;
extern crate elsa;

use std::fs;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::newline,
    combinator::{value, map},
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u32 as nom_u32;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::ops::AddAssign;
use std::ops::Sub;


// ======= Parsing =======

fn input() -> Result<Vec<Motion>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_09.txt")?;
    match Motion::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}

type Dist = usize;

#[derive(Debug, Copy, Clone)]
enum Dir { U, D, L, R }

#[derive(Debug, Copy, Clone)]
struct Motion {
    dir: Dir,
    dist: Dist
}

impl Dir {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(Dir::U, tag("U")),
            value(Dir::D, tag("D")),
            value(Dir::L, tag("L")),
            value(Dir::R, tag("R")),
        ))(input)
    }
}

impl Display for Dir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {Dir::U => 'U', Dir::D => 'D', Dir::L => 'L', Dir::R => 'R', })
    }
}

impl Motion {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                Dir::parse,
                tag(" "),
                nom_u32,
            )),
            |(dir, _, num)| Motion {dir, dist: num as Dist}
        )(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }
}

impl Display for Motion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.dir, self.dist)
    }
}

// ======= Processing =======

type Pos = i32;

#[derive(Debug,  Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Coord(Pos,Pos);

#[derive(Debug)]
/// A "rope" is more like a chain -- it consists of a series of "knots" each of which
/// is at a location. The "head" is the first link and the "tail" is the last link.
struct Rope {
    links: Vec<Coord>
}

#[derive(Debug)]
struct Grid {
    rope: Rope,
    tail_visited: BTreeSet<Coord>,
}

impl From<Dir> for Coord {
    fn from(dir: Dir) -> Self {
        match dir {
            Dir::U => Self(0, 1),
            Dir::D => Self(0, -1),
            Dir::L => Self(-1, 0),
            Dir::R => Self(1, 0),
        }
    }
}


impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}


impl Sub for Coord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl AddAssign for Coord {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl Rope {
    fn new(len: usize) -> Self {
        assert!(len >= 1); // No ropes of length zero!
        Rope{links: vec![Coord(0,0); len]}
    }

    /// Returns the Coord of the tail of the rope.
    fn tail(&self) -> Coord {
        self.links[self.links.len() - 1]
    }

    /// Iterate through the links (from head to tail), with the ability to
    /// mutate each one (but NOT to mutate the list!).
    fn iter_mut(&mut self) -> impl Iterator<Item=&mut Coord> {
        self.links.iter_mut()
    }
}

impl Display for Rope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rope(")?;
        let mut it = self.links.iter();
        write!(f, "{}", it.next().unwrap())?;
        for link in it {
            write!(f, ", {}", link)?;
        }
        write!(f, ")")
    }
}


impl Grid {
    fn new(rope: Rope) -> Self {
        let mut tail_visited = BTreeSet::new();
        tail_visited.insert(rope.tail());
        Self{rope, tail_visited}
    }

    /// Applies a motion to this Grid.
    #[allow(overlapping_range_endpoints)]
    fn apply_motion(&mut self, motion: &Motion) {
        for _ in 0..motion.dist {
            let mut links = self.rope.iter_mut(); // mutable iterator of "&mut link"s
            // --- move the head (just by one step) ---
            let head = links.next().unwrap(); // we ARE guaranteed at least 1 link in a rope
            *head += motion.dir.into();
            // --- move the other links, each following the previous ---
            let mut prev = head;
            for link in links {
                match *prev - *link {
                    // still touching; tail doesn't move
                    Coord(-1 ..= 1, -1 ..= 1) => {},
                    // two steps in one direction
                    Coord(-2,  0) => *link += Coord(-1,  0),
                    Coord( 2,  0) => *link += Coord( 1,  0),
                    Coord( 0, -2) => *link += Coord( 0, -1),
                    Coord( 0,  2) => *link += Coord( 0,  1),
                    // not touching
                    Coord(-2 ..= -1, -2 ..= -1) => *link += Coord(-1, -1),
                    Coord(-2 ..= -1,  1 ..=  2) => *link += Coord(-1,  1),
                    Coord( 1 ..=  2, -2 ..= -1) => *link += Coord( 1, -1),
                    Coord( 1 ..=  2,  1 ..=  2) => *link += Coord( 1,  1),
                    // other moves should not be possible
                    Coord(_, _) => panic!(
                        "Should not be possible after just one step. Link {} follows {}",
                        *link, *prev
                    ),
                }
                prev = link;
            }
            // --- update tail_visited ---
            self.tail_visited.insert(*prev);
        }
    }

    /// Returns a count of the places the tail has visited
    fn count_visited(&self) -> usize {
        self.tail_visited.len()
    }
}


/// Define a wrapper used to support Display of positions in a grid.
struct Positions<'a>(&'a Grid);

impl<'a> Display for Positions<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (mut max_x, mut min_x, mut max_y, mut min_y) = (0, 0, 0, 0);
        for coord in self.0.tail_visited.iter() {
            if coord.0 < min_x {min_x = coord.0;}
            if coord.0 > max_x {max_x = coord.0;}
            if coord.1 < min_y {min_y = coord.1;}
            if coord.1 > max_y {max_y = coord.1;}
        }
        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                write!(f, "{}", if self.0.tail_visited.contains(&Coord(x,y)) {'#'} else {'.'})?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

// ======= main() =======

fn part_a(motions: &Vec<Motion>) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    let mut grid = Grid::new(Rope::new(2));
    for motion in motions {
        grid.apply_motion(&motion);
    }
    println!("Rope: {}", grid.rope);
    println!("{}", Positions(&grid));
    println!("Which is a total of {} positions visited.", grid.count_visited());
    Ok(())
}


fn part_b(motions: &Vec<Motion>) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    let mut grid = Grid::new(Rope::new(10));
    for motion in motions {
        grid.apply_motion(&motion);
    }
    println!("Rope: {}", grid.rope);
    println!("{}", Positions(&grid));
    println!("Which is a total of {} positions visited.", grid.count_visited());
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
