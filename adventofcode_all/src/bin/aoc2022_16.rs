
extern crate anyhow;

//use std::cmp::Ordering;
use std::fs;
use nom;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::line_ending,
};
use nom::character::complete::u32 as nom_Num;
use std::collections::{/*BinaryHeap, BTreeMap,*/ HashMap};
//use im::ordset::OrdSet;
//use im::Vector;
//use itertools::Itertools;
use std::fmt::{Debug, Display, Formatter};
use itertools::Itertools;


// ======= Constants =======

// FIXME: Remove
//const PRINT_WORK: bool = true;
//const PRINT_RESULTS: bool = false;
//const MAX_STEPS: usize = 30;

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


/// Represents the name of a valve (which is always 2 upper-case ascii letters).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct ValveName {
    code: u16,
}


#[derive(Debug)]
struct ValveDesc {
    name: ValveName,
    flow_rate: Num,
    leads_to: Vec<ValveName>,
}


impl ValveName {
    /// Construct a ValveName from a vector of exactly 2 chars, each of which is in ['A'..'Z'].
    fn new(vec: Vec<char>) -> Self {
        assert!(vec.len() == 2);
        assert!(vec[0] >= 'A' && vec[0] <= 'Z');
        assert!(vec[1] >= 'A' && vec[1] <= 'Z');
        let digit0 = (vec[0] as u16) - ('A' as u16);
        let digit1 = (vec[1] as u16) - ('A' as u16);
        let code = digit0 * 26 + digit1;
        ValveName{code}
    }

    /// Construct the starting ValveName.
    const START: Self = ValveName{code: 0};

    /// Parse a ValveName
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        nom::combinator::map(
            nom::multi::count(
                nom::character::complete::satisfy(|c| c >= 'A' && c <= 'Z'),
                2
            ),
            ValveName::new
        )(input)
    }

    /// Returns the chars making up this ValveName.
    fn chars(&self) -> [char; 2] {
        [
            ((self.code / 26) as u8 + 'A' as u8) as char,
            ((self.code % 26) as u8 + 'A' as u8) as char,
        ]
    }
}


impl Display for ValveName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let [c0,c1] = self.chars();
        write!(f, "{}{}", c0, c1)
    }
}

impl Debug for ValveName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}


/// Convert strings to ValveNames. Will panic if the strings isn't a valid ValveName.
impl From<&str> for ValveName {
    fn from(s: &str) -> Self {
        assert!(s.len() == 2);
        ValveName::new(s.chars().collect())
    }
}


//Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
impl ValveDesc {

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        nom::combinator::map(
            nom::sequence::tuple((
                tag("Valve "),
                ValveName::parse,
                tag(" has flow rate="),
                nom_Num,
                nom::branch::alt((
                    tag("; tunnels lead to valves "),
                    tag("; tunnel leads to valve "),
                )),
                nom::multi::separated_list1( tag(", "), ValveName::parse ),
            )),
            |(_, name, _, flow_rate, _, leads_to)| ValveDesc{
                name,
                flow_rate,
                leads_to
            }
        )(input)
    }

    /// Parses a newline-terminated list of LineSpecs
    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        nom::multi::many1( nom::sequence::terminated(Self::parse, line_ending) )(input)
    }

}



// ======= Part 1 Compute =======

/// Represents the best possible path between two particular (non-zero-flow) nodes.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct BestPath {
    path: Vec<ValveName>, // intermediate valves NOT including start but including end node
}

type PathMap = HashMap<(ValveName,ValveName), BestPath>;

/// A fixed structure of the non-zero valves and the distances between them.
#[derive(Debug)]
struct ValveMatrix {
    key_valves: HashMap<ValveName,Num>,
    dist: PathMap,
}


impl BestPath {
    fn cost(&self) -> usize {
        self.path.len()
    }
}

impl Display for BestPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.cost(), self.path.iter().join("-"))
    }
}

