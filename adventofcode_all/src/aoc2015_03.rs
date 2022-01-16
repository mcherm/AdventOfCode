use std::fs;
use std::io;
use itertools::Itertools;
use std::collections::HashSet;


fn input() -> Result<Vec<Direction>, io::Error> {
    let s = fs::read_to_string("input/2015/03/input.txt")?;
    Ok(s.chars().map_into().collect())
}


enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn deltas(&self) -> (i32,i32) {
        match self {
            Direction::Up => (0,1),
            Direction::Down => (0,-1),
            Direction::Left => (-1,0),
            Direction::Right => (1,0),
        }
    }
}

impl From<char> for Direction {
    fn from(c: char) -> Self {
        match c {
            '^' => Direction::Up,
            'v' => Direction::Down,
            '<' => Direction::Left,
            '>' => Direction::Right,
            _ => panic!("Invalid input")
        }
    }
}


fn part_a(route: &Vec<Direction>) -> Result<(), io::Error> {
    let mut visited: HashSet<(i32,i32)> = HashSet::new();
    let mut loc = (0,0);
    visited.insert(loc);
    for d in route {
        let deltas = d.deltas();
        loc = (loc.0 + deltas.0, loc.1 + deltas.1);
        visited.insert(loc);
    }
    println!("We visited {} houses.", visited.len());
    Ok(())
}

fn part_b(route: &Vec<Direction>) -> Result<(), io::Error> {
    let mut visited: HashSet<(i32,i32)> = HashSet::new();
    let mut locs = [(0,0), (0,0)];
    visited.insert(locs[0]);
    for (i, d) in route.iter().enumerate() {
        let deltas = d.deltas();
        locs[i%2] = (locs[i%2].0 + deltas.0, locs[i%2].1 + deltas.1);
        visited.insert(locs[i%2]);
    }
    println!("With robo-santa, we visited {} houses.", visited.len());
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let route = input()?;
    part_a(&route)?;
    part_b(&route)?;
    Ok(())
}
