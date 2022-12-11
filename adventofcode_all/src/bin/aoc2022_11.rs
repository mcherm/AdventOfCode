
extern crate anyhow;

use std::fmt::{Display, Formatter};
use std::fs;
use anyhow::anyhow;
use nom;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::map,
    multi::many0,
    sequence::{terminated, pair, tuple},
};
use nom::character::complete::u32 as nom_u32;


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

#[derive(Debug)]
enum Operation {
    Mult(WorryLevel),
    Add(WorryLevel),
    Square,
}

#[derive(Debug)]
struct ThrowingRule {
    divide_by: WorryLevel,
    true_dest: usize,
    false_dest: usize,
}

#[derive(Debug)]
struct Monkey {
    monkey_num: usize,
    items: Vec<Item>,
    operation: Operation,
    throwing_rule: ThrowingRule,
}

#[derive(Debug)]
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
            },
        )(input)
    }
}

impl MonkeyTroop {
    // NOTE: I would rather have this return an error. But I can't figure out
    //   how do do that within the parser. So it will panic instead.
    fn new(monkeys: Vec<Monkey>) -> Self {
        for (i, monkey) in monkeys.iter().enumerate() {
            if monkey.monkey_num != i {
                panic!("Monkey {} has number {}.", i, monkey.monkey_num);
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

}


// ======= Calculations =======


// ======= main() =======

fn part_a(monkey_troop: &MonkeyTroop) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    println!("{:?}", monkey_troop);
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
