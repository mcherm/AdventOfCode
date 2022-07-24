
extern crate anyhow;

use std::fs;
use anyhow::Error;
use itertools::Itertools;
use std::iter::once;


fn input() -> Result<String, Error> {
    Ok(fs::read_to_string("input/2016/input_18.txt")?)
}


/// Given a row, returns the next one.
fn next_row(row: &str) -> String {
    once('.').chain(row.chars()).chain(once('.'))
        .tuple_windows()
        .map(|x| match x {
            ('^','^','.') => '^',
            ('.','^','^') => '^',
            ('^','.','.') => '^',
            ('.','.','^') => '^',
            _ => '.',
        })
        .collect()
}

/// Returns the number of safe tiles in a row.
fn count_safe(row: &str) -> usize {
    row.chars().filter(|x| *x == '.').count()
}


fn generate_block(row_one: &String, num_rows: usize) -> usize {
    let mut safe = 0;
    let mut row = row_one.clone();
    safe += count_safe(&row);
    for _ in 0..num_rows - 1 {
        row = next_row(&row);
        safe += count_safe(&row);
        println!("{}", row);
    }
    safe
}



const NUM_ROWS_A: usize = 40;
const NUM_ROWS_B: usize = 400000;

fn part_a(row_one: &String) {
    println!("\nPart a:");
    let safe = generate_block(row_one, NUM_ROWS_A);
    println!("A total of {} safe tiles", safe);
}


fn part_b(row_one: &String) {
    println!("\nPart b:");
    let safe = generate_block(row_one, NUM_ROWS_B);
    println!("A total of {} safe tiles", safe);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
