
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::fmt::{Debug, Display, Formatter};



/// A trait for the states that the system can be in.
pub trait State : Display + Clone + Eq + Hash {
    type TMove: Clone; // associated type for the moves between states

    /// Returns true if this state is one that satisfies our goal.
    fn is_winning(&self) -> bool;

    /// Returns an heuristic for the number of moves needed to win from this state. The
    /// heuristic does not need to be correct, but it MUST not be too small: it should
    /// be accurate or an overestimate.
    fn min_moves_to_win(&self) -> usize;

    /// Returns the list of moves that can be made from this State.
    ///
    /// NOTE: It would be nice to return an iterator rather than insisting that it be a Vec,
    ///   but returning "impl Iterator" is not possible in a trait.
    fn avail_moves(&self) -> &Vec<Self::TMove>;

    /// Returns the new state achieved by performing one of the moves. The move provided
    /// MUST be one of those returned by avail_moves() or we risk a panic.
    fn enact_move(&self, mv: &Self::TMove) -> Self;

    /// For display purposes, prints to stdout the information about howe we are doing in our
    /// A* search. Is really more of a helper function for display rather than part of the State
    /// trait, but we declare it here so that implementors of the trait can override this.
    fn show_state(
        &self,
        loop_ctr: usize,
        move_count: usize,
        visited_from: &HashMap<Self, Option<(Self, Self::TMove, usize)>>,
        queue: &VecDeque<StateToConsider<Self>>
    ) {
        println!(
            "\nAt {} went {} moves; at least {} to go for a total of {}:{:}. Have visited {} states and have {} queued.",
            loop_ctr,
            move_count,
            self.min_moves_to_win(),
            move_count + self.min_moves_to_win(),
            self,
            visited_from.len(),
            queue.len()
        );
    }

}




/// This is what we insert into the queue while doing an A* search. It has a State and the
/// number of moves it took to get there. They are sortable (because the queue is kept
/// sorted) and the sort order is by move_count + state.min_movess_to_win()
pub struct StateToConsider<TS: State> {
    state: TS, // the state we will consider
    prev: Option<(TS, TS::TMove, usize)>, // Some(the previous state, the move from it, and the num_moves to get here) or None if this is the FIRST state.
}


impl<TS: State> StateToConsider<TS> {
    pub fn state(&self) -> &TS {
        &self.state
    }

    fn sort_score(&self) -> usize {
        let move_count = match self.prev {
            None => 0,
            Some((_,_,count)) => count
        };
        let answer = move_count + self.state.min_moves_to_win();
        answer
    }
}


impl<TS: State> Debug for StateToConsider<TS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateToConsider[{}] worth {}", self.state, self.sort_score())
    }
}



