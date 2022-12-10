
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
struct Grid {
    head: Coord,
    tail: Coord,
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

// FIXME: Remove
// impl From<Motion> for Coord {
//     fn from(motion: Motion) -> Self {
//         match motion.dir {
//             Dir::U => Self(0, motion.dist as Pos),
//             Dir::D => Self(0, -1 *(motion.dist as Pos)),
//             Dir::L => Self(-1 * (motion.dist as Pos), 0),
//             Dir::R => Self(motion.dist as Pos, 0),
//         }
//     }
// }

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

impl Grid {
    fn new() -> Self {
        let head = Coord(0,0);
        let tail = Coord(0,0);
        let mut tail_visited = BTreeSet::new();
        tail_visited.insert(tail);
        Grid{head, tail, tail_visited}
    }

    /// Applies a motion to this Grid.
    fn apply_motion(&mut self, motion: &Motion) {
        for _ in 0..motion.dist {
            // --- move the head ---
            self.head += motion.dir.into();
            // --- move the tail ---
            match self.head - self.tail {
                // still touching; tail doesn't move
                Coord(-1 ..= 1, -1 ..= 1) => {},
                // two steps in one direction
                Coord(-2, 0) => self.tail += Coord(-1, 0),
                Coord(2, 0) => self.tail += Coord(1, 0),
                Coord(0, -2) => self.tail += Coord(0, -1),
                Coord(0, 2) => self.tail += Coord(0, 1),
                // not touching
                Coord(-2, -1) => self.tail += Coord(-1, -1),
                Coord(-2, 1) => self.tail += Coord(-1, 1),
                Coord(2, -1) => self.tail += Coord(1, -1),
                Coord(2, 1) => self.tail += Coord(1, 1),
                Coord(-1, -2) => self.tail += Coord(-1, -1),
                Coord(1, -2) => self.tail += Coord(1, -1),
                Coord(-1, 2) => self.tail += Coord(-1, 1),
                Coord(1, 2) => self.tail += Coord(1, 1),
                // other moves should not be possible
                Coord(_, _) => panic!("Should not be possible after just one step."),
            }
            // --- update tail_visited ---
            self.tail_visited.insert(self.tail);
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
        for y in (min_x..=max_y).rev() {
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
    let mut grid = Grid::new();
    for motion in motions {
        grid.apply_motion(&motion);
    }
    println!("Head: {}", grid.head);
    println!("Tail: {}", grid.tail);
    println!("{}", Positions(&grid));
    println!("Which is a total of {} positions visited.", grid.count_visited());
    Ok(())
}


fn part_b(_input: &Vec<Motion>) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
