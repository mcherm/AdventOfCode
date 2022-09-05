
extern crate anyhow;

use std::fs;
use anyhow::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::collections::BTreeMap;
use advent_lib::astar::{
    solve_with_astar, State,
    grid::{Coord, GridVec, GridMove, taxicab_dist, moves_from}
};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::map,
    multi::many1,
    sequence::terminated,
};


const PRINT_EVERY_N_MOVES: usize = 0;


fn input() -> Result<Grid, Error> {
    let s = fs::read_to_string("input/2016/input_24.txt")?;
    match Grid::parse(&s) {
        Ok(("", grid)) => Ok(grid),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}




type PointNum = u8;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum Cell {
    Wall,
    Open,
    Point(PointNum),
}


#[derive(Debug, Eq, PartialEq)]
struct Grid {
    nodes: GridVec<Cell>,
    points: BTreeMap<PointNum,Coord>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct RobotState<'a> {
    grid: &'a Grid,
    goal: &'a Coord,
    robot_pos: Coord,
}



impl Cell {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map(tag("#"), |_| Cell::Wall),
            map(tag("."), |_| Cell::Open),
            map(tag("0"), |_| Cell::Point(0)),
            map(tag("1"), |_| Cell::Point(1)),
            map(tag("2"), |_| Cell::Point(2)),
            map(tag("3"), |_| Cell::Point(3)),
            map(tag("4"), |_| Cell::Point(4)),
            map(tag("5"), |_| Cell::Point(5)),
            map(tag("6"), |_| Cell::Point(6)),
            map(tag("7"), |_| Cell::Point(7)),
            map(tag("8"), |_| Cell::Point(8)),
            map(tag("9"), |_| Cell::Point(9)),
        ))(input)
    }
}

impl Grid {
    fn parse_line(input: &str) -> IResult<&str, Vec<Cell>> {
        terminated(
            many1(Cell::parse),
            line_ending
        )(input)
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            many1(Self::parse_line),
            |data: Vec<Vec<Cell>>| Grid::from_vec2d(data)
        )(input)
    }

    /// Constructor which works on a 2 dimensional vector. Panics if the vector isn't rectangular
    /// or if it isn't at least 1x1. Also panics if it finds there is a duplicate PointNum or
    /// the PointNum 0 is missing.
    fn from_vec2d(data: Vec<Vec<Cell>>) -> Self {
        let nodes = GridVec::from_vec2d(&data);

        let mut points = BTreeMap::new();
        for c in nodes.iter_indexes() {
            let cell = nodes.get(&c);
            match cell {
                Cell::Point(point_num) => {
                    assert!(!points.contains_key(point_num));
                    points.insert(*point_num, c);
                },
                _ => {}, // ignore anything else
            }
        }
        assert!(points.contains_key(&0));

        Grid{nodes, points}
    }


    /// Returns the dimensions of the grid.
    fn size(&self) -> Coord {
        self.nodes.size()
    }


    /// Returns a list of the PointNums appearing in the grid. Panics if any PointNum is
    /// not unique. The PointNums will be in sorted order.
    fn get_points(&self) -> Vec<PointNum> {
        let mut answer: Vec<PointNum> = self.points.keys().map(|x| *x).collect();
        answer.sort();
        answer
    }

    /// Returns the state where the robot is on the specified coord and wants to go to
    /// the specified location.
    fn robot_at_point<'a>(&'a self, start: &Coord, goal: &'a Coord) -> RobotState<'a> {
        let robot_pos = *start;
        let grid = self;
        RobotState{grid, goal, robot_pos}
    }

    /// Find the number of moves needed to go between the two PointNums. (Panics if they aren't
    /// valid PointNums in this diagram.) Returns count_of_moves or panics if there is no
    /// way to get between those two points.
    fn find_pairwise_distance(&self, p1: PointNum, p2: PointNum) -> usize {
        let start: Coord = *self.points.get(&p1).unwrap();
        let goal: Coord = *self.points.get(&p2).unwrap();
        let initial_state = self.robot_at_point(&start, &goal);
        if let Some(solution) = solve_with_astar(&initial_state, PRINT_EVERY_N_MOVES) {
            solution.len()
        } else {
            panic!("No path between points {} and {}.", p1, p2);
        }
    }
}

impl<'a> Display for RobotState<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "@({},{})", self.robot_pos.0, self.robot_pos.1)
    }
}

impl<'a> Hash for RobotState<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.robot_pos.hash(state);
    }
}

impl<'a> State for RobotState<'a> {
    type TMove = GridMove;

    fn is_winning(&self) -> bool {
        self.robot_pos == *self.goal
    }

    fn min_moves_to_win(&self) -> usize {
        taxicab_dist(self.robot_pos, *self.goal)
    }

    fn avail_moves(&self) -> Vec<Self::TMove> {
        moves_from(self.robot_pos, self.grid.size())
    }

    fn enact_move(&self, mv: &Self::TMove) -> Self {
        RobotState{robot_pos: mv.to(), ..*self}
    }
}


impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Open => write!(f, "."),
            Cell::Wall => write!(f, "#"),
            Cell::Point(point_num) => write!(f, "{}", point_num),
        }
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for c in self.nodes.iter_indexes() {
            if c.0 == 0 {
                writeln!(f)?; // newline before each row
            }
            write!(f, "{}", self.nodes.get(&c))?;
        }
        writeln!(f)
    }
}




#[allow(dead_code)]
fn part_a(grid: &Grid) {
    println!("\nPart a:");
    println!("Grid = {}", grid);
    let points = grid.get_points();
    println!("points = {:?}", points);
    for (p1_pos, p1) in points.iter().enumerate() {
        for p2 in points[(p1_pos + 1)..].iter() {
            println!("From {} to {} takes {} moves.", p1, p2, grid.find_pairwise_distance(*p1, *p2));
        }
    }
}



fn part_b(_grid: &Grid) {
    println!("\nPart b:");
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}



// ==========================================================================================

#[cfg(test)]
mod tests {
    use super::*;

}
