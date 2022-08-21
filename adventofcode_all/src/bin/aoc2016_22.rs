
extern crate anyhow;

use std::{fs, io};
use anyhow::Error;
use std::cmp::{max, min, Ordering};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::BufRead;
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


const VERIFY_AVAIL_STEP_LOGIC: bool = false;
const PRINT_EVERY_N_STEPS: usize = 1;
const PRINT_WHEN_CLASSIFYING: bool = false;


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
    size: Coord,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Direction {
    Up, Down, Left, Right
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct GridStep {
    from: Coord,
    dir: Direction,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct NodeSpace {
    used: usize,
    avail: usize,
}

/// This is a data structure for storing items indexed by a Coord. It provide O(1) lookup
/// and also provides Eq and Hash.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct GridVec<T: Eq + Hash + Clone> {
    size: Coord,
    data: Vec<T>,
}

/// A generalized State implementation
#[derive(Clone, Debug, Eq)]
struct GenState {
    nodes: GridVec<NodeSpace>,
    goal_data_loc: Coord,
    avail_steps: Vec<GridStep>,
}

/// A specialized version of GenState that is optimized for the case where there is a single
/// location that can be moved around (except to some places) and that's the only way to
/// move the goal data around. It's a weird special case, but one which arose in my own
/// data and frankly the solve time is unreasonable if this DOESN'T apply.
#[derive(Clone, Debug, Eq)]
struct SingleSpaceState {
    base: GenState,
    min_blocker_content: usize,
    open_space_loc: Coord,
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
    } // the default is to print nothing
}

/// This is what we insert into the queue while doing an A* search. It has a State and the
/// number of steps it took to get there. They are sortable (because the queue is kept
/// sorted) and the sort order is by step_count + state.min_steps_to_win()
struct StateToConsider<S: State> {
    state: S, // the state we will consider
    prev: Option<(S, GridStep, usize)>, // Some(the previous state, the step from it, and the num_steps to get here) or None if this is the FIRST state.
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
        let mut nodes: HashMap<Coord,Node> = HashMap::new();
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


#[derive(Debug, Clone)]
struct NotASpecialGridError;

impl Display for NotASpecialGridError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not a special grid error")
    }
}

impl std::error::Error for NotASpecialGridError {
}


enum NodeClassification {
    Goal, // The node containing the goal data; must qualify as a FillerNode.
    Empty, // The node with no data; must qualify as a FillerNode.
    Filler, // Big enough to move data into, but too big to combine
    Blocker, // So big it can't move into any Filler spot
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

    fn goal_data_loc(&self) -> Coord {
        (self.size.0 - 1, 0)
    }

    /// Given a Grid, this generates the starting State
    fn get_initial_genstate(&self) -> GenState {
        let nodes: GridVec<NodeSpace> = self.nodes.iter()
            .map(|(coord, node)| (*coord, NodeSpace{used: node.used, avail: node.avail})).collect();
        let avail_steps = derive_avail_steps(&nodes);
        GenState{nodes, goal_data_loc: self.goal_data_loc(), avail_steps }
    }

    /// Attempts to find the SingleSpaceState -- but returns None if the grid doesn't conform
    /// to the expectations of that.
    fn get_initial_singlespacestate(&self) -> Option<SingleSpaceState> {
        match self.attempt_classification() {
            Err(NotASpecialGridError) => None,
            Ok((open_space_loc, min_blocker_content)) => {
                let base = self.get_initial_genstate();
                Some(SingleSpaceState{base, min_blocker_content, open_space_loc})
            }
        }
    }