/// Uses Dijkstra's Algorithm to find the best paths from one particular node to other key values.
fn find_best_paths_from_valve(valve_descs: &Vec<ValveDesc>, key_valves: &HashMap<ValveName,Num>, start_valve: ValveName) -> Vec<(ValveName, BestPath)> {
    // for a given valve, gives the distance from start_valve
    let mut dist: HashMap<ValveName, Option<usize>> =
        valve_descs.iter().map(|x| (x.name, None)).collect(); // initialize to "don't know anything"
    // for a given valve, gives the previous valve in the path from start_valve
    let mut prev: HashMap<ValveName, Option<ValveName>> =
        valve_descs.iter().map(|x| (x.name, None)).collect(); // initialize to "don't know anything"
    dist.insert(start_valve, Some(0));
    // list of ValveNames we have not yet explored
    let mut unexplored: Vec<ValveName> =
        valve_descs.iter().map(|x| (x.name)).collect(); // initialize to "all of them"
    println!("dist: {:?}", dist); // FIXME: Remove
    println!("prev: {:?}", prev); // FIXME: Remove
    println!("Unexplored: {:?}", unexplored); // FIXME: Remove
    while !unexplored.is_empty() {
        // println!("Unexplored: {:?}", unexplored); // FIXME: Remove
        // println!("dist: {:?}", dist); // FIXME: Remove
        let min_dist = unexplored.iter().filter_map(|x| *dist.get(x).unwrap()).min().unwrap();
        let min_item_index = unexplored.iter().position(|x| {
            let d = dist.get(x).unwrap();
            d.is_some() && d.unwrap() == min_dist
        }).unwrap();
        let this_valve = unexplored.swap_remove(min_item_index); // make progress by mapping one new valve in each loop
        let this_dist = dist.get(&this_valve).unwrap().expect("Graph is not connected");
        let this_neighbors = &valve_descs.iter().find(|x| x.name == this_valve).unwrap().leads_to;
        for neighbor in this_neighbors {
            let found_better_path = match dist.get(neighbor).unwrap() {
                None => true, // if neighbor was never connected, we'll update it for sure
                Some(old_dist) => this_dist + 1 < *old_dist, // update only if new distance is better
            };
            if found_better_path {
                dist.insert(*neighbor, Some(this_dist + 1));
                prev.insert(*neighbor, Some(this_valve));
            }
        }
    }

    // --- Having found them all, construct the result ---
    let make_path = |valve: ValveName| -> BestPath {
        let cost = dist.get(&valve).unwrap().unwrap();
        let mut path = Vec::with_capacity(cost - 1);
        let mut vv = valve;
        while vv != start_valve {
            path.insert(0, vv);
            vv = prev.get(&vv).unwrap().unwrap();
        }
        BestPath{path}
    };

    // return only paths between START and the various key values
    key_valves.keys()
        .filter(|x| **x != start_valve)
        .map(|v| (*v, make_path(*v)))
        .collect()
}


/// Uses Dijkstra's Algorithm to find the best paths between key values.
fn find_best_paths(valve_descs: &Vec<ValveDesc>, key_valves: &HashMap<ValveName,Num>) -> PathMap {
    let mut answer = PathMap::new();
    // find paths from every key value (non-zero flow) and also from START
    for start_valve in key_valves.keys().chain(std::iter::once(&ValveName::START)) {
        for (end_valve, best_path) in find_best_paths_from_valve(valve_descs, key_valves, *start_valve) {
            answer.insert((*start_valve, end_valve), best_path);
        }
    }
    answer
}

impl ValveMatrix {
    /// Construct a ValveMatrix from a list of ValveDescs.
    fn new(valve_descs: &Vec<ValveDesc>) -> Self {
        let key_valves = valve_descs.iter()
            .filter_map(|x| match x.flow_rate {0 => None, _ => Some((x.name, x.flow_rate))})
            .collect();
        let dist = find_best_paths(valve_descs, &key_valves);
        ValveMatrix{key_valves, dist}
    }
}


impl Display for ValveMatrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "==== ValveMatrix ====")?;
        writeln!(f, "Valves:")?;
        for name in self.key_valves.keys().sorted() {
            writeln!(f, "    {} -> {}", name, self.key_valves.get(name).unwrap())?;
        }
        writeln!(f, "Distances:")?;
        for ((start_valve, end_valve), best_path) in self.dist.iter().sorted() {
            writeln!(f, "    {},{} -> {}", start_valve, end_valve, best_path)?;
        }
        writeln!(f, "=====================")
    }
}

