
extern crate anyhow;

/// This module provides a SortedVec which is a wrapper around Vec that guarantees
/// the Vec will be kept in sorted order. The wrapper only bothers to implement
/// the specific features of Vec that we're using in this file. We have this
/// because we want a Vec of items for which order is not significant for equality
/// testing or hashing.
mod sorted_vec {
    use core::hash::Hash;
    use std::ops::{Deref, DerefMut};

    #[derive(Debug, Hash, Eq, PartialEq, Clone)]
    pub struct SortedVec<T>(Vec<T>)
        where T: Hash + Ord;

    impl<T: Hash + Ord> SortedVec<T> {
        pub fn new() -> Self {
            SortedVec(Vec::new())
        }

        pub fn push(&mut self, value: T) {
            let insert_pos = self.0.binary_search(&value).unwrap_or_else(|e| e);
            self.0.insert(insert_pos, value);
        }

        pub fn retain<F>(&mut self, f: F) where F: FnMut(&T) -> bool {
            self.0.retain(f);
        }
    }

    impl<T: Hash + Ord> Deref for SortedVec<T> {
        type Target = [T];

        fn deref(&self) -> &Self::Target {
            self.0.deref()
        }

    }

    impl<T: Hash + Ord> DerefMut for SortedVec<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0.deref_mut()
        }
    }

    impl<T: Hash + Ord> IntoIterator for SortedVec<T> {
        type Item = T;
        type IntoIter = std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}


use sorted_vec::SortedVec;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use anyhow::Error;


use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, newline},
    combinator::{value, map, opt},
    multi::{many0, separated_list1},
    sequence::{terminated, tuple},
};


const PRINT_WORK: bool = false;


fn input() -> Result<Vec<FloorDescription>, Error> {
    let s = fs::read_to_string("input/2016/input_11.txt")?;
    match FloorDescription::parse_list(&s) {
        Ok(("", floor_descriptions)) => Ok(floor_descriptions),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ItemType {
    Generator, Microchip
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Item {
    name: String,
    item_type: ItemType,
}



impl Item {
    fn parse_generator<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("a "),
                alpha1,
                tag(" generator")
            )),
            |(_, name, _): (&str, &str, &str)| Item{name: name.to_string(), item_type: ItemType::Generator}
        )(input)
    }

    fn parse_microchip<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("a "),
                alpha1,
                tag("-compatible microchip")
            )),
            |(_, name, _): (&str, &str, &str)| Item{name: name.to_string(), item_type: ItemType::Microchip}
        )(input)
    }

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            Self::parse_generator,
            Self::parse_microchip,
        ))(input)
    }

    fn parse_list_0<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        value(Vec::new(), tag("nothing relevant"))(input)
    }

    fn parse_list_1<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        map(
            Self::parse,
            |x| vec![x]
        )(input)
    }

    fn parse_list_2plus<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        map(
            tuple((
                separated_list1( tag(", "), Self::parse ),
                opt(tag(",")),
                tag(" and "),
                Self::parse,
            )),
            |(mut items, _, _, last_item)| {
                items.push(last_item);
                items
            }
        )(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        alt((
            Self::parse_list_2plus,
            Self::parse_list_1,
            Self::parse_list_0,
        ))(input)
    }

}


fn ordinal_to_num(s: &str) -> u8 {
    match s {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "sixth" => 6,
        "seventh" => 7,
        "eighth" => 8,
        "nineth" => 9,
        "tenth" => 10,
        _ => panic!("Ordinal not supported"),
    }
}



#[derive(Debug)]
pub struct FloorDescription {
    floor_num: FloorNum,
    items: Vec<Item>
}


impl FloorDescription {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("The "),
                alpha1,
                tag(" floor contains "),
                Item::parse_list,
                tag("."),
            )),
            |(_, floor_name, _, items, _)| FloorDescription{
                floor_num: ordinal_to_num(floor_name),
                items
            }
        )(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }
}


type FloorNum = u8;