    /// This attempts to classify every cell of the grid in the special set of restricted
    /// rules that I observed for my own data and which allows for a SingleFreeSpaceWithWallsState
    /// to be used. If successful it return Ok((open_space_loc, min_blocker_content)), otherwise it returns NotASpecialGridError
    fn attempt_classification(&self) -> Result<(Coord, usize), NotASpecialGridError> {
        let state = self.get_initial_genstate();

        // --- values we observed in the data named foo_seen ---
        // --- constraints we will apply named foo_rule ---
        let goal_data_size_seen = state.nodes.get(&state.goal_data_loc).used;
        let zero_used_count_seen = state.nodes.data.iter()
            .filter(|x| x.used == 0)
            .count();
        if zero_used_count_seen != 1 {
            return Err(NotASpecialGridError);
        }
        let open_space_loc = state.nodes.iter_indexes()
            .filter(|c| state.nodes.get(c).used == 0)
            .nth(0).unwrap();
        let max_nonzero_avail_seen = state.nodes.data.iter()
            .filter(|x| x.used != 0)
            .map(|x| x.avail)
            .max().ok_or(NotASpecialGridError)?;
        let max_avail_rule: usize = max_nonzero_avail_seen;
        let min_nonzero_size_seen = state.nodes.data.iter()
            .filter(|x| x.used != 0)
            .map(|x| x.used)
            .min().ok_or(NotASpecialGridError)?;
        if max_nonzero_avail_seen >= min_nonzero_size_seen {
            return Err(NotASpecialGridError);
        }
        let min_filler_content_rule: usize = min_nonzero_size_seen;
        let max_filler_capacity_rule: usize = min_filler_content_rule * 2 - 1;
        let min_blocker_content_rule: usize = max_filler_capacity_rule + 1;
        let max_filler_content_seen = state.nodes.data.iter()
            .filter(|x| x.used < min_blocker_content_rule)
            .map(|x| x.used)
            .max().ok_or(NotASpecialGridError)?;
        let max_filler_content_rule: usize = max_filler_content_seen;
        let min_filler_capacity_rule: usize = max_filler_content_rule;

        // --- these asserts are checking my logic in this method NOT the input data ---
        assert!(max_avail_rule < min_filler_content_rule); // no filler can be moved into anywhere but the empty space
        assert!(max_filler_capacity_rule < 2 * min_filler_content_rule); // two fillers can never fit into any filler location
        assert!(max_filler_content_rule <= min_filler_capacity_rule); // any filler can be moved into any other filler location
        assert!(min_blocker_content_rule > max_filler_capacity_rule); // no blocker can be moved into a filler

        let valid_filler = |used, avail|
            used >= min_filler_content_rule &&
            used <= max_filler_content_rule &&
            used + avail >= min_filler_capacity_rule &&
            used + avail <= max_filler_capacity_rule &&
            avail <= max_avail_rule;

        let valid_empty = |avail|
            avail >= min_filler_capacity_rule &&
            avail <= max_filler_capacity_rule;

        let valid_blocker = |used, avail|
            used >= min_blocker_content_rule &&
            avail <= max_avail_rule;

        let classify = |c: &Coord| -> Result<NodeClassification, NotASpecialGridError> {
            let node_space: &NodeSpace = state.nodes.get(c);
            if *c == state.goal_data_loc {
                match node_space {
                    NodeSpace{used, avail} if valid_filler(*used, *avail) => Ok(NodeClassification::Goal),
                    _ => Err(NotASpecialGridError),
                }
            } else {
                match node_space {
                    NodeSpace{used: 0, avail} if valid_empty(*avail) => Ok(NodeClassification::Empty),
                    NodeSpace{used, avail} if valid_filler(*used, *avail) => Ok(NodeClassification::Filler),
                    NodeSpace{used, avail} if valid_blocker(*used, *avail) => Ok(NodeClassification::Blocker),
                    _ => Err(NotASpecialGridError),
                }
            }
        };

        if PRINT_WHEN_CLASSIFYING {
            println!("goal_data_size_seen = {}", goal_data_size_seen);
            println!("zero_used_count_seen = {}", zero_used_count_seen);
            println!("open_space_loc = {:?}", open_space_loc);
            println!("max_nonzero_avail_seen = {}", max_nonzero_avail_seen);
            println!("max_avail_rule = {}", max_avail_rule);
            println!("min_nonzero_size_seen = {}", min_nonzero_size_seen);
            println!("min_filler_content_rule = {}", min_filler_content_rule);
            println!("max_filler_capacity_rule = {}", max_filler_capacity_rule);
            println!("min_blocker_content_rule = {}", min_blocker_content_rule);
            println!("max_filler_content_seen = {}", max_filler_content_seen);
            println!("max_filler_content_rule = {}", max_filler_content_rule);
            println!("min_filler_capacity_rule = {}", min_filler_capacity_rule);
            for c in state.nodes.iter_indexes() {
                if c.0 == 0 {
                    println!(); // newline at the start of each row
                }
                let ch = match classify(&c)? {
                    NodeClassification::Goal => 'X',
                    NodeClassification::Empty => '.',
                    NodeClassification::Filler => 'o',
                    NodeClassification::Blocker => 'H',
                };
                print!("{}", ch);
            }
            println!(); // newline at the end of the grid
        } else {
            for c in state.nodes.iter_indexes() {
                classify(&c)?;
            }
        }

        Ok((open_space_loc, min_blocker_content_rule))
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
/// FIXME: A better version might use default values for missing items and would avoid O(n^2) performance
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


impl GridStep {
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
    fn  is_legal(&self, nodes: &GridVec<NodeSpace>) -> bool {
        let fr = nodes.get(&self.from);
        let to = nodes.get(&self.to());
        fr.used > 0 && to.avail >= fr.used
    }

    /// Given a move, this returns a move that goes from the destination to the start.
    fn inverse(&self) -> Self {
        Self{from: self.to(), dir: self.dir.inverse()}
    }
}


impl Display for GridStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})->({},{})", self.from.0, self.from.1, self.to().0, self.to().1)
    }
}


