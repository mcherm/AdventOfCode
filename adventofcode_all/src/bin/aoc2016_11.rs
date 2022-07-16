
extern crate anyhow;

use std::fs;
use anyhow::Error;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, newline},
    combinator::{value, map},
    multi::{many0, separated_list1},
    sequence::{terminated, tuple},
};


fn input() -> Result<Vec<FloorDescription>, Error> {
    let s = fs::read_to_string("input/2016/input_11.txt")?;
    match FloorDescription::parse_list(&s) {
        Ok(("", floor_descriptions)) => Ok(floor_descriptions),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



#[derive(Debug, Clone)]
pub enum Item {
    Generator(String),
    Microchip(String),
}

impl Item {
    fn parse_generator<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("a "),
                alpha1,
                tag(" generator")
            )),
            |(_, name, _): (&str, &str, &str)| Item::Generator(name.to_string())
        )(input)
    }

    fn parse_microchip<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("a "),
                alpha1,
                tag("-compatible microchip")
            )),
            |(_, name, _): (&str, &str, &str)| Item::Microchip(name.to_string())
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
                tag(", and "),
                Self::parse,
            )),
            |(mut items, _, last_item)| {
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
    floor_num: u8,
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


fn part_a(floor_descriptions: &Vec<FloorDescription>) {
    println!("\nPart a:");
    for floor_description in floor_descriptions {
        println!("{:?}", floor_description);
    }
}


fn part_b(_floor_descriptions: &Vec<FloorDescription>) {
    println!("\nPart b:");
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