/// States that are equivalent if you replace one name with another one are identical for
/// purposes of the problem and we don't need to explore all of them. So our list of
/// visited states, instead of storing a State, will store this object which represents
/// a state without respect for which name is which. This depends on (and verifies) an
/// assumption that each chip has a corresponding generator, and vice versa.
#[derive(Hash, Eq, PartialEq, Clone)]
struct StateIgnoringType {
    elevator: FloorNum,
    pairs: SortedVec<(FloorNum, FloorNum)>,
}


/// This represents a single state that the system can be in.
#[derive(Hash, Eq, PartialEq, Clone)]
struct State {
    elevator: FloorNum,
    data: Vec<SortedVec<Item>>,
}

impl State {
    /// Initialize a State from the descriptions
    fn from_descriptions(floor_descriptions: &Vec<FloorDescription>) -> Self {
        let mut data = Vec::new();
        let num_floors = FloorNum::try_from(floor_descriptions.len()).unwrap();
        for floor in 0..num_floors {
            for floor_description in floor_descriptions {
                if FloorNum::from(floor_description.floor_num) == floor + 1 {
                    let mut floor_items = SortedVec::new();
                    for item in &floor_description.items {
                        floor_items.push(item.clone())
                    }
                    data.push(floor_items);
                    break;
                }
            }
        }
        let elevator = 0;
        State{elevator, data}
    }

