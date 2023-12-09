use anyhow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use itertools::Itertools;

// ======= Constants =======

const PRINT: bool = true;


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
                    nom::character::complete::alphanumeric1,
                    tag(" = ("),
                    nom::character::complete::alphanumeric1,
                    tag(", "),
                    nom::character::complete::alphanumeric1,
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
/// of steps taken.
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


impl Input {
    /// Returns a list of where the ghosts should start
    fn ghost_starts<'a>(&'a self) -> Vec<&'a String> {
        self.nodes.iter()
            .filter(|node| node.name.ends_with('A'))
            .map(|node| &node.name)
            .collect()
    }
}


/// This object can be walked through the grid starting from a certain location and it will
/// collect information about where the directions will lead.
#[derive(Debug)]
struct Ghost {
    stem_len: usize,
    cycle_len: usize,
    stem_ends: Vec<usize>, // steps to get to each end that happens before we enter the cycle
    cycle_ends: Vec<usize>, // steps WITHIN the cycle to get to each end that happens in the cycle
}


impl Ghost {
    /// Initialize a Ghost from a given start position.
    fn create<'a>(input: &'a Input, start: &'a String) -> Self {
        let node_map: HashMap<String, &Node> = input.nodes.iter()
            .map(|node: &Node| (node.name.clone(), node))
            .collect();
        let path_len = input.path.len();
        let return_to: usize;
        let stem_len: usize;
        let cycle_len: usize;
        let stem_ends: Vec<usize>;
        let cycle_ends: Vec<usize>;
        let mut node_name: &'a String = start;
        let mut path_iter = input.path.iter().cycle();
        let mut visit_order: Vec<&'a String> = Vec::new();
        let mut ends: Vec<usize> = Vec::new();
        loop {
            let mut prev_cycle_pos: usize = visit_order.len();
            while prev_cycle_pos >= path_len {
                prev_cycle_pos -= path_len;
                if visit_order[prev_cycle_pos] == node_name {
                    return_to = prev_cycle_pos;
                    stem_len = return_to;
                    cycle_len = visit_order.len() - stem_len;
                    stem_ends = ends.iter()
                        .copied()
                        .filter(|x: &usize| *x < return_to)
                        .collect_vec();
                    cycle_ends = ends.iter()
                        .copied()
                        .filter(|x: &usize| *x >= return_to)
                        .map(|x: usize| x - return_to)
                        .collect_vec();
                    return Ghost{stem_len, cycle_len, stem_ends, cycle_ends};
                }
            }
            if node_name.ends_with('Z') {
                ends.push(visit_order.len())
            }
            visit_order.push(node_name);
            node_name = node_map.get(node_name).unwrap().next(path_iter.next().unwrap());
        }
    }


    fn cycle_len(&self) -> usize {
        self.cycle_len
    }

    fn offset(&self) -> usize {
        self.stem_len
    }

    fn iter_exits(&self) -> impl Iterator<Item=usize> + '_ {
        self.stem_ends.iter()
            .map(|x| *x)
            .chain(
                (0..)
                    .map(move |iter_count| {
                        self.cycle_ends.iter()
                            .map(move |x| self.stem_len + (iter_count * self.cycle_len()) + *x)
                    })
                    .flatten()
            )
    }
}

impl Display for Ghost {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ghost{{cycle_len: {}, offset: {}, stem_ends: {:?}, cycle_ends: {:?} }}",
            self.cycle_len(),
            self.offset(),
            self.stem_ends,
            self.cycle_ends,
        )
    }
}




/// Given 2 ghosts, this finds the first place they overlap. It works (I think) for ANY
/// possible input, not JUST for ones where each ghost has a loop with a single exit.
fn find_first_overlap(ghosts: &Vec<Ghost>) -> usize {
    let mut exit_iterators = ghosts.into_iter()
        .map(|ghost| ghost.iter_exits())
        .collect_vec();
    let mut exits: Vec<usize> = exit_iterators.iter_mut()
        .map(|it| it.next().unwrap())
        .collect_vec();
    let mut loop_counter = 0;
    loop {
        loop_counter += 1;
        if PRINT && loop_counter % 1000000 == 0 {
            println!("Entering loop: exits = {:?}", exits);
        }
        if exits.iter().all_equal() {
            return exits[0];
        }
        let biggest = exits.iter().max().copied().unwrap();
        // advance until each one is at least caught up with the previous biggest one
        for i in 0..exits.len() {
            while exits[i] < biggest {
                exits[i] = exit_iterators[i].next().unwrap(); // exit iterators are infinite, so unwrap is safe
            }
        }
    }
}


// ======= main() =======


fn part_a(data: &Input) {
    println!("\nPart a:");
    let steps = traverse(data);
    println!("It took {} steps.", steps);
}


fn part_b(data: &Input) {
    println!("\nPart b:");
    let ghost_starts = data.ghost_starts();
    if PRINT {
        println!("ghost_starts: {:?}", ghost_starts);
    }
    let ghosts = ghost_starts.iter()
        .map(|start| Ghost::create(data, start))
        .collect_vec();
    if PRINT {
        for ghost in ghosts.iter() {
            println!("ghost: {}", ghost);
        }
    }
    let first_overlap = find_first_overlap(&ghosts);
    println!("The first 2 overlap at {}", first_overlap);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
