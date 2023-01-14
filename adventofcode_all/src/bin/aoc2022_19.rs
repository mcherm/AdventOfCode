
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
    use strum::IntoEnumIterator;
    use strum_macros::{Display as StrumDisplayMacro, EnumIter, EnumCount as EnumCountMacro};

    const MAX_MINUTES: Num = 24;
    const PRINT_WORK: bool = true;

    #[derive(Debug, Copy, Clone, StrumDisplayMacro, EnumCountMacro, EnumIter)]
    enum Resource {Ore, Clay, Obsidian, Geode}
    use Resource::{Ore, Clay, Obsidian, Geode};


    #[derive(Debug, Copy, Clone)]
    enum Action {
        BuildRobot(Resource),
        Wait1Min,
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    struct ResourceState {
        stuff: Num,
        robots: Num,
        cooking: Num,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    struct State {
        minute: Num,
        by_resource: Vec<ResourceState>, // one for each resource; can be indexed by resource.index()
    }


    /// Looks at the blueprint and returns the cost in to_spend to build a robot for
    /// harvesting to_build.
    fn build_cost(bp: &Blueprint, to_build: Resource, to_spend: Resource) -> Num {
        match (to_build, to_spend) {
            (Ore, Ore) => bp.ore_robot_ore,
            (Clay, Ore) => bp.clay_robot_ore,
            (Obsidian, Ore) => bp.obsidian_robot_ore,
            (Obsidian, Clay) => bp.obsidian_robot_clay,
            (Geode, Ore) => bp.geode_robot_ore,
            (Geode, Obsidian) => bp.geode_robot_obsidian,
            (_, _) => 0,
        }
    }

    impl Resource {
        fn index(&self) -> usize {
            match self {
                Ore => 0,
                Clay => 1,
                Obsidian => 2,
                Geode => 3,
            }
        }
    }


    impl Display for Action {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Action::BuildRobot(resource) => write!(f, "Build{resource}Robot"),
                Action::Wait1Min => write!(f, "Wait1Min"),
            }
        }
    }

    impl State {
        /// Returns the list of possible actions to take from this state. They are in order by their
        /// likely value.
        fn possible_actions(&self, bp: &Blueprint) -> Vec<Action> {
            let mut actions = Vec::new();
            if self.minute < MAX_MINUTES - 1 { // if there's time for a robot to do any good...
                if self.stuff(Ore) >= bp.geode_robot_ore && self.stuff(Obsidian) >= bp.geode_robot_obsidian {
                    actions.push(Action::BuildRobot(Geode));
                }
                if self.stuff(Ore) >= bp.obsidian_robot_ore && self.stuff(Clay) >= bp.obsidian_robot_clay {
                    actions.push(Action::BuildRobot(Obsidian));
                }
                if self.stuff(Ore) >= bp.clay_robot_ore {
                    actions.push(Action::BuildRobot(Clay));
                }
                if self.stuff(Ore) >= bp.ore_robot_ore {
                    actions.push(Action::BuildRobot(Ore));
                }
            }
            if self.minute < MAX_MINUTES { // FIXME: is this an off-by-one error?
                actions.push(Action::Wait1Min);
            }
            actions
        }

        /// Returns the State reached by applying the given action.
        fn apply(&self, bp: &Blueprint, action: Action) -> Self {
            let answer = match action {
                Action::BuildRobot(build_resource) => {
                    let mut value: Self = self.clone();
                    value.by_resource[build_resource.index()].cooking += 1;
                    for cost_resource in Resource::iter() {
                        value.by_resource[cost_resource.index()].stuff -=
                            build_cost(bp, build_resource, cost_resource);
                    }
                    value
                },
                Action::Wait1Min => Self{
                    minute: self.minute + 1,
                    by_resource: self.by_resource.iter()
                        .map(|x| ResourceState{
                            stuff: x.stuff + x.robots,
                            robots: x.robots + x.cooking,
                            cooking: 0,
                        })
                        .collect(),
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

        /// Returns the amount of the resource we have in storage
        fn stuff(&self, r: Resource) -> Num {
            self.by_resource[r.index()].stuff
        }

        /// Returns the amount of robots we have for this resource
        fn robots(&self, r: Resource) -> Num {
            self.by_resource[r.index()].robots
        }

        /// Returns the amount of robots cooking we have for this resource
        fn cooking(&self, r: Resource) -> Num {
            self.by_resource[r.index()].cooking
        }

        /// Returns the minimum number of that resource we are guaranteed to have at the end.
        fn min_by_end(&self, r: Resource) -> Num {
            let time_left = MAX_MINUTES - self.minute;
            self.stuff(r) + time_left * self.robots(r) + time_left.saturating_sub(1) * self.cooking(r)
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
                by_resource: vec![
                    ResourceState{stuff: 0, robots: 1, cooking: 0},
                    ResourceState{stuff: 0, robots: 0, cooking: 0},
                    ResourceState{stuff: 0, robots: 0, cooking: 0},
                    ResourceState{stuff: 0, robots: 0, cooking: 0},
                ],
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
                answer = self.min_by_end(Geode).cmp(&other.min_by_end(Geode)); // sort by geodes_at_end
            }
            if answer == Ordering::Equal {
                answer = self.min_by_end(Obsidian).cmp(&other.min_by_end(Obsidian)); // sort by obsidian_by_end
            }
            if answer == Ordering::Equal {
                answer = self.min_by_end(Clay).cmp(&other.min_by_end(Clay)); // sort by clay_by_end
            }
            if answer == Ordering::Equal {
                answer = self.min_by_end(Ore).cmp(&other.min_by_end(Ore)); // sort by ore_by_end
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
                self.stuff(Ore), self.stuff(Clay), self.stuff(Obsidian), self.stuff(Geode),
                self.robots(Ore), self.robots(Clay), self.robots(Obsidian), self.robots(Geode),
                self.min_by_end(Ore), self.min_by_end(Clay), self.min_by_end(Obsidian), self.min_by_end(Geode),
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
                    return best_state.stuff(Geode);
                }
                Some(state) => {
                    // Make sure it's still POSSIBLE for this one to beat the best
                    if true /*state.max_geodes_by_end() > best_geodes_by_end*/ { // FIXME: I've disabled pruning
                        // Add the possible next states onto the list
                        for next_state in state.next_states(bp) {
                            if true /*next_state.max_geodes_by_end() > best_geodes_by_end*/ { // FIXME: I've disabled pruning
                                states_to_try.push(next_state); // they get sorted as they are inserted
                            }
                        }
                        // Check if this one is the new best state
                        if state.stuff(Geode) > best_state.stuff(Geode) {
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

