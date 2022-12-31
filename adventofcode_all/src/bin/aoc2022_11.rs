
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
    sequence::{preceded, tuple},
};
use nom::character::complete::u32 as nom_u32;
use std::collections::{BTreeMap, BTreeSet};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};


// ======= Switches =======

const PRINT_WORK: bool = false;

// ======= Parsing =======

fn input() -> Result<MonkeyTroopTemplate, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_11.txt")?;
    match MonkeyTroopTemplate::parse(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}

type WorryLevel = u32;

const REDUCE_FACTOR: WorryLevel = 3;

#[derive(Debug)]
enum Item {
    SimpleValue {
        worry_level: WorryLevel
    },
    ModularValue {
        initial_worry_level: WorryLevel,
        remainders: BTreeMap<WorryLevel, WorryLevel>,
    }
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

#[derive(Debug)]
struct MonkeyTemplate {
    monkey_num: usize,
    items: Vec<WorryLevel>,
    operation: Operation,
    throwing_rule: ThrowingRule,
}

#[derive(Debug)]
struct Monkey {
    monkey_num: usize,
    items: VecDeque<Item>, // use a VecDeque because we'll need to pop items off the front
    operation: Operation,
    throwing_rule: ThrowingRule,
    actions: usize,
}

#[derive(Debug)]
struct MonkeyTroopTemplate {
    monkeys: Vec<MonkeyTemplate>
}

#[derive(Debug)]
struct MonkeyTroop {
    reduce_worry: bool, // if true, we use Item::SimpleValue, otherwise we use Item::ModularValue
    monkeys: Vec<Monkey>
}


impl Item {
    /// Construct a SimpleValue. It can't grow bigger than WorryLevel::MAX, but it supports
    /// dividing by arbitrary numbers.
    fn new_simple_value(initial_worry_level: WorryLevel) -> Self {
        Self::SimpleValue{worry_level: initial_worry_level}
    }

    /// Construct a ModularValue. It can grow arbitrarily large, but it doesn't support being
    /// divided by a number. To specify it, you must provide the set of all values you will
    /// ever want to mod it by.
    fn new_modular_value(initial_worry_level: WorryLevel, divisors: &BTreeSet<WorryLevel>) -> Self {
        let mut remainders = BTreeMap::new();
        for divisor in divisors.into_iter() {
            remainders.insert(*divisor, initial_worry_level % *divisor);
        }
        Self::ModularValue{initial_worry_level, remainders}
    }

    /// Squares the current worry level.
    fn square(&mut self) {
        match self {
            Item::SimpleValue{ref mut worry_level} => {
                *worry_level *= *worry_level;
            }
            Item::ModularValue{remainders, ..} => {
                for (divide_by, remainder) in remainders.iter_mut() {
                    *remainder = *remainder * *remainder;
                    *remainder %= divide_by;
                }
            }
        }
    }

    /// Returns the remainder when divided by val. If this is a ModularValue, then val must be one
    /// of the divide_by values the Item was set up with, and it will panic if not.
    fn get_remainder(&self, val: WorryLevel) -> WorryLevel {
        match self {
            Item::SimpleValue{worry_level} => {
                worry_level % val
            }
            Item::ModularValue{remainders, ..} => {
                assert!(remainders.contains_key(&val));
                *remainders.get(&val).unwrap()
            }
        }
    }
}

impl std::ops::MulAssign<WorryLevel> for Item {
    fn mul_assign(&mut self, val: WorryLevel) {
        match self {
            Item::SimpleValue{ref mut worry_level} => {
                *worry_level *= val;
            }
            Item::ModularValue{remainders, ..} => {
                for (divide_by, remainder) in remainders.iter_mut() {
                    *remainder *= val;
                    *remainder %= divide_by;
                }
            }
        }
    }
}

impl std::ops::AddAssign<WorryLevel> for Item {
    fn add_assign(&mut self, val: WorryLevel) {
        match self {
            Item::SimpleValue{ref mut worry_level} => {
                *worry_level += val;
            }
            Item::ModularValue{remainders, ..} => {
                for (divide_by, remainder) in remainders.iter_mut() {
                    *remainder += val;
                    *remainder %= divide_by;
                }
            }
        }
    }
}

impl std::ops::DivAssign<WorryLevel> for Item {
    fn div_assign(&mut self, rhs: WorryLevel) {
        match self {
            Item::SimpleValue{ref mut worry_level} => {
                *worry_level /= rhs;
            }
            Item::ModularValue{..} => {
                panic!("Item::ModularValue does not support division!");
            }
        }
    }
}



impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::SimpleValue{worry_level} => write!(f, "{}", worry_level),
            Item::ModularValue{initial_worry_level, remainders} => write!(f, "[{}>> {}]",
                initial_worry_level,
                remainders.iter().map(|(d,r)| format!("{}:{}", d, r)).join(", ")
            )
        }
    }
}

