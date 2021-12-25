use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;


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


#[derive(Debug)]
struct ScoresAtPosition {

}

#[derive(Debug)]
struct GameMetaState {
    // if p1, p2 are player positions counting from zero, then positions[p1 * 10 + p2] is
    // the count of universes with those positions
    positions: [usize;100],
    // All scores are 0..=20, or else 21+ (for a total of 22 options). if s1 and s2 are
    // player scores (treating 21 as "21 and up") then scores[s1 * 22 + s2] is the count of
    // universes with that pair of scores.
    scores: [usize;484],
}


/// Given a pair of positions (indexed from 0), return the key for it
fn pos_key(p1: usize, p2: usize) -> usize {
    p1 * 10 + p2
}

/// Given a key, return the pair of positions (indexed from 0)
fn unkey_pos(key: usize) -> [usize;2] {
    [key / 10, key % 10]
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
        let mut positions: [usize;100] = [0;100];
        let scores: [usize;484] = [0;484];
        positions[pos_key(pos[0] - 1, pos[1] - 1)] = 1;
        GameMetaState{positions, scores}
    }

    fn take_turn(&mut self, player: usize) {
        let mut new_positions = [0;100];
        for old_key in 0..100 {
            let [p0_pos_old, p1_pos_old] = unkey_pos(old_key);
            for (roll, weight) in ROLL_PROBS {
                let p0_pos_new;
                let p1_pos_new;
                match player {
                    0 => {
                        p0_pos_new = (p0_pos_old + roll) % 10;
                        p1_pos_new = p1_pos_old;
                    },
                    1 => {
                        p0_pos_new = p0_pos_old;
                        p1_pos_new = (p1_pos_old + roll) % 10;
                    },
                    _ => panic!("Invalid player"),
                }
                let new_key = pos_key(p0_pos_new, p1_pos_new);
                new_positions[new_key] += weight * self.positions[old_key];
            }
        }
        self.positions = new_positions;
    }
}

impl fmt::Display for GameMetaState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for key in 0..100 {
            let universes = self.positions[key];
            if universes > 0 {
                let [p0_pos, p1_pos] = unkey_pos(key);
                writeln!(f, "There are {} universes with players at {} and {}.", universes, p0_pos + 1, p1_pos + 1)?;
            }
        }
        writeln!(f)
    }
}


fn run() -> Result<(),InputError> {
    let starts = read_dice_game_file()?;
    println!("starts: ({},{})", starts[0], starts[1]);

    let mut game = GameMetaState::new(starts);
    println!("GameMetaState = \n{}", game);
    let mut player = 0;
    game.take_turn(player);
    println!("GameMetaState = \n{}", game);
    // player = (player + 1) % 2;

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
}
