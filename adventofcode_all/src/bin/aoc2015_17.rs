use advent_lib::eznom;

use std::fs;
use std::io;
use std::cmp::Ordering;
use nom::multi::many0 as nom_many0;
use nom::character::complete::u32 as nom_value;
use nom::sequence::tuple as nom_tuple;
use nom::character::complete::newline as nom_newline;
use eznom::type_builder;


const EGGNOG_SUPPLY: u32 = 150;

fn parse(input: &str) -> nom::IResult<&str, u32> {
    let recognize = |s| nom_tuple((nom_value, nom_newline))(s);
    let build = |(size, _): (u32, char)| size;
    type_builder(recognize, build)(input)
}

pub fn parse_list(input: &str) -> nom::IResult<&str, Vec<u32>> {
    nom_many0(parse)(input)
}



fn input() -> Result<Vec<u32>, io::Error> {
    let s = fs::read_to_string("input/2015/17/input.txt")?;
    match parse_list(&s) {
        Ok(("", containers)) => Ok(containers),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



fn count_fills(containers: &Vec<u32>, position: usize, eggnog_left: u32) -> usize {
    assert!(eggnog_left > 0);
    if position == containers.len() {
        return 0; // We ran right off the end of the list
    }
    match eggnog_left.cmp(&containers[position]) {
        Ordering::Less => {
            0 // the list is sorted, so nothing else will work. We prune the search tree now.
        },
        Ordering::Equal => {
            // Found a way to fill it
            1 + count_fills(containers, position + 1, eggnog_left)
        },
        Ordering::Greater => {
            // Filled it and there's more left over
            count_fills(containers, position + 1, eggnog_left - containers[position]) +
                count_fills(containers, position + 1, eggnog_left)
        },
    }
}


fn count_fills_by_size(
    containers: &Vec<u32>,
    position: usize,
    eggnog_left: u32,
    containers_used: usize,
    min_containers_usable: usize,
    ways_to_fill_containers: usize,
) -> (usize, usize) { // returns (min_containers_usable, ways_to_fill_containers)
    assert!(eggnog_left > 0);
    if position == containers.len() {
        // We ran right off the end of the list
        return (min_containers_usable, ways_to_fill_containers)
    }
    if containers_used == min_containers_usable {
        // we've already used the max # of containers; nothing more will be found!
        return (min_containers_usable, ways_to_fill_containers)
    }
    match eggnog_left.cmp(&containers[position]) {
        Ordering::Less => {
            // the list is sorted, so nothing else will work. We prune the search tree now.
            (min_containers_usable, ways_to_fill_containers)
        },
        Ordering::Equal => {
            // Found a way to fill it
            let used = containers_used + 1;
            match used.cmp(&min_containers_usable) {
                Ordering::Less => {
                    // found a new minimal size
                    (used, 1)
                },
                Ordering::Equal => {
                    // found a new one of the existing size
                    //   add 1, then recurse with this container empty...
                    count_fills_by_size(containers, position + 1, eggnog_left, containers_used, min_containers_usable, ways_to_fill_containers + 1)
                },
                Ordering::Greater => {
                    // found something greater; should have pruned already
                    panic!()
                },
            }
        },
        Ordering::Greater => {
            // Filled it and there's more left over
            let if_fill_this = count_fills_by_size(containers, position + 1, eggnog_left - containers[position], containers_used + 1, min_containers_usable, ways_to_fill_containers);
            let if_not_fill = count_fills_by_size(containers, position + 1, eggnog_left, containers_used, min_containers_usable, ways_to_fill_containers);
            match if_fill_this.0.cmp(&if_not_fill.0) {
                Ordering::Less => {
                    if_fill_this
                },
                Ordering::Equal => {
                    (if_fill_this.0, if_fill_this.1 + if_not_fill.1)
                },
                Ordering::Greater => {
                    if_not_fill
                },
            }
        },
    }
}



fn part_a(containers: &Vec<u32>) -> Result<(), io::Error> {
    let mut containers = containers.clone();
    containers.sort();
    let fills = count_fills(&containers, 0, EGGNOG_SUPPLY);
    println!("There are {} ways to fill it.", fills);
    Ok(())
}


fn part_b(containers: &Vec<u32>) -> Result<(), io::Error> {
    let mut containers = containers.clone();
    containers.sort();
    let (min_containers_usable, ways_to_fill_containers) = count_fills_by_size(&containers, 0, EGGNOG_SUPPLY, 0, usize::MAX, 0);
    println!("With minimal container count of {} there are {} ways to fill it.", min_containers_usable, ways_to_fill_containers);
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
