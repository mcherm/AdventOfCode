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
use std::collections::{VecDeque, HashMap, BTreeMap, HashSet};
use std::hash::Hash;
use num;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum ModuleKind {
    Broadcaster, FlipFlop, Conjunction, Output
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Module {
    name: String,
    kind: ModuleKind,
    destinations: Vec<String>,
}


/// To protect against bugs, I want to create a variant of HashMap that does not let you
/// alter the keys. It has only the actual calls I happen to be using, and it only allows
/// Strings as keys.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct FixedStringMap<V: Hash>(BTreeMap<String,V>);

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

impl<V: Hash> FixedStringMap<V> {
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
    fn values(&self) -> std::collections::btree_map::Values<'_, String, V> {
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
        let flip_flops_map: BTreeMap<String,bool> = modules.iter()
            .filter_map(|m| if m.kind == FlipFlop {Some((m.name.clone(), false))} else {None})
            .collect();
        let flip_flops = FixedStringMap(flip_flops_map);

        // -- construct Conjunction state --
        let conjunctions_map: BTreeMap<String, FixedStringMap<PulseKind>> = modules.iter()
            .filter(|m| m.kind == Conjunction)
            .map(|m| {
                let source_state_map: BTreeMap<String,PulseKind> = modules.iter()
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
        let mut module_map: BTreeMap<String, Module> = modules.into_iter().map(|m| (m.name.clone(), m)).collect();
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
    /// This injects the given Pulse into the Machine and triggers all subsequent internal
    /// pulses until things settle down. For different parts of the problem, we want it to
    /// DO different things, so we provide a pulse_func, which is called each time the
    /// Machine sends any Pulse to anywhere.
    fn inject_pulse<T: FnMut(&Pulse)>(&mut self, mut pulse_func: T, pulse: Pulse) {
        use PulseKind::*;
        use ModuleKind::*;
        let mut pulses: VecDeque<Pulse> = VecDeque::new();
        pulses.push_back(pulse);
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

    /// The naive implementation of pressing a button. It simulates all the pulses
    /// resulting from a button press. This uses the existing state of everything in
    /// the machine. For different parts of the problem, we want it to DO different
    /// things, so we provide a pulse_func, which is called each time the Machine
    /// sends any Pulse. For instance, for part 1 we will use a function that simply
    /// increments the pulse_count.
    fn button_push<T: FnMut(&Pulse)>(&mut self, pulse_func: T) {
        let pulse = Pulse{kind: PulseKind::Low, source: "button", destination: "broadcaster"};
        self.inject_pulse(pulse_func, pulse);
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


/// This just solves part 2 in a very straightforward manner. It may run way too long.
/// It repetedly presses the button until a pulse of kind_needed is sent to the module
/// named module_to_watch (which must exist or this panics!).
/// It is not at all generalized... for instance, it looks for the specific field "rx"
/// as its output. It returns the number of pushes needed before the component "rx"
/// (which must exist or this panics) receives a low pulse.
fn pushes_until_pulse_received(input: &Machine, module_to_watch: &str, kind_needed: PulseKind) -> usize {
    // -- create machine --
    let mut machine = input.clone();

    // -- here is the state we'll be updating --
    let mut done: bool = false;

    // -- send the pushes (using this function) --
    let mut pushes: usize = 0;
    while !done {
        pushes += 1;
        // -- create counter function that updates <done> --
        let mut pulse_func = |pulse: &Pulse| {
            if pulse.destination == module_to_watch && pulse.kind == kind_needed {
                done = true;
            }
        };
        machine.button_push(&mut pulse_func);
    }

    // -- return the answer --
    pushes
}


/// An Isolate is a very particular structure, which would be quite specialized and unlikely
/// in general problems, but which happens to occur within the particular Machines that are
/// given out for this problem. It consists of a particular subset of the nodes of some
/// Machine, which have the following properties:
///   (1) This subset of nodes may have many connections between members of the set, but
///       it has only one incoming link and one outgoing link.
///   (2) The incoming link always receives a series of "Low" pulses.
#[derive(Debug)]
struct Isolate<'a> {
    /// The original machine which we read from. It will NOT be run, but will get
    /// duplicated and the duplicate executed.
    original_machine: &'a Machine,
    /// The set of nodes in the machine. All must be names of Modules in the source_machine.
    members: HashSet<&'a str>,
    /// The module (in the subset) that receives the series of Low pulses.
    start_module: &'a str,
    /// The name to use for the source of the input pulses
    source: &'a str,
    /// The module (NOT in the subset) that receives the output
    exit_module: &'a str,
}


/// This represents a stream of PulseTypes (abstracting away what it is going to and coming
/// from) which is assumed to repeat.
struct PulseStream {
    /// Just a list of (PulseKind, number-of-repeats) pairs.
    items: Vec<(PulseKind, usize)>,
}

/// The state of a Machine.
#[derive(Hash, Eq, PartialEq, Clone)]
struct MachineState {
    flip_flops: FixedStringMap<bool>,
    conjunctions: FixedStringMap<FixedStringMap<PulseKind>>,
}


/// Can't easily create this as a constant, so here's a function to creat a PulseStream
/// of all Low Pulses.
#[allow(dead_code)]
fn all_low() -> PulseStream {
    PulseStream{items: vec![(PulseKind::Low, 1)]}
}

struct PulseStreamIter<'a> {
    stream: &'a PulseStream,
    item: usize, // next item to read from
    pos: usize, // next pos in that item
}


impl PulseStream {
    /// Length before this repeats. Always at least 1.
    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.items.iter().map(|(_,x)| x).sum()
    }

    /// Iterate through the PulseStream.
    fn iter(&self) -> PulseStreamIter {
        PulseStreamIter{stream: self, item: 0, pos: 0}
    }
}

impl<'a> Iterator for PulseStreamIter<'a> {
    type Item = PulseKind;

    fn next(&mut self) -> Option<Self::Item> {
        let (next, count) = self.stream.items[self.item];
        self.pos += 1;
        if self.pos == count {
            self.pos = 0;
            self.item += 1;
            if self.item == self.stream.items.len() {
                self.item = 0; // because we loop around infinitely
            }
        }
        Some(next)
    }
}


impl MachineState {
    /// Get the state from a machine.
    fn snag_state(machine: &Machine) -> Self {
        MachineState{
            flip_flops: machine.flip_flops.clone(),
            conjunctions: machine.conjunctions.clone(),
        }
    }
}


impl<'a> Isolate<'a> {
    /// Create a new Isolate. All items in members should be names of Modules in source_machine.
    /// start_module should be a name in members and exit_module should be a name NOT in
    /// members that is the name of some other Module in source_machine.
    fn new(
        original_machine: &'a Machine,
        members: HashSet<&'a str>,
        start_module: &'a str,
        source: &'a str,
        exit_module: &'a str
    ) -> Self {
        Isolate{original_machine, members, start_module, source, exit_module}
    }

    /// Finds the output of this Isolate (the hard way... by trying it until it repeats). It
    /// returns a tuple with the number of input pulses needed until it repeats and the
    /// PulseStream that it emits.
    ///
    /// Unfortunately, this did NOT turn out to be useful in solving the problem. So now
    /// it is just dead code.
    #[allow(dead_code)]
    fn find_output(&self, input_stream: PulseStream) -> (usize, PulseStream) {
        // -- make a Machine for just this isolate, which we'll run until it repeats a state --
        let modules = self.members.iter()
            .map(|name| self.original_machine.modules.get(name).clone())
            .collect();
        let mut mini_machine: Machine = Machine::new(modules);

        // -- we'll iterate through the input --
        let mut iter = input_stream.iter();

        // -- keep a list of all the states we've ever been in. and if they were new --
        let mut state_was_new: bool;
        let mut states_seen: HashSet<MachineState> = HashSet::new();
        states_seen.insert(MachineState::snag_state(&mini_machine));

        // -- collect the data we need in order to return a PulseStream --
        let mut output_items: Vec<(PulseKind, usize)> = Vec::new();

        // -- loop through until it repeats --
        let mut input_pulse_count = 0;
        loop {
            // there could be multiple output pulses from this one input... collect them here
            let mut output_pulse_kinds: Vec<PulseKind> = Vec::new();
            let pulse_func = |pulse: &Pulse| {
                if pulse.destination == self.exit_module {
                    output_pulse_kinds.push(pulse.kind);
                }
            };
            let pulse: Pulse = Pulse{kind: iter.next().unwrap(), source: self.source, destination: self.start_module};
            mini_machine.inject_pulse(pulse_func, pulse);

            // -- this one counts, so record the outputs --
            input_pulse_count += 1;
            for pulse_kind in output_pulse_kinds {
                match output_items.pop() {
                    Some( (prev_pulse_kind, prev_pulse_count) ) => {
                        if prev_pulse_kind == pulse_kind {
                            output_items.push( (pulse_kind, prev_pulse_count + 1) );
                        } else {
                            output_items.push( (prev_pulse_kind, prev_pulse_count) );
                            output_items.push( (pulse_kind, 1) );
                        }
                    }
                    None => {
                        output_items.push( (pulse_kind, 1) );
                    }
                }
            }

            // -- see if it's time to exit --
            state_was_new = states_seen.insert(MachineState::snag_state(&mini_machine));
            if !state_was_new {
                break;
            }
        }

        // -- return the output --
        (input_pulse_count, PulseStream{items: output_items})
    }


    /// We expect (because that's what they do in the problems that they give us, not because
    /// this is somehow normal) that when our Isolate receives an infinite stream of Low
    /// pulses, it will emit some (0 or more) High pulses each of the first N-1 times the Low
    /// pulse is received. Then the next (Nth) time a Low pulse is received, it will emit a
    /// single Low pulse followed by (0 or more) High pulses. Then the NEXT time the button is
    /// pressed it will repeat from the beginning.
    ///
    /// What this does is to simulate button presses, verifying that we get exactly that
    /// behavior. If we DO get exactly that behavior, then it returns Ok(N) (the number of
    /// times that will generate the single High pulse). If we DON'T get exactly that
    /// behavior then this returns Err(msg), where msg is a message telling what went wrong.
    fn seek_expected_output(&self) -> Result<usize,String> {
        use PulseKind::*;

        // -- make a Machine for just this isolate, which we'll run until it repeats a state --
        let modules = self.members.iter()
            .map(|name| self.original_machine.modules.get(name).clone())
            .collect();
        let mut mini_machine: Machine = Machine::new(modules);

        // -- keep a list of all the states we've ever been in. and if they were new --
        let mut states_seen: HashSet<MachineState> = HashSet::new();
        states_seen.insert(MachineState::snag_state(&mini_machine));

        let input_pulse = || Pulse{kind: Low, source: self.source, destination: self.start_module};

        // -- loop through until it repeats --
        let mut input_pulse_count = 0;
        loop {
            // there could be multiple output pulses from this one input... collect them here
            let mut output_pulse_kinds: Vec<PulseKind> = Vec::new();
            let pulse_func = |pulse: &Pulse| {
                if pulse.destination == self.exit_module {
                    output_pulse_kinds.push(pulse.kind);
                }
            };
            mini_machine.inject_pulse(pulse_func, input_pulse());
            input_pulse_count += 1;

            // -- so record the outputs --
            if output_pulse_kinds.iter().all(|kind| *kind == High) {
                // -- a normal step, let's make sure we haven't repeated a state --
                let state_was_new = states_seen.insert(MachineState::snag_state(&mini_machine));
                if !state_was_new {
                    return Err("one section entered an infinite loop before emitting a low pulse".to_string());
                }
            } else {
                // -- OK, let's see if it started with a single Low --
                let mut iter = output_pulse_kinds.iter();
                if *iter.next().unwrap() == High { // safe to unwrap: if it were length 0 we'd do the other branch
                    return Err("one section had a button press emit some low pulses, but not the FIRST pulse from that button press".to_string());
                }
                // -- OK, check if the rest in this block are Low --
                if ! iter.all(|x| *x == High) {
                    return Err("one section had a button press emit more than one low pulse".to_string());
                }
                // -- OK, make sure the NEXT step repeats the loop --
                let pulse_func = |_: &Pulse| {};
                mini_machine.inject_pulse(pulse_func, input_pulse());
                let state_was_new = states_seen.insert(MachineState::snag_state(&mini_machine));
                if state_was_new {
                    return Err("one section, after a button press emitted a Low pulse, did not repeat".to_string());
                }
                return Ok(input_pulse_count);
            }
        }
    }
}


/// Based on the input I saw, I expect other machines to be like mine: they'll have
/// a Broadcast module that leads to several "stacks" of FlipFlop modules, which
/// connect to a single Collecter module and all that (except the Broadcast module)
/// forms one Isolate. We then expect each of these to each lead to a single Collecter
/// used as an inverter, then all those join in one Collecter which leads to "rx".
///
/// This function is passed a machine, and attempts to locate that exact pattern. If
/// it succeeds in finding the pattern, it returns a successful Result with the Isolates;
/// if the pattern does not seem to hold then it returns an Err Result with a &str
/// containing a reason why.
fn build_expected_isolates<'a>(machine: &'a Machine) -> Result< Vec<Isolate<'a>>, String > {
    use ModuleKind::*;
    let source = "broadcaster";
    let starts: Vec<&str> = machine.modules.get(source).destinations.iter()
        .map(|s| s.as_str()).collect();
    let mut isolates: Vec<Isolate> = Vec::new();
    for start_module in starts {
        let prefix = || format!("in the section starting from {},", start_module);

        // -- for this start_module, we will find an Isolate (or fail to and bail) --
        let mut modules_to_check: Vec<&str> = vec![start_module];
        let mut the_collector_opt: Option<&str> = None;
        let mut exit_module_opt: Option<&str> = None;
        let mut members: HashSet<&str> = HashSet::new();

        // -- keep looping, adding in everything linked to other than the exit_module --
        while let Some(name) = modules_to_check.pop() {
            if !members.contains(name) { // if we didn't already do this one
                let module = machine.modules.get(name);
                match module.kind {
                    Broadcaster => {
                        return Err(format!("{} we found a broadcaster module", prefix()));
                    }
                    Output => {
                        return Err(format!("{} we found an output module", prefix()));
                    }
                    Conjunction => {
                        match the_collector_opt {
                            None => {
                                the_collector_opt = Some(name);
                                members.insert(name); // the collector is one of our members (with a bunch of FlipFlops)
                                // Things linked to by the_collector could be the exit node,
                                //   so we have to check the type of each one before adding
                                //   it to modules_to_check.
                                for next_module in module.destinations.iter() {
                                    if machine.modules.get(next_module.as_str()).kind == Conjunction {
                                        exit_module_opt = Some(next_module.as_str()); // Found the exit point!
                                    } else {
                                        modules_to_check.push(next_module.as_str()); // Not the exit; so we'll consider it.
                                    }
                                }
                            }
                            Some(collector) => {
                                if collector == name {
                                    // to be expected -- we found the collector again. Nothing to do.
                                } else {
                                    return Err(format!("{} we found multiple conjunction modules", prefix()));
                                }
                            }
                        }
                    }
                    FlipFlop => {
                        members.insert(name); // this is one of the FlipFlops
                        for next_module in module.destinations.iter() { // consider everything it connects to
                            modules_to_check.push(next_module.as_str());
                        }
                    }
                }
            }
        }

        // -- check that it's valid --
        if the_collector_opt.is_none() {
            return Err(format!("{} we didn't find a collector", prefix()));
        }
        if exit_module_opt.is_none() {
            return Err(format!("{} the collector didn't have a single exit leading to another collector", prefix()));
        }
        let exit_module = exit_module_opt.unwrap();

        // -- create the Isolate --
        let isolate = Isolate::new(machine, members, start_module, source, exit_module);
        isolates.push(isolate);
    }

    // -- make sure we had at least 1 isolate --
    if isolates.is_empty() {
        return Err("we didn't find even one isolated section with a single collector as an exit".to_string());
    }

    // -- now check on what comes OUT of the isolates --
    let mut final_point_opt: Option<&str> = None;
    for isolate in isolates.iter() {
        let inverter_destinations = &machine.modules.get(isolate.exit_module).destinations;
        if inverter_destinations.len() != 1 {
            return Err("an inverter after an isolate didn't go exactly 1 place".to_string());
        }
        match final_point_opt {
            None => {
                final_point_opt = Some(&inverter_destinations[0]);
            }
            Some(final_point) => {
                if final_point != &inverter_destinations[0] {
                    return Err("different inverters (after isolates) went to different places".to_string());
                }
            }
        }
    }
    let final_point = final_point_opt.unwrap(); // we checked that there was at least 1 so it's safe
    let final_module = machine.modules.get(final_point);
    if final_module.kind != Conjunction {
        return Err("the final module the inverters lead to isn't a conjunction".to_string());
    }
    if final_module.destinations != ["rx"] {
        return Err("the final module the inverters lead to doesn't connect to exactly one thing named \"rx\"".to_string());
    }

    // -- It worked! We can return the Isolates. --
    Ok(isolates)
}


/// Finds the least common multiple of a list of numbers. Can return None if the list is empty.
fn lcm(numbers: &Vec<usize>) -> Option<usize> {
    numbers.into_iter().cloned().reduce(|acc: usize, x: usize| num::integer::lcm(acc,x))
}


/// This attempts to solve (part 2 of) the problem quickly by knowing FAR too much about exactly
/// how it is laid out. It either succeeds, and returns the number of pushes required to output
/// the first Low pulse, or it fails and returns a string describing why it couldn't solve it.
fn maybe_solve_fast(machine: &Machine) -> Result<usize,String> {
    let isolates = build_expected_isolates(machine)?;

    let numbers = isolates.iter()
        .map(|i| i.seek_expected_output())
        .collect::<Result<Vec<usize>,String>>()?;
    println!(
        "Aha! This machine appears to be finding the least common multiple of the numbers {}.",
        itertools::join(numbers.clone(), ", ")
    );
    assert!(numbers.len() > 0);
    Ok(lcm(&numbers).unwrap())
}


/// This attempts to solve (part 2 of) the problem quickly and reverts to solving it the slow
/// way if that doesn't work.
fn solve_fast_or_slow(machine: &Machine) -> usize {
    match maybe_solve_fast(machine) {
        Ok(answer) => answer,
        Err(err_msg) => {
            println!("We did NOT succeed in solving it using our knowledge of the layout. The problem was {}", err_msg);
            println!("So instead, we will solve it the slow way.");
            println!();
            pushes_until_pulse_received(machine, "rx", PulseKind::Low)
        }
    }
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let pulse_count_code = count_pulses(input, 1000);
    println!("The pulse count code is {}", pulse_count_code);
}


fn part_b(input: &Input) {
    println!("\nPart b:");
    let pushes_needed = solve_fast_or_slow(input);
    println!("It took {} pushes before rs first received a Low pulse.", pushes_needed);
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
    fn isolate() {
        use ModuleKind::*;
        use PulseKind::*;
        let module_data: Vec<(&str,ModuleKind,Vec<&str>)> = vec![
            ("mg", FlipFlop, vec!["fj", "dt"]),
            ("fj", FlipFlop, vec!["tr", "dt"]),
            ("tr", FlipFlop, vec!["dt"]),
            ("dt", Conjunction, vec!["mg", "cl"]),
        ];
        let modules = module_data.iter().map(|(a,b,c)| Module::new(*a,*b,c.clone())).collect();
        let machine = Machine::new(modules);
        let members: HashSet<&str> = module_data.iter().map(|x| x.0).collect();
        let start_module = "mg";
        let source = "broadcaster";
        let exit_module = "cl";
        let isolate = Isolate::new(&machine, members, start_module, source, exit_module);

        let (input_pulse_count, output_pulse_stream) = isolate.find_output(all_low());

        assert_eq!(input_pulse_count, 7);
        assert_eq!(output_pulse_stream.len(), 14);
        assert_eq!(
            output_pulse_stream.items,
            vec![(High,10), (Low,1), (High,3)]
        );
    }
}
