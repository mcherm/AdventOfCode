use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use std::cmp;

/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    InvalidStartPositionLine,
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl From<std::num::ParseIntError> for InputError {
    fn from(error: std::num::ParseIntError) -> Self {
        InputError::BadInt(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err) => write!(f, "{}", err),
            InputError::BadInt(err) => write!(f, "{}", err),
            InputError::InvalidStartPositionLine => write!(f, "Invalid starting position"),
        }
    }
}

/// Read in the input file.
fn read_dice_game_file() -> Result<[usize;2], InputError> {
    // --- open file ---
    let filename = "data/2021/day/21/input.txt";
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    // --- read start positions ---
    let start_position_line_regex = Regex::new(
        r"^Player [12] starting position: (\d+)$"
    ).unwrap();

    let text: String = lines.next().ok_or(InputError::InvalidStartPositionLine)??;
    let capture = start_position_line_regex.captures(&text).ok_or(InputError::InvalidStartPositionLine)?;
    let p1_pos: usize = capture.get(1).unwrap().as_str().parse()?;

    let text: String = lines.next().ok_or(InputError::InvalidStartPositionLine)??;
    let capture = start_position_line_regex.captures(&text).ok_or(InputError::InvalidStartPositionLine)?;
    let p2_pos: usize = capture.get(1).unwrap().as_str().parse()?;

    // --- return result ---
    Ok([p1_pos, p2_pos])
}



const NUM_GAMESTATES: usize = 48400;

#[derive(Debug)]
struct GameMetaState {
    /// If p0 and p1 are player positions (indexed from zero) and s0 and s1 are player
    /// scores (letting 21 mean "21 or higher") then positions[key(p0,p1,s0,s1)] is the
    /// number of universes with that set of positions and scores.
    universes: [usize;NUM_GAMESTATES],
}


/// Given a pair of positions (indexed from 0), and a pair of keys, return the key for it.
/// This will always be a number from 0 to just-below 48400.
fn key(p1: usize, p2: usize, s1: usize, s2: usize) -> usize {
    (((p1 * 10) + p2) * 22 + s1) * 22 + s2
}

/// Given a key, return the pair of positions (indexed from 0) and scores in the order
/// (p0,p1,s0,s1)
fn unkey(key: usize) -> (usize, usize, usize, usize) {
    let s1 = key % 22;
    let remainder_1 = key / 22;
    let s0 = remainder_1 % 22;
    let remainder_2 = remainder_1 / 22;
    let p1 = remainder_2 % 10;
    let p0 = remainder_2 / 10;
    (p0,p1,s0,s1)
}


/// this represents all (roll, #-of-universes-it-happens-in) pairs
const ROLL_PROBS: [(usize, usize);7] = [
    (3, 1),
    (4, 3),
    (5, 6),
    (6, 7),
    (7, 6),
    (8, 3),
    (9, 1),
];


impl GameMetaState {
    fn new(pos: [usize;2]) -> Self {
        let mut universes = [0;48400];
        universes[key(pos[0], pos[1], 0, 0)] = 1;
        GameMetaState{universes}
    }

    /// Takes a turn, populating new universes. Returns true if at least one new universe
    /// was created; false if NO new universes were created.
    fn take_turn(&mut self, player: usize) -> bool {
        let mut did_something_new: bool = false;
        let mut new_universes = [0;48400];
        for old_key in 0..NUM_GAMESTATES {
            let (p0_old, p1_old, s0_old, s1_old)  = unkey(old_key);

            if s0_old == 21 || s1_old == 21 {
                // After someone wins, no one moves, no one scores, and no new universes are created
                new_universes[old_key] += self.universes[old_key];
            } else {
                did_something_new = true;
                for (roll, weight) in ROLL_PROBS {
                    let new_p = |p_old| (p_old + roll) % 10;
                    let new_score = |s_old, p_new| cmp::min(21, s_old + p_new + 1);
                    let p0_new;
                    let p1_new;
                    let s0_new;
                    let s1_new;
                    match player {
                        0 => {
                            p0_new = new_p(p0_old);
                            p1_new = p1_old;
                            s0_new = new_score(s0_old, p0_new);
                            s1_new = s1_old;
                        },
                        1 => {
                            p0_new = p0_old;
                            p1_new = new_p(p1_old);
                            s0_new = s0_old;
                            s1_new = new_score(s1_old, p1_new);
                        },
                        _ => panic!("Invalid player"),
                    }
                    let new_key = key(p0_new, p1_new, s0_new, s1_new);
                    new_universes[new_key] += weight * self.universes[old_key];
                }
            }
        }
        self.universes = new_universes;
        did_something_new
    }

    /// Returns the number of winning universes for [player1, player2]. If there is any
    /// universe in which both players won or neither player won then this panics.
    fn num_winning_universes(&self) -> [usize;2] {
        let mut p1_wins = 0;
        let mut p2_wins = 0;
        for key in 0..NUM_GAMESTATES {
            let count = self.universes[key];
            match unkey(key) {
                (_, _, 21, 21) => panic!("Both players won a game."),
                (_, _, 21, _) => p1_wins += count,
                (_, _, _, 21) => p2_wins += count,
                (_, _, _, _) => panic!("There was a game that no player won."),
            }
        }
        [p1_wins, p2_wins]
    }
}

impl fmt::Display for GameMetaState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for key in 0..NUM_GAMESTATES {
            let universes = self.universes[key];
            if universes > 0 {
                let (p0,p1,s0,s1) = unkey(key);
                writeln!(f, "There are {} universes with players at {} and {} and scores of {} and {}.",
                         universes, p0 + 1, p1 + 1, s0, s1)?;
            }
        }
        writeln!(f)
    }
}


fn run() -> Result<(),InputError> {
    println!("The size 'usize' goes up to {}", usize::MAX);
    println!("The size 'u32' goes up to {}", u32::MAX);
    println!("The size 'u64' goes up to {}", u64::MAX);

    let starts = read_dice_game_file()?;
    println!("starts: ({},{})", starts[0], starts[1]);

    let mut game = GameMetaState::new(starts);
    println!("GameMetaState = \n{}", game);
    let mut player = 0;
    let mut turn = 0;
    while game.take_turn(player) {
        turn += 1;
        println!("Completed turn {}.", turn);
        player = (player + 1) % 2;
    }
    let [p1_wins, p2_wins] = game.num_winning_universes();
    println!("The winner who won more won in {} universes.", cmp::max(p1_wins, p2_wins));

    Ok(())
}


pub fn main() {
    match run() {
        Ok(()) => {
            println!("Done");
        },
        Err(err) => println!("Error: {}", err),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file() {
        let _ = read_dice_game_file().unwrap();
    }

    #[test]
    fn test_pos_key() {
        assert_eq!(0, key(0, 0, 0, 0));
        assert_eq!(21, key(0, 0, 0, 21));
        assert_eq!(22, key(0, 0, 1, 0));
        assert_eq!(28, key(0, 0, 1, 6));
        assert_eq!(484-1, key(0, 0, 21, 21));
        assert_eq!(NUM_GAMESTATES - 1, key(9, 9, 21, 21));
    }

    #[test]
    fn test_unpos_key() {
        assert_eq!((0,0,0,0), unkey(0));
        assert_eq!((0,0,0,5), unkey(5));
        assert_eq!((0,0,5,0), unkey(110));
        assert_eq!((0,5,0,0), unkey(2420));
        assert_eq!((5,0,0,0), unkey(24200));
        assert_eq!((9,9,21,21), unkey(NUM_GAMESTATES - 1));
    }
}