// FIXME: Remove
// #[deprecated(note = "Need better approach")]
// #[derive(Debug, Eq, PartialEq, Clone)]
// struct Valve {
//     flow_rate: Num,
//     leads_to: Vec<ValveName>,
// }

// FIXME: Remove
// #[deprecated(note = "Need better approach")]
// #[derive(Debug, Eq, PartialEq)]
// struct ValveMaze {
//     valves: BTreeMap<ValveName, Valve>
// }

/// A single step that a single actor can take.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Step {
    OpenValve(ValveName),
    MoveTo(ValveName),
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
    Solo(ValveName),
    Pair(ValveName, ValveName),
}


// FIXME: Remove
// #[deprecated(note = "Need better approach")]
// #[derive(Debug, Eq, PartialEq)]
// struct SolverState<'a> {
//     valve_maze: &'a ValveMaze,
//     location: GroupLocation,
//     time_completed: usize,
//     prev_steps: Vector<GroupStep>,
//     unopened_valves: OrdSet<ValveName>,
//     unopened_flow_rates: Vec<usize>,
//     score: [usize; 2], // the score is [pressure_released, possible_release]
// }


// FIXME: Remove
// impl ValveMaze {
//     /// Construct a ValveMaze from the list of ValveDescs.
//     ///
//     /// NOTE: This *could* confirm that every place led to is one that exists (and return an
//     ///   error if not), but for now it is not checking that.
//     fn new(input: &Vec<ValveDesc>) -> Self {
//         let mut valves = BTreeMap::new();
//         for ValveDesc{name, flow_rate, leads_to} in input {
//             let valve = Valve{flow_rate: flow_rate.clone(), leads_to: leads_to.clone()};
//             valves.insert(name.clone(), valve);
//         }
//         ValveMaze{valves}
//     }
// }


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

    // FIXME: Remove
    // /// This is passed an existing set of unopened valves, the corresponding (sorted) unopened
    // /// flow rates, and the current pressure_released. It returns the new values for all three
    // /// of these fields that we will obtain if we enact this GroupStep.
    // fn apply_to_valves(
    //     &self,
    //     valve_maze: &ValveMaze,
    //     time_remaining: usize,
    //     old_unopened_valves: &OrdSet<ValveName>,
    //     old_unopened_flow_rates: &Vec<usize>,
    //     old_pressure_released: usize,
    // ) -> (
    //     OrdSet<ValveName>, // new_unopened_valves
    //     Vec<usize>, // new_unopened_flow_rates
    //     usize, // new_pressure_released
    // ) {
    //     let valves_to_open: Vec<&ValveName> = match self {
    //         GroupStep::Solo(Step::OpenValve(my_valve)) => vec![my_valve],
    //         GroupStep::Solo(Step::MoveTo(_)) => vec![],
    //         GroupStep::Pair(Step::OpenValve(my_valve), Step::OpenValve(el_valve)) if my_valve == el_valve => vec![], // no both opening same valve
    //         GroupStep::Pair(Step::OpenValve(my_valve), Step::OpenValve(el_valve)) => vec![my_valve, el_valve],
    //         GroupStep::Pair(Step::OpenValve(my_valve), Step::MoveTo(_)) => vec![my_valve],
    //         GroupStep::Pair(Step::MoveTo(_), Step::OpenValve(el_valve)) => vec![el_valve],
    //         GroupStep::Pair(Step::MoveTo(_), Step::MoveTo(_)) => vec![],
    //         GroupStep::Training => vec![],
    //     };
    //
    //     let mut new_unopened_valves = old_unopened_valves.clone();
    //     let mut new_unopened_flow_rates = old_unopened_flow_rates.clone();
    //     let mut pressure_released_this_time = 0;
    //     for valve in valves_to_open.iter() {
    //         new_unopened_valves.remove(*valve);
    //         let valve_flow_rate = valve_maze.valves.get(*valve).unwrap().flow_rate as usize;
    //         let pos_to_remove = new_unopened_flow_rates.iter().position(|x| *x == valve_flow_rate).unwrap();
    //         new_unopened_flow_rates.remove(pos_to_remove);
    //         pressure_released_this_time += (valve_flow_rate as usize) * time_remaining;
    //     }
    //     let new_pressure_released = old_pressure_released + pressure_released_this_time;
    //     (new_unopened_valves, new_unopened_flow_rates, new_pressure_released)
    // }

}


