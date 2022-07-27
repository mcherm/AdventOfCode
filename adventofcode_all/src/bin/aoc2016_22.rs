
extern crate anyhow;

use std::fs;
use anyhow::Error;
use std::cmp::max;
use std::collections::HashMap;


use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{multispace1, line_ending, not_line_ending},
    combinator::map,
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u16 as nom_u16;


fn input() -> Result<Grid, Error> {
    let s = fs::read_to_string("input/2016/input_22.txt")?;
    match GridLoader::parse(&s) {
        Ok(("", grid_loader)) => Ok(grid_loader.make_grid()),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



#[derive(Copy, Clone, Debug)]
struct Node {
    x: usize,
    y: usize,
    #[allow(dead_code)]
    size: usize,
    used: usize,
    avail: usize,
}

struct GridLoader {
    nodes: Vec<Node>,
}

struct Grid {
    nodes: HashMap<(usize,usize),Node>,
    size: (usize,usize)
}



fn nom_usize(input: &str) -> IResult<&str, usize> {
    map(
        nom_u16,
        |x| usize::from(x)
    )(input)
}

fn nom_line(input: &str) -> IResult<&str, &str> {
    terminated( not_line_ending, line_ending )(input)
}


impl Node {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("/dev/grid/node-x"),
                nom_usize,
                tag("-y"),
                nom_usize,
                multispace1,
                nom_usize,
                tag("T"),
                multispace1,
                nom_usize,
                tag("T"),
                multispace1,
                nom_usize,
                tag("T"),
                multispace1,
                nom_usize,
                tag("%"),
                line_ending,
            )),
            |(_, x, _, y, _, size, _, _, used, _, _, avail, _, _, _, _, _)| Node{x, y, size, used, avail}
        )(input)
    }
}

impl GridLoader {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                nom_line,
                nom_line,
                many0(Node::parse),
            )),
            |(_, _, nodes,)| Self{ nodes }
        )(input)
    }

    fn make_grid(&self) -> Grid {
        let mut max_x = 0;
        let mut max_y = 0;
        let mut nodes: HashMap<(usize,usize),Node> = HashMap::new();
        for node in self.nodes.iter() {
            max_x = max(max_x, node.x);
            max_y = max(max_y, node.y);
            nodes.insert((node.x, node.y), node.clone());
        }
        assert_eq!( nodes.len(), (max_x + 1) * (max_y + 1) ); // Guarantees we got all of them
        Grid{nodes, size: (max_x + 1, max_y + 1)}
    }
}

impl Grid {
    fn count_viable_pairs(&self) -> usize {
        let mut count = 0;
        for y1 in 0..self.size.1 {
            for x1 in 0..self.size.0 {
                let n1: &Node = self.nodes.get(&(x1,y1)).unwrap();
                if n1.used != 0 {
                    for y2 in 0..self.size.1 {
                        for x2 in 0..self.size.0 {
                            if (x1,y1) != (x2,y2) {
                                let n2: &Node = self.nodes.get(&(x2,y2)).unwrap();
                                if n1.used <= n2.avail {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        count
    }
}



fn part_a(grid: &Grid) {
    println!("\nPart a:");
    let pair_count = grid.count_viable_pairs();
    println!("There are {} viable pairs.", pair_count);
}



fn part_b(_grid: &Grid) {
    println!("\nPart b:");
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}



// ==========================================================================================

#[cfg(test)]
mod tests {
    use super::*;

}