    /// Tests whether a state is legal.
    fn is_legal(&self) -> bool {
        for floor_items in &self.data {
            for item in floor_items.iter() {
                if matches!(item.item_type, ItemType::Microchip)  {
                    let mut microchip_plugged_in = false;
                    let mut other_generator_present = false;
                    for other_item in floor_items.iter() {
                        if matches!(other_item.item_type, ItemType::Generator) {
                            if item.name == other_item.name {
                                microchip_plugged_in = true;
                            } else {
                                other_generator_present = true;
                            }
                        }
                    }
                    if other_generator_present && !microchip_plugged_in {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Returns true if the object has all items on the top floor (so it wins); false
    /// if not.
    fn winning(&self) -> bool {
        for (floor_num, floor_items) in self.data.iter().enumerate() {
            if floor_num < self.data.len() - 1 {
                if floor_items.len() > 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Returns a new State identical to this, but the given item is removed
    /// from the floor with the current elevator and put on the floor
    /// new_elevator, and the elevator is moved there also.
    fn move_item(&self, new_elevator: FloorNum, moved_items: Vec<&Item>) -> State {
        let mut new_data = Vec::new();
        for (i, floor_items) in self.data.iter().enumerate() {
            let floor_num = FloorNum::try_from(i).unwrap();
            new_data.push(match floor_num {
                _ if floor_num == self.elevator => {
                    // leave off items
                    let mut new_items = floor_items.clone();
                    new_items.retain(|i| !moved_items.contains(&i));
                    new_items
                },
                _ if floor_num == new_elevator => {
                    // add item
                    let mut new_items = floor_items.clone();
                    for moved_item in moved_items.iter() {
                        new_items.push((*moved_item).clone())
                    }
                    new_items
                },
                _ => {
                    // exactly as-is
                    floor_items.clone()
                },
            });
        }
        State{elevator: new_elevator, data: new_data}
    }

    /// Returns a list of all possible next states (legal or not).
    fn possible_next_states(&self) -> Vec<State> {
        let mut answer = Vec::new();
        let floors = FloorNum::try_from(self.data.len()).unwrap();
        let elevator_floor_items: &SortedVec<Item> = self.data.get(usize::from(self.elevator)).unwrap();

        let mut possible_new_floors = Vec::new();
        if self.elevator < floors - 1 {
            // --- We can try things that move the elevator up ---
            possible_new_floors.push(self.elevator + 1);
        }
        if self.elevator > 0 {
            // --- We can try things that move the elevator down ---
            possible_new_floors.push(self.elevator - 1);
        }

        for new_floor in possible_new_floors {
            // -- Try two things in the elevator --
            if elevator_floor_items.len() >= 2 {
                for p1 in 0..(elevator_floor_items.len() - 1) {
                    for p2 in (p1 + 1)..elevator_floor_items.len() {
                        let new_floor_items = self.move_item(new_floor, vec![
                            elevator_floor_items.get(p1).unwrap(),
                            elevator_floor_items.get(p2).unwrap(),
                        ]);
                        answer.push(new_floor_items);
                    }
                }
            }
            // -- Try just one thing in the elevator --
            for item in elevator_floor_items.iter() {
                let new_floor_items = self.move_item(new_floor, vec![&item]);
                answer.push(new_floor_items);
            }
        }
        answer
    }

    fn to_state_ignoring_type(&self) -> StateIgnoringType {
        let mut pairs = SortedVec::new();
        let mut found_item_floors: HashMap<String, FloorNum> = HashMap::new();
        for (i, floor_data) in self.data.iter().enumerate() {
            let floor_num = FloorNum::try_from(i).unwrap();
            for item in floor_data.iter() {
                match found_item_floors.remove(&item.name) {
                    None => {found_item_floors.insert(item.name.clone(), floor_num);},
                    Some(other_floor_num) => {pairs.push(
                        match item.item_type {
                            ItemType::Generator => (other_floor_num, floor_num),
                            ItemType::Microchip => (floor_num, other_floor_num),
                        }
                    );},
                }
            }
        }
        assert!(found_item_floors.len() == 0);
        StateIgnoringType{elevator: self.elevator, pairs}
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, floor_items) in self.data.iter().enumerate().rev() {
            let floor_num = FloorNum::try_from(i).unwrap();
            write!(f, "F{} {}", floor_num + 1, if self.elevator == floor_num {'E'} else {' '})?;
            for item in floor_items.iter() {
                write!(
                    f,
                    " {}{}",
                    item.name.chars().nth(0).unwrap_or(' '),
                    match item.item_type { ItemType::Generator => 'G', ItemType::Microchip => 'M'}
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}


fn explore_states(initial_state: State) {
    if initial_state.winning() {
        println!("That starting state is already a winner.");
        return;
    }

    let mut visited_states: HashSet<StateIgnoringType> = HashSet::new();
    visited_states.insert(initial_state.to_state_ignoring_type());
    let mut available_states: VecDeque<(usize, State)> = VecDeque::new(); // (num_steps, state)
    available_states.push_back((0,initial_state));
    let mut number_of_steps = 0;

    loop {
        // -- grab the state that's on the front of the queue --
        let (steps, from_state) = available_states.pop_front().expect("There is no way to find a solution.");

        // -- print out info about how our breadth-first search is going --
        match steps - number_of_steps {
            0 => {}, // same number of steps as last time
            1 => {
                println!("Now searching solutions that require {} steps.", steps);
                number_of_steps = steps;
            },
            _ => panic!("Apparently we are not searching a breadth-first search."),
        }

        // -- loop through possible next steps --
        for s in from_state.possible_next_states() {
            if !visited_states.contains(&s.to_state_ignoring_type()) && s.is_legal() {
                if s.winning() {
                    println!("**** FOUND A WINNER ****");
                    println!("In {} steps:", steps + 1);
                    println!("{}", s);
                    return;
                }
                if PRINT_WORK {
                    println!("Going {} steps (we've tried {} legal states):", steps + 1, visited_states.len() + 1);
                    println!("{}", s);
                }
                visited_states.insert(s.to_state_ignoring_type());
                available_states.push_back((steps + 1, s));
            }
        }
    }
}


fn part_a(floor_descriptions: &Vec<FloorDescription>) {
    println!("\nPart a:");
    let initial_state = State::from_descriptions(floor_descriptions);
    if PRINT_WORK {
        println!("Initial State:");
        println!("{}", initial_state);
    }
    explore_states(initial_state);
}


fn part_b(floor_descriptions: &Vec<FloorDescription>) {
    println!("\nPart b:");
    let mut initial_state = State::from_descriptions(floor_descriptions);
    initial_state.data[0].push(Item{name: "elerium".to_string(), item_type: ItemType::Generator});
    initial_state.data[0].push(Item{name: "elerium".to_string(), item_type: ItemType::Microchip});
    initial_state.data[0].push(Item{name: "dilithium".to_string(), item_type: ItemType::Generator});
    initial_state.data[0].push(Item{name: "dilithium".to_string(), item_type: ItemType::Microchip});
    if PRINT_WORK {
        println!("Initial State:");
        println!("{}", initial_state);
    }
    explore_states(initial_state);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
