
extern crate anyhow;

use std::fs;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::newline,
    combinator::{value, map},
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u64 as nom_file_size;



fn input() -> Result<Vec<Command>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_07.txt")?;
    match Command::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type FileSize = u64;

#[derive(Debug, Clone)]
enum CdDestination {
    Root,
    Up,
    Down(String),
}

#[derive(Debug, Clone)]
enum DirEntry {
    Dir(String),
    File(FileSize, String),
}

#[derive(Debug, Clone)]
enum Command {
    Cd(CdDestination),
    Ls(Vec<DirEntry>),
}



fn parse_filename<'a>(input: &'a str) -> IResult<&'a str, String> {
    map(
        take_while1(|c| (c as char).is_ascii_alphabetic() || (c as char) == '.'),
        |s: &str| s.to_string()
    )(input)
}

impl CdDestination {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(CdDestination::Root, tag("/")),
            value(CdDestination::Up, tag("..")),
            map(parse_filename, |s| CdDestination::Down(s)),
        ))(input)
    }
}

impl DirEntry {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                tuple((
                    tag("dir "),
                    parse_filename
                )),
                |(_, s)| DirEntry::Dir(s)
            ),
            map(
                tuple((
                    nom_file_size,
                    tag(" "),
                    parse_filename,
                )),
                |(size, _, name)| DirEntry::File(size, name)
            ),
        ))(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }

}

impl Command {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                tuple((tag("$ cd "), CdDestination::parse, newline)),
                |(_, cd_dest, _)| Command::Cd(cd_dest)
            ),
            map(
                tuple((tag("$ ls"), newline, DirEntry::parse_list)),
                |(_, _, dir_entries)| Command::Ls(dir_entries)
            ),
        ))(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0(Self::parse)(input)
    }
}



fn part_a(input: &Vec<Command>) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    println!("READ: {:?}", input);
    Ok(())
}


fn part_b(_input: &Vec<Command>) -> Result<(), anyhow::Error> {
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