/// Uses A* to find a minimal solution starting from the given initial_state. Returns the
/// list of moves (or None if there isn't a solution).
///
/// print_every_n_moves either 0 (for don't print) or a number (eg: 1000) to print out some
///   progress notes every that many moves so we can tell it's still going.
pub fn solve_with_astar<TS: State>(initial_state: &mut TS, print_every_n_moves: usize) -> Option<Vec<TS::TMove>> {
    println!("Starting state: {:}", initial_state);

    // visited_from maps from a state (which we have considered and explored its neighbors) to how
    // we got there: (prev_state, prev_move, move_count).
    let mut visited_from: HashMap<TS, Option<(TS, TS::TMove, usize)>> = HashMap::new();

    // queue is a collection of states we will consider. What we store is
    //   StateToConsider. The queue is kept sorted by sort_score()
    let mut queue: VecDeque<StateToConsider<TS>> = VecDeque::new();
    queue.push_back(StateToConsider{state: initial_state.clone(), prev: None});

    let mut loop_ctr: usize = 0;
    loop {
        loop_ctr += 1;

        match queue.pop_front() {
            None => {
                return None; // we ran out of places to go. Guess it's not solvable!
            }
            Some(StateToConsider{state, prev}) => {
                let move_count = match prev {
                    None => 0,
                    Some((_,_, move_count)) => move_count,
                };

                // What to do if we visited this before?
                if let Some(prev) = visited_from.get(&state) {
                    let been_here_same_or_better = match prev {
                        None => true,
                        Some((_visited_state, _grid_move, prev_moves)) => *prev_moves <= move_count, // FIXME: think carefully about off-by-one error
                    };
                    if been_here_same_or_better {
                        // been here before, and it took same-or-fewer moves, so don't bother to re-examine
                        continue;
                    }
                }


                // -- Every so often, print it out so we can monitor progress --
                if loop_ctr % print_every_n_moves == 0 {
                    if print_every_n_moves > 1 || !visited_from.contains_key(&state.clone()) {
                        state.show_state(loop_ctr, move_count, &visited_from, &queue);
                    }
                }

                // -- mark that we have (or now will!) visited this one --
                assert!(!visited_from.contains_key(&state)); // Assert that we haven't been here before
                visited_from.insert(state.clone(), prev);

                // -- try each move from here --
                for mv in state.avail_moves() {
                    let next_state: TS = state.enact_move(mv);
                    let next_moves = move_count + 1;

                    // -- maybe we've already been to this one --
                    let earlier_visit = visited_from.get(&next_state);
                    // -- decide whether to put next_state onto the queue... --
                    let try_next_state = match earlier_visit {
                        None => true, // never seen it, certainly want to try it out
                        Some(None) => false, // the earlier visit was our starting position
                        Some(Some((_, _, earlier_move_count))) => {
                            match earlier_move_count.cmp(&next_moves) {
                                Ordering::Greater => panic!("Found faster way to a visited site."),
                                Ordering::Equal => false, // been here same distance; don't try it
                                Ordering::Less => false, // been here better distance; don't try it
                            }
                        }
                    };

                    if try_next_state {
                        if next_state.is_winning() {
                            println!("\nSOLVED!! {}", next_state);
                            let winning_moves = Some({
                                let mut moves: Vec<TS::TMove> = Vec::new();
                                moves.push(mv.clone());
                                let mut state_var: &TS = &state;
                                while let Some((prev_state, prev_move, _)) = visited_from.get(&state_var).unwrap() {
                                    moves.push((*prev_move).clone());
                                    state_var = prev_state;
                                }
                                moves.reverse();
                                moves
                            });
                            return winning_moves
                        } else {
                            // -- Actually add this to the queue --
                            let to_insert = StateToConsider{
                                state: next_state,
                                prev: Some((state.clone(), mv.clone(), move_count + 1))
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



/// A module that specializes astar for the case where we're dealing with things in a grid.
pub mod grid {
    use std::fmt::{Debug, Display, Formatter};
    use std::hash::Hash;
    use itertools::Itertools;


    pub type Coord = (usize, usize);

    #[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct GridMove {
        from: Coord,
        dir: Direction,
    }

    #[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub enum Direction {
        Up, Down, Left, Right
    }

    /// This is a data structure for storing items indexed by a Coord. It provide O(1) lookup
    /// and also provides Eq and Hash.
    #[derive(Clone, Debug, Eq, PartialEq, Hash)]
    pub struct GridVec<T: Eq + Hash + Clone> {
        size: Coord,
        data: Vec<T>,
    }

    /// This is used just to return an iterator of Coords.
    struct GridVecCoordIter {
        size: Coord,
        next_val: Option<Coord>,
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

        /// Useful sometimes for rendering a GridMove.
        pub fn to_ascii_picture(&self) -> char {
            match self {
                Direction::Down => '^',
                Direction::Up => 'v',
                Direction::Left => '>',
                Direction::Right => '<',
            }
        }
    }


    impl GridMove {
        /// Useful sometimes for rendering a GridMove.
        pub fn direction_to_ascii_picture(&self) -> char {
            self.dir.to_ascii_picture()
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

    impl<T: Eq + Hash + Clone> GridVec<T> {
        /// Construct from a vec (which must be rectangular and at least 1x1 in size or it panics).
        pub fn from_vec2d(data_vec: &Vec<Vec<T>>) -> Self {
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

        pub fn index_to_coord(&self, idx: usize) -> Coord {
            (idx % self.size.0, idx / self.size.0)
        }

        /// Returns the (x_size,y_size) of the GridVec.
        pub fn size(&self) -> Coord {
            self.size
        }

        /// Iterate through (references to) the items.
        pub fn iter(&self) -> impl Iterator<Item = &T> {
            self.data.iter()
        }

        /// This is used to iterate through the indexes of the coord. It happens to
        /// loop through x faster than y.
        pub fn iter_indexes(&self) -> impl Iterator<Item = Coord> {
            GridVecCoordIter{size: self.size, next_val: Some((0,0))}
        }

        /// This returns (in O(1) time) the item in the GridVec at the given coord. If
        /// the coord is not within size, then this panics.
        pub fn get(&self, coord: &Coord) -> &T {
            let idx = self.coord_to_index(coord);
            self.data.get(idx).unwrap()
        }

        /// This returns (in O(1) time) a mutable reference to the item in the GridVec at the
        /// given coord. If the coord is not within size then this panics.
        pub fn get_mut(&mut self, coord: &Coord) -> &mut T {
            let idx = self.coord_to_index(coord);
            self.data.get_mut(idx).unwrap()
        }
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

    impl GridMove {
        /// Construct a GridMove from a coordinate and a direction.
        pub fn new(from: Coord, dir: Direction) -> Self {
            GridMove{from, dir}
        }

        /// Returns the place that the move ends up at.
        pub fn to(&self) -> Coord {
            match self.dir {
                Direction::Up => (self.from.0, self.from.1 - 1),
                Direction::Down => (self.from.0, self.from.1 + 1),
                Direction::Left => (self.from.0 - 1, self.from.1),
                Direction::Right => (self.from.0 + 1, self.from.1),
            }
        }

        /// Returns the place that the move starts from.
        pub fn from(&self) -> Coord {
            self.from
        }

        /// Given a move, this returns a move that goes from the destination to the start.
        pub fn inverse(&self) -> Self {
            Self{from: self.to(), dir: self.dir.inverse()}
        }
    }

    impl Display for GridMove {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "({},{})->({},{})", self.from.0, self.from.1, self.to().0, self.to().1)
        }
    }

    /// This returns a list of the directions reachable from this coordinate.
    pub fn neighbor_dirs(coord: Coord, size: Coord) -> Vec<Direction> {
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

}