impl Display for NodeSpace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:3}T/{:3}T", self.used, self.used + self.avail)
    }
}


/// One might want to look at the diagonal lines of coords that are within a certain rectangle.
/// That's what this function is for. It is passed a Coord which represents the outer bound
/// of the rectangle, and it returns a Vec of the diagonals. It does NOT include the diagonal
/// that contains the bound itself.
/// FIXME: Could use a better explanation. Also, maybe should return an iterable not a Vec.
fn diagonals(bound: Coord) -> Vec<Vec<Coord>> {
    // each diagonal has a certain taxi_distance from the origin
    let mut answer: Vec<Vec<Coord>> = Vec::new();
    for taxi_dist in 0..(bound.0 + bound.1) {
        let lowbound = if taxi_dist > bound.1 {taxi_dist - bound.1} else {0};
        let highbound = min(taxi_dist, bound.0);
        let mut diagonal: Vec<Coord> = Vec::new();
        for x in lowbound..=highbound {
            diagonal.push((x, taxi_dist - x))
        }
        answer.push(diagonal);
    }
    answer
}



/// This the slow but sure way to find the available moves for a given set of nodes.
/// It finds them but we really only use it to find the initial
/// set of moves because most of the time it's more efficient to only tweak the list
/// of moves slightly since only 2 nodes have changed.
fn derive_avail_steps(nodes: &GridVec<NodeSpace>) -> Vec<GridStep> {
    let indexes = 0..(nodes.size.0 * nodes.size.1);
    indexes.flat_map(|idx| {
        let coord = nodes.index_to_coord(idx);
        neighbor_dirs(coord, nodes.size).into_iter()
            .map(move |dir| GridStep {from: coord, dir})
            .filter(|s| s.is_legal(&nodes))
    }).collect()
}


impl GenState {
    #[allow(unused)]
    /// A simple heuristic: returns the taxicab distance from here to the goal.
    fn taxicab_distance_from_goal_data_to_corner(&self) -> usize {
        self.goal_data_loc.0 + self.goal_data_loc.1
    }