impl GroupLocation {
    fn num_actors(&self) -> usize {
        match self {
            GroupLocation::Solo(_) => 1,
            GroupLocation::Pair(_, _) => 2,
        }
    }
}


// FIXME: Remove
// impl<'a> SolverState<'a> {
//
//     /// Returns a cap on the the maximum possible future release. The heuristic used
//     /// may change over time, but for now it assumes instantaneous travel to all locations.
//     fn calc_score(
//         time_completed: usize,
//         unopened_flow_rates: &Vec<usize>,
//         pressure_released: usize,
//         num_actors: usize,
//     ) -> [usize;2] {
//         let mut remaining_steps = MAX_STEPS - time_completed;
//         let mut possible_release = 0;
//         let mut flow_rate_iter = unopened_flow_rates.iter();
//         while remaining_steps > 0 {
//             if remaining_steps > 0 {
//                 remaining_steps -= 1; // have to open the valve
//                 for _actor_num in 0..num_actors {
//                     let flow_rate = match flow_rate_iter.next() {
//                         None => break,
//                         Some(flow_rate) => flow_rate
//                     };
//                     possible_release += flow_rate * remaining_steps;
//                 }
//                 if remaining_steps > 0 {
//                     remaining_steps -= 1; // have to walk to a new location
//                 }
//             }
//         }
//         [pressure_released, possible_release]
//     }
//
//     fn pressure_released(&self) -> usize {
//         return self.score[0];
//     }
//
//     fn possible_release(&self) -> usize {
//         return self.score[1];
//     }
//
//     /// Returns (an overestimate of) the largest score it's possible to get, starting from this location.
//     fn max_possible(&self) -> usize {
//         self.pressure_released() + self.possible_release()
//     }
//
//     /// The initial SolverState
//     fn initial(valve_maze: &'a ValveMaze, has_elephant: bool) -> Self {
//         let location: GroupLocation = if has_elephant {
//             GroupLocation::Pair(ValveName::START, ValveName::START)
//         } else {
//             GroupLocation::Solo(ValveName::START)
//         };
//         let time_completed = 0;
//         let prev_steps = Vector::new();
//         let unopened_valves: OrdSet<ValveName> = valve_maze.valves.iter()
//             .filter_map(|(name, valve)| if valve.flow_rate == 0 {None} else {Some(*name)})
//             .collect();
//         let mut unopened_flow_rates: Vec<usize> = unopened_valves.iter()
//             .map(|name| (valve_maze.valves.get(name).unwrap().flow_rate as usize))
//             .collect();
//         unopened_flow_rates.sort_by_key(|x| std::cmp::Reverse(*x)); // put big ones first
//         let pressure_released = 0;
//         let score = Self::calc_score(time_completed, &unopened_flow_rates, pressure_released, location.num_actors());
//         SolverState{
//             valve_maze,
//             location,
//             time_completed,
//             prev_steps,
//             unopened_valves,
//             unopened_flow_rates,
//             score,
//         }
//     }
//
//     /// Returns the list of possible next steps for an agent starting from the given location
//     fn next_moves_from_loc(&self, location: ValveName) -> Vec<Step> {
//         let mut answer = Vec::new();
//         if self.time_completed < MAX_STEPS {
//             if self.unopened_valves.contains(&location) {
//                 answer.push(Step::OpenValve(location));
//             }
//             for next_location in &self.valve_maze.valves.get(&location).unwrap().leads_to {
//                 answer.push(Step::MoveTo(next_location.clone()));
//             }
//         }
//         answer
//     }
//
//     /// Returns the list of possible next moves that can be made from the current state.
//     fn next_steps_from_state(&self) -> Vec<GroupStep> {
//         match &self.location {
//             GroupLocation::Solo(loc_me) => {
//                 self.next_moves_from_loc(*loc_me).into_iter()
//                     .map(|step| GroupStep::Solo(step))
//                     .collect_vec()
//             }
//             GroupLocation::Pair(loc_me, loc_elephant) => {
//                 if self.time_completed < 4 {
//                     vec![GroupStep::Training]
//                 } else {
//                     let my_moves = self.next_moves_from_loc(*loc_me);
//                     let elephant_moves = self.next_moves_from_loc(*loc_elephant);
//                     my_moves.into_iter().cartesian_product(elephant_moves.into_iter())
//                         .map(|(my_step, elephant_step)| GroupStep::Pair(my_step, elephant_step))
//                         .collect_vec()
//                 }
//             }
//         }
//     }
//
//     /// Returns the list of possible next states from this state.
//     fn next_states(&self) -> Vec<Self> {
//         self.next_steps_from_state().into_iter()
//             .map(|group_step| self.build_next_state(group_step))
//             .collect_vec()
//     }
//
//     /// Given a GroupStep to take from here, this returns the new state that step would reach.
//     fn build_next_state(&self, group_step: GroupStep) -> Self {
//         let valve_maze = self.valve_maze;
//         let location = group_step.apply_to_location(&self.location);
//         let time_completed = self.time_completed + 1;
//         let time_remaining = MAX_STEPS - self.time_completed - 1;
//         let (unopened_valves, unopened_flow_rates, new_pressure_released) = group_step.apply_to_valves(
//             valve_maze,
//             time_remaining,
//             &self.unopened_valves,
//             &self.unopened_flow_rates,
//             self.pressure_released(),
//         );
//         let score = Self::calc_score(time_completed, &unopened_flow_rates, new_pressure_released, location.num_actors());
//         let mut prev_steps = self.prev_steps.clone();
//         prev_steps.push_back(group_step);
//         SolverState{
//             valve_maze,
//             location,
//             time_completed,
//             prev_steps,
//             unopened_valves,
//             unopened_flow_rates,
//             score,
//         }
//     }
//
// }
//
// impl<'a> PartialOrd for SolverState<'a> {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other)) // just default to the total ordering
//     }
// }
//
// impl<'a> Ord for SolverState<'a> {
//     fn cmp(&self, other: &Self) -> Ordering {
//         let mut answer = self.score.cmp(&other.score); // sort by score
//         if answer == Ordering::Equal {
//             answer = self.prev_steps.cmp(&other.prev_steps); // break ties with path
//         }
//         answer
//     }
// }


