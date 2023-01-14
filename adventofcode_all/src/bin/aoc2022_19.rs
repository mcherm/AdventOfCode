
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fs;
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
        combinator::{opt, map, peek},
        character::complete::{multispace1, line_ending, u32 as nom_Num},
        sequence::tuple,
        multi::many0,
    };


    pub fn input() -> Result<Vec<Blueprint>, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_19.txt")?;
        match Blueprint::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    pub type Num = u32;


    #[derive(Debug, Copy, Clone)]
    pub struct Blueprint {
        pub id: Num,
        pub ore_robot_ore: Num,
        pub clay_robot_ore: Num,
        pub obsidian_robot_ore: Num,
        pub obsidian_robot_clay: Num,
        pub geode_robot_ore: Num,
        pub geode_robot_obsidian: Num,
    }

    impl Blueprint {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            map(
                tuple((
                    tuple((
                        tag("Blueprint "),
                        nom_Num,
                        tag(":"),
                        multispace1,
                    )),
                    tuple((
                        tag("Each ore robot costs "),
                        nom_Num,
                        tag(" ore."),
                        multispace1,
                    )),
                    tuple((
                        tag("Each clay robot costs "),
                        nom_Num,
                        tag(" ore."),
                        multispace1,
                    )),
                    tuple((
                        tag("Each obsidian robot costs "),
                        nom_Num,
                        tag(" ore and "),
                        nom_Num,
                        tag(" clay."),
                        multispace1,
                    )),
                    tuple((
                        tag("Each geode robot costs "),
                        nom_Num,
                        tag(" ore and "),
                        nom_Num,
                        tag(" obsidian."),
                    )),
                )),
                |(
                    (_, id, _, _,),
                    (_, ore_robot_ore, _, _),
                    (_, clay_robot_ore, _, _),
                    (_, obsidian_robot_ore, _, obsidian_robot_clay, _, _),
                    (_, geode_robot_ore, _, geode_robot_obsidian, _,),
                 )| Blueprint{
                    id,
                    ore_robot_ore,
                    clay_robot_ore,
                    obsidian_robot_ore,
                    obsidian_robot_clay,
                    geode_robot_ore,
                    geode_robot_obsidian,
                }
            )(input)
        }

        /// Parses a newline-terminated list of Blueprints
        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            many0(
                map(
                    tuple((
                        Self::parse,
                        opt(tuple(( line_ending, peek(line_ending) ))),
                        line_ending,
                    )),
                    |(blueprint, _, _)| blueprint
                )
            )(input)
        }
    }

}



// ======= Part 1 Compute =======

mod maxbuild {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;
    use std::fmt::{Display, Formatter};
    use crate::parse::{Blueprint, Num};

    const MAX_MINUTES: Num = 24;
    const PRINT_WORK: bool = true;


    #[derive(Debug, Copy, Clone)]
    enum Action {
        BuildOreRobot,
        BuildClayRobot,
        BuildObsidianRobot,
        BuildGeodeRobot,
        Wait1Min,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct State {
        minute: Num,
        ore: Num,
        clay: Num,
        obsidian: Num,
        geodes: Num,
        ore_robots: Num,
        clay_robots: Num,
        obsidian_robots: Num,
        geode_robots: Num,
        ore_robots_cooking: Num,
        clay_robots_cooking: Num,
        obsidian_robots_cooking: Num,
        geode_robots_cooking: Num,
    }


