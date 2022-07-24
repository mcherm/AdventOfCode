
extern crate anyhow;

use std::fs;
use anyhow::Error;

use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::newline,
    combinator::map,
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u32 as nom_u32;


fn input() -> Result<Vec<DiskPlacement>, Error> {
    let s = fs::read_to_string("input/2016/input_15.txt")?;
    match DiskPlacement::parse_list(&s) {
        Ok(("", disk_placements)) => Ok(disk_placements),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}

#[derive(Debug, Copy, Clone)]
struct DiskPlacement {
    disk_num: u32,
    positions: u32,
    pos_at_0: u32,
}

impl DiskPlacement {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("Disc #"),
                nom_u32,
                tag(" has "),
                nom_u32,
                tag(" positions; at time=0, it is at position "),
                nom_u32,
                tag("."),
            )),
            |(_, disk_num, _, positions, _, pos_at_0, _)| Self{disk_num, positions, pos_at_0}
        )(input)
    }

    fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }
}


#[derive(Debug, Copy, Clone)]
struct DiskPos {
    positions: u32,
    pos_to_hit: u32,
}

impl DiskPos {
    fn rotate(&mut self, n: u32) {
        self.pos_to_hit = (self.pos_to_hit + n) % self.positions;
    }
}


fn part_a(disk_placements: &Vec<DiskPlacement>) {
    println!("\nPart a:");

    let mut dps: Vec<DiskPos> = disk_placements.iter().map(|x: &DiskPlacement| DiskPos{
        positions: x.positions,
        pos_to_hit: (x.pos_at_0 + x.disk_num) % x.positions,
    }).collect();

    // REALLY could just solve this with math instead of computer.
    let mut turns = 0;
    loop {
        if dps.iter().all(|x| x.pos_to_hit == 0) {
            break;
        }
        turns += 1;
        for dp in dps.iter_mut() {
            dp.rotate(1);
        }
    }
    println!("It first works on turn {}.", turns);
}


fn part_b(_disk_placements: &Vec<DiskPlacement>) {
    println!("\nPart b:");
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}


// For my problem it's LESS than 431705
