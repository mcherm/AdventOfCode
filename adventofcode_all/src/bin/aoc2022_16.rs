
extern crate anyhow;

use std::fs;
use nom;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::line_ending,
};
use nom::character::complete::u32 as nom_Num;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use itertools::Itertools;


// ======= Constants =======

const MAX_STEPS: usize = 4;

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

#[derive(Debug)]
struct Valve {
    flow_rate: Num,
    leads_to: Vec<String>,
}

#[derive(Debug)]
struct ValveMaze {
    valves: BTreeMap<String, Valve>
}

#[derive(Debug, Clone)]
enum Step {
    OpenValve(String),
    MoveTo(String),
}

#[derive(Debug)]
struct SolverState {
    location: String,
    time_completed: usize,
    prev_steps: Vec<Step>,
    opened_valves: BTreeSet<String>,
    pressure_released: usize,
}

struct Solver<'a> {
    valve_maze: &'a ValveMaze,
    // FIXME: Remove the next lines
    // states_to_try: VecDeque<SolverState>,
    // best_state: Option<SolverState>,
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


impl SolverState {

    /// The initial SolverState
    fn initial() -> Self {
        SolverState{
            location: "AA".to_string(),
            time_completed: 0,
            prev_steps: Vec::new(),
            opened_valves: BTreeSet::new(),
            pressure_released: 0,
        }
    }

    /// Returns the list of possible next states after this one
    fn next_states(&self, valve_maze: &ValveMaze) -> Vec<Self> {
        let mut answer = Vec::new();
        if self.time_completed < MAX_STEPS {
            if ! self.opened_valves.contains(&self.location) {
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
        let mut prev_steps = self.prev_steps.clone();
        prev_steps.push(Step::OpenValve(self.location.clone()));
        let mut opened_valves = self.opened_valves.clone();
        opened_valves.insert(self.location.clone());
        let new_pressure_released = (flow_rate as usize) * (MAX_STEPS - self.time_completed);
        SolverState{
            location: self.location.clone(),
            time_completed: self.time_completed + 1,
            prev_steps,
            opened_valves,
            pressure_released: self.pressure_released + new_pressure_released,
        }
    }

    /// The state we get to by taking a step from here. Assumes that it's valid to do. the
    /// step is passed in.
    fn do_go_to(&self, next_location: &String) -> Self {
        let mut prev_steps = self.prev_steps.clone();
        prev_steps.push(Step::MoveTo(next_location.clone()));
        SolverState{
            location: next_location.clone(),
            time_completed: self.time_completed + 1,
            prev_steps,
            opened_valves: self.opened_valves.clone(),
            pressure_released: self.pressure_released,
        }
    }
}


// struct Solver<'a> {
//     valve_maze: &'a ValveMaze,
//     states_to_try: VecDeque<SolverState>
// }

impl<'a> Solver<'a> {
    fn new(valve_maze: &'a ValveMaze) -> Self {
        Solver{valve_maze}
    }

    /// Solves it, returning the final state
    fn solve(&self) -> SolverState {
        let mut best_state: Option<SolverState> = None;
        let mut states_to_try = VecDeque::from([SolverState::initial()]);
        loop {
            match states_to_try.pop_front() {
                None => {
                    // Nothing left to try so we've solved it
                    return best_state.unwrap();
                }
                Some(state) => {
                    states_to_try.extend(state.next_states(self.valve_maze));
                    match &best_state {
                        None => best_state = Some(state),
                        Some(best) => if state.pressure_released > best.pressure_released {
                            best_state = Some(state);
                        }
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
    let solver = Solver::new(&valve_maze);
    let solved_state = solver.solve();
    println!("Path {:?}", solved_state.prev_steps);
    println!("Releases {}", solved_state.pressure_released);
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

