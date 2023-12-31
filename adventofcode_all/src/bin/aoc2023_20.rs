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
    /// to count pulses
    pulse_count: HashMap<PulseKind,usize>,
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

        // -- empty pulse count to start with --
        let mut pulse_count = HashMap::with_capacity(2);
        pulse_count.insert(Low, 0);
        pulse_count.insert(High, 0);

        // -- construct it --
        Self{modules, flip_flops, conjunctions, pulse_count}
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
    /// resulting from a button press. This uses the existing state of everything,
    /// and it increments the pulse_count.
    fn button_push(&mut self) {
        use PulseKind::*;
        use ModuleKind::*;
        let mut pulses: VecDeque<Pulse> = VecDeque::new();
        pulses.push_back(Pulse{kind: Low, source: "button", destination: "broadcaster"});
        while let Some(pulse) = pulses.pop_front() {
            // increment the count of pulses sent
            *(self.pulse_count.get_mut(&pulse.kind).unwrap()) += 1;

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

    /// Returns the product of low pulses sent and high pulses sent.
    fn pulse_count_code(&self) -> usize {
        use PulseKind::*;
        self.pulse_count.get(&High).unwrap() * self.pulse_count.get(&Low).unwrap()
    }
}

// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let mut machine = input.clone();
    for _ in 0..1000 {
        machine.button_push();
    }
    let pulse_count_code = machine.pulse_count_code();
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
