
extern crate anyhow;

use std::fs;
use itertools::Itertools;
use nom;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::map,
    sequence::tuple,
};
use nom::character::complete::u32 as nom_u32;

// ======= Switches =======

const PRINT_WORK: bool = false;

// ======= Parsing =======

fn input() -> Result<MonkeyTroop, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_11.txt")?;
    match MonkeyTroop::parse(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}

type WorryLevel = u32;

#[derive(Debug, Copy, Clone)]
struct Item  {
    worry_level: WorryLevel,
}

#[derive(Debug, Copy, Clone)]
enum Operation {
    Mult(WorryLevel),
    Add(WorryLevel),
    Square,
}

#[derive(Debug, Copy, Clone)]
struct ThrowingRule {
    divide_by: WorryLevel,
    true_dest: usize,
    false_dest: usize,
}

#[derive(Debug, Clone)]
struct Monkey {
    monkey_num: usize,
    items: Vec<Item>,
    operation: Operation,
    throwing_rule: ThrowingRule,
    actions: usize,
}

#[derive(Debug, Clone)]
struct MonkeyTroop {
    monkeys: Vec<Monkey>
}


impl Item {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            nom_u32,
            |worry_level| Item{worry_level}
        )(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        nom::multi::separated_list0(tag(", "), Self::parse)(input)
    }
}

impl Operation {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        nom::sequence::preceded(
            tag("new = "),
            alt((
                map(
                    tuple((
                        tag("old"),
                        tag(" * "),
                        nom_u32,
                    )),
                    |(_, _, val)| Operation::Mult(val)
                ),
                map(
                    tuple((
                        tag("old"),
                        tag(" + "),
                        nom_u32,
                    )),
                    |(_, _, val)| Operation::Add(val)
                ),
                map(
                    tuple((
                        tag("old"),
                        tag(" * "),
                        tag("old"),
                    )),
                    |(_, _, _)| Operation::Square
                ),
            ))
        )(input)
    }

    /// Performs the operation on an item, and prints if appropriate
    fn perform(&self, item: &mut Item) {
        match self {
            Operation::Mult(val) => {
                item.worry_level *= val;
                if PRINT_WORK {println!("    Worry level is multiplied by {} to {}.", val, item.worry_level);}
            }
            Operation::Add(val) => {
                item.worry_level += val;
                if PRINT_WORK {println!("    Worry level increases by {} to {}.", val, item.worry_level);}
            }
            Operation::Square => {
                item.worry_level *= item.worry_level;
                if PRINT_WORK {println!("    Worry level is multiplied by itself to {}.", item.worry_level);}
            }
        }
    }
}

impl ThrowingRule {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("  Test: divisible by "),
                nom_u32,
                line_ending,
                tag("    If true: throw to monkey "),
                nom_u32,
                line_ending,
                tag("    If false: throw to monkey "),
                nom_u32,
                line_ending,
            )),
            |(_, divide_by, _, _, true_dest, _, _, false_dest, _)| ThrowingRule{
                divide_by,
                true_dest: true_dest as usize,
                false_dest: false_dest as usize,
            }
        )(input)
    }

    /// Is passed an item, and decides what to do with it, returning a ThrownItem. May also
    /// print stuff if appropriate.
    fn perform(&self, item: Item) -> ThrownItem {
        let remainder = item.worry_level % self.divide_by;
        let target = match remainder {
            0 => {
                if PRINT_WORK {println!("    Current worry level is divisible by {}.", self.divide_by);}
                if PRINT_WORK {println!("    Item with worry level {} is thrown to monkey {}.", item.worry_level, self.true_dest);}
                self.true_dest
            }
            _ => {
                if PRINT_WORK {println!("    Current worry level is not divisible by {}.", self.divide_by);}
                if PRINT_WORK {println!("    Item with worry level {} is thrown to monkey {}.", item.worry_level, self.false_dest);}
                self.false_dest
            }
        };
        ThrownItem::new(item, target)
    }
}

impl Monkey {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("Monkey "),
                nom_u32,
                tag(":"),
                line_ending,

