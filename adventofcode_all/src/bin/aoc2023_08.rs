use std::fmt::{Display, Formatter};
use anyhow;
use std::collections::HashMap;


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Direction {
    R, L
}

#[derive(Debug)]
pub struct Node {
    name: String,
    left: String,
    right: String,
}

#[derive(Debug)]
pub struct Input {
    path: Vec<Direction>,
    nodes: Vec<Node>,
}



mod parse {
    use std::fs;
    use super::{Input, Node, Direction};
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
    };


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_08.txt")?;
        match Input::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    impl Direction {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::branch::alt((
                    tag("R"),
                    tag("L"),
                )),
                |c: &str| match c {
                    "R" => Direction::R,
                    "L" => Direction::L,
                    _ => panic!("Invalid parse.")
                }
            )(input)
        }

        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            nom::sequence::terminated(
                nom::multi::many1( Self::parse ),
                nom::character::complete::line_ending,
            )(input)
        }
    }

    impl Node {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom::character::complete::alpha1,
                    tag(" = ("),
                    nom::character::complete::alpha1,
                    tag(", "),
                    nom::character::complete::alpha1,
                    tag(")"),
                    nom::character::complete::line_ending,
                )),
                |(name, _, left, _, right, _, _): (&str,_,&str,_,&str,_,_)| Self{
                    name: name.to_string(),
                    left: left.to_string(),
                    right: right.to_string(),
                }
            )(input)
        }
    }

    impl Input {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    Direction::parse_list,
                    nom::character::complete::line_ending,
                    nom::multi::many1( Node::parse )
                )),
                |(path, _, nodes)| Input{path, nodes}
            )(input)
        }
    }
}


// ======= Compute =======


impl Node {
    /// Given a Direction, this finds the next Node's name
    fn next(&self, dir: &Direction) -> &String {
        match dir {
            Direction::L => &self.left,
            Direction::R => &self.right,
        }
    }
}


/// A simple traversal function that starts from AAA and goes to ZZZ and returns the number
/// of steps taken (printing out the nodes as it goes).
fn traverse(input: &Input) -> usize {
    let node_map: HashMap<String, &Node> = input.nodes.iter()
        .map(|node: &Node| (node.name.clone(), node))
        .collect();
    let mut steps = 0;
    let mut node: &String = &"AAA".to_string();
    let mut path_iter = input.path.iter().cycle();
    while node != "ZZZ" {
        steps += 1;
        node = node_map.get(node).unwrap().next(path_iter.next().unwrap());
    }
    steps
}


// ======= main() =======


fn part_a(data: &Input) {
    println!("\nPart a:");
    let steps = traverse(data);
    println!("It took {} steps.", steps);
}


fn part_b(_data: &Input) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
