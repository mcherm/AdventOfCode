
extern crate anyhow;

use std::{fs, io};
use anyhow::Error;
use std::cmp::{max, min};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::BufRead;
use advent_lib::astar::{
    State, StateToConsider, solve_with_astar,
    grid::{GridVec, GridMove, Coord, Direction, neighbor_dirs},
};


use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{multispace1, line_ending, not_line_ending},
    combinator::map,
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u16 as nom_u16;


const PLAY_MANUAL_GAME: bool = false;
const VERIFY_AVAIL_STEP_LOGIC: bool = false;
const PRINT_EVERY_N_MOVES: usize = 1000;
const PRINT_WHEN_CLASSIFYING: bool = false;
const VERBOSE_STATE: bool = false;


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


struct Grid {
    nodes: HashMap<Coord,Node>,
    size: Coord,
}


#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct NodeSpace {
    used: usize,
    avail: usize,
}

/// Returns whether a given move is legal for a given arrangement of nodes.
fn is_legal(mv: &GridMove, nodes: &GridVec<NodeSpace>) -> bool {
    let fr = nodes.get(&mv.from());
    let to = nodes.get(&mv.to());
    fr.used > 0 && to.avail >= fr.used
}


/// A generalized State implementation
#[derive(Clone, Debug, Eq)]
struct GenState {
    nodes: GridVec<NodeSpace>,
    goal_data_loc: Coord,
    avail_moves: Vec<GridMove>,
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
        let avail_moves = derive_avail_moves(&nodes);
        GenState{nodes, goal_data_loc: self.goal_data_loc(), avail_moves }
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
        let zero_used_count_seen = state.nodes.iter()
            .filter(|x| x.used == 0)
            .count();
        if zero_used_count_seen != 1 {
            return Err(NotASpecialGridError);
        }
        let open_space_loc = state.nodes.iter_indexes()
            .filter(|c| state.nodes.get(c).used == 0)
            .nth(0).unwrap();
        let max_nonzero_avail_seen = state.nodes.iter()
            .filter(|x| x.used != 0)
            .map(|x| x.avail)
            .max().ok_or(NotASpecialGridError)?;
        let max_avail_rule: usize = max_nonzero_avail_seen;
        let min_nonzero_size_seen = state.nodes.iter()
            .filter(|x| x.used != 0)
            .map(|x| x.used)
            .min().ok_or(NotASpecialGridError)?;
        if max_nonzero_avail_seen >= min_nonzero_size_seen {
            return Err(NotASpecialGridError);
        }
        let min_filler_content_rule: usize = min_nonzero_size_seen;
        let max_filler_capacity_rule: usize = min_filler_content_rule * 2 - 1;
        let min_blocker_content_rule: usize = max_filler_capacity_rule + 1;
        let max_filler_content_seen = state.nodes.iter()
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


impl Display for NodeSpace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:3}T/{:3}T", self.used, self.used + self.avail)
    }
}


