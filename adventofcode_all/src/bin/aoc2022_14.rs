
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
use std::cmp::{min, max};


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
        if points.len() < 2 {
            return Err(anyhow!("Line must have at least 2 points"));
        }
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
        nom::multi::many1( nom::sequence::terminated(Self::parse, line_ending) )(input)
    }

    /// Draws this on a grid's cells, setting the appropriate cells to "Wall".
    fn draw(&self, size: &Point, cells: &mut Vec<GridCell>) {
        fn set_wall(size: &Point, cells: &mut Vec<GridCell>, p: &Point) {
            cells[grid_idx(size, p)] = GridCell::Wall;
        }
        // draw line segments
        for pair in self.points.windows(2) {
            let start = pair[0];
            let end = pair[1];
            if start.0 == end.0 {
                // vertical line
                let x = start.0;
                for y in min(start.1, end.1) ..= max(start.1, end.1) {
                    set_wall(size, cells, &Point(x,y));
                }
            } else if start.1 == end.1 {
                // horizontal line
                let y = start.1;
                for x in min(start.0, end.0) ..= max(start.0, end.0) {
                    set_wall(size, cells, &Point(x,y));
                }
            } else {
                panic!("Diagonal line");
            }
        }
    }
}


// ======= Compute =======

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GridCell { Empty, Wall, Sand }


#[derive(Debug)]
struct Grid {
    size: Point,
    cells: Vec<GridCell>,
}

/// Returns the index into a vector for a grid of size size. Assumes everything is the right
/// size or this will panic.
fn grid_idx(size: &Point, p: &Point) -> usize {
    assert!(p.0 < size.0);
    assert!(p.1 < size.1);
    (p.1 as usize) * (size.0 as usize) + (p.0 as usize)
}


impl Grid {
    fn new(lines: &Vec<LineSpec>) -> Self {
        assert!(lines.len() > 0);
        assert!(lines[0].points.len() > 1);
        let max_x = lines.iter().map(|line| line.points.iter().map(|p| p.0).max().unwrap()).max().unwrap();
        let max_y = lines.iter().map(|line| line.points.iter().map(|p| p.1).max().unwrap()).max().unwrap();
        let size = Point(max_x + 2, max_y + 2); // ensure there's 1 extra space
        let mut cells = vec![GridCell::Empty; (size.0 * size.1) as usize];
        for line in lines.iter() {
            line.draw(&size, &mut cells);
        }
        Grid{size, cells}
    }

    fn idx(&self, p: Point) -> usize {
        grid_idx(&self.size, &p)
    }

    fn get(&self, p: Point) -> GridCell {
        *self.cells.get(self.idx(p)).unwrap()
    }

    fn set(&mut self, p: Point, val: GridCell) {
        let idx = self.idx(p);
        *self.cells.get_mut(idx).unwrap() = val;
    }

    /// This returns where a piece of sand added at Point(500,0) will come to rest, or None
    /// if it will fall into the void.
    fn sand_resting_place(&self) -> Option<Point> {
        let mut s = Point(500,0); // set the starting point for the sand
        loop {
            if s.1 + 1 == self.size.1 {
                return None; // it fell to infinity
            }
            assert!(s.1 < self.size.1); // there's enough space it should never go out of bounds
            let below = Point(s.0, s.1 + 1);
            if self.get(below) == GridCell::Empty {
                s = below;
                continue;
            }
            let below_left = Point(s.0 - 1, s.1 + 1);
            if self.get(below_left) == GridCell::Empty {
                s = below_left;
                continue;
            }
            let below_right = Point(s.0 + 1, s.1 + 1);
            if self.get(below_right) == GridCell::Empty {
                s = below_right;
                continue;
            }
            // if we get here, it must be in its resting place
            return Some(s);
        }
    }

    /// This adds sand until it no longer fits. It returns the count of sand added.
    fn pour_sand(&mut self) -> usize {
        let mut count = 0;
        loop {
            let rest_at = self.sand_resting_place();
            match rest_at {
                Some(p) => {
                    count += 1;
                    self.set(p, GridCell::Sand)
                },
                None => {
                    return count;
                },
            }
        }
    }
}

impl Display for GridCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            GridCell::Empty => ".",
            GridCell::Wall => "#",
            GridCell::Sand => "o",
        };
        write!(f, "{}", ch)
    }
}


impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                write!(f, "{}", self.get(Point(x,y)))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}


// ======= main() =======

fn part_a(input: &Vec<LineSpec>) {
    println!("\nPart a:");
    let mut grid = Grid::new(input);
    println!("{}", grid);
    println!();
    println!("After pouring:");
    let count = grid.pour_sand();
    println!("{}", grid);
    println!();
    println!("That was {} grains of sand.", count);
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