    impl Display for Action {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", match self {
                Action::BuildOreRobot => "BuildOreRobot",
                Action::BuildClayRobot => "BuildClayRobot",
                Action::BuildObsidianRobot => "BuildObsidianRobot",
                Action::BuildGeodeRobot => "BuildGeodeRobot",
                Action::Wait1Min => "Wait1Min",
            })
        }
    }

    impl State {
        /// Returns the list of possible actions to take from this state. They are in order by their
        /// likely value.
        fn possible_actions(&self, bp: &Blueprint) -> Vec<Action> {
            let mut actions = Vec::new();
            if self.ore >= bp.geode_robot_ore && self.obsidian >= bp.geode_robot_obsidian {
                actions.push(Action::BuildGeodeRobot);
            }
            if self.ore >= bp.obsidian_robot_ore && self.clay >= bp.obsidian_robot_clay {
                actions.push(Action::BuildObsidianRobot);
            }
            if self.ore >= bp.clay_robot_ore {
                actions.push(Action::BuildClayRobot);
            }
            if self.ore >= bp.ore_robot_ore {
                actions.push(Action::BuildOreRobot);
            }
            if self.minute < MAX_MINUTES { // FIXME: is this an off-by-one error?
                actions.push(Action::Wait1Min);
            }
            actions
        }

        /// Returns the State reached by applying the given action.
        fn apply(&self, bp: &Blueprint, action: Action) -> Self {
            let answer = match action {
                Action::BuildOreRobot => Self{
                    ore_robots_cooking: self.ore_robots_cooking + 1,
                    ore: self.ore - bp.ore_robot_ore,
                    ..*self
                },
                Action::BuildClayRobot => Self{
                    clay_robots_cooking: self.clay_robots_cooking + 1,
                    ore: self.ore - bp.clay_robot_ore,
                    ..*self
                },
                Action::BuildObsidianRobot => Self{
                    obsidian_robots_cooking: self.obsidian_robots_cooking + 1,
                    ore: self.ore - bp.obsidian_robot_ore,
                    clay: self.clay - bp.obsidian_robot_clay,
                    ..*self
                },
                Action::BuildGeodeRobot => Self{
                    geode_robots_cooking: self.geode_robots_cooking + 1,
                    ore: self.ore - bp.geode_robot_ore,
                    obsidian: self.obsidian - bp.geode_robot_obsidian,
                    ..*self
                },
                Action::Wait1Min => Self{
                    minute: self.minute + 1,
                    ore: self.ore + self.ore_robots,
                    clay: self.clay + self.clay_robots,
                    obsidian: self.obsidian + self.obsidian_robots,
                    geodes: self.geodes + self.geode_robots,
                    ore_robots: self.ore_robots + self.ore_robots_cooking,
                    clay_robots: self.clay_robots + self.clay_robots_cooking,
                    obsidian_robots: self.obsidian_robots + self.obsidian_robots_cooking,
                    geode_robots: self.geode_robots + self.geode_robots_cooking,
                    ore_robots_cooking: 0,
                    clay_robots_cooking: 0,
                    obsidian_robots_cooking: 0,
                    geode_robots_cooking: 0,
                },
            };
            println!("        From {} if I {} we get {}", self, action, answer);
            answer
        }

        /// Returns a vector of the possible next states from here.
        fn next_states(&self, bp: &Blueprint) -> Vec<State> {
            self.possible_actions(bp).iter()
                .map(|a| self.apply(bp, *a))
                .collect()
        }

        /// Returns the minimum number of geodes at the end we are guaranteed to have.
        fn geodes_by_end(&self) -> Num {
            self.geodes + (MAX_MINUTES - self.minute) * (self.geode_robots + self.geode_robots_cooking)
        }

        /// Returns the minimum number of geodes at the end we are guaranteed to have.
        fn obsidian_by_end(&self) -> Num {
            self.obsidian + (MAX_MINUTES - self.minute) * (self.obsidian_robots + self.obsidian_robots_cooking)
        }

        /// Returns the minimum number of geodes at the end we are guaranteed to have.
        fn clay_by_end(&self) -> Num {
            self.clay + (MAX_MINUTES - self.minute) * (self.clay_robots + self.clay_robots_cooking)
        }

        /// Returns the minimum number of geodes at the end we are guaranteed to have.
        fn ore_by_end(&self) -> Num {
            self.ore + (MAX_MINUTES - self.ore) * (self.ore_robots + self.ore_robots_cooking)
        }

        // FIXME: If I am do do pruning, I will need this.
        // /// Returns the maximum possible number of geodes at the end we could
        // /// possibly have. This is a heuristic.
        // fn max_geodes_by_end(&self) -> Num {
        //     todo!()
        // }
    }

    impl Default for State {
        fn default() -> Self {
            State{
                minute: 0,
                ore: 0,
                clay: 0,
                obsidian: 0,
                geodes: 0,
                ore_robots: 1,
                clay_robots: 0,
                obsidian_robots: 0,
                geode_robots: 0,
                ore_robots_cooking: 0,
                clay_robots_cooking: 0,
                obsidian_robots_cooking: 0,
                geode_robots_cooking: 0,
            }
        }
    }

    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other)) // just default to the total ordering
        }
    }

    impl Ord for State {
        fn cmp(&self, other: &Self) -> Ordering {
            let mut answer = Ordering::Equal;
            if answer == Ordering::Equal {
                answer = self.geodes_by_end().cmp(&other.geodes_by_end()); // sort by geodes_at_end
            }
            if answer == Ordering::Equal {
                answer = self.obsidian_by_end().cmp(&other.obsidian_by_end()); // sort by obsidian_by_end
            }
            if answer == Ordering::Equal {
                answer = self.clay_by_end().cmp(&other.clay_by_end()); // sort by obsidian_by_end
            }
            if answer == Ordering::Equal {
                answer = self.ore_by_end().cmp(&other.ore_by_end()); // sort by obsidian_by_end
            }
            if answer == Ordering::Equal {
                answer = other.minute.cmp(&self.minute); // reverse sort by minute
            }
            answer
        }
    }

    impl Display for State {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "State({}: {},{},{},{} / {},{},{},{} min {},{},{},{})",
                self.minute,
                self.ore, self.clay, self.obsidian, self.geodes,
                self.ore_robots, self.clay_robots, self.obsidian_robots, self.geodes,
                self.ore_by_end(), self.clay_by_end(), self.obsidian_by_end(), self.geodes_by_end(),
            )
        }
    }


    /// Finds the maximum number of geodes that can be built and returns it.
    pub fn max_build(bp: &Blueprint) -> Num {
        let state = State::default();
        println!("Start state: {state}");
        println!("Blueprint: {:?}", bp);

        let mut states_tried = 0;
        let mut states_to_try: BinaryHeap<State> = BinaryHeap::from([State::default()]);
        let mut best_state: State = State::default();
        loop {
            if states_tried % 1 == 0 {
                print!("    {states_tried} states tried and {} in the queue. ", states_to_try.len());
                if states_to_try.is_empty() {
                    println!("ALL DONE");
                } else {
                    println!("Next: {}", states_to_try.peek().unwrap());
                }
            }
            states_tried += 1;
            match states_to_try.pop() {
                None => {
                    // Nothing left to try so we've solved it
                    return best_state.geodes_by_end();
                }
                Some(state) => {
                    // Make sure it's still POSSIBLE for this one to beat the best
                    let best_geodes_by_end = best_state.geodes_by_end();
                    if true /*state.max_geodes_by_end() > best_geodes_by_end*/ { // FIXME: I've disabled pruning
                        // Add the possible next states onto the list
                        for next_state in state.next_states(bp) {
                            if true /*next_state.max_geodes_by_end() > best_geodes_by_end*/ { // FIXME: I've disabled pruning
                                states_to_try.push(next_state); // they get sorted as they are inserted
                            }
                        }
                        // Check if this one is the new best state
                        if state.geodes_by_end() > best_geodes_by_end {
                            if PRINT_WORK {
                                println!("New best: {state} after trying {states_tried} states.");
                            }
                            best_state = state;
                        }
                    }
                }
            }
        }
    }
}




// ======= main() =======

use crate::parse::{Blueprint, input};
use crate::maxbuild::max_build;



fn part_a(input: &Vec<Blueprint>) {
    println!("\nPart a:");
    let bp = input[0];
    let geodes = max_build(&bp);
    println!("We produced {geodes} geodes.")
}


fn part_b(_input: &Vec<Blueprint>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}

