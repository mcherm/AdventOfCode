#![allow(dead_code)] // FIXME: Remove this

extern crate anyhow;

use std::fs;
use anyhow::Error;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use itertools::Itertools;


use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending},
    combinator::map,
    multi::many1,
    sequence::terminated,
};




fn input() -> Result<Grid, Error> {
    let s = fs::read_to_string("input/2016/input_24.txt")?;
    match Grid::parse(&s) {
        Ok(("", grid)) => Ok(grid),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}




type Coord = (usize, usize);

type PointNum = u8;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum Cell {
    Wall,
    Open,
    Point(PointNum),
}

/// This is a data structure for storing items indexed by a Coord. It provide O(1) lookup
/// and also provides Eq and Hash.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct GridVec<T: Eq + Hash + Clone> {
    size: Coord,
    data: Vec<T>,
}

#[derive(Debug)]
struct Grid {
    nodes: GridVec<Cell>,
}

#[allow(dead_code)] // FIXME: Remove
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Direction {
    Up, Down, Left, Right
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct GridStep {
    from: Coord,
    dir: Direction,
}



trait State : Display + Clone + Eq + Hash {
    fn is_winning(&self) -> bool;
    fn min_steps_to_win(&self) -> usize;
    fn avail_steps(&self) -> &Vec<GridStep>;
    fn enact_step(&self, step: &GridStep) -> Self;

    // FIXME: This is probably temporary
    fn show_state(
        &self,
        loop_ctr: usize,
        step_count: usize,
        visited_from: &HashMap<Self, Option<(Self, GridStep, usize)>>,
        queue: &VecDeque<StateToConsider<Self>>
    ) {
        println!(
            "\nAt {} went {} steps; at least {} to go for a total of {}:{:}. Have visited {} states and have {} queued.",
            loop_ctr,
            step_count,
            self.min_steps_to_win(),
            step_count + self.min_steps_to_win(),
            self,
            visited_from.len(),
            queue.len()
        );
    }
}


/// This is what we insert into the queue while doing an A* search. It has a State and the
/// number of steps it took to get there. They are sortable (because the queue is kept
/// sorted) and the sort order is by step_count + state.min_steps_to_win()
struct StateToConsider<S: State> {
    state: S, // the state we will consider
    prev: Option<(S, GridStep, usize)>, // Some(the previous state, the step from it, and the num_steps to get here) or None if this is the FIRST state.
}




fn nom_line(input: &str) -> IResult<&str, &str> {
    terminated( not_line_ending, line_ending )(input)
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
    /// or if it isn't at least 1x1.
    fn from_vec2d(data: Vec<Vec<Cell>>) -> Self {
        let nodes = GridVec::from_vec2d(&data);
        Grid{nodes}
    }
}


/// This returns a list of the directions reachable from this coordinate.
fn neighbor_dirs(coord: Coord, size: Coord) -> Vec<Direction> {
    let mut answer = Vec::with_capacity(4);
    if coord.0 > 0 {
        answer.push(Direction::Left);
    }
    if coord.1 > 0 {
        answer.push(Direction::Up);
    }
    if coord.1 + 1 < size.1 {
        answer.push(Direction::Down);
    }
    if coord.0 + 1 < size.0 {
        answer.push(Direction::Right);
    }
    answer
}



impl Grid {
}



impl Direction {
    /// Returns the opposite of this direction
    fn inverse(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}


/// This is used just to return an iterator of Coords.
struct GridVecCoordIter {
    size: Coord,
    next_val: Option<Coord>,
}

impl Iterator for GridVecCoordIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        let answer = self.next_val;
        match self.next_val {
            None => {
                self.next_val = Some((0,0));
            },
            Some((x, y)) => {
                self.next_val = if x + 1 < self.size.0 {
                    Some((x + 1, y))
                } else if y + 1 < self.size.1 {
                    Some((0, y + 1))
                } else {
                    None
                };
            },
        }
        answer
    }
}

impl<T: Eq + Hash + Clone> GridVec<T> {
    /// Construct from a vec (which must be rectangular and at least 1x1 in size or it panics).
    fn from_vec2d(data_vec: &Vec<Vec<T>>) -> Self {
        let height = data_vec.len();
        assert!(height >= 1);
        let width = data_vec.first().unwrap().len();
        assert!(width >= 1);
        let size = (width, height);
        assert!(data_vec.iter().all(|x| x.len() == width));

        let mut data = Vec::with_capacity(width * height);
        for row in data_vec.iter() {
            for cell in row.iter() {
                data.push(cell.clone());
            }
        }

        GridVec{size, data}
    }

    fn coord_to_index(&self, coord: &Coord) -> usize {
        if coord.0 >= self.size.0 || coord.1 >= self.size.1 {
            panic!("Coord {:?} is out of bounds.", coord);
        }
        coord.1 * self.size.0 + coord.0
    }

    fn index_to_coord(&self, idx: usize) -> Coord {
        (idx % self.size.0, idx / self.size.0)
    }

    /// This is used to iterate through the indexes of the coord. It happens to
    /// loop through x faster than y.
    fn iter_indexes(&self) -> impl Iterator<Item = Coord> {
        GridVecCoordIter{size: self.size, next_val: Some((0,0))}
    }

    /// This returns (in O(1) time) the item in the GridVec at the given coord. If
    /// the coord is not within size, then this panics.
    fn get(&self, coord: &Coord) -> &T {
        let idx = self.coord_to_index(coord);
        self.data.get(idx).unwrap()
    }

    /// This returns (in O(1) time) a mutable reference to the item in the GridVec at the
    /// given coord. If the coord is not within size then this panics.
    fn get_mut(&mut self, coord: &Coord) -> &mut T {
        let idx = self.coord_to_index(coord);
        self.data.get_mut(idx).unwrap()
    }
}


impl<T: Eq + Hash + Clone> IntoIterator for GridVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}


/// This converts from an iterator of (Coord,T) into a GridVec<T>. The current version
/// will panic if there isn't exactly one value for each Coord.
impl<T: Eq + Hash + Clone + Debug> FromIterator<(Coord, T)> for GridVec<T> {
    fn from_iter<U: IntoIterator<Item=(Coord, T)>>(iter: U) -> Self {
        let staging: Vec<(Coord, T)> = iter.into_iter().collect_vec();
        let max_x = staging.iter().map(|(c,_)| c.0).max().unwrap_or(0);
        let max_y = staging.iter().map(|(c,_)| c.1).max().unwrap_or(0);
        let size: Coord = (max_x + 1, max_y + 1);

        let get_value = |idx: usize| {
            let coord = (idx % size.0, idx / size.0);
            for (c,v) in &staging {
                if *c == coord {
                    return v.clone()
                }
            }
            panic!("No value provided for coordinate {:?}", coord)
        };

        let indexes = 0..(size.0 * size.1);
        let data: Vec<T> = indexes.map(get_value).collect();

        GridVec{size, data}
    }
}



impl<T: State> StateToConsider<T> {
    fn sort_score(&self) -> usize {
        let step_count = match self.prev {
            None => 0,
            Some((_,_,count)) => count
        };
        let answer = step_count + self.state.min_steps_to_win();
        answer
    }
}


impl<S: State> Debug for StateToConsider<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateToConsider[{}] worth {}", self.state, self.sort_score())
    }
}





#[allow(dead_code)]
fn part_a(_grid: &Grid) {
    println!("\nPart a:");

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
