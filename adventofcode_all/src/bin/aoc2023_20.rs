///
/// Part 2 of this advent of code problem has done something which rather annoys me. The
/// problem to solve is one which is straightforward to code in a simple fashion, but it's
/// execution time for the input problem when coded that way is too long. So far, that's
/// perfectly fine, but there is (as far as I know, and I think this may be provable) no way
/// to write an algorithm that is fundamentally faster and works for any possible input.
///
/// The PARTICULAR input we have received is of a very particular form, which CAN (if you
/// analyze it) be solved. I analyzed my own input by hand and created a diagram, which
/// you can see in "/notes/2023/aoc2023_20_diagram.svg". It has basically been set up to
/// [NOT SURE OF THIS... PROVE IT LATER] multiply 4 large numbers.
///
/// What I am going to set out to do is (1) write the naive code that will technically solve
/// this for any possible input, (2) detect whether the input is of the specific form that
/// mine is, and if so, solve it faster. That way I will technically have something that can
/// solve any possible input, and can solve the actual inputs people get in reasonable time.
///


use std::fmt::{Debug, Display, Formatter};
use anyhow;
use std::collections::{VecDeque, HashMap, HashSet, hash_map};


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ModuleKind {
    Broadcaster, FlipFlop, Conjunction, Output
}

#[derive(Debug, Clone)]
pub struct Module {
    name: String,
    kind: ModuleKind,
    destinations: Vec<String>,
}


/// To protect against bugs, I want to create a variant of HashMap that does not let you
/// alter the keys. It has only the actual calls I happen to be using, and it only allows
/// Strings as keys.
#[derive(Debug, Clone)]
struct FixedStringMap<V>(HashMap<String,V>);

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
enum PulseKind { High, Low }

#[derive(Debug, Clone)]
pub struct Machine {
    modules: FixedStringMap<Module>,
    /// for each FlipFlop, true means "on", false means "off"
    flip_flops: FixedStringMap<bool>,
    /// for each conjunction, for each input, existing and true means it was high, missing or false means it was low
    conjunctions: FixedStringMap<FixedStringMap<PulseKind>>,
}

type Input = Machine;



impl Module {
    fn new<T1: ToString, T2: ToString>(name: T1, kind: ModuleKind, destinations: Vec<T2>) -> Self {
        Module{name: name.to_string(), kind, destinations: destinations.iter().map(|x| x.to_string()).collect()}
    }
}

impl<V> FixedStringMap<V> {
    /// Retrieves a reference to the value from the map, but because the strings are known, this
    /// one returns &T (not Option<&T> like HashMap) and it panics if the string passed isn't
    /// valid.
    fn get(&self, key: &str) -> &V {
        self.0.get(key).expect(&format!("can't find string {}", key))
    }

    /// Retrieves a mutable reference to the value from the map, but because the strings are known,
    /// this one returns &mut T (not Option<&mut T> like HashMap) and it panics if the string
    /// passed isn't valid.
    fn get_mut(&mut self, key: &str) -> &mut V {
        self.0.get_mut(key).unwrap()
    }

    /// Iterator for the values in the map, just like in HashMap.
    fn values(&self) -> hash_map::Values<'_, String, V> {
        self.0.values()
    }
}

impl Display for PulseKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use PulseKind::*;
        write!(f, "{}", match self {High => '+', Low => '-'})
    }
}

impl Machine {
    fn new(modules: Vec<Module>) -> Self {
        use ModuleKind::*;
        use PulseKind::*;
        // -- construct FlipFlop state --
        let flip_flops_map: HashMap<String,bool> = modules.iter()
            .filter_map(|m| if m.kind == FlipFlop {Some((m.name.clone(), false))} else {None})
            .collect();
        let flip_flops = FixedStringMap(flip_flops_map);

        // -- construct Conjunction state --
        let conjunctions_map: HashMap<String, FixedStringMap<PulseKind>> = modules.iter()
            .filter(|m| m.kind == Conjunction)
            .map(|m| {
                let source_state_map: HashMap<String,PulseKind> = modules.iter()
                    .filter(|source_m| source_m.destinations.contains(&m.name))
                    .map(|source_m| (source_m.name.clone(), Low))
                    .collect();
                (m.name.clone(), FixedStringMap(source_state_map))
            })
            .collect();
        let conjunctions = FixedStringMap(conjunctions_map);

        // -- construct the map of states, including the dummy "Output" states for any not mentioned --
        let all_names_sent_to: HashSet<String> = modules.iter()
            .flat_map(|m| m.destinations.iter().map(|dest| dest.clone()))
            .collect();
        let mut module_map: HashMap<String, Module> = modules.into_iter().map(|m| (m.name.clone(), m)).collect();
        for name in all_names_sent_to {
            if !module_map.contains_key(&name) {
                let nowhere: Vec<&str> = Vec::new();
                module_map.insert(name.clone(), Module::new(name, Output, nowhere));
            }
        }
        let modules = FixedStringMap(module_map);

        // -- construct it --
        Self{modules, flip_flops, conjunctions}
    }
}


