
extern crate anyhow;
extern crate md5;

use std::fs;
use anyhow::Error;
use itertools::Itertools;


fn input() -> Result<String, Error> {
    Ok(fs::read_to_string("input/2016/input_14.txt")?)
}

/// Returns a list of all characters repeated n times in a row within the string.
fn n_in_a_row(s: &str, n: usize) -> Vec<char> {
    assert!(n > 0);
    let mut answer = Vec::new();
    let mut cs = s.chars();
    let mut window: Vec<char> = Vec::with_capacity(n);
    for _ in 0..n {
        window.push(match cs.next() {
            None => return answer,
            Some(c) => c,
        });
    }

    loop {
        if window.iter().all_equal() {
            let c = *window.first().unwrap();
            if !answer.contains(&c) {
                answer.push(c);
            };
        }
        window.remove(0); // move everything down one slot
        window.push(match cs.next() {
            None => break,
            Some(c) => c,
        });
    }
    answer
}

/// Asserts c is a hex digit and returns the numeric value.
fn hex_val(c: char) -> usize {
    match c {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' => 10,
        'b' => 11,
        'c' => 12,
        'd' => 13,
        'e' => 14,
        'f' => 15,
        _ => panic!("{} is not a hex char", c),
    }
}


fn find_keys(salt: &str, key_stretching: usize) -> Vec<u64> {
    let mut counter: u64 = 0;
    let mut stop_after: Option<u64> = None; // We can stop examining numbers after we find this
    let mut pending: [Vec<u64>; 16] = Default::default();
    let mut keys: Vec<u64> = Vec::new();
    while counter < stop_after.unwrap_or(u64::MAX) {
        let s = format!("{}{}", salt, counter);
        let mut hex = format!("{:x}", md5::compute(s));
        for _ in 0..key_stretching {
            hex = format!("{:x}", md5::compute(hex));
        }
        for c in n_in_a_row(&hex, NUM_LATER_MATCHES) {
            for n in pending[hex_val(c)].iter() {
                if counter - n <= NUM_STEPS_LATER {
                    keys.push(*n);
                    println!("Now have {} keys after adding {}", keys.len(), keys.last().unwrap()); // FIXME: Remove
                    if keys.len() == NUM_KEYS_TO_FIND && stop_after.is_none() {
                        // We found enough keys. BUT -- we don't find the keys IN ORDER... perhaps
                        // there's an even earlier key that will have a NUM_LATER_MATCHES match with
                        // the very next hash after this one! So we'll set stop_after so as to
                        // continue on for NUM_STEPS_LATER more steps, and THEN exit. (And sort
                        // the list afterward.)
                        stop_after = Some(counter + NUM_STEPS_LATER);
                    }
                }
            }
            pending[hex_val(c)].clear(); // each was too old or already triggered
        }
        if let Some(c) = n_in_a_row(&hex, NUM_INITIAL_MATCHES).first() {
            pending[hex_val(*c)].push(counter);
        }
        counter += 1;
    }
    keys.sort();
    keys
}


const NUM_KEYS_TO_FIND: usize = 64;
const NUM_INITIAL_MATCHES: usize = 3;
const NUM_LATER_MATCHES: usize = 5;
const NUM_STEPS_LATER: u64 = 1000;

fn part_a(salt: &String) {
    println!("\nPart a:");

    let keys = find_keys(salt, 0);
    println!("{:?}", keys);
    println!(
        "Got keys. Key # {} comes from index {}.",
        NUM_KEYS_TO_FIND,
        keys.get(NUM_KEYS_TO_FIND - 1).unwrap(),
    );
}


fn part_b(salt: &String) {
    println!("\nPart b:");

    let keys = find_keys(salt, 2016);
    println!("{:?}", keys);
    println!(
        "Got keys. Key # {} comes from index {}.",
        NUM_KEYS_TO_FIND,
        keys.get(NUM_KEYS_TO_FIND - 1).unwrap(),
    );
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
