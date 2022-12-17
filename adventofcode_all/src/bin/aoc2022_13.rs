
extern crate anyhow;

use std::cmp::Ordering;
use std::fs;
use nom;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::map,
    sequence::tuple,
};
use nom::character::complete::u8 as nom_u8;
use std::iter::zip;



// ======= Parsing =======

fn input() -> Result<Vec<PacketPair>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_13.txt")?;
    match PacketPair::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Num = u8;

#[derive(Debug)]
enum PacketElem {
    List(Vec<Box<PacketElem>>),
    Value(Num),
}

#[derive(Debug)]
struct PacketPair(PacketElem, PacketElem);


impl PacketElem {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                nom_u8,
                |x| PacketElem::Value(x)
            ),
            map(
                nom::sequence::delimited(
                    tag("["),
                    nom::multi::separated_list0(tag(","), PacketElem::parse),
                    tag("]")
                ),
                |elems| PacketElem::List(elems.into_iter().map(|x| Box::new(x)).collect())
            ),
        ))(input)
    }
}


impl PartialEq for PacketElem {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for PacketElem {
}

impl PartialOrd for PacketElem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl Ord for PacketElem {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PacketElem::Value(a), PacketElem::Value(b)) => a.cmp(b),
            (PacketElem::List(a_list), PacketElem::List(b_list)) => {
                for (a,b) in zip(a_list, b_list) {
                    match a.cmp(b) {
                        Ordering::Less => return Ordering::Less,
                        Ordering::Greater => return Ordering::Greater,
                        Ordering::Equal => {}, // continue looping
                    }
                }
                // made it to the end of the shorter list with everything matching
                a_list.len().cmp(&b_list.len())
            },
            (PacketElem::Value(a), PacketElem::List(b_list)) => {
                let a_list = vec![Box::new(PacketElem::Value(a.clone()))];
                a_list.cmp(b_list)
            }
            (PacketElem::List(a_list), PacketElem::Value(b)) => {
                let b_list = vec![Box::new(PacketElem::Value(b.clone()))];
                a_list.cmp(&b_list)
            }
        }
    }
}

impl PacketPair {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                PacketElem::parse,
                line_ending,
                PacketElem::parse,
                line_ending,
            )),
            |(a, _, b, _)| PacketPair(a,b)
        )(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        nom::multi::separated_list0( line_ending, Self::parse )(input)
    }

    fn is_ordered(&self) -> bool {
        self.0 < self.1
    }
}


// ======= Compute =======

// ======= main() =======

fn part_a(input: &Vec<PacketPair>) {
    println!("\nPart a:");
    let mut sum = 0;
    for (i, pair) in input.iter().enumerate() {
        let idx = i + 1;
        let ordered = pair.is_ordered();
        if ordered {
            sum += idx;
        }
    }
    println!("The relevant sum is {}", sum);
}


fn part_b(input: &Vec<PacketPair>) {
    println!("\nPart b:");
    let divider_packets: PacketPair = PacketPair::parse("[[2]]\n[[6]]\n").unwrap().1;
    let mut packets: Vec<&PacketElem> = input.iter()
        .flat_map(|pair| [&pair.0, &pair.1])
        .collect();
    packets.push(&divider_packets.0);
    packets.push(&divider_packets.1);
    packets.sort();
    let pos_0 = packets.iter().position(|x| **x == divider_packets.0).unwrap() + 1;
    let pos_1 = packets.iter().position(|x| **x == divider_packets.1).unwrap() + 1;
    let product = pos_0 * pos_1;
    println!("The product we want is {}", product);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