    #[allow(unused)]
    /// In this heuristic we return the taxicab distance from here to the goal PLUS
    /// one for each diagonal where every cell has less available space than the size
    /// of the goal data.
    fn taxicab_plus_moving(&self) -> usize {
        let mut answer = 0;
        let goal_data_size = self.nodes.get(&self.goal_data_loc).used;
        for diagonal in diagonals(self.goal_data_loc) {
            answer += 1; // have to move the goal data into this diagonal
            let largest_avail = diagonal.iter().map(|x| self.nodes.get(x).avail).max().unwrap();
            if largest_avail < goal_data_size {
                answer += 1; // have to move something out to make space for it
            }
        }
        answer
    }
}


impl State for GenState {
    /// This indicates whether a state is a "winning" state.
    fn is_winning(&self) -> bool {
        self.goal_data_loc == (0,0)
    }

    /// This is the heuristic used for the A* search algorithm -- it returns an estimate of
    /// the minimum possible steps needed to win (this estimate can be too low, but must
    /// never be too high). To make it pluggable, we have a set of different heuristics
    /// and this is coded to call one of them.
    fn min_steps_to_win(&self) -> usize {
        self.taxicab_plus_moving()
    }

    fn avail_steps(&self) -> &Vec<GridStep> {
        &self.avail_steps
    }


    /// Returns the new State achieved by performing this step (which ought to be one of the
    /// valid Steps for this State).
    fn enact_step(&self, step: &GridStep) -> Self {
        // --- set the nodes ---
        let mut nodes = self.nodes.clone();
        let from_node = nodes.get_mut(&step.from);
        let amt_moved: usize = from_node.used;
        from_node.avail += amt_moved;
        from_node.used -= amt_moved;
        assert_eq!(from_node.used, 0);
        let to_node = nodes.get_mut(&step.to());
        assert!(to_node.avail >= amt_moved);
        to_node.avail -= amt_moved;
        to_node.used += amt_moved;

        // --- set the goal_data ---
        let goal_data_loc = if step.from == self.goal_data_loc {
            step.to()
        } else {
            self.goal_data_loc
        };

        // --- set the avail_moves ---
        // copy existing avail_moves EXCEPT those that enter or leave step.from or step.to()
        let mut avail_steps: Vec<GridStep> = self.avail_steps.iter()
            .filter(|x| x.from != step.from && x.to() != step.from && x.from != step.to() && x.to() != step.to())
            .copied()
            .collect();
        // re-consider everything that enters or leaves step.from or step.to()
        let steps_out: Vec<GridStep> = neighbor_dirs(step.from, self.nodes.size).into_iter()
            .map(|dir| GridStep {from: step.from, dir}) // steps from the "from" location
            .chain(
                neighbor_dirs(step.to(), self.nodes.size).into_iter()
                    .map(|dir| GridStep {from: step.to(), dir}) // steps from the "to" location
                    .filter(|s| s.to() != step.from) // except the one going to "from" location; we already got the reverse of that
            ).collect();
        avail_steps.extend(steps_out.iter().filter(|m| m.is_legal(&nodes))); // add the legal "out" steps
        avail_steps.extend(steps_out.iter().map(|m| m.inverse()).filter(|m| m.is_legal(&nodes))); // add the legal "in" steps

        if VERIFY_AVAIL_STEP_LOGIC {
            let mut sorted_steps: Vec<GridStep> = avail_steps.clone();
            let mut correct_steps: Vec<GridStep> = derive_avail_steps(&nodes);
            sorted_steps.sort();
            correct_steps.sort();
            assert_eq!(sorted_steps, correct_steps);
        }

        // --- return the result ---
        GenState{nodes, goal_data_loc, avail_steps }
    }
}




