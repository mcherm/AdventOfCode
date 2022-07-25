
extern crate anyhow;

use std::fs;
use anyhow::Error;


fn input() -> Result<String, Error> {
    Ok(fs::read_to_string("input/2016/input_19.txt")?)
}

fn part_a(s: &String) {
    println!("\nPart a:");
    let n: u64 = s.parse().unwrap();
    // find the position of the biggest 1 bit
    let mut v = n;
    let mut r = 0;
    while v != 0 {
        v = v >> 1;
        r += 1;
    }
    // calculate the answer (see https://en.wikipedia.org/wiki/Josephus_problem )
    let answer = 2 * (n - (1 << (r - 1))) + 1;
    println!("answer: {}", answer);
}



fn part_b(s: &String) {
    println!("\nPart b:");
    let n: usize = s.parse().unwrap();

    // I can't find a way to map the well-known solution to the Josephus problem onto this
    // so I'm going to have to solve it.
    let mut nums: Vec<u32> = (1..=n).map(|x| u32::try_from(x).unwrap()).collect();
    let true_len = nums.len();
    let mut num_alive = true_len;
    let mut die_pos = true_len / 2;

    while num_alive > 1 {
        nums[die_pos] = 0; // set them to zero when they die
        num_alive -= 1;
        let mut move_by = if num_alive % 2 == 0 {2} else {1};
        while move_by > 0 {
            die_pos = (die_pos + 1) % true_len;
            if nums[die_pos] != 0 {
                move_by -= 1;
            }
        }
    }
    // -- find the one value not zero'ed out--
    let mut answer: u32 = 0;
    for num in nums {
        if num != 0 {
            answer = num;
            break;
        }
    }
    println!("The survivor started as number {}", answer);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