// FIXME: Remove
// /// Solves it, returning the final state
// fn solve(valve_maze: &ValveMaze, has_elephant: bool) -> SolverState {
//     let mut states_tried = 0;
//     let mut best_state = SolverState::initial(valve_maze, has_elephant);
//     let mut states_to_try: BinaryHeap<SolverState> = BinaryHeap::from([SolverState::initial(valve_maze, has_elephant)]);
//     loop {
//         states_tried += 1;
//         match states_to_try.pop() {
//             None => {
//                 // Nothing left to try so we've solved it
//                 println!("Tried a total of {} states.", states_tried);
//                 return best_state;
//             }
//             Some(state) => {
//                 // Add the possible next states onto the list, but ONLY if it's POSSIBLE for one to beat the best
//                 let best_released = best_state.pressure_released();
//                 for next_state in state.next_states() {
//                     if next_state.max_possible() > best_released {
//                         states_to_try.push(next_state); // they get sorted as they are inserted
//                     }
//                 }
//                 // Check if this one is the new best state
//                 if state.pressure_released() > best_state.pressure_released() {
//                     if PRINT_WORK {
//                         println!(
//                             "New best: [{}, {}] -> {} (tried {}, have {} more; next has {}) {:?}",
//                             state.pressure_released(),
//                             state.possible_release(),
//                             state.max_possible(),
//                             states_tried,
//                             states_to_try.len(),
//                             states_to_try.peek().map_or("N/A".to_string(), |x| x.max_possible().to_string()),
//                             state.prev_steps,
//                         );
//                     }
//                     best_state = state;
//                 }
//             }
//         }
//     }
// }


/// Used inside pretty_print_path().
fn pretty_print_step(actor: &str, step: &Step) {
    match step {
        Step::OpenValve(loc) => {
            println!("{} open valve {}.", actor, loc);
        }
        Step::MoveTo(loc) => {
            println!("{} move to valve {}.", actor, loc);
        }
    }
}

