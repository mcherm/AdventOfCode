
extern crate anyhow;

use std::fs;
use anyhow::Error;
use std::cmp::Ordering;
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


const PRINT_EVERY_N_STEPS: usize = 1;


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



#[allow(dead_code)] // FIXME: Remove
fn solve_with_astar<S: State>(initial_state: &mut S) -> Option<Vec<GridStep>> {
    println!("Starting state: {:}", initial_state);

    // visited_from maps from a state (which we have considered and explored its neighbors) to how
    // we got there: (prev_state, prev_step, step_count).
    let mut visited_from: HashMap<S, Option<(S, GridStep, usize)>> = HashMap::new();

    // queue is a collection of states we will consider. What we store is
    //   StateToConsider. The queue is kept sorted by sort_score()
    let mut queue: VecDeque<StateToConsider<S>> = VecDeque::new();
    queue.push_back(StateToConsider{state: initial_state.clone(), prev: None});

    let mut loop_ctr: usize = 0;
    loop {
        loop_ctr += 1;

        match queue.pop_front() {
            None => {
                return None; // we ran out of places to go. Guess it's not solvable!
            }
            Some(StateToConsider{state, prev}) => {
                let step_count = match prev {
                    None => 0,
                    Some((_,_,step_count)) => step_count,
                };

                // What to do if we visited this before?
                if let Some(prev) = visited_from.get(&state) {
                    let been_here_same_or_better = match prev {
                        None => true,
                        Some((_visited_state, _grid_step, prev_steps)) => *prev_steps <= step_count, // FIXME: think carefully about off-by-one error
                    };
                    if been_here_same_or_better {
                        // been here before, and it took same-or-fewer steps, so don't bother to re-examine
                        continue;
                    }
                }


                // -- Every so often, print it out so we can monitor progress --
                if loop_ctr % PRINT_EVERY_N_STEPS == 0 {
                    if PRINT_EVERY_N_STEPS > 1 || !visited_from.contains_key(&state.clone()) {
                        state.show_state(loop_ctr, step_count, &visited_from, &queue);
                    }
                }

                // -- mark that we have (or now will!) visited this one --
                assert!(!visited_from.contains_key(&state)); // FIXME: Assert that we haven't been here before
                visited_from.insert(state.clone(), prev);

                // -- try each step from here --
                for step in state.avail_steps() {
                    let next_state: S = state.enact_step(step);
                    let next_steps = step_count + 1;

                    // -- maybe we've already been to this one --
                    let earlier_visit = visited_from.get(&next_state);
                    // -- decide whether to put next_state onto the queue... --
                    let try_next_state = match earlier_visit {
                        None => true, // never seen it, certainly want to try it out
                        Some(None) => false, // the earlier visit was our starting position
                        Some(Some((_, _, earlier_step_count))) => {
                            match earlier_step_count.cmp(&next_steps) {
                                Ordering::Greater => panic!("Found faster way to a visited site."),
                                Ordering::Equal => false, // been here same distance; don't try it
                                Ordering::Less => false, // been here better distance; don't try it
                            }
                        }
                    };

                    if try_next_state {
                        if next_state.is_winning() {
                            println!("\nSOLVED!! {}", next_state);
                            let winning_steps = Some({
                                let mut steps: Vec<GridStep> = Vec::new();
                                steps.push(*step);
                                let mut state_var: &S = &state;
                                while let Some((prev_state, prev_step, _)) = visited_from.get(&state_var).unwrap() {
                                    steps.push((*prev_step).clone());
                                    state_var = prev_state;
                                }
                                steps.reverse();
                                steps
                            });
                            return winning_steps
                        } else {
                            // -- Actually add this to the queue --
                            let to_insert = StateToConsider{
                                state: next_state,
                                prev: Some((state.clone(), *step, step_count + 1))
                            };
                            let insert_idx = queue.partition_point(|x| x.sort_score() < to_insert.sort_score());
                            queue.insert(insert_idx, to_insert);
                        }
                    }
                }
            }
        }
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