impl Operation {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        preceded(
            tag("new = "),
            alt((
                map(
                    preceded(tag("old * "), nom_u32),
                    |x| Operation::Mult(x)
                ),
                map(
                    preceded(tag("old + "), nom_u32),
                    |x| Operation::Add(x)
                ),
                map(
                    tag("old * old"),
                    |_| Operation::Square
                ),
            ))
        )(input)
    }

    /// Performs the operation on an item, and prints if appropriate
    fn perform(&self, item: &mut Item) {
        match self {
            Operation::Mult(val) => {
                (*item) *= *val;
                if PRINT_WORK {println!("    Worry level is multiplied by {} to {}.", val, item);}
            }
            Operation::Add(val) => {
                (*item) += *val;
                if PRINT_WORK {println!("    Worry level increases by {} to {}.", val, item);}
            }
            Operation::Square => {
                (*item).square();
                if PRINT_WORK {println!("    Worry level is multiplied by itself to {}.", item);}
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
        let remainder = item.get_remainder(self.divide_by);
        let target = match remainder {
            0 => {
                if PRINT_WORK {println!("    Current worry level is divisible by {}.", self.divide_by);}
                if PRINT_WORK {println!("    Item with worry level {} is thrown to monkey {}.", item, self.true_dest);}
                self.true_dest
            }
            _ => {
                if PRINT_WORK {println!("    Current worry level is not divisible by {}.", self.divide_by);}
                if PRINT_WORK {println!("    Item with worry level {} is thrown to monkey {}.", item, self.false_dest);}
                self.false_dest
            }
        };
        ThrownItem::new(item, target)
    }
}



impl MonkeyTemplate {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("Monkey "),
                nom_u32,
                tag(":"),
                line_ending,

                tag("  Starting items: "),
                nom::multi::separated_list0(tag(", "), nom_u32),
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
             )| Self{
                monkey_num: monkey_num as usize,
                items,
                operation,
                throwing_rule,
            },
        )(input)
    }
}

impl Monkey {
    /// Creates a monkey that uses Item::SimpleValue.
    fn new_reduced_worry_monkey(template: &MonkeyTemplate) -> Self {
        let items = template.items.iter()
            .map(|x| Item::new_simple_value(*x))
            .collect();
        Self {
            monkey_num: template.monkey_num,
            items,
            operation: template.operation,
            throwing_rule: template.throwing_rule,
            actions: 0,
        }
    }

    /// Creates a monkey that uses Item::ModularValue.
    fn new_very_worrisome_monkey(template: &MonkeyTemplate, divisors: &BTreeSet<WorryLevel>) -> Self {
        let items = template.items.iter()
            .map(|x| Item::new_modular_value(*x, divisors))
            .collect();
        Self {
            monkey_num: template.monkey_num,
            items,
            operation: template.operation,
            throwing_rule: template.throwing_rule,
            actions: 0,
        }
    }