// FIXME: Remove
// #[deprecated(note = "Need better approach")]
// fn released_per_turn(state: &SolverState) -> usize {
//     let open_valves = state.valve_maze.valves.iter()
//         .filter(|(_name,valve)| valve.flow_rate != 0)
//         .filter(|(name,_valve)| !state.unopened_valves.contains(*name))
//         .collect_vec();
//     let flow_rates: usize = open_valves.iter().map(|(_name, valve)| valve.flow_rate as usize).sum();
//     let open_ones = open_valves.iter().map(|(name,_valve)| name).join(", ");
//     println!("Valves {} are open, releasing {:?} pressure.", open_ones, flow_rates);
//     flow_rates
// }
//
// #[deprecated(note = "Need better approach")]
// /// Prints out (in pretty fashion) a path.
// fn pretty_print_path(valve_maze: &ValveMaze, has_elephant: bool, path: &Vector<GroupStep>) {
//     let mut step_iter = path.iter();
//     let mut current_state = SolverState::initial(valve_maze, has_elephant);
//     let mut total_released = 0;
//     for timer in 1..MAX_STEPS {
//         println!("== Minute {} ==", (timer as i32) + if has_elephant {-4} else {1});
//         let step_opt = step_iter.next();
//         match step_opt {
//             None => {
//                 println!("Nothing to do.");
//                 total_released += released_per_turn(&current_state);
//             },
//             Some(step) => {
//                 current_state = current_state.build_next_state(step.clone());
//                 total_released += released_per_turn(&current_state);
//                 match step {
//                     GroupStep::Solo(ref my_step) => {
//                         pretty_print_step("You", my_step);
//                     },
//                     GroupStep::Pair(ref my_step, ref el_step) => {
//                         pretty_print_step("You", my_step);
//                         pretty_print_step("The elephant", el_step);
//                     },
//                     GroupStep::Training => {
//                         println!("You train the elephant.");
//                     }
//                 }
//             },
//         }
//         println!();
//     }
//     println!("Released a total of {}.", total_released);
// }



// ======= main() =======

fn part_a(input: &Vec<ValveDesc>) {
    println!("\nPart a:");
    println!("{:?}", input); // FIXME: Remove
    let valve_matrix = ValveMatrix::new(input);
    println!("valve_matrix: {}", valve_matrix);
    // let has_elephant = false;
    // let solved_state = solve(&valve_maze, has_elephant);
    // println!("Path {:?}", solved_state.prev_steps);
    // println!("Releases {}", solved_state.pressure_released());
}


fn part_b(_input: &Vec<ValveDesc>) {
    println!("\nPart b:");
    // let valve_maze = ValveMaze::new(input);
    // let has_elephant = true;
    // let solved_state = solve(&valve_maze, has_elephant);
    // println!("Path {:?}", solved_state.prev_steps);
    // println!("Releases {}", solved_state.pressure_released());
    // if PRINT_RESULTS {
    //     println!();
    //     pretty_print_path(solved_state.valve_maze, has_elephant, &solved_state.prev_steps);
    // }
    // println!("===========================");
    // let known_path: Vector<GroupStep> = vec![
    //     GroupStep::Training, GroupStep::Training, GroupStep::Training, GroupStep::Training,
    //     GroupStep::Pair(Step::MoveTo("II".into()), Step::MoveTo("DD".into())),
    //     GroupStep::Pair(Step::MoveTo("JJ".into()), Step::OpenValve("DD".into())),
    //     GroupStep::Pair(Step::OpenValve("JJ".into()), Step::MoveTo("EE".into())),
    //     GroupStep::Pair(Step::MoveTo("II".into()), Step::MoveTo("FF".into())),
    //     GroupStep::Pair(Step::MoveTo("AA".into()), Step::MoveTo("GG".into())),
    //     GroupStep::Pair(Step::MoveTo("BB".into()), Step::MoveTo("HH".into())),
    //     GroupStep::Pair(Step::OpenValve("BB".into()), Step::OpenValve("HH".into())),
    //     GroupStep::Pair(Step::MoveTo("CC".into()), Step::MoveTo("GG".into())),
    //     GroupStep::Pair(Step::OpenValve("CC".into()), Step::MoveTo("FF".into())),
    //     GroupStep::Pair(Step::MoveTo("DD".into()), Step::MoveTo("EE".into())),
    //     GroupStep::Pair(Step::MoveTo("CC".into()), Step::OpenValve("EE".into())),
    // ].into();
    // pretty_print_path(&valve_maze, has_elephant, &known_path);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}


// ======= Tests =======