mod parse {
    use super::{Input, Module, ModuleKind};
    use std::fs;
    use nom;
    use nom::IResult;
    use crate::Machine;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_20.txt")?;
        match Input::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    impl Module {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom::branch::alt(( // this will return a (ModuleKind, name) tuple
                        nom::combinator::map(
                            nom::bytes::complete::tag("broadcaster"),
                            |s| (ModuleKind::Broadcaster, s)
                        ),
                        nom::combinator::map(
                            nom::sequence::tuple((
                                nom::bytes::complete::tag("%"),
                                nom::character::complete::alpha1,
                            )),
                            |(_, name)| (ModuleKind::FlipFlop, name)
                        ),
                        nom::combinator::map(
                            nom::sequence::tuple((
                                nom::bytes::complete::tag("&"),
                                nom::character::complete::alpha1,
                            )),
                            |(_, name)| (ModuleKind::Conjunction, name)
                        ),
                    )),
                    nom::bytes::complete::tag(" -> "),
                    nom::multi::separated_list1(
                        nom::bytes::complete::tag(", "),
                        nom::character::complete::alpha1
                    ),
                )),
                |((kind, name), _, destinations)| Module::new(name, kind, destinations)
            )(input)
        }
    }

    impl Machine {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::multi::many1(
                    nom::sequence::terminated(
                        Module::parse,
                        nom::character::complete::line_ending,
                    )
                ),
                |modules| Machine::new(modules)
            )(input)
        }
    }

}


// ======= Compute =======

#[derive(Debug, Copy, Clone)]
struct Pulse<'a> {
    kind: PulseKind,
    source: &'a str,
    destination: &'a str,
}

impl<'a> Display for Pulse<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} sends {} to {})", self.source, self.kind, self.destination)
    }
}

impl Machine {
    /// The naive implementation of pressing a button. It simulates all the pulses
    /// resulting from a button press. This uses the existing state of everything in
    /// the machine. For different parts of the problem, we want it to DO different
    /// things, so we provide a pulse_func, which is called each time the Machine
    /// sends any Pulse. For instance, for part 1 we will use a function that simply
    /// increments the pulse_count.
    fn button_push<T: FnMut(&Pulse)>(&mut self, mut pulse_func: T) {
        use PulseKind::*;
        use ModuleKind::*;
        let mut pulses: VecDeque<Pulse> = VecDeque::new();
        pulses.push_back(Pulse{kind: Low, source: "button", destination: "broadcaster"});
        while let Some(pulse) = pulses.pop_front() {
            // invoke the pulse_func
            pulse_func(&pulse);

            let module = self.modules.get(pulse.destination);
            let new_pulse_kind: Option<PulseKind> = match module.kind {
                Broadcaster => Some(pulse.kind),
                FlipFlop => match pulse.kind {
                    High => None,
                    Low => {
                        let state: &mut bool = self.flip_flops.get_mut(&module.name);
                        *state = !*state; // flip it
                        match state {
                            true => Some(High),
                            false => Some(Low),
                        }
                    }
                }
                Conjunction => {
                    let conjunction_state: &mut FixedStringMap<PulseKind> = self.conjunctions.get_mut(&module.name);
                    let this_source_state: &mut PulseKind = conjunction_state.get_mut(pulse.source);
                    *this_source_state = pulse.kind;
                    if conjunction_state.values().all(|x| *x == High) { // if all are High
                        Some(Low)
                    } else {
                        Some(High)
                    }
                }
                Output => None,
            };
            if let Some(kind) = new_pulse_kind {
                let source = &module.name;
                for destination in module.destinations.iter() {
                    pulses.push_back(Pulse{kind, source, destination})
                }
            }
        }
    }

}

/// This solves part 1. It sends the given number button-pushes and counts the number of high
/// and low pulses that occur, then multiplies those and returns the answer, the
/// "pulse_count_code".
fn count_pulses(input: &Machine, pushes: usize) -> usize {
    use PulseKind::*;
    // -- create machine --
    let mut machine = input.clone();

    // -- empty pulse count to start with --
    let mut pulse_count: HashMap<PulseKind,usize> = HashMap::with_capacity(2);
    pulse_count.insert(Low, 0);
    pulse_count.insert(High, 0);

    // -- create counter function that increments the pulse count --
    let mut pulse_func = |pulse: &Pulse| {
        *(pulse_count.get_mut(&pulse.kind).unwrap()) += 1;
    };

    // -- send the pushes (using this function) --
    for _ in 0..pushes {
        machine.button_push(&mut pulse_func);
    }

    // -- return the answer --
    pulse_count.get(&High).unwrap() * pulse_count.get(&Low).unwrap()
}



// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let pulse_count_code = count_pulses(input, 1000);
    println!("The pulse count code is {}", pulse_count_code);
}


fn part_b(_input: &Input) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn none() {
    }
}
