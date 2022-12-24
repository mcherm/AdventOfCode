
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

const PRINT_WORK: bool = true;
const MAX_STEPS: usize = 5;

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

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Step {
    OpenValve(String),
    MoveTo(String),
}

#[derive(Debug, Eq, PartialEq)]
struct SolverState<'a> {
    valve_maze: &'a ValveMaze,
    location: String,
    time_completed: usize,
    prev_steps: Vector<Step>,
    unopened_valves: OrdSet<String>, // FIXME: Can this be &'a str instead?
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


impl<'a> SolverState<'a> {

    /// Returns a cap on the the maximum possible future release. The heuristic used
    /// may change over time, but for now it assumes instantaneous travel to all locations.
    fn calc_score(valve_maze: &'a ValveMaze, time_completed: usize, unopened_valves: &OrdSet<String>, pressure_released: usize) -> [usize;2] {
        let remaining_steps = MAX_STEPS - time_completed;
        let possible_release = unopened_valves.iter()
            .map(|name| (valve_maze.valves.get(name).unwrap().flow_rate as usize) * remaining_steps)
            .product();
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
    fn initial(valve_maze: &'a ValveMaze) -> Self {
        let location = "AA".to_string();
        let time_completed = 0;
        let prev_steps = Vector::new();
        let unopened_valves = valve_maze.valves.iter()
            .filter_map(|(name, valve)| if valve.flow_rate == 0 {None} else {Some(name.clone())})
            .collect();
        let pressure_released = 0;
        let score = Self::calc_score(valve_maze, time_completed, &unopened_valves, pressure_released);
        SolverState{
            valve_maze,
            location,
            time_completed,
            prev_steps,
            unopened_valves,
            score,
        }
    }

    /// Returns the list of possible next states after this one
    fn next_states(&self, valve_maze: &ValveMaze) -> Vec<Self> {
        let mut answer = Vec::new();
        if self.time_completed < MAX_STEPS {
            if self.unopened_valves.contains(&self.location) {
                let flow_rate = valve_maze.valves.get(&self.location).unwrap().flow_rate;
                answer.push(self.do_open_valve(flow_rate));
            }
            for next_location in &valve_maze.valves.get(&self.location).unwrap().leads_to {
                answer.push(self.do_go_to(&next_location));
            }
        }
        answer
    }

    /// The state we get to by opening a valve from here. Assumes that it's valid to do. The
    /// flow_rate of this valve is passed in.
    fn do_open_valve(&self, flow_rate: Num) -> Self {
        let valve_maze = self.valve_maze;
        let location = self.location.clone();
        let mut prev_steps = self.prev_steps.clone();
        prev_steps.push_back(Step::OpenValve(self.location.clone()));
        let mut unopened_valves = self.unopened_valves.clone();
        unopened_valves.remove(&self.location);
        let new_pressure_released = (flow_rate as usize) * (MAX_STEPS - self.time_completed);
        let time_completed = self.time_completed + 1;
        let score = Self::calc_score(
            self.valve_maze,
            time_completed,
            &unopened_valves,
            self.pressure_released() + new_pressure_released
        );
        SolverState{
            valve_maze,
            location,
            time_completed,
            prev_steps,
            unopened_valves,
            score
        }
    }

    /// The state we get to by taking a step from here. Assumes that it's valid to do. the
    /// step is passed in.
    fn do_go_to(&self, next_location: &String) -> Self {
        let valve_maze = self.valve_maze;
        let location = next_location.clone();
        let time_completed = self.time_completed + 1;
        let mut prev_steps = self.prev_steps.clone();
        prev_steps.push_back(Step::MoveTo(next_location.clone()));
        let unopened_valves = self.unopened_valves.clone();
        let score = Self::calc_score(valve_maze, time_completed, &unopened_valves, self.pressure_released());
        SolverState{
            valve_maze,
            location,
            time_completed,
            prev_steps,
            unopened_valves,
            score,
        }
    }

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
fn solve(valve_maze: &ValveMaze) -> SolverState {
    let mut states_tried = 0;
    let mut best_state = SolverState::initial(valve_maze);
    let mut states_to_try: BinaryHeap<SolverState> = BinaryHeap::from([SolverState::initial(valve_maze)]);
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
                if state.max_possible() > best_released {
                    for next_state in state.next_states(valve_maze) {
                        if next_state.max_possible() > best_released {
                            states_to_try.push(next_state); // they get sorted as they are inserted
                        }
                    }
                }
                // Check if this one is the new best state
                if state.pressure_released() > best_state.pressure_released() {
                    if PRINT_WORK {
                        println!("New best: [{}, {}] -> {} {:?}", state.pressure_released(), state.possible_release(), state.max_possible(), state.prev_steps); // FIXME: debugging
                    }
                    best_state = state;
                } else {
                    if PRINT_WORK {
                        println!("   tried: [{}, {}] -> {} {:?}", state.pressure_released(), state.possible_release(), state.max_possible(), state.prev_steps); // FIXME: debugging
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
    let solved_state = solve(&valve_maze);
    println!("Path {:?}", solved_state.prev_steps);
    println!("Releases {}", solved_state.pressure_released());
}


fn part_b(_input: &Vec<ValveDesc>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}


// ======= Tests =======

