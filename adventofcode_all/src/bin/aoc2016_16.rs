
extern crate anyhow;

use std::fmt::{Display, Formatter};
use std::fs;
use anyhow::Error;


const PART_A_DISK_LEN: usize = 272;
const PART_B_DISK_LEN: usize = 35651584;


fn input() -> Result<String, Error> {
    Ok(fs::read_to_string("input/2016/input_16.txt")?)
}


#[derive(Debug, Clone)]
struct Dragon {
    bits: Vec<bool>,
}

impl Dragon {
    fn len(&self) -> usize {
        self.bits.len()
    }

    fn make_dragon(s: &String) -> Self {
        Dragon{
            bits: s.chars().map(|c| match c {
                '1' => true,
                '0' => false,
                _ => panic!("Unexpected char in string."),
            }).collect()
        }
    }

    /// This returns a new dragon which is just over 2x as long.
    fn grow(&self) -> Self {
        let mut new_bits: Vec<bool> = Vec::with_capacity(self.bits.len() * 2 + 1);
        new_bits.extend(self.bits.iter());
        new_bits.push(false);
        new_bits.extend(self.bits.iter().rev().map(|x| !x));
        Self{bits: new_bits}
    }

    /// Returns a new dragon which is exactly the given size by growing, then
    /// truncating.
    fn grow_to(&self, new_size: usize) -> Dragon {
        let mut dr: Dragon = self.clone();
        while dr.len() <= new_size {
            dr = dr.grow()
        }
        dr.bits.truncate(new_size);
        dr
    }

    fn immediate_checksum(&self) -> Dragon {
        let mut new_bits: Vec<bool> = Vec::with_capacity((self.len() + 1) / 2);
        new_bits.extend(self.bits.chunks(2).map(|x| x[0] == x[1]));
        Dragon{bits: new_bits}
    }

    fn checksum(&self) -> Dragon {
        let mut answer = self.immediate_checksum();
        while answer.len() % 2 == 0 {
            answer = answer.immediate_checksum();
        }
        answer
    }

}

impl Display for Dragon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for b in self.bits.iter() {
            write!(f, "{}", if *b {'1'} else {'0'})?
        }
        Ok(())
    }
}


fn part_a(s: &String) {
    println!("\nPart a:");
    let mut dr = Dragon::make_dragon(s);
    dr = dr.grow_to(PART_A_DISK_LEN);
    println!("Dragon is {}", dr);
    let checksum = dr.checksum();
    println!("Checksum is {}", checksum);
}

fn part_b(s: &String) {
    println!("\nPart b:");
    // FIXME: I used brute strength here... probably should have been more clever.
    let mut dr = Dragon::make_dragon(s);
    dr = dr.grow_to(PART_B_DISK_LEN);
    let checksum = dr.checksum();
    println!("Checksum is {}", checksum);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
