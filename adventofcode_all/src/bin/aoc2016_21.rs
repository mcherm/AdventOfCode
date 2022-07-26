
extern crate anyhow;
extern crate lazy_static;

use std::collections::BTreeMap;
use std::fs;
use std::sync::Mutex;
use anyhow::Error;
use lazy_static::lazy_static;


use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, newline},
    combinator::{opt, map},
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u32 as nom_u32;


fn input() -> Result<Vec<Operation>, Error> {
    let s = fs::read_to_string("input/2016/input_21.txt")?;
    match Operation::parse_list(&s) {
        Ok(("", operations)) => Ok(operations),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



#[derive(Copy, Clone, Debug)]
enum Operation {
    SwapPosition(usize, usize),
    SwapLetter(char, char),
    ReverseRange(usize, usize),
    RotateLeft(usize),
    RotateRight(usize),
    RotateByLetter(char),
    MovePosition(usize, usize),
}


impl Operation {

    fn parse_swap_position(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("swap position "),
                nom_u32,
                tag(" with position "),
                nom_u32,
            )),
            |(_, p1, _, p2)| Operation::SwapPosition(usize::try_from(p1).unwrap(), usize::try_from(p2).unwrap())
        )(input)
    }

    fn parse_swap_letter(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("swap letter "),
                anychar,
                tag(" with letter "),
                anychar,
            )),
            |(_, c1, _, c2)| Operation::SwapLetter(c1, c2)
        )(input)
    }

    fn parse_reverse_range(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("reverse positions "),
                nom_u32,
                tag(" through "),
                nom_u32,
            )),
            |(_, p1, _, p2)| Operation::ReverseRange(usize::try_from(p1).unwrap(), usize::try_from(p2).unwrap())
        )(input)
    }

    fn parse_rotate_left(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("rotate left "),
                nom_u32,
                tag(" step"),
                opt(tag("s")), // could be singular or plural
            )),
            |(_, n, _, _)| Operation::RotateLeft(usize::try_from(n).unwrap())
        )(input)
    }

    fn parse_rotate_right(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("rotate right "),
                nom_u32,
                tag(" step"),
                opt(tag("s")), // could be singular or plural
            )),
            |(_, n, _, _)| Operation::RotateRight(usize::try_from(n).unwrap())
        )(input)
    }

    fn parse_rotate_by_letter(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("rotate based on position of letter "),
                anychar,
            )),
            |(_, c)| Operation::RotateByLetter(c)
        )(input)
    }

    fn parse_move_position(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("move position "),
                nom_u32,
                tag(" to position "),
                nom_u32,
            )),
            |(_, p1, _, p2)| Operation::MovePosition(usize::try_from(p1).unwrap(), usize::try_from(p2).unwrap())
        )(input)
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            Self::parse_swap_position,
            Self::parse_swap_letter,
            Self::parse_reverse_range,
            Self::parse_rotate_left,
            Self::parse_rotate_right,
            Self::parse_rotate_by_letter,
            Self::parse_move_position,
        ))(input)
    }

    fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }
}


/// This function computes the reverse map of the oddly-specific rotate_by_letter algorithm. It
/// caches the result (which can be expensive to compute the first time). Pass it the length of
/// the array and the position in the array and it returns the number of steps to the right it
/// should be rotated to restore it. HOWEVER, if it ever is asked to return a value which can
/// be mapped to multiple locations, then it PANICS.
fn reverse_map_for_rotate_by_letter(len: usize, pos: usize) -> usize {
    lazy_static! {
        static ref MEMOIZED: Mutex<BTreeMap<usize,BTreeMap<usize,Option<usize>>>> = Mutex::new(BTreeMap::new());
    }
    let mut cache = MEMOIZED.lock().unwrap();
    match cache.get(&len) {
        None => {
            // This length hasn't been computed yet
            let mut new_map: BTreeMap<usize,Option<usize>> = BTreeMap::new();
            for start_at in 0..len {
                let end_at = (start_at * 2 + (if start_at >= 4 {2} else {1})) % len;
                if new_map.contains_key(&end_at) {
                    new_map.insert(end_at, None); // multiple ways to get this output
                } else {
                    let steps_to_right = (start_at + len - end_at) % len;
                    new_map.insert(end_at, Some(steps_to_right));
                }
            }
            let answer_opt: Option<usize> = *new_map.get(&pos).unwrap();
            cache.insert(len, new_map);
            match answer_opt {
                None => panic!("Cannot perform reverse map of {} for length {}", pos, len),
                Some(x) => x.clone()
            }
        }
        Some(precomputed_map) => {
            match precomputed_map.get(&pos).unwrap() {
                None => panic!("Cannot perform reverse map of {} for length {}", pos, len),
                Some(x) => x.clone()
            }
        }
    }
}



