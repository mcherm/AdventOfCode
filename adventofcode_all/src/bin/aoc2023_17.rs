use std::fmt::{Debug, Display, Formatter};
use anyhow;
use advent_lib::grid::{Coord, Grid, Direction};
use std::collections::HashSet;
use std::cmp::Reverse;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Copy, Clone)]
pub struct HeatLoss(u8);

pub type HeatLossGrid = Grid<HeatLoss>;


type Input = HeatLossGrid;


/// An error type when reading a character which should be a HeatLoss.
#[derive(Debug)]
pub struct InvalidHeatLoss(char);


impl TryFrom<char> for HeatLoss {
    type Error = InvalidHeatLoss;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '1'..='9' => Ok(HeatLoss(value.to_string().parse::<u8>().unwrap())),
            _ => Err(InvalidHeatLoss(value))?,
        }
    }
}


impl Display for HeatLoss {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl Display for InvalidHeatLoss {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidHeadLoss({:?})", self.0)
    }
}

impl std::error::Error for InvalidHeatLoss {}




mod parse {
    use super::{Input, HeatLossGrid};
    use std::fs;

    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_17.txt")?;
        Ok(HeatLossGrid::from_char_string(&s)?)
    }


}


// ======= Compute =======


/// A specific state it can be in.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct SolverState {
    coord: Coord,
    dir: Option<Direction>, // will be None if we teleported here; otherwise always Some()
    steps_gone_straight: u8,
}


/// The combination of a SolverState and the cost to get there.
struct SolverPos {
    state: SolverState,
    cost: u32,
}

enum CrucibleType {
    Normal, Ultra
}



impl SolverState {
    /// Returns true if this is at the exit of the grid; false if not.
    fn at_grid_exit(&self, grid: &HeatLossGrid) -> bool {
        self.coord.x() + 1 == grid.bound().x() && self.coord.y() + 1 == grid.bound().y()
    }
}


impl Debug for SolverPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dir_string = match self.state.dir {
            Some(dir) => dir.to_string(),
            None => "X".to_string(),
        };
        write!(f, "Pos[{}, {} to {} for {}]", self.cost, dir_string, self.state.coord, self.state.steps_gone_straight)
    }
}


impl CrucibleType {
    /// Returns the minimum number of steps in a straight line for this crucible type.
    fn min_straight(&self) -> u8 {
        match self {
            CrucibleType::Normal => 1,
            CrucibleType::Ultra => 4,
        }
    }

    /// Returns the maximum number of steps in a straight line for this crucible type.
    fn max_straight(&self) -> u8 {
        match self {
            CrucibleType::Normal => 3,
            CrucibleType::Ultra => 10,
        }
    }

    fn can_stop(&self, state: &SolverState) -> bool {
        state.steps_gone_straight >= self.min_straight()
    }
}


/// Given a HeatLossGrid, this finds the one of the fastest paths from the top-left to
/// the bottom-right and then returns the total cost of that (excluding the cost to enter
/// the top-left).
fn solve(grid: &HeatLossGrid, crucible: CrucibleType) -> u32 {
    let mut available_positions: Vec<SolverPos> = Vec::new(); // we'll keep this sorted with least cost at the end
    let mut visited_states: HashSet<SolverState> = HashSet::new();
    let start_pos = SolverPos{
        state: SolverState{
            coord: Coord(0,0),
            dir: None,
            steps_gone_straight: 0,
        },
        cost: 0,
    };
    available_positions.push(start_pos);

    // Loop until we find the answer (or run out of choices!)
    while let Some(pos) = available_positions.pop() {

        if pos.state.at_grid_exit(grid) && crucible.can_stop(&pos.state) {
            return pos.cost;
        }

        // Mark this one as having been tried
        let state = &pos.state;
        let was_new = visited_states.insert(state.clone());

        if was_new { // if it wasn't new, then we've done this before and can skip the rest

            // Figure out the directions we can go (ignoring out-of-bounds)
            let min_straight = crucible.min_straight();
            let max_straight = crucible.max_straight();
            assert!(state.steps_gone_straight <= max_straight);
            let mut next_directions: HashSet<Direction> = Direction::ALL.iter().copied().collect();
            if let Some(dir) = state.dir { // except for when we teleported in
                next_directions.remove(&dir.reverse()); // can't reverse
                if state.steps_gone_straight < min_straight { // can't turn if less than min_straight
                    next_directions.remove(&dir.clockwise());
                    next_directions.remove(&dir.counter_clockwise());
                }
                if state.steps_gone_straight == max_straight {
                    next_directions.remove(&dir); // can't go more than max_straight steps in a line
                }
            }

            // Add the new possible positions to available_states
            for next_direction in next_directions {
                if let Some(coord) = state.coord.bounded_step(next_direction, grid.bound()) { // here we check for out-of-bounds
                    // Create the new state
                    let dir = Some(next_direction);
                    let steps_gone_straight = if dir == state.dir {state.steps_gone_straight + 1} else {1};
                    let new_state = SolverState{coord, dir, steps_gone_straight};

                    // Check if this state is already visited (which must have been at the same or cheaper cost)
                    if !visited_states.contains(&new_state) {
                        // We're going to insert this in available_positions!

                        // Create the new pos.
                        let cost = pos.cost + (grid.get(coord).0 as u32);
                        let new_pos = SolverPos{state: new_state, cost};

                        // define the function for sorting
                        fn sort_key(pos: &SolverPos) -> Reverse<(u32, usize, usize, u8, u8)> {
                            use Direction::*;
                            let dir_num: u8 = match pos.state.dir {
                                None => 0,
                                Some(East) => 1,
                                Some(South) => 2,
                                Some(West) => 3,
                                Some(North) => 4,
                            };
                            Reverse((pos.cost, pos.state.coord.1, pos.state.coord.0, dir_num, pos.state.steps_gone_straight))
                        }

                        // insert it in the proper location in the list
                        match available_positions.binary_search_by_key(&sort_key(&new_pos), sort_key) {
                            Ok(_i) => { // a matching pos is already in the list at location i
                                // we don't do anything... that one is already covered
                            }
                            Err(i) => { // this pos is NOT in the list, but it would belong at location i
                                available_positions.insert(i, new_pos);
                            }
                        }
                    }
                }
            }
        }
    }
    println!("We ran out of states without solving it!");
    panic!("Should not run out of states without solving it.");
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let heat_loss = solve(input, CrucibleType::Normal);
    println!("When solved, it loses at least {} heat.", heat_loss);
}


fn part_b(input: &Input) {
    println!("\nPart b:");
    let heat_loss = solve(input, CrucibleType::Ultra);
    println!("When solved, it loses at least {} heat.", heat_loss);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