/// One might want to look at the diagonal lines of coords that are within a certain rectangle.
/// That's what this function is for. It is passed a Coord which represents the outer bound
/// of the rectangle, and it returns a Vec of the diagonals. It does NOT include the diagonal
/// that contains the bound itself.
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
fn derive_avail_moves(nodes: &GridVec<NodeSpace>) -> Vec<GridMove> {
    let indexes = 0..(nodes.size().0 * nodes.size().1);
    indexes.flat_map(|idx| {
        let coord = nodes.index_to_coord(idx);
        neighbor_dirs(coord, nodes.size()).into_iter()
            .map(move |dir| GridMove::new(coord, dir))
            .filter(|mv| is_legal(mv, &nodes))
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
    type TMove = GridMove;

    /// This indicates whether a state is a "winning" state.
    fn is_winning(&self) -> bool {
        self.goal_data_loc == (0,0)
    }

    /// This is the heuristic used for the A* search algorithm -- it returns an estimate of
    /// the minimum possible moves needed to win (this estimate can be too low, but must
    /// never be too high). To make it pluggable, we have a set of different heuristics
    /// and this is coded to call one of them.
    fn min_moves_to_win(&self) -> usize {
        self.taxicab_plus_moving()
    }

    fn avail_moves(&self) -> &Vec<GridMove> {
        &self.avail_moves
    }


    /// Returns the new State achieved by performing this move (which ought to be one of the
    /// valid Moves for this State).
    fn enact_move(&self, mv: &GridMove) -> Self {
        // --- set the nodes ---
        let mut nodes = self.nodes.clone();
        let from_node = nodes.get_mut(&mv.from());
        let amt_moved: usize = from_node.used;
        from_node.avail += amt_moved;
        from_node.used -= amt_moved;
        assert_eq!(from_node.used, 0);
        let to_node = nodes.get_mut(&mv.to());
        assert!(to_node.avail >= amt_moved);
        to_node.avail -= amt_moved;
        to_node.used += amt_moved;

        // --- set the goal_data ---
        let goal_data_loc = if mv.from() == self.goal_data_loc {
            mv.to()
        } else {
            self.goal_data_loc
        };

        // --- set the avail_moves ---
        // copy existing avail_moves EXCEPT those that enter or leave move.from or move.to()
        let mut avail_moves: Vec<GridMove> = self.avail_moves.iter()
            .filter(|x| x.from() != mv.from() && x.to() != mv.from() && x.from() != mv.to() && x.to() != mv.to())
            .copied()
            .collect();
        // re-consider everything that enters or leaves move.from or move.to()
        let moves_out: Vec<GridMove> = neighbor_dirs(mv.from(), self.nodes.size()).into_iter()
            .map(|dir| GridMove::new(mv.from(), dir)) // moves from the "from" location
            .chain(
                neighbor_dirs(mv.to(), self.nodes.size()).into_iter()
                    .map(|dir| GridMove::new(mv.to(), dir)) // moves from the "to" location
                    .filter(|s| s.to() != mv.from()) // except the one going to "from" location; we already got the reverse of that
            ).collect();
        avail_moves.extend(moves_out.iter().filter(|m| is_legal(m, &nodes))); // add the legal "out" movess
        avail_moves.extend(moves_out.iter().map(|m| m.inverse()).filter(|m| is_legal(m, &nodes))); // add the legal "in" moves

        if VERIFY_AVAIL_STEP_LOGIC {
            let mut sorted_moves: Vec<GridMove> = avail_moves.clone();
            let mut correct_moves: Vec<GridMove> = derive_avail_moves(&nodes);
            sorted_moves.sort();
            correct_moves.sort();
            assert_eq!(sorted_moves, correct_moves);
        }

        // --- return the result ---
        GenState{nodes, goal_data_loc, avail_moves }
    }
}




impl SingleSpaceState {
    /// Number of moves the open space must take to get beside the goal_data without
    /// moving the goal_data
    fn min_movess_open_space_must_take(&self) -> usize {
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
            let open_space_loc = self.open_space_loc;
            let min_dist_to_target_loc = target_locs.iter().map(|target_loc| {
                if *target_loc == open_space_loc {
                    0 // already there!
                } else {
                    target_loc.0.abs_diff(open_space_loc.0) + // x distance to get there
                        target_loc.1.abs_diff(open_space_loc.1) + // y distance to get there
                        if goal_data_loc.0 == 0 && open_space_loc.0 == 0 && open_space_loc.1 > goal_data_loc.1 {2} else {0} + // go-around-penalty
                        if goal_data_loc.1 == 0 && open_space_loc.1 == 0 && open_space_loc.0 > goal_data_loc.0 {2} else {0} // go-around-penalty
                }
            }).min().unwrap();
            min_dist_to_target_loc
        }
    }

    /// Returns a dummy SingleSpaceState with the specified goal_data_loc and open_space_loc.
    fn dummy_with_space_at(&self, open_space_loc: Coord) -> Self {
        SingleSpaceState{open_space_loc,  ..self.clone()}
    }
}


impl State for SingleSpaceState {
    type TMove = GridMove;

    /// This indicates whether a state is a "winning" state.
    fn is_winning(&self) -> bool {
        self.base.is_winning()
    }

    /// This is the heuristic used for the A* search algorithm -- it returns an estimate of
    /// the minimum possible movess needed to win (this estimate can be too low, but must
    /// never be too high). To make it pluggable, we have a set of different heuristics
    /// and this is coded to call one of them.
    fn min_moves_to_win(&self) -> usize {
        1 + (self.base.taxicab_distance_from_goal_data_to_corner() - 1) * 5 +
            self.min_movess_open_space_must_take()
    }

    fn avail_moves(&self) -> &Vec<GridMove> {
        self.base.avail_moves()
    }


    /// Returns the new State achieved by performing this move (which ought to be one of the
    /// valid Moves for this State).
    fn enact_move(&self, mv: &GridMove) -> Self {
        let base = self.base.enact_move(mv);
        let min_blocker_content = self.min_blocker_content;

        // --- set open_space_loc ---
        let open_space_loc = if mv.to() == self.open_space_loc {
            mv.from()
        } else {
            self.open_space_loc
        };

        // --- return the result ---
        SingleSpaceState{base, min_blocker_content, open_space_loc}
    }


    /// Override show_state to display much more information including an ascii picture.
    fn show_state(
        &self,
        loop_ctr: usize,
        move_count: usize,
        visited_from: &HashMap<Self, Option<(Self, GridMove, usize)>>,
        queue: &VecDeque<StateToConsider<Self>>)
    {
        println!(
            "\nAt {} went {} moves; at least {} to go for a total of {}.",
            loop_ctr,
            move_count,
            self.min_moves_to_win(),
            move_count + self.min_moves_to_win()
        );
        for c in self.base.nodes.iter_indexes() {
            if c.0 == 0 {
                println!(); // newline before each line
            }
            let ch = match c {
                c if c == self.open_space_loc => '.',
                c if c == self.base.goal_data_loc => 'X',
                c if self.base.nodes.get(&c).used >= self.min_blocker_content => '#',
                c if visited_from.contains_key(&self.dummy_with_space_at(c)) => match visited_from.get(&self.dummy_with_space_at(c)).unwrap() {
                    None => '@',
                    Some((_, grid_move, _)) => grid_move.direction_to_ascii_picture(),
                },
                c if queue.iter().any(|x| {
                    x.state().base.goal_data_loc == self.base.goal_data_loc &&
                        x.state().open_space_loc == c
                }) => '&',
                _ => 'o',
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
        self.base.nodes.size().eq(&other.base.nodes.size()) &&
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
        self.base.nodes.size().hash(state);
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
        for mv in &self.avail_moves {
            write!(f, "{} ", mv)?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}


impl Display for SingleSpaceState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if VERBOSE_STATE {
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
            for mv in &self.base.avail_moves {
                write!(f, "{} ", mv)?;
            }
            writeln!(f, "]")?;
        } else {
            write!(f, "SingleSpaceState{{goal:{:?}, space: {:?}}}", self.base.goal_data_loc, self.open_space_loc)?;
        }
        Ok(())
    }
}



fn find_winning_moves(grid: &Grid) -> Option<Vec<GridMove>> {
    match grid.get_initial_singlespacestate() {
        Some(mut initial_state) => {
            solve_with_astar(&mut initial_state, PRINT_EVERY_N_MOVES)
        },
        None => {
            let mut initial_state = grid.get_initial_genstate();
            solve_with_astar(&mut initial_state, PRINT_EVERY_N_MOVES)
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

    let winning_moves = find_winning_moves(grid);
    match winning_moves {
        None => println!("Could not find a solution."),
        Some(moves) => {
            print!("Winning moves are: ");
            for mv in &moves {
                print!("{}, ", mv);
            }
            println!();
            println!("Which took {} moves.", moves.len());
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
        let ch = match (c, state.base.nodes.get(&c)) {
            (c,_) if c == goal_data_loc => 'X',
            (c,_) if c == open_space_loc => '.',
            (_, NodeSpace{used, ..}) if *used >= state.min_blocker_content => 'H',
            _ => 'o',
        };
        print!("{}", ch);
    }
    println!(); // newline at the end of the grid
}

enum ManualAction<'a> {
    Quit,
    Echo(&'a str),
    Move(Direction),
    Undo,
}

/// Plays the game interactively
#[allow(dead_code)]
fn play_manually(grid: &Grid) {
    println!("Use the following commands (hit return after each character):");
    println!("  q - Quit");
    println!("  w - Move empty space up");
    println!("  s - Move empty space down");
    println!("  a - Move empty space left");
    println!("  d - Move empty space right");
    println!("  z - Undo previous move");
    match grid.get_initial_singlespacestate() {
        Some(initial_state) => {
            let mut moves_taken: Vec<GridMove> = Vec::new();
            let mut state = initial_state.clone();
            loop {
                println!("After {} moves:", moves_taken.len());
                print_for_play(&state);
                let input = read_line();
                let manual_action: ManualAction = match input.as_str() {
                    "q" => ManualAction::Quit,
                    "w" => ManualAction::Move(Direction::Up),
                    "s" => ManualAction::Move(Direction::Down),
                    "a" => ManualAction::Move(Direction::Left),
                    "d" => ManualAction::Move(Direction::Right),
                    "z" => ManualAction::Undo,
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
                        let mv = GridMove::new(state.open_space_loc, dir).inverse();
                        println!("Step {}", mv);
                        state = state.enact_move(&mv);
                        moves_taken.push(mv);
                    },
                    ManualAction::Undo => {
                        match moves_taken.pop() {
                            None => {
                                println!("Nothing more to undo.");
                            },
                            Some(prev_move) => {
                                println!("Undo");
                                state = state.enact_move(&prev_move.inverse());
                            }
                        }
                    }
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
    if PLAY_MANUAL_GAME {
        play_manually(&data);
    } else {
        part_a(&data);
        part_b(&data);
    }
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

    #[test]
    fn test_min_moves_open_space_must_take() {
        /// Builds a SingleSpaceState that's 6x6 with the goal at goal_coord and the
        /// space at space_coord.
        fn build_state(goal_coord: Coord, space_coord: Coord) -> SingleSpaceState {
            let size = (6,6);
            assert_ne!(space_coord, (0, size.1 - 1)); // won't work right if the top-right is the space_coord
            let mut nodes: HashMap<Coord,Node> = HashMap::new();
            for y in 0..6 {
                for x in 0..6 {
                    if (x,y) == space_coord {
                        nodes.insert((x,y), Node{x, y, size: 60, used: 0, avail: 60});
                    } else {
                        nodes.insert((x,y), Node{x, y, size: 60, used: 50, avail: 10});
                    }
                }
            }
            let grid = Grid{nodes, size};
            let mut sss = grid.get_initial_singlespacestate().unwrap();
            sss.base.goal_data_loc = goal_coord;
            sss
        }

        let expected_data = vec![
            // ( goal_loc, space_loc, expect_movess )
            ((3,0), (4,0), 4),
            ((4,0), (3,0), 0),
            ((1,0), (0,1), 1),
            ((2,2), (1,2), 0),
            ((2,2), (2,1), 0),
            ((2,2), (1,1), 1),
            ((2,2), (0,0), 3),
            ((2,2), (3,2), 2),
            ((2,2), (2,3), 2),
            ((2,2), (0,1), 2),
            ((3,0), (3,1), 2),
        ];

        for (goal_coord, space_coord, expect_moves) in expected_data {
            let moves = build_state(goal_coord, space_coord).min_movess_open_space_must_take();
            println!("Testing {:?}, {:?}, {}", goal_coord, space_coord, expect_moves); // so I can tell which one failed
            assert_eq!(expect_moves, moves);
        }
    }
}