                tag("  Starting items: "),
                Item::parse_list,
                line_ending,

                tag("  Operation: "),
                Operation::parse,
                line_ending,

                ThrowingRule::parse,
            )),
            |(
                _, monkey_num, _, _,    // monkey line
                _, items, _,            // items line
                _, operation, _,        // operation line
                throwing_rule,          // throwing rule lines
             )| Monkey{
                monkey_num: monkey_num as usize,
                items,
                operation,
                throwing_rule,
                actions: 0,
            },
        )(input)
    }

    /// When this is called, the Monkey examines its items, adjusts the worry
    /// level of each, then removes the thrown items from its own list and
    /// returns them to the caller (which can place them in the right location).
    fn perform(&mut self) -> Vec<ThrownItem> {
        let mut answer = Vec::new();
        if PRINT_WORK {println!("Monkey {}:", self.monkey_num);}
        for mut item in self.items.iter_mut() {
            self.actions += 1;
            if PRINT_WORK {println!("  Monkey inspects an item with a worry level of {}.", item.worry_level);}
            self.operation.perform(&mut item);
            item.worry_level /= 3;
            if PRINT_WORK {println!("    Monkey gets bored with item. Worry level is divided by 3 to {}.", item.worry_level);}
            answer.push( self.throwing_rule.perform(*item) );
        }
        self.items.clear(); // delete them all
        answer
    }

    /// Gives an item to a monkey.
    fn give_item(&mut self, item: Item) {
        self.items.push(item);
    }
}

impl MonkeyTroop {
    // NOTE: I would rather have this return an error. But I can't figure out
    //   how do do that within the parser. So it will panic instead.
    fn new(monkeys: Vec<Monkey>) -> Self {
        let count = monkeys.len();
        for (i, monkey) in monkeys.iter().enumerate() {
            if monkey.monkey_num != i {
                panic!("Monkey {} has number {}.", i, monkey.monkey_num);
            }
            if monkey.throwing_rule.true_dest >= count || monkey.throwing_rule.false_dest >= count {
                panic!("Monkey {} can throw things out of bounds.", monkey.monkey_num);
            }
        }
        Self{monkeys}
    }

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            nom::multi::separated_list0(line_ending, Monkey::parse),
            |monkeys| MonkeyTroop::new(monkeys)
        )(input)
    }

    /// Performs one round of the monkeys.
    fn perform_round(&mut self) {
        for monkey_num in 0..self.monkeys.len() {
            let monkey = self.monkeys.get_mut(monkey_num).unwrap();
            let thrown_items = monkey.perform();
            for thrown_item in thrown_items {
                self.monkeys.get_mut(thrown_item.target).unwrap().give_item(thrown_item.item);
            }
        }
    }

    /// Prints a description to stdout
    fn show_holdings(&self) {
        for monkey in self.monkeys.iter() {
            println!("Monkey {}: ({} actions) {:?}", monkey.monkey_num, monkey.actions, monkey.items.iter().map(|x| x.worry_level).collect_vec());
        }
    }

    /// Calculates the monkey business.
    fn monkey_business(&self) -> usize {
        self.monkeys.iter()
            .map(|x| x.actions)
            .sorted()
            .rev()
            .take(2)
            .product()
    }
}


// ======= Calculations =======

#[derive(Debug)]
struct ThrownItem {
    item: Item,
    target: usize,
}

impl ThrownItem {
    fn new(item: Item, target: usize) -> Self {
        ThrownItem{item, target}
    }
}

// ======= main() =======

fn part_a(monkey_troop: &MonkeyTroop) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    let mut monkey_troop = monkey_troop.clone();
    monkey_troop.show_holdings();
    for round in 0..20 {
        monkey_troop.perform_round();
        println!();
        println!("After round {}", round + 1);
        monkey_troop.show_holdings();
    }
    println!();
    println!("The monkey business is {}", monkey_troop.monkey_business());
    Ok(())
}


fn part_b(_monkey_troop: &MonkeyTroop) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