impl SingleSpaceState {
    /// Number of steps the open space must take to get beside the goal_data without
    /// moving the goal_data
    fn min_steps_open_space_must_take(&self) -> usize {
        let goal_data_loc = self.base.goal_data_loc;
        let mut target_locs: Vec<Coord> = Vec::new(); // places open_space might need to go to
        if goal_data_loc.0 > 0 {
            target_locs.push((goal_data_loc.0 - 1, goal_data_loc.1));
        }
        if goal_data_loc.1 > 0 {
            target_locs.push((goal_data_loc.0, goal_data_loc.1 - 1));
        }
        if target_locs.len() == 0 {
            return 0; // we're already sitting at (0,0)
        } else {
            let min_dist_to_target_loc = target_locs.iter().map(|target_loc| {
                target_loc.0.abs_diff(self.open_space_loc.0) + // x distance to get there
                    target_loc.1.abs_diff(self.open_space_loc.1) + // y distance to get there
                    if target_loc.0 == 0 && self.open_space_loc.0 == 0 {2} else {0} + // go-around-penalty
                    if target_loc.1 == 0 && self.open_space_loc.1 == 0 {2} else {0} // go-around-penalty
            }).min().unwrap();
            min_dist_to_target_loc
        }
    }

    // FIXME: This might be temporary
    /// Returns a dummy SingleSpaceState with the specified goal_data_loc and open_space_loc.
    fn dummy_with_space_at(&self, open_space_loc: Coord) -> Self {
        SingleSpaceState{open_space_loc,  ..self.clone()}
    }
}


impl State for SingleSpaceState {
    /// This indicates whether a state is a "winning" state.
    fn is_winning(&self) -> bool {
        self.base.is_winning()
    }

    /// This is the heuristic used for the A* search algorithm -- it returns an estimate of
    /// the minimum possible steps needed to win (this estimate can be too low, but must
    /// never be too high). To make it pluggable, we have a set of different heuristics
    /// and this is coded to call one of them.
    fn min_steps_to_win(&self) -> usize {
        // FIXME: The below is correct. But not working.
        1 + (self.base.taxicab_distance_from_goal_data_to_corner() - 1) * 5 +
            self.min_steps_open_space_must_take()
        // FIXME: The below is what I'm trying right now.
        // self.min_steps_open_space_must_take()
    }

    fn avail_steps(&self) -> &Vec<GridStep> {
        self.base.avail_steps()
    }


    /// Returns the new State achieved by performing this step (which ought to be one of the
    /// valid Steps for this State).
    fn enact_step(&self, step: &GridStep) -> Self {
        let base = self.base.enact_step(step);
        let min_blocker_content = self.min_blocker_content;

        // --- set open_space_loc ---
        let open_space_loc = if step.to() == self.open_space_loc {
            step.from
        } else {
            self.open_space_loc
        };

        // --- return the result ---
        SingleSpaceState{base, min_blocker_content, open_space_loc}
    }


    // FIXME: This might be temporary
    fn show_state(
        &self,
        loop_ctr: usize,
        step_count: usize,
        visited_from: &HashMap<Self, Option<(Self, GridStep, usize)>>,
        queue: &VecDeque<StateToConsider<Self>>)
    {
        println!(
            "\nAt {} went {} steps; at least {} to go for a total of {}.",
            loop_ctr,
            step_count,
            self.min_steps_to_win(),
            step_count + self.min_steps_to_win()
        );
        for c in self.base.nodes.iter_indexes() {
            if c.0 == 0 {
                println!(); // newline before each line
            }
            let ch = match c {
                c if c == self.open_space_loc => '.',
                c if c == self.base.goal_data_loc => 'X',
                c if visited_from.contains_key(&self.dummy_with_space_at(c)) => '*',
                c if queue.iter().any(|x| {
                    x.state.base.goal_data_loc == self.base.goal_data_loc &&
                        x.state.open_space_loc == c
                }) => '&',
                _ => '#',
            };
            print!("{}", ch);
        }
        println!(); // newline after last line
        println!(
            "Have visited {} states and have {} queued.",
            visited_from.len(),
            queue.len()
        );
    }

}


/// States are equal if their nodes and goal_data is equal.
impl PartialEq for GenState {
    fn eq(&self, other: &Self) -> bool {
        self.goal_data_loc == other.goal_data_loc && self.nodes == other.nodes
    }
}


