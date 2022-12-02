
extern crate anyhow;

use std::fs;
use std::cmp::max;


fn input() -> Result<String, anyhow::Error> {
    Ok(fs::read_to_string("input/2022/input_01.txt")?)
}



fn part_a(input: &str) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    let mut max_load: u32 = 0;
    let mut load: u32 = 0;
    for line in input.lines() {
        if line.len() == 0 {
            max_load = max(max_load, load);
            load = 0;
        } else {
            let item: u32 = line.parse()?;
            load += item;
        }
    }
    println!("The largest load is {}.", max_load);
    Ok(())
}


fn part_b(input: &str) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    let mut loads: Vec<u32> = Vec::new();
    let mut load: u32 = 0;
    for line in input.lines() {
        if line.len() == 0 {
            loads.push(load);
            load = 0;
        } else {
            load += line.parse::<u32>()?;
        }
    }
    loads.sort_unstable_by(|x,y| y.cmp(x)); // sort reversed
    let top_three_loads: u32 = loads[..3].iter().sum();
    println!("The sum of the top three loads is {}.", top_three_loads);
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
