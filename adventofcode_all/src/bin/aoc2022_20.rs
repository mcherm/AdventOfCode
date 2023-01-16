
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fs;

    pub fn input() -> Result<Vec<i64>, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_20.txt")?;
        Ok(s.lines()
            .map(|line| line.parse::<i64>())
            .collect::<Result<_, _>>()?)
    }

}



// ======= Compute =======

mod solve_1 {
    use std::fmt::{Display, Formatter};
    use itertools::Itertools;

    #[derive(Debug)]
    pub struct Item {
        pub orig_pos: usize,
        pub value: i64,
    }

    #[derive(Debug)]
    pub struct NumList {
        items: Vec<Item>,
    }


    impl NumList {
        /// Creates a new NumList.
        pub fn new(input: &Vec<i64>) -> Self {
            let items = input.iter()
                .enumerate()
                .map(|(orig_pos, &value)| Item{orig_pos, value})
                .collect();
            NumList{items}
        }

        /// Moves the item at position from_pos by a total of move_by.
        fn move_item(&mut self, from_pos: usize) {
            let item = self.items.remove(from_pos);
            let new_pos = (item.value + (from_pos as i64)).rem_euclid(self.items.len() as i64);
            let new_pos = if new_pos == 0 { self.items.len() } else { new_pos as usize };
            self.items.insert(new_pos, item);
        }

        /// Performs the "mix" operation once.
        pub fn mix(&mut self) {
            for orig_pos in 0..self.items.len() {
                let current_pos = self.items.iter().position(|x| x.orig_pos == orig_pos).unwrap();
                self.move_item(current_pos);
            }
        }

        /// Performs "grove coordinate" calculation
        pub fn grove_coord_sum(&self) -> i64 {
            let zero_pos = self.items.iter().position(|x| x.value == 0).unwrap();
            [1000, 2000, 3000].iter()
                .map(|offset| self.items[(zero_pos + offset) % self.items.len()].value)
                .sum()
        }
    }

    impl Display for NumList {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}]", self.items.iter().map(|x| x.value.to_string()).join(", "))
        }
    }
}


// ======= main() =======

use crate::parse::input;
use crate::solve_1::NumList;


fn part_a(input: &Vec<i64>) {
    println!("\nPart a:");
    let mut nums = NumList::new(input);
    nums.mix();
    let sum = nums.grove_coord_sum();
    println!("The grove coordinate sum is {}", sum);
}


fn part_b(input: &Vec<i64>) {
    println!("\nPart b:");
    let decryption_key = 811589153;
    let bigger_vals = input.iter().map(|x| x * decryption_key).collect();
    let mut nums = NumList::new(&bigger_vals);
    for _ in 0..10 {
        nums.mix();
    }
    let sum = nums.grove_coord_sum();
    println!("The grove coordinate sum is {}", sum);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}

