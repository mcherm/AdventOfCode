
extern crate anyhow;

use std::cmp::Ordering;
use std::fs;
use nom;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::line_ending,
};
use nom::character::complete::u32 as nom_Num;
use std::collections::{BinaryHeap, BTreeMap};
use im::ordset::OrdSet;
use im::Vector;
use itertools::Itertools;


// ======= Constants =======

const PRINT_WORK: bool = false;
const MAX_STEPS: usize = 30;

// ======= Parsing =======

fn input() -> Result<Vec<ValveDesc>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_16.txt")?;
    match ValveDesc::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Num = u32;

#[derive(Debug)]
struct ValveDesc {
    name: String,
    flow_rate: Num,
    leads_to: Vec<String>,
}


//Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
impl ValveDesc {

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        nom::combinator::map(
            nom::sequence::tuple((
                tag("Valve "),
                nom::character::complete::alpha1,
                tag(" has flow rate="),
                nom_Num,
                nom::branch::alt((
                    tag("; tunnels lead to valves "),
                    tag("; tunnel leads to valve "),
                )),
                nom::multi::separated_list1(
                    tag(", "),
                    nom::character::complete::alpha1
                ),
            )),
            |(_, name, _, flow_rate, _, leads_to): (_, &'a str, _, Num, _, Vec<&'a str>)| ValveDesc{
                name: name.to_string(),
                flow_rate,
                leads_to: leads_to.iter().map(|x| x.to_string()).collect_vec()
            }
        )(input)
    }

    /// Parses a newline-terminated list of LineSpecs
    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        nom::multi::many1( nom::sequence::terminated(Self::parse, line_ending) )(input)
    }

}



// ======= Part 1 Compute =======

#[derive(Debug, Eq, PartialEq)]
struct Valve {
    flow_rate: Num,
    leads_to: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
struct ValveMaze {
    valves: BTreeMap<String, Valve>
}

/// A single step that a single actor can take.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Step {
    OpenValve(String),
    MoveTo(String),
}

/// All the steps that the entire group of actors can take.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum GroupStep {
    Solo(Step),
    Pair(Step, Step),
    Training,
}

/// The location of the entire group of actors.
#[derive(Debug, Eq, PartialEq)]
enum GroupLocation {
    Solo(String),
    Pair(String, String),
}


#[derive(Debug, Eq, PartialEq)]
struct SolverState<'a> {
    valve_maze: &'a ValveMaze,
    location: GroupLocation,
    time_completed: usize,
    prev_steps: Vector<GroupStep>,
    unopened_valves: OrdSet<String>,
    unopened_flow_rates: Vec<usize>,
    score: [usize; 2], // the score is [pressure_released, possible_release]
}


impl ValveMaze {
    /// Construct a ValveMaze from the list of ValveDescs.
    ///
    /// NOTE: This *could* confirm that every place led to is one that exists (and return an
    ///   error if not), but for now it is not checking that.
    fn new(input: &Vec<ValveDesc>) -> Self {
        let mut valves = BTreeMap::new();
        for ValveDesc{name, flow_rate, leads_to} in input {
            let valve = Valve{flow_rate: flow_rate.clone(), leads_to: leads_to.clone()};
            valves.insert(name.clone(), valve);
        }
        ValveMaze{valves}
    }
}


impl GroupStep {
    fn apply_to_location(&self, from_loc: &GroupLocation) -> GroupLocation {
        match from_loc {
            GroupLocation::Solo(my_loc) => match self {
                GroupStep::Solo(Step::MoveTo(new_loc)) => GroupLocation::Solo(new_loc.clone()),
                GroupStep::Solo(Step::OpenValve(_)) => GroupLocation::Solo(my_loc.clone()),
                GroupStep::Pair(_, _) => panic!("Pair step from Solo Location"),
                GroupStep::Training => panic!("Training step from Solo Location"),
            }
            GroupLocation::Pair(my_loc, el_loc) => match self {
                GroupStep::Pair(Step::MoveTo(my_new_loc), Step::MoveTo(el_new_loc)) => GroupLocation::Pair(my_new_loc.clone(), el_new_loc.clone()),
                GroupStep::Pair(Step::MoveTo(my_new_loc), Step::OpenValve(_)) => GroupLocation::Pair(my_new_loc.clone(), el_loc.clone()),
                GroupStep::Pair(Step::OpenValve(_), Step::MoveTo(el_new_loc)) => GroupLocation::Pair(my_loc.clone(), el_new_loc.clone()),
                GroupStep::Pair(Step::OpenValve(_), Step::OpenValve(_)) => GroupLocation::Pair(my_loc.clone(), el_loc.clone()),
                GroupStep::Training => GroupLocation::Pair(my_loc.clone(), el_loc.clone()),
                GroupStep::Solo(_) => panic!("Solo step from Pair Location"),
            }
        }
    }

