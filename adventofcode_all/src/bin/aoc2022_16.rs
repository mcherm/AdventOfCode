
extern crate anyhow;



// ======= Constants =======

// FIXME: Remove
const PRINT_WORK: bool = true;
// const PRINT_RESULTS: bool = false;
const MAX_STEPS: usize = 30;

// ======= Parsing =======

mod parse {
    use std::fs;
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
        character::complete::line_ending,
    };
    use nom::character::complete::u32 as nom_Num;
    use std::fmt::{Debug, Display, Formatter};


    pub fn input() -> Result<Vec<ValveDesc>, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_16.txt")?;
        match ValveDesc::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    pub type Num = u32;


    /// Represents the name of a valve (which is always 2 upper-case ascii letters).
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct ValveName {
        code: u16,
    }


    #[derive(Debug)]
    pub struct ValveDesc {
        pub name: ValveName,
        pub flow_rate: Num,
        pub leads_to: Vec<ValveName>,
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
        pub const START: Self = ValveName{code: 0};

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

}



// ======= Part 1 Compute =======

mod matrix {
    use crate::parse::{Num, ValveName, ValveDesc};
    use std::fmt::{Debug, Display, Formatter};
    use std::collections::HashMap;
    use itertools::Itertools;


    /// Represents the best possible path between two particular (non-zero-flow) nodes.
    #[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct BestPath {
        pub path: Vec<ValveName>, // intermediate valves NOT including start but including end node
    }

    type PathMap = HashMap<(ValveName,ValveName), BestPath>;

    /// A fixed structure of the non-zero valves and the distances between them.
    #[derive(Debug, Eq, PartialEq)]
    pub struct ValveMatrix {
        pub key_valves: HashMap<ValveName,Num>,
        pub dist: PathMap,
    }


    impl BestPath {
        pub fn cost(&self) -> usize {
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
        while !unexplored.is_empty() {
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
        pub fn new(valve_descs: &Vec<ValveDesc>) -> Self {
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

}


mod solve {
    use crate::matrix::ValveMatrix;
    use crate::parse::{Num, ValveName};
    use crate::{MAX_STEPS, PRINT_WORK};
    use std::collections::BinaryHeap;
    use std::fmt::{Display, Formatter};
    use std::cmp::Ordering;
    use itertools::Itertools;


    /// A single step that a single actor can take.
    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub enum Step {
        OpenValve(ValveName),
        MoveTo(ValveName),
    }

    /// The action of traveling from a certain location to another, then opening the valve there.
    #[derive(Debug)]
    pub struct Action {
        start: ValveName,
        end: ValveName,
    }


    #[derive(Debug, Eq, PartialEq)]
    pub struct SolverState1 {
        location: ValveName,
        unopened_valves: Vec<ValveName>, // names of the valves that aren't open yet
        unopened_flow: Vec<Num>, // SORTED (biggest first) list of the flow rates for the unopened valves
        score: [Num; 2], // the score is [pressure_released, possible_release]
        steps: Vec<Step>,
    }



    impl Display for Step {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Step::MoveTo(loc) => write!(f, "M({})", loc),
                Step::OpenValve(loc) => write!(f, "O({})", loc),
            }
        }
    }

    impl Action {
        fn new(start: ValveName, end: ValveName) -> Self {
            Action{start, end}
        }

        /// Returns the number of steps needed to get there and then get the valve open.
        fn cost(&self, valve_matrix: &ValveMatrix) -> usize {
            valve_matrix.dist.get(&(self.start, self.end)).unwrap().cost() + 1
        }
    }

    impl Display for Action {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Do({})", self.end)
        }
    }


    /// Solves it, returning the final state
    pub fn solve_1(valve_matrix: &ValveMatrix) -> SolverState1 {
        let mut states_tried = 0;
        let mut states_to_try: BinaryHeap<SolverState1> = BinaryHeap::from([SolverState1::initial(valve_matrix)]);
        let mut best_state: SolverState1 = SolverState1::initial(valve_matrix);
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
                    for next_state in state.next_states(valve_matrix) {
                        if next_state.max_possible() > best_released {
                            states_to_try.push(next_state); // they get sorted as they are inserted
                        }
                    }
                    // Check if this one is the new best state
                    if state.pressure_released() > best_state.pressure_released() {
                        if PRINT_WORK {
                            println!(
                                "New best: [{}, {}] -> {} (tried {}, have {} more; next has [{}]) {}",
                                state.pressure_released(),
                                state.possible_release(),
                                state.max_possible(),
                                states_tried,
                                states_to_try.len(),
                                states_to_try.peek().map_or("N/A".to_string(), |x| x.max_possible().to_string()),
                                state.steps.iter().join(","),
                            );
                        }
                        best_state = state;
                    }
                }
            }
        }
    }


    impl SolverState1 {

        /// Returns a score which requires it to calculate a cap on the the maximum possible future
        /// release. The heuristic used may change over time, but for now it assumes 2 steps to
        /// each location.
        fn calc_score(
            time_completed: usize,
            unopened_flow: &Vec<Num>, // will be sorted with biggest first
            total_pressure_released: Num,
        ) -> [Num;2] {
            let mut remaining_steps = MAX_STEPS - time_completed;
            let mut possible_release = 0;
            let mut flow_iter = unopened_flow.iter();
            while remaining_steps > 0 {
                if remaining_steps > 0 {
                    remaining_steps -= 1; // have to open the valve
                    let flow_rate = match flow_iter.next() {
                        None => break,
                        Some(flow_rate) => flow_rate
                    };
                    possible_release += flow_rate * (remaining_steps as Num);
                    if remaining_steps > 0 {
                        remaining_steps -= 1; // have to walk to a new location
                    }
                }
            }
            [total_pressure_released, possible_release]
        }

        pub fn pressure_released(&self) -> Num {
            return self.score[0];
        }

        fn possible_release(&self) -> Num {
            return self.score[1];
        }

        /// Returns (an overestimate of) the largest score it's possible to get, starting from this location.
        fn max_possible(&self) -> Num {
            self.pressure_released() + self.possible_release()
        }

        /// Returns the number of steps we can take after this one.
        pub fn remaining_steps(&self) -> usize {
            MAX_STEPS - self.time_completed()
        }

        /// Returns the number of steps already taken in reaching this state.
        pub fn time_completed(&self) -> usize {
            self.steps.len()
        }

        /// Returns the list of steps to take to arrive at this state.
        pub fn steps(&self) -> &Vec<Step> {
            &self.steps
        }

        /// The initial SolverState
        fn initial(valve_matrix: &ValveMatrix) -> Self {
            let location = ValveName::START;
            let time_completed = 0;
            let unopened_valves: Vec<ValveName> = valve_matrix.key_valves.keys()
                .map(|x| *x)
                .collect();
            let unopened_flow: Vec<Num> = valve_matrix.key_valves.values()
                .sorted_by_key(|x| std::cmp::Reverse(*x))
                .map(|x| *x)
                .collect();
            let total_pressure_released = 0;
            let score = Self::calc_score(time_completed, &unopened_flow, total_pressure_released);
            let steps = Vec::new();
            Self{location, unopened_valves, unopened_flow, score, steps}
        }

        /// Returns the list of possible next states from this state.
        fn next_states(&self, valve_matrix: &ValveMatrix) -> Vec<Self> {
            self.next_actions_from_state(valve_matrix).into_iter()
                .map(|group_step| self.build_next_state(valve_matrix, group_step))
                .collect_vec()
        }

        /// Returns the list of possible next actions that can be taken from the current state.
        fn next_actions_from_state(&self, valve_matrix: &ValveMatrix) -> Vec<Action> {
            self.next_actions_from_loc(valve_matrix, self.location)
        }

        /// Returns the list of possible next steps for an agent starting from the given location
        fn next_actions_from_loc(&self, valve_matrix: &ValveMatrix, my_loc: ValveName) -> Vec<Action> {
            // We could move to everything there's still time to reach...
            self.unopened_valves.iter()
                .filter_map(|x| {
                    if my_loc == *x {
                        None
                    } else {
                        let action = Action::new(my_loc, *x);
                        // must be LESS (not equal) so we complete the opening and gain SOME benefit from the release
                        if action.cost(valve_matrix) < self.remaining_steps() {
                            Some(Action::new(my_loc, *x))
                        } else {
                            None
                        }
                    }
                })
                .collect()
        }

        /// Given an Action to take from here, this returns the new state that step would reach.
        fn build_next_state(&self, valve_matrix: &ValveMatrix, action: Action) -> Self {
            let location = action.end;

            let mut unopened_valves = self.unopened_valves.clone();
            let index_to_delete = unopened_valves.iter().position(|x| *x == location).unwrap();
            unopened_valves.swap_remove(index_to_delete); // don't have to maintain order

            let mut unopened_flow = self.unopened_flow.clone();
            let flow_to_delete = valve_matrix.key_valves.get(&location).unwrap();
            let index_to_delete = unopened_flow.iter().position(|x| x == flow_to_delete).unwrap();
            unopened_flow.remove(index_to_delete); // must maintain order

            let mut steps = self.steps.clone();
            let best_path = valve_matrix.dist.get(&(action.start, action.end)).unwrap();
            steps.extend(best_path.path.iter().map(|loc| Step::MoveTo(*loc)));
            steps.push(Step::OpenValve(location));

            let new_pressure_released = *flow_to_delete * ((MAX_STEPS - steps.len()) as Num);
            let total_pressure_released = self.pressure_released() + new_pressure_released;

            let score: [Num; 2] = Self::calc_score(steps.len(), &unopened_flow, total_pressure_released);

            Self{location, unopened_valves, unopened_flow, score, steps}
        }

    }

    impl PartialOrd for SolverState1 {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other)) // just default to the total ordering
        }
    }

    impl Ord for SolverState1 {
        fn cmp(&self, other: &Self) -> Ordering {
            let mut answer = self.score.cmp(&other.score); // sort by score
            if answer == Ordering::Equal {
                answer = self.steps().cmp(&other.steps()); // break ties with path
            }
            answer
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

use crate::parse::{ValveDesc, input};
use crate::matrix::ValveMatrix;



fn part_a(input: &Vec<ValveDesc>) {
    println!("\nPart a:");
    let valve_matrix = ValveMatrix::new(input);
    let solved_state = solve::solve_1(&valve_matrix);
    println!("Path {:?}", solved_state.steps());
    println!("Releases {}", solved_state.pressure_released());
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

