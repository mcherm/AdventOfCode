use std::fs;
use std::io;
use std::collections::HashSet;


fn input() -> Result<Vec<String>, io::Error> {
    let s = fs::read_to_string("input/2015/05/input.txt")?;
    Ok(s.lines().map(|x| x.to_string()).collect())
}

const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
const PROHIBITED_PAIRS: [(char,char); 4] = [
    ('a', 'b'),
    ('c', 'd'),
    ('p', 'q'),
    ('x', 'y'),
];

fn is_nice_first(s: &str) -> bool {
    let mut vowel_count = 0;
    let mut doubled_letter_count = 0;
    let mut prev_char: Option<char> = None;
    for c in s.chars() {
        if VOWELS.contains(&c) {
            vowel_count += 1;
        }
        match prev_char {
            Some(prev) => {
                if prev == c {
                    doubled_letter_count += 1;
                }
                if PROHIBITED_PAIRS.contains(&(prev, c)) {
                    return false;
                }
            },
            None => {},
        }
        prev_char = Some(c);
    }
    vowel_count >= 3 && doubled_letter_count >= 1
}


fn is_nice_second(s: &str) -> bool {
    let mut pairs_seen: HashSet<(char,char)> = HashSet::new();

    let mut nonoverlapping_pair_dups: bool = false;
    let mut repeat_with_one_between: bool = false;
    let mut prev_char: Option<char> = None;
    let mut third_back_char: Option<char> = None;
    for c in s.chars() {
        match (third_back_char, prev_char) {
            (Some(third), Some(prev)) => {
                if third == c {
                    repeat_with_one_between = true;
                    if nonoverlapping_pair_dups {
                        return true;
                    }
                }
                if pairs_seen.contains(&(prev, c)) {
                    nonoverlapping_pair_dups = true;
                    if repeat_with_one_between {
                        return true;
                    }
                }
                // now we can insert the previous pair into pairs_seen
                pairs_seen.insert((third, prev));
            },
            _ => {}, // not yet far enough into the string
        }
        third_back_char = prev_char;
        prev_char = Some(c);
    }
    false
}


fn part_a(strings: &Vec<String>) -> Result<(), io::Error> {
    let nice_count = strings.iter().filter(|x| is_nice_first(x)).count();
    println!("There are {} nice strings by the first rules.", nice_count);
    Ok(())
}

fn part_b(strings: &Vec<String>) -> Result<(), io::Error> {
    let nice_count = strings.iter().filter(|x| is_nice_second(x)).count();
    println!("There are {} nice strings by the second rules.", nice_count);
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let route = input()?;
    part_a(&route)?;
    part_b(&route)?;
    Ok(())
}