impl PartialEq for SingleSpaceState {
    fn eq(&self, other: &Self) -> bool {
        // ONLY the locations of these two spots matter in equality testing
        self.base.nodes.size.eq(&other.base.nodes.size) &&
            self.open_space_loc.eq(&other.open_space_loc) &&
            self.base.goal_data_loc.eq(&other.base.goal_data_loc)
    }
}


impl Hash for GenState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.goal_data_loc.hash(state);
        self.nodes.hash(state);
    }
}


impl Hash for SingleSpaceState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // ONLY the locations of these two spots matter in equality testing
        self.base.nodes.size.hash(state);
        self.open_space_loc.hash(state);
        self.base.goal_data_loc.hash(state);
    }
}

impl Display for GenState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for c in self.nodes.iter_indexes() {
            if c.0 == 0 {
                writeln!(f)?; // newline before each row
            }
            write!(f, "{:1}{}{:1} ",
                   if c == self.goal_data_loc {'['} else {' '},
                   self.nodes.get(&c),
                   if c == self.goal_data_loc {']'} else {' '},
            )?;
        }
        writeln!(f)?; // newline after last row
        write!(f, "[")?;
        for step in &self.avail_steps {
            write!(f, "{} ", step)?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}


impl Display for SingleSpaceState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for c in self.base.nodes.iter_indexes() {
            if c.0 == 0 {
                writeln!(f)?; // newline before each row
            }
            let is_space = c == self.open_space_loc;
            let is_goal = c == self.base.goal_data_loc;
            let is_wall = self.base.nodes.get(&c).used >= self.min_blocker_content;
            let ch = if is_goal {'X'} else if is_space {'.'} else if is_wall {'#'} else {'o'};
            write!(f, "{}", ch)?;
        }
        writeln!(f)?; // newline after last row
        write!(f, "[")?;
        for step in &self.base.avail_steps {
            write!(f, "{} ", step)?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}


impl<T: State> StateToConsider<T> {
    fn sort_score(&self) -> usize {
        let step_count = match self.prev {
            None => 0,
            Some((_,_,count)) => count
        };
        step_count + self.state.min_steps_to_win()
    }
}



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

                // -- Every so often, print it out so we can monitor progress --
                if loop_ctr % PRINT_EVERY_N_STEPS == 0 {
                    if PRINT_EVERY_N_STEPS > 1 || !visited_from.contains_key(&state.clone()) {
                        state.show_state(loop_ctr, step_count, &visited_from, &queue);
                    }
                }

                // -- mark that we have (or now will!) visited this one --
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

fn find_winning_steps(grid: &Grid) -> Option<Vec<GridStep>> {
    match grid.get_initial_singlespacestate() {
        Some(mut initial_state) => {
            solve_with_astar(&mut initial_state)
        },
        None => {
            let mut initial_state = grid.get_initial_genstate();
            solve_with_astar(&mut initial_state)
        }
    }
}



#[allow(dead_code)]
fn part_a(grid: &Grid) {
    println!("\nPart a:");
    let pair_count = grid.count_viable_pairs();
    println!("There are {} viable pairs.", pair_count);
}



#[allow(dead_code)]
fn part_b(grid: &Grid) {
    println!("\nPart b:");

    let winning_steps = find_winning_steps(grid);
    match winning_steps {
        None => println!("Could not find a solution."),
        Some(steps) => {
            print!("Winning steps are: ");
            for step in &steps {
                print!("{}, ", step);
            }
            println!();
            println!("Which took {} steps.", steps.len());
        }

    }
    println!("Done.");
}

fn read_line() -> String {
    io::stdin().lock()
        .lines()
        .next()
        .expect("there was no next line")
        .expect("the line could not be read")
}


