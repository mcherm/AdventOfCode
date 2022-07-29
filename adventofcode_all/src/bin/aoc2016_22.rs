
extern crate anyhow;

use std::fs;
use anyhow::Error;
use std::cmp::max;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use itertools::Itertools;


use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{multispace1, line_ending, not_line_ending},
    combinator::map,
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u16 as nom_u16;


fn input() -> Result<Grid, Error> {
    let s = fs::read_to_string("input/2016/input_22.txt")?;
    match GridLoader::parse(&s) {
        Ok(("", grid_loader)) => Ok(grid_loader.make_grid()),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



#[derive(Copy, Clone, Debug)]
struct Node {
    x: usize,
    y: usize,
    #[allow(dead_code)]
    size: usize,
    used: usize,
    avail: usize,
}

struct GridLoader {
    nodes: Vec<Node>,
}

type Coord = (usize, usize);

struct Grid {
    nodes: HashMap<Coord,Node>,
    size: (usize,usize)
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    Up, Down, Left, Right
}

#[derive(Copy, Clone, Debug)]
struct Step {
    from: Coord,
    dir: Direction,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct NodeSpace {
    used: usize,
    avail: usize,
}

#[derive(Clone, Debug)]
struct State {
    nodes: HashMap<Coord,NodeSpace>,
    goal_data: Coord,
    avail_steps: Vec<Step>,
}



fn nom_usize(input: &str) -> IResult<&str, usize> {
    map(
        nom_u16,
        |x| usize::from(x)
    )(input)
}

fn nom_line(input: &str) -> IResult<&str, &str> {
    terminated( not_line_ending, line_ending )(input)
}


impl Node {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("/dev/grid/node-x"),
                nom_usize,
                tag("-y"),
                nom_usize,
                multispace1,
                nom_usize,
                tag("T"),
                multispace1,
                nom_usize,
                tag("T"),
                multispace1,
                nom_usize,
                tag("T"),
                multispace1,
                nom_usize,
                tag("%"),
                line_ending,
            )),
            |(_, x, _, y, _, size, _, _, used, _, _, avail, _, _, _, _, _)| Node{x, y, size, used, avail}
        )(input)
    }
}


