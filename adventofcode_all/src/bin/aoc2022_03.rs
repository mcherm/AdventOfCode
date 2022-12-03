
extern crate anyhow;

use std::fs;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::collections::HashSet;
use nom::{
    IResult,
    character::complete::{one_of, newline},
    combinator::map,
    multi::many0,
    sequence::{terminated, pair},
};



fn input() -> Result<Vec<Rucksack>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_03.txt")?;
    match Rucksack::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}

/// The allowed letters, in order by value.
const ALLOWED_LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct ElfItem {
    c: char,
}

#[derive(Debug)]
struct Compartment {
    items: Vec<ElfItem>,
}

#[derive(Debug)]
struct Rucksack {
    compartments: [Compartment; 2],
}

struct ElfGroup<'a> {
    rucksacks: [&'a Rucksack; 3],
}


impl ElfItem {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            one_of(ALLOWED_LETTERS),
            |c| Self{c}
        )(input)
    }

    /// Returns the priority of the item.
    fn priority(&self) -> u32 {
        u32::try_from(ALLOWED_LETTERS.find(self.c).unwrap()).unwrap() + 1
    }
}

impl Compartment {
    fn new<'a>(items: impl IntoIterator<Item=&'a ElfItem>) -> Self {
        Compartment{items: items.into_iter().map(|x| *x).collect()}
    }

    /// Returns the set of items in this compartment.
    fn item_set(&self) -> HashSet<ElfItem> {
        HashSet::from_iter(self.items.iter().map(|x| *x))
    }
}

impl Rucksack {
    /// Given a vector of Items, this splits them into two compartments and returns a Rucksack.
    /// The vector MUST have a length that is even, or this panics.
    fn new(all_items: Vec<ElfItem>) -> Self {
        assert!(all_items.len() % 2 == 0);
        let mid = all_items.len() / 2;
        let compartments = [
            Compartment::new(&all_items[..mid]),
            Compartment::new(&all_items[mid..]),
        ];
        Rucksack{compartments}
    }

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            many0(pair(ElfItem::parse, ElfItem::parse)), // collect pairs, so we know it's an even number of items
            |item_pairs| Rucksack::new(item_pairs.iter().flat_map(|(a,b)| [*a,*b]).collect())
        )(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }

    /// Returns a HashSet of the ElfItems which are common across both compartments of the Rucksack.
    fn shared_items(&self) -> HashSet<ElfItem> {
        let set_0 = self.compartments[0].item_set();
        let set_1 = self.compartments[1].item_set();
        return set_0.intersection(&set_1).map(|x| *x).collect()
    }

    /// Returns the set of items in this rucksack.
    fn item_set(&self) -> HashSet<ElfItem> {
        let set_0 = self.compartments[0].item_set();
        let set_1 = self.compartments[1].item_set();
        return set_0.union(&set_1).map(|x| *x).collect()
    }
}

impl<'a> ElfGroup<'a> {
    /// Given the input list of rucksacks, returns the ElfGroups. Will panic if the rucksacks
    /// cannot be divided evenly into ElfGroups.
    fn form_groups(input: &'a Vec<Rucksack>) -> Vec<ElfGroup<'a>> {
        assert!(input.len() % 3 == 0);
        input.chunks_exact(3).map(|x| {
            let rucksacks: [&'a Rucksack; 3] = [&x[0], &x[1], &x[2]];
            ElfGroup{rucksacks}
        }).collect()
    }

    /// Finds the items common to an ElfGroup.
    fn shared_items(&self) -> HashSet<ElfItem> {
        let set_0 = self.rucksacks[0].item_set();
        let set_1 = self.rucksacks[1].item_set();
        let set_2 = self.rucksacks[2].item_set();
        let set_01: HashSet<ElfItem> = set_0.intersection(&set_1).map(|x| *x).collect();
        set_01.intersection(&set_2).map(|x| *x).collect()
    }
}

impl Display for ElfItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.c)
    }
}

impl Display for Compartment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl Display for Rucksack {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Rucksack{{{}, {}}}", self.compartments[0], self.compartments[1])
    }
}



fn part_a(input: &Vec<Rucksack>) {
    println!("\nPart a:");
    let mut priority_sum = 0;
    for rucksack in input.iter() {
        let shared_items = rucksack.shared_items();
        assert!(shared_items.len() == 1);
        let shared_item = shared_items.iter().next().unwrap();
        let priority = shared_item.priority();
        priority_sum += priority;
        println!("{} ({}) shared in {}", shared_item, priority, rucksack);
    }
    println!("The total priority is {}", priority_sum);
}


fn part_b(input: &Vec<Rucksack>) {
    println!("\nPart b:");
    let elf_groups = ElfGroup::form_groups(input);
    let mut priority_sum = 0;
    for elf_group in elf_groups.iter() {
        let shared_items = elf_group.shared_items();
        assert!(shared_items.len() == 1);
        let shared_item = shared_items.iter().next().unwrap();
        let priority = shared_item.priority();
        priority_sum += priority;
        println!("{} ({}) shared in an elf group", shared_item, priority);
    }
    println!("The total priority is {}", priority_sum);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
