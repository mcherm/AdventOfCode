
extern crate anyhow;

use std::fs;
use anyhow::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::collections::BTreeMap;
use itertools::Itertools;
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
use traveling_salesman::{Distances, solve_with_brute_force};


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

    /// Returns a boolean indicating if the given location is a wall.
    fn is_wall(&self, coord: Coord) -> bool {
        match self.nodes.get(&coord) {
            Cell::Wall => true,
            _ => false,
        }
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
        moves_from(self.robot_pos, self.grid.size()).into_iter()
            .filter(|mv| !self.grid.is_wall(mv.to()))
            .collect_vec()
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


/// This module contains the logic for finding an OPTIMAL solution to a variant of the
/// traveling salesman problem. The variant from the standard problem is that the problem
/// specifies a starting location (but not an ending one) and it does NOT require returning
/// to the start position.
///
/// My intention is to implement a couple of different algorithms. Normally traveling
/// salesman is solved with heuristics, but I need optimal solutions so I'll be using
/// approaches like brute force and branch-and-prune.
mod traveling_salesman {
    use itertools::Itertools;
    use std::fmt::{Display, Formatter};
    use super::PointNum;

    type NodeId = PointNum;


    /// Contains the distances between a set of nodes. The nodes are identified by integers
    /// starting from 0. Distances are usize.
    pub struct Distances {
        size: NodeId,
        dist: Vec<usize>, // there are size*size elements
    }

    /// Contains a specific path.
    #[derive(Debug)]
    pub struct Path {
        moves: usize,
        nodes: Vec<NodeId>,
    }


    impl Distances {
        /// Returns a new Distances which has the given size and with all distances set
        /// to zero.
        pub fn new_zeros(size: NodeId) -> Self {
            let dist = vec![0; usize::from(size) * usize::from(size)];
            Distances{size, dist}
        }

        /// Returns the distance from n1 to n2. Panics if either is not a node in this.
        pub fn dist(&self, n1: NodeId, n2: NodeId) -> usize {
            assert!(n1 < self.size);
            assert!(n2 < self.size);
            *self.dist.get(self.idx(n1,n2)).unwrap()
        }

        /// Given two nodes, this finds the index into dist for their distance. Order matters;
        /// swapping n1 and n2 will give two different indexes (although we ensure that those
        /// locations will always store the same value).
        fn idx(&self, n1: NodeId, n2: NodeId) -> usize {
            usize::from(n1) * usize::from(self.size) + usize::from(n2)
        }

        /// Sets the distance between n1 and n2 to be d. Distances are symmetric, so this
        /// sets both directions to the same value.
        pub fn set_dist(&mut self, n1: NodeId, n2: NodeId, d: usize) {
            let idx1 = self.idx(n1,n2);
            let idx2 = self.idx(n2,n1);
            self.dist[idx1] = d;
            self.dist[idx2] = d;
        }
    }


    impl Path {
        fn new(nodes: Vec<NodeId>, moves: usize) -> Self {
            Path{nodes: nodes, moves}
        }

        /// Returns the number of moves for this path
        pub fn moves(&self) -> usize {
            self.moves
        }
    }

    impl Display for Path {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            assert!(self.nodes.len() >= 1);
            write!(f, "{}", self.nodes.first().unwrap())?;
            for node in self.nodes.iter().skip(1) {
                write!(f, "->{}", node)?;
            }
            Ok(())
        }
    }


    /// Solver that uses brute force. It returns the minimum path distance starting from
    /// node zero and visiting all other nodes.
    ///
    /// FIXME: This should return the particular order
    pub fn solve_with_brute_force(distances: &Distances) -> Path {
        let size = distances.size;
        let start_node: NodeId = 0;
        let mut best_path: Option<Path> = None;
        for rest_of_path in ((start_node + 1)..size).permutations(usize::from(size - 1)) {
            let dist = distances.dist(0, *rest_of_path.first().unwrap()) +
                rest_of_path.windows(2).map(|pair| distances.dist(pair[0],pair[1])).sum::<usize>();
            let make_some_path = || {
                let mut nodes = Vec::with_capacity(usize::from(size));
                nodes.push(start_node);
                nodes.extend(rest_of_path);
                Some(Path::new(nodes, dist))
            };
            match &best_path {
                None => {
                    best_path = make_some_path()
                },
                Some(prev_best_path) => {
                    if dist < prev_best_path.moves {
                        best_path = make_some_path()
                    }
                }
            }
        }

        match best_path {
            Some(path) => path,
            None => panic!("Should have found at least one path!"),
        }
    }
}




fn part_a(grid: &Grid) {
    println!("\nPart a:");
    println!("Grid = {}", grid);
    let points = grid.get_points();
    if points.is_empty() {
        panic!("No numbered points in the maze.");
    }
    if points.len() != usize::from(*points.last().unwrap()) + 1 { // points are known to be unique and sorted
        panic!("Numbered points in the maze are skipping some value.");
    }
    let size_as_point_num = PointNum::try_from(points.len()).unwrap();
    let mut distances = Distances::new_zeros(size_as_point_num);
    for (p1_pos, p1) in points.iter().enumerate() {
        for p2 in points[(p1_pos + 1)..].iter() {
            let dist = grid.find_pairwise_distance(*p1, *p2);
            println!("From {} to {} takes {} moves.", p1, p2, dist);
            distances.set_dist(*p1, *p2, dist);
        }
    }

    let min_path = solve_with_brute_force(&distances);
    println!("The minimal path is {} steps with path {}.", min_path.moves(),  min_path);
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


// 250 is too low.