impl GridLoader {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                nom_line,
                nom_line,
                many0(Node::parse),
            )),
            |(_, _, nodes,)| Self{ nodes }
        )(input)
    }

    fn make_grid(&self) -> Grid {
        let mut max_x = 0;
        let mut max_y = 0;
        let mut nodes: HashMap<(usize,usize),Node> = HashMap::new();
        for node in self.nodes.iter() {
            max_x = max(max_x, node.x);
            max_y = max(max_y, node.y);
            nodes.insert((node.x, node.y), node.clone());
        }
        assert_eq!( nodes.len(), (max_x + 1) * (max_y + 1) ); // Guarantees we got all of them
        Grid{nodes, size: (max_x + 1, max_y + 1)}
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
    fn count_viable_pairs(&self) -> usize {
        let mut count = 0;
        for y1 in 0..self.size.1 {
            for x1 in 0..self.size.0 {
                let n1: &Node = self.nodes.get(&(x1,y1)).unwrap();
                if n1.used != 0 {
                    for y2 in 0..self.size.1 {
                        for x2 in 0..self.size.0 {
                            if (x1,y1) != (x2,y2) {
                                let n2: &Node = self.nodes.get(&(x2,y2)).unwrap();
                                if n1.used <= n2.avail {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        count
    }

    /// Given a Grid, this generates the starting State
    fn get_initial_state(&self) -> State {
        let nodes: HashMap<Coord,NodeSpace> = self.nodes.iter()
            .map(|(coord, node)| (*coord, NodeSpace{used: node.used, avail: node.avail})).collect();
        let goal_data = (self.size.0 - 1, 0); // top-right corner
        let avail_moves = self.nodes.iter().flat_map(|(coord, _)| {
            neighbor_dirs(*coord, self.size).into_iter()
                .map(|dir| Step{from: *coord, dir})
                .filter(|s| s.is_legal(&nodes))
        }).collect();
        State{nodes, goal_data, avail_steps: avail_moves }
    }
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


impl Step {
    /// Returns the place that the step ends up at.
    fn to(&self) -> Coord {
        match self.dir {
            Direction::Up => (self.from.0, self.from.1 - 1),
            Direction::Down => (self.from.0, self.from.1 + 1),
            Direction::Left => (self.from.0 - 1, self.from.1),
            Direction::Right => (self.from.0 + 1, self.from.1),
        }
    }

    /// Returns true if the step is allowed given state constraints; false otherwise.
    /// Bases that on the provided set of node sizes.
    fn  is_legal(&self, nodes: &HashMap<Coord,NodeSpace>) -> bool {
        let fr = nodes.get(&self.from).unwrap();
        let to = nodes.get(&self.to()).unwrap();
        fr.avail > 0 && to.avail >= fr.used && to.used == 0
    }

    /// Given a move, this returns a move that goes from the destination to the start.
    fn inverse(&self) -> Self {
        Self{from: self.to(), dir: self.dir.inverse()}
    }
}


impl Display for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})->({},{})", self.from.0, self.from.1, self.to().0, self.to().1)
    }
}


impl Display for NodeSpace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:2}T/{:2}T", self.used, self.used + self.avail)
    }
}


impl State {
    /// This indicates whether a state is a "winning" state.
    fn is_winning(&self) -> bool {
        self.goal_data == (0,0)
    }

    /// This is used within Eq and Hash. It returns a sorted list of the content of Node.
    fn get_sorted_nodes(&self) -> Vec<(Coord,NodeSpace)> {
        self.nodes.iter().map(|x| (x.0.clone(), x.1.clone())).sorted().collect()
    }

    /// Returns the new State achieved by performing this step (which ought to be one of the
    /// valid Steps for this State).
    fn enact_step(&self, step: &Step, size: Coord) -> State {
        // --- set the nodes ---
        let mut nodes = self.nodes.clone();
        let from_node = nodes.get_mut(&step.from).unwrap();
        let amt_moved: usize = from_node.used;
        from_node.avail += amt_moved;
        from_node.used -= amt_moved;
        assert_eq!(from_node.used, 0);
        let to_node = nodes.get_mut(&step.to()).unwrap();
        assert!(to_node.avail >= amt_moved);
        to_node.avail -= amt_moved;
        to_node.used += amt_moved;

        // --- set the goal_data ---
        let goal_data = if step.from == self.goal_data {
            step.to()
        } else {
            self.goal_data
        };

        // --- set the avail_moves ---
        // copy existing avail_moves EXCEPT those that enter or leave step.from or step.to()
        let mut avail_moves: Vec<Step> = self.avail_steps.iter()
            .filter(|x| x.from != step.from && x.to() != step.from && x.to() != step.from && x.to() != step.to())
            .copied()
            .collect();
        // re-consider everything that enters or leaves step.from or step.to()
        let moves_out: Vec<Step> = neighbor_dirs(step.from, size).into_iter()
            .map(|dir| Step{from: step.from, dir}) // steps from the "from" location
            .chain(
                neighbor_dirs(step.to(), size).into_iter()
                    .map(|dir| Step{from: step.to(), dir}) // steps from the "to" location
                    .filter(|s| s.to() != step.from) // except the one going to "from" location; we already got the reverse of that
            ).collect();
        avail_moves.extend(moves_out.iter().filter(|m| m.is_legal(&nodes))); // add the legal "out" steps
        avail_moves.extend(moves_out.iter().map(|m| m.inverse()).filter(|m| m.is_legal(&nodes))); // add the legal "in" steps

        // --- return the result ---
        State{nodes, goal_data, avail_steps: avail_moves }
    }
}


/// States are equal if their nodes and goal_data is equal.
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.goal_data == other.goal_data && self.get_sorted_nodes() == other.get_sorted_nodes()
    }
}

impl Eq for State {}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.goal_data.hash(state);
        self.get_sorted_nodes().hash(state);
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let max_x = self.nodes.keys().map(|x| x.0).max().unwrap() + 1;
        let max_y = self.nodes.keys().map(|x| x.1).max().unwrap() + 1;
        writeln!(f)?;
        for y in 0..max_y {
            for x in 0..max_x {
                write!(f, "{}{:1}  ",
                    self.nodes.get(&(x,y)).unwrap(),
                    if (x,y) == self.goal_data {'G'} else {' '}
                )?;
            }
            writeln!(f)?;
        }
        write!(f, "[")?;
        for step in &self.avail_steps {
            write!(f, "{} ", step)?;
        }
        writeln!(f, "]")
    }
}


fn part_a(grid: &Grid) {
    println!("\nPart a:");
    let pair_count = grid.count_viable_pairs();
    println!("There are {} viable pairs.", pair_count);
}



fn part_b(grid: &Grid) {
    println!("\nPart b:");
    let initial_state = grid.get_initial_state();
    println!("State: {:}", initial_state);
    let mut visited: HashSet<State> = HashSet::new();
    visited.insert(initial_state.clone());
    let mut queue: VecDeque<State> = VecDeque::new();
    queue.push_back(initial_state);

    loop {
        match queue.pop_front() {
            None => {
                println!("Could not find a solution.");
                break;
            }
            Some(state) => {
                println!("Moving from state {}", state);
                for step in &state.avail_steps {
                    println!("    Consider step {:}", step);
                    let next_state = state.enact_step(step, grid.size);
                    println!("       puts us in state {:}", next_state);
                    if !visited.contains(&next_state) {
                        if next_state.is_winning() {
                            println!("SOLVED!! {}", next_state);
                            queue.clear(); // so we will exit the bigger loop
                            break;
                        }
                        visited.insert(next_state.clone());
                        queue.push_back(next_state);
                    }
                }
            }
        }
    }
    println!("Done.");
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
