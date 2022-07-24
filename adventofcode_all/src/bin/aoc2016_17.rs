
extern crate anyhow;

use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs;
use md5;
use anyhow::Error;

const WINNER_ROOM: Coord = (3,3);


fn input() -> Result<String, Error> {
    Ok(fs::read_to_string("input/2016/input_17.txt")?)
}


type Coord = (usize, usize);

enum Direction {
    Up, Down, Left, Right,
}

use Direction::*;

struct State {
    full_str: String,
    room: (usize, usize),
}


impl Direction {
    /// Returns a new coord that is one step in the given direction.
    /// Will panic if that's out of bounds.
    fn step(&self, c: &Coord) -> Coord {
        match self {
            Up => {
                assert!(c.1 > 0);
                (c.0, c.1 - 1)
            },
            Down => {
                assert!(c.1 < WINNER_ROOM.1);
                (c.0, c.1 + 1)
            },
            Left => {
                assert!(c.0 > 0);
                (c.0 - 1, c.1)
            },
            Right => {
                assert!((c.0 < WINNER_ROOM.0));
                (c.0 + 1, c.1)
            },
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {Up=>'U', Down=>'D', Left=>'L', Right=>'R',})
    }
}

impl State {
    /// Gives the (one) starting state.
    fn starting_state(passcode: &String) -> State {
        State{full_str: passcode.clone(), room: (0,0)}
    }

    /// Gives the state reached by moving in the given direction.
    fn move_to(&self, direction: Direction) -> State {
        State{
            full_str: format!("{}{}", self.full_str, direction),
            room: direction.step(&self.room),
        }
    }

    fn is_winner(&self) -> bool {
        self.room == WINNER_ROOM
    }

    fn get_solution(&self, passcode: &String) -> String {
        self.full_str.strip_prefix(passcode).unwrap().to_string()
    }

    /// Returns the MD5 hash of the current state as a String.
    fn get_hash(&self) -> String {
        format!("{:x}", md5::compute(&self.full_str))
    }

    fn get_available_doors(&self) -> Vec<Direction> {
        let mut answer = Vec::new();
        let hash = self.get_hash();
        for (i, hash_char) in hash.chars().take(4).enumerate() {
            let unlocked: bool = match hash_char {
                'b' | 'c' | 'd' | 'e' | 'f' => true,
                _ => false
            };
            if unlocked {
                match i {
                    0 if self.room.1 > 0             => {answer.push(Up);},
                    1 if self.room.1 < WINNER_ROOM.1 => {answer.push(Down);},
                    2 if self.room.0 > 0             => {answer.push(Left);},
                    3 if self.room.0 < WINNER_ROOM.0 => {answer.push(Right);},
                    0|1|2|3                          => {}, // room would be off the edge of the map
                    _ => panic!("Invalid position.")
                };
            }
        }
        answer
    }
}

#[derive(Copy, Clone)]
enum Extreme {
    Shortest, Longest
}

/// Given a passcode, this explores the maze. If it can't be solved, it returns None, if it
/// be then it returns Some(s) where s is (one of) the shortest / longest route(s) to WINNER_ROOM.
fn find_solution(passcode: &String, extreme: Extreme) -> Option<String> {
    let mut states: VecDeque<State> = VecDeque::new();
    let mut longest_winner: Option<String> = None; // used if extreme == Extreme::Longest
    states.push_back(State::starting_state(passcode));

    while let Some(old_state) = states.pop_front() {
        for direction in old_state.get_available_doors() {
            let new_state = old_state.move_to(direction);
            if new_state.is_winner() {
                let solution = new_state.get_solution(passcode);
                match extreme {
                    Extreme::Shortest => return Some(solution), // short-circuit exit!
                    Extreme::Longest => longest_winner = Some(solution),
                }
            } else {
                states.push_back(new_state);
            }
        }
    }
    longest_winner
}



fn part_a(passcode: &String) {
    println!("\nPart a:");
    println!("{}", passcode);

    match find_solution(passcode, Extreme::Shortest) {
        None => println!("No solution exists."),
        Some(solution) =>  println!("Solved it with the following directions: {}", solution),
    };
}


fn part_b(passcode: &String) {
    println!("\nPart b:");

    match find_solution(passcode, Extreme::Longest) {
        None => println!("No solution exists."),
        Some(solution) => {
            println!("Solved it with the following directions: {}", solution);
            println!("...which has a length of {}", solution.len());
        },
    };
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