    /// When this is called, the Monkey examines its items, adjusts the worry
    /// level of each, then removes the thrown items from its own list and
    /// returns them to the caller (which can place them in the right location).
    fn perform(&mut self, reduce_worry: bool) -> Vec<ThrownItem> {
        let mut answer = Vec::new();
        if PRINT_WORK {println!("Monkey {}:", self.monkey_num);}
        loop { // loop through self.items in order, removing them as we use them
            match self.items.pop_front() {
                None => break, // self.items is now empty
                Some(mut item) => {
                    self.actions += 1;
                    if PRINT_WORK {println!("  Monkey inspects an item with a worry level of {}.", item);}
                    self.operation.perform(&mut item);
                    if reduce_worry {
                        item /= REDUCE_FACTOR;
                        if PRINT_WORK { println!("    Monkey gets bored with item. Worry level is divided by 3 to {}.", item); }
                    }
                    answer.push( self.throwing_rule.perform(item) );
                }
            }
        }
        self.items.clear(); // delete them all
        answer
    }

    /// Gives an item to a monkey.
    fn give_item(&mut self, item: Item) {
        self.items.push_back(item);
    }
}

impl MonkeyTroopTemplate {
    // NOTE: I would rather have this return an error. But I can't figure out
    //   how do do that within the parser. So it will panic instead.
    fn new(monkeys: Vec<MonkeyTemplate>) -> Self {
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
            nom::multi::separated_list0(line_ending, MonkeyTemplate::parse),
            |monkeys| MonkeyTroopTemplate::new(monkeys)
        )(input)
    }
}

impl MonkeyTroop {
    /// Constructs a MonkeyTroop from a template and a choice of whether we'll reduce values or not.
    fn new(template: &MonkeyTroopTemplate, reduce_worry: bool) -> Self {
        let monkeys = if reduce_worry {
            template.monkeys.iter().map(|x| Monkey::new_reduced_worry_monkey(x)).collect()
        } else {
            let divisors: BTreeSet<WorryLevel> = BTreeSet::from_iter(
                template.monkeys.iter().map(|x| x.throwing_rule.divide_by)
            );
            template.monkeys.iter().map(|x| Monkey::new_very_worrisome_monkey(x, &divisors)).collect()
        };
        Self{reduce_worry, monkeys}
    }

    /// Performs one round of the monkeys.
    fn perform_round(&mut self) {
        for monkey_num in 0..self.monkeys.len() {
            let monkey = self.monkeys.get_mut(monkey_num).unwrap();
            let thrown_items = monkey.perform(self.reduce_worry);
            for thrown_item in thrown_items {
                self.monkeys.get_mut(thrown_item.target).unwrap().give_item(thrown_item.item);
            }
        }
    }

    /// Prints a description to stdout
    fn show_holdings(&self) {
        for monkey in self.monkeys.iter() {
            println!(
                "Monkey {}: ({} actions) {}",
                monkey.monkey_num,
                monkey.actions,
                monkey.items.iter().map(|x| format!("{}", x)).join(", ")
            );
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

fn part_a(monkey_troop_template: &MonkeyTroopTemplate) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    let reduce_worry = true;
    let mut monkey_troop = MonkeyTroop::new(monkey_troop_template, reduce_worry);
    if PRINT_WORK {monkey_troop.show_holdings();}
    for round in 0..20 {
        monkey_troop.perform_round();
        if PRINT_WORK {
            println!();
            println!("After round {}", round + 1);
            monkey_troop.show_holdings();
        }
    }
    println!();
    println!("The monkey business is {}", monkey_troop.monkey_business());
    Ok(())
}


fn part_b(monkey_troop_template: &MonkeyTroopTemplate) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    let reduce_worry = false;
    let mut monkey_troop = MonkeyTroop::new(monkey_troop_template, reduce_worry);
    if PRINT_WORK {monkey_troop.show_holdings();}
    for _ in 0..10000 {
        monkey_troop.perform_round();
    }
    println!();
    println!("The monkey business is {}", monkey_troop.monkey_business());
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