fn print_for_play(state: &SingleSpaceState) {
    let open_space_loc = state.open_space_loc;
    let goal_data_loc = state.base.goal_data_loc;
    for c in state.base.nodes.iter_indexes() {
        if c.0 == 0 {
            println!(); // newline at the start of each row
        }
        print!(
            "{:1}{:3}+{:3}{:1}",
            match c {
                _ if c == goal_data_loc => '[',
                _ if c == open_space_loc => '(',
                _ => ' ',
            },
            state.base.nodes.get(&c).used,
            state.base.nodes.get(&c).avail,
            match c {
                _ if c == goal_data_loc => ']',
                _ if c == open_space_loc => ')',
                _ => ' ',
            },
        );
    }
    println!(); // newline at the end of the grid
}

enum ManualAction<'a> {
    Quit,
    Echo(&'a str),
    Move(Direction),
}

/// Plays the game interactively
#[allow(dead_code)]
fn play_manually(grid: &Grid) {
    match grid.get_initial_singlespacestate() {
        Some(initial_state) => {
            let mut step_count: usize = 0;
            let mut state = initial_state.clone();
            loop {
                println!("After {} steps:", step_count);
                println!();
                print_for_play(&state);
                let input = read_line();
                let manual_action: ManualAction = match input.as_str() {
                    "q" => ManualAction::Quit,
                    "w" => ManualAction::Move(Direction::Up),
                    "s" => ManualAction::Move(Direction::Down),
                    "a" => ManualAction::Move(Direction::Left),
                    "d" => ManualAction::Move(Direction::Right),
                    other => ManualAction::Echo(other),
                };
                match manual_action {
                    ManualAction::Quit => {
                        println!("Exiting.");
                        break;
                    },
                    ManualAction::Echo(input) => {
                        println!("Command '{}' is unknown.", input);
                    },
                    ManualAction::Move(dir) => {
                        let step = GridStep{from: state.open_space_loc, dir}.inverse();
                        println!("Step {}", step);
                        state = state.enact_step(&step);
                        step_count += 1;
                    },
                }
            }
        },
        None => {
            println!("NOT A SINGLE SPACE GAME");
        }
    }
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    play_manually(&data); // FIXME: Restore?
    // part_a(&data); // FIXME: Restore
    // part_b(&data); // FIXME: Restore
    Ok(())
}



// ==========================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagonals() {
        let result = diagonals((4,2));
        assert_eq!(result, vec![
            vec![(0,0)],
            vec![(0,1), (1,0)],
            vec![(0,2), (1,1), (2,0)],
            vec![(1,2), (2,1), (3,0)],
            vec![(2,2), (3,1), (4,0)],
            vec![(3,2), (4,1)],
            // vec![(4,2)],
        ]);
    }

    #[test]
    fn test_read() {
        let size = (3,2);
        let mut nodes: HashMap<Coord,Node> = HashMap::new();
        nodes.insert((0,0), Node{x: 0, y: 0, size: 6, used: 1, avail: 5});
        nodes.insert((1,0), Node{x: 1, y: 0, size: 6, used: 2, avail: 4});
        nodes.insert((2,0), Node{x: 2, y: 0, size: 6, used: 3, avail: 3});
        nodes.insert((0,1), Node{x: 0, y: 1, size: 5, used: 0, avail: 5});
        nodes.insert((1,1), Node{x: 1, y: 1, size: 5, used: 4, avail: 1});
        nodes.insert((2,1), Node{x: 2, y: 1, size: 5, used: 5, avail: 0});
        let grid = Grid{nodes, size};

        let genstate = grid.get_initial_genstate();
        assert_eq!(genstate.goal_data_loc, (2,0));
        assert_eq!(genstate.nodes.get(&(0,0)).used, 1);
        assert_eq!(genstate.nodes.get(&(1,0)).used, 2);
        assert_eq!(genstate.nodes.get(&(2,0)).used, 3);
        assert_eq!(genstate.nodes.get(&(0,1)).used, 0);
        assert_eq!(genstate.nodes.get(&(1,1)).used, 4);
        assert_eq!(genstate.nodes.get(&(2,1)).used, 5);
    }

}


// 297 too big
