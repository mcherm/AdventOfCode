
extern crate anyhow;

use std::fs;
use std::cmp::max;


const PRINT_WORK: bool = false;


fn input() -> Result<Vec<u8>, anyhow::Error> {
    Ok(fs::read("input/2022/input_06.txt")?)
}

struct DupFound {
    new_start_at: usize,
    new_checked_to: usize,
}


fn check_for_dups(data: &[u8], scan_len: usize, start_at: usize, checked_to: usize) -> Option<DupFound> {
    if PRINT_WORK { println!("check_for_dups(data, {}, {}, {})", scan_len, start_at, checked_to); }
    for low in (start_at..(start_at + scan_len)).rev() {
        for high in (max(low, checked_to) + 1)..(start_at + scan_len) {
            if PRINT_WORK { println!("  Compare {} to {} (that is, {} to {})", low, high, data[low] as char, data[high] as char); }
            if data[low] == data[high] {
                return Some(DupFound{new_start_at: low + 1, new_checked_to: high});
            }
        }
    }
    None // did NOT find any dups
}

fn scan_for_nondups(data: &[u8], scan_len: usize) {
    let mut start_at: usize = 0;
    let mut checked_to: usize = 0;
    loop {
        match check_for_dups(data, scan_len, start_at, checked_to) {
            None => {
                println!("Found no dups in range from {} to {}", start_at + 1, start_at + scan_len);
                return ();
            },
            Some(DupFound{new_start_at, new_checked_to}) => {
                start_at = new_start_at;
                checked_to = new_checked_to;
            }
        }
        if start_at + scan_len > data.len() {
            println!("There was NO place of length {} without dups.", scan_len);
            return ();
        }
    }
}


fn part_a(input: &Vec<u8>) {
    println!("\nPart a:");
    scan_for_nondups(input, 4);
}


fn part_b(input: &Vec<u8>) {
    println!("\nPart b:");
    scan_for_nondups(input, 14);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