impl Operation {
    /// This modifies (in place) the vector passed to it according to the rule for the
    /// particular operation being applied.
    fn apply(&self, data: &mut Vec<char>) {
        match self {
            Operation::SwapPosition(p1, p2) => {
                data.swap(*p1, *p2);
            }
            Operation::SwapLetter(c1, c2) => {
                for x in data.iter_mut() {
                    if *x == *c1 {
                        *x = *c2;
                    } else if *x == *c2 {
                        *x = *c1;
                    }
                };
            }
            Operation::ReverseRange(p1, p2) => {
                let (mut x1, mut x2) = (*p1, *p2);
                while x1 < x2 {
                    data.swap(x1, x2);
                    x1 += 1;
                    x2 -= 1;
                }
            }
            Operation::RotateLeft(x) => {
                let len = data.len();
                data.rotate_left(*x % len);
            }
            Operation::RotateRight(x) => {
                let len = data.len();
                data.rotate_right(*x % len);
            }
            Operation::RotateByLetter(c) => {
                let pos_opt = data.iter().position(|x| *x == *c);
                if let Some(pos) = pos_opt {
                    let rotate_by = pos + (if pos >= 4 {2} else {1});
                    let len = data.len();
                    data.rotate_right(rotate_by % len);
                }
            }
            Operation::MovePosition(p1, p2) => {
                let c = data.remove(*p1);
                data.insert(*p2, c);
            }
        }
    }

    /// This modifies (in place) the vector passed to it according to the REVERSE of the rule for
    /// the particular operation being applied.
    fn reverse(&self, data: &mut Vec<char>) {
        match self {
            Operation::SwapPosition(_, _) => {
                self.apply(data);
            }
            Operation::SwapLetter(_, _) => {
                self.apply(data);
            }
            Operation::ReverseRange(_, _) => {
                self.apply(data);
            }
            Operation::RotateLeft(x) => {
                Operation::RotateRight(*x).apply(data);
            }
            Operation::RotateRight(x) => {
                Operation::RotateLeft(*x).apply(data);
            }
            Operation::RotateByLetter(c) => {
                match data.iter().position(|x| *x == *c) {
                    None => return, // rotated by a letter that didn't exist did nothing
                    Some(current_pos) => {
                        let steps_to_right = reverse_map_for_rotate_by_letter(data.len(), current_pos);
                        Operation::RotateRight(steps_to_right).apply(data);
                    }
                }
            }
            Operation::MovePosition(p1, p2) => {
                Operation::MovePosition(*p2, *p1).apply(data);
            }
        }
    }
}

const PART_A_INPUT_STRING: &str = "abcdefgh";
const PART_B_INPUT_STRING: &str = "fbgdceah";


fn part_a(operations: &Vec<Operation>) {
    println!("\nPart a:");

    let mut data = PART_A_INPUT_STRING.chars().collect();
    for op in operations {
        op.apply(&mut data);
    }
    let s: String = data.iter().collect();
    println!("The output string is \"{}\".", s);
}



fn part_b(operations: &Vec<Operation>) {
    println!("\nPart b:");

   let mut data: Vec<char> = PART_B_INPUT_STRING.chars().collect();
    for op in operations.iter().rev() {
        op.reverse(&mut data);
    }
    let s: String = data.iter().collect();
    println!("The output string is \"{}\".", s);
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

    /// To make it easy to write lots of similar tests, this applies an provided operation
    /// to the string "abcdeff" and asserts that the result matches a provided string.
    fn assert_result(op: Operation, expect: &str) {
        let mut d = "abcdeff".chars().collect();
        op.apply(&mut d);
        assert_eq!(d, expect.chars().collect::<Vec<char>>());
    }

    #[test]
    fn test_swap_position() {
        assert_result(Operation::SwapPosition(2, 4), "abedcff");
    }

    #[test]
    fn test_swap_letter() {
        assert_result(Operation::SwapLetter('b', 'e'), "aecdbff");
        assert_result(Operation::SwapLetter('f', 'c'), "abfdecc");
    }

    #[test]
    fn test_reverse_range() {
        assert_result(Operation::ReverseRange(2, 4), "abedcff");
    }

    #[test]
    fn test_rotate() {
        assert_result(Operation::RotateLeft(3), "deffabc");
        assert_result(Operation::RotateRight(3), "effabcd");
    }

    #[test]
    fn test_rotate_by_letter() {
        assert_result(Operation::RotateByLetter('b'), "ffabcde");
        assert_result(Operation::RotateByLetter('d'), "deffabc");
        assert_result(Operation::RotateByLetter('e'), "bcdeffa");
        assert_result(Operation::RotateByLetter('f'), "abcdeff");
    }

    #[test]
    fn test_move_position() {
        assert_result(Operation::MovePosition(2,5), "abdefcf");
    }

}