    /// This is passed an existing set of unopened valves, the corresponding (sorted) unopened
    /// flow rates, and the current pressure_released. It returns the new values for all three
    /// of these fields that we will obtain if we enact this GroupStep.
    fn apply_to_valves(
        &self,
        valve_maze: &ValveMaze,
        time_remaining: usize,
        old_unopened_valves: &OrdSet<String>,
        old_unopened_flow_rates: &Vec<usize>,
        old_pressure_released: usize,
    ) -> (
        OrdSet<String>, // new_unopened_valves
        Vec<usize>, // new_unopened_flow_rates
        usize, // new_pressure_released
    ) {
        let valves_to_open: Vec<&String> = match self {
            GroupStep::Solo(Step::OpenValve(my_valve)) => vec![my_valve],
            GroupStep::Solo(Step::MoveTo(_)) => vec![],
            GroupStep::Pair(Step::OpenValve(my_valve), Step::OpenValve(el_valve)) if my_valve == el_valve => vec![], // no both opening same valve
            GroupStep::Pair(Step::OpenValve(my_valve), Step::OpenValve(el_valve)) => vec![my_valve, el_valve],
            GroupStep::Pair(Step::OpenValve(my_valve), Step::MoveTo(_)) => vec![my_valve],
            GroupStep::Pair(Step::MoveTo(_), Step::OpenValve(el_valve)) => vec![el_valve],
            GroupStep::Pair(Step::MoveTo(_), Step::MoveTo(_)) => vec![],
            GroupStep::Training => vec![],
        };

        let mut new_unopened_valves = old_unopened_valves.clone();
        let mut new_unopened_flow_rates = old_unopened_flow_rates.clone();
        let mut pressure_released_this_time = 0;
        for valve in valves_to_open.iter() {
            new_unopened_valves.remove(*valve);
            let valve_flow_rate = valve_maze.valves.get(*valve).unwrap().flow_rate as usize;
            let pos_to_remove = new_unopened_flow_rates.iter().position(|x| *x == valve_flow_rate).unwrap();
            new_unopened_flow_rates.remove(pos_to_remove);
            pressure_released_this_time += (valve_flow_rate as usize) * time_remaining;
        }
        let new_pressure_released = old_pressure_released + pressure_released_this_time;
        (new_unopened_valves, new_unopened_flow_rates, new_pressure_released)
    }

}


impl GroupLocation {
    fn num_actors(&self) -> usize {
        match self {
            GroupLocation::Solo(_) => 1,
            GroupLocation::Pair(_, _) => 2,
        }
    }
}


impl<'a> SolverState<'a> {

    /// Returns a cap on the the maximum possible future release. The heuristic used
    /// may change over time, but for now it assumes instantaneous travel to all locations.
    fn calc_score(
        time_completed: usize,
        unopened_flow_rates: &Vec<usize>,
        pressure_released: usize,
        num_actors: usize,
    ) -> [usize;2] {
        let mut remaining_steps = MAX_STEPS - time_completed;
        let mut possible_release = 0;
        let mut flow_rate_iter = unopened_flow_rates.iter();
        while remaining_steps > 0 {
            for _actor_num in 0..num_actors {
                if remaining_steps > 0 {
                    remaining_steps -= 1; // have to open the valve
                    let flow_rate = match flow_rate_iter.next() {
                        None => break,
                        Some(flow_rate) => flow_rate
                    };
                    possible_release += flow_rate * remaining_steps;
                    if remaining_steps > 0 {
                        remaining_steps -= 1; // have to walk to a new location
                    }
                }
            }
        }
        [pressure_released, possible_release]
    }

    fn pressure_released(&self) -> usize {
        return self.score[0];
    }

    fn possible_release(&self) -> usize {
        return self.score[1];
    }

    /// Returns (an overestimate of) the largest score it's possible to get, starting from this location.
    fn max_possible(&self) -> usize {
        self.pressure_released() + self.possible_release()
    }

    /// The initial SolverState
    fn initial(valve_maze: &'a ValveMaze, has_elephant: bool) -> Self {
        let start_loc = "AA";
        let location: GroupLocation = if has_elephant {
            GroupLocation::Pair(start_loc.to_string(), start_loc.to_string())
        } else {
            GroupLocation::Solo(start_loc.to_string())
        };
        let time_completed = 0;
        let prev_steps = Vector::new();
        let unopened_valves: OrdSet<String> = valve_maze.valves.iter()
            .filter_map(|(name, valve)| if valve.flow_rate == 0 {None} else {Some(name)})
            .collect();
        let mut unopened_flow_rates: Vec<usize> = unopened_valves.iter()
            .map(|name| (valve_maze.valves.get(name).unwrap().flow_rate as usize))
            .collect();
        unopened_flow_rates.sort_by_key(|x| std::cmp::Reverse(*x)); // put big ones first
        let pressure_released = 0;
        let score = Self::calc_score(time_completed, &unopened_flow_rates, pressure_released, location.num_actors());
        SolverState{
            valve_maze,
            location,
            time_completed,
            prev_steps,
            unopened_valves,
            unopened_flow_rates,
            score,
        }
    }

    /// Returns the list of possible next steps for an agent starting from the given location
    fn next_moves_from_loc(&self, location: &String) -> Vec<Step> {
        let mut answer = Vec::new();
        if self.time_completed < MAX_STEPS {
            if self.unopened_valves.contains(location) {
                answer.push(Step::OpenValve(location.to_string()));
            }
            for next_location in &self.valve_maze.valves.get(location).unwrap().leads_to {
                answer.push(Step::MoveTo(next_location.clone()));
            }
        }
        answer
    }

    /// Returns the list of possible next moves that can be made from the current state.
    fn next_steps_from_state(&self) -> Vec<GroupStep> {
        match &self.location {
            GroupLocation::Solo(loc_me) => {
                self.next_moves_from_loc(&loc_me).into_iter()
                    .map(|step| GroupStep::Solo(step))
                    .collect_vec()
            }
            GroupLocation::Pair(loc_me, loc_elephant) => {
                if self.time_completed < 4 {
                    vec![GroupStep::Training]
                } else {
                    let my_moves = self.next_moves_from_loc(&loc_me);
                    let elephant_moves = self.next_moves_from_loc(&loc_elephant);
                    my_moves.into_iter().cartesian_product(elephant_moves.into_iter())
                        .map(|(my_step, elephant_step)| GroupStep::Pair(my_step, elephant_step))
                        .collect_vec()
                }
            }
        }
    }

    /// Returns the list of possible next states from this state.
    fn next_states(&self) -> Vec<Self> {
        self.next_steps_from_state().into_iter()
            .map(|group_step| self.build_next_state(group_step))
            .collect_vec()
    }


    /// Given a GroupStep to take from here, this returns the new state that step would reach.
    fn build_next_state(&self, group_step: GroupStep) -> Self {
        let valve_maze = self.valve_maze;
        let location = group_step.apply_to_location(&self.location);
        let time_completed = self.time_completed + 1;
        let time_remaining = MAX_STEPS - self.time_completed - 1;
        let (unopened_valves, unopened_flow_rates, new_pressure_released) = group_step.apply_to_valves(
            valve_maze,
            time_remaining,
            &self.unopened_valves,
            &self.unopened_flow_rates,
            self.pressure_released(),
        );
        let score = Self::calc_score(time_completed, &unopened_flow_rates, new_pressure_released, location.num_actors());
        let mut prev_steps = self.prev_steps.clone();
        prev_steps.push_back(group_step);
        SolverState{
            valve_maze,
            location,
            time_completed,
            prev_steps,
            unopened_valves,
            unopened_flow_rates,
            score,
        }
    }

    // /// The state we get to by opening a valve from here. Assumes that it's valid to do. The
    // /// flow_rate of this valve is passed in.
    // fn do_open_valve(&self, flow_rate: Num) -> Self {
    //     let valve_maze = self.valve_maze;
    //     let location = self.location.clone();
    //     let mut prev_steps = self.prev_steps.clone();
    //     prev_steps.push_back(Step::OpenValve(self.location.to_string()));
    //     let mut unopened_valves = self.unopened_valves.clone();
    //     unopened_valves.remove(&self.location);
    //     let mut unopened_flow_rates: Vec<usize> = self.unopened_flow_rates.clone();
    //     let newly_opened_valve_flow_rate = valve_maze.valves.get(location).unwrap().flow_rate as usize;
    //     let pos_to_remove = unopened_flow_rates.iter().position(|x| *x == newly_opened_valve_flow_rate).unwrap();
    //     unopened_flow_rates.remove(pos_to_remove);
    //     let new_pressure_released = (flow_rate as usize) * (MAX_STEPS - self.time_completed - 1);
    //     let time_completed = self.time_completed + 1;
    //     let score = Self::calc_score(
    //         time_completed,
    //         &unopened_flow_rates,
    //         self.pressure_released() + new_pressure_released
    //     );
    //     SolverState{
    //         valve_maze,
    //         location,
    //         time_completed,
    //         prev_steps,
    //         unopened_valves,
    //         unopened_flow_rates,
    //         score
    //     }
    // }

    // /// The state we get to by taking a step from here. Assumes that it's valid to do. the
    // /// step is passed in.
    // fn do_go_to(&self, next_location: &'a str) -> Self {
    //     let valve_maze = self.valve_maze;
    //     let location = next_location;
    //     let time_completed = self.time_completed + 1;
    //     let mut prev_steps = self.prev_steps.clone();
    //     prev_steps.push_back(Step::MoveTo(next_location.to_string()));
    //     let unopened_valves = self.unopened_valves.clone();
    //     let unopened_flow_rates = self.unopened_flow_rates.clone();
    //     let score = Self::calc_score(time_completed, &unopened_flow_rates, self.pressure_released());
    //     SolverState{
    //         valve_maze,
    //         location,
    //         time_completed,
    //         prev_steps,
    //         unopened_valves,
    //         unopened_flow_rates,
    //         score,
    //     }
    // }

}

impl<'a> PartialOrd for SolverState<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other)) // just default to the total ordering
    }
}

impl<'a> Ord for SolverState<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut answer = self.score.cmp(&other.score); // sort by score
        if answer == Ordering::Equal {
            answer = self.prev_steps.cmp(&other.prev_steps); // break ties with path
        }
        answer
    }
}


/// Solves it, returning the final state
fn solve(valve_maze: &ValveMaze, has_elephant: bool) -> SolverState {
    let mut states_tried = 0;
    let mut best_state = SolverState::initial(valve_maze, has_elephant);
    let mut states_to_try: BinaryHeap<SolverState> = BinaryHeap::from([SolverState::initial(valve_maze, has_elephant)]);
    loop {
        states_tried += 1;
        match states_to_try.pop() {
            None => {
                // Nothing left to try so we've solved it
                println!("Tried a total of {} states.", states_tried);
                return best_state;
            }
            Some(state) => {
                // Add the possible next states onto the list, but ONLY if it's POSSIBLE for one to beat the best
                let best_released = best_state.pressure_released();
                for next_state in state.next_states() {
                    if next_state.max_possible() > best_released {
                        states_to_try.push(next_state); // they get sorted as they are inserted
                    }
                }
                // Check if this one is the new best state
                if state.pressure_released() > best_state.pressure_released() {
                    if PRINT_WORK {
                        println!("New best: [{}, {}] -> {} {:?}", state.pressure_released(), state.possible_release(), state.max_possible(), state.prev_steps);
                    }
                    best_state = state;
                } else {
                    if PRINT_WORK {
                        println!("   tried: [{}, {}] -> {} {:?}", state.pressure_released(), state.possible_release(), state.max_possible(), state.prev_steps);
                    }
                }
            }
        }
    }
}


// ======= main() =======

fn part_a(input: &Vec<ValveDesc>) {
    println!("\nPart a:");
    let valve_maze = ValveMaze::new(input);
    let has_elephant = false;
    let solved_state = solve(&valve_maze, has_elephant);
    println!("Path {:?}", solved_state.prev_steps);
    println!("Releases {}", solved_state.pressure_released());
}


fn part_b(input: &Vec<ValveDesc>) {
    println!("\nPart b:");
    let valve_maze = ValveMaze::new(input);
    let has_elephant = true;
    let solved_state = solve(&valve_maze, has_elephant);
    println!("Path {:?}", solved_state.prev_steps);
    println!("Releases {}", solved_state.pressure_released());
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}


// ======= Tests =======

