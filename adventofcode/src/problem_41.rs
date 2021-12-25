use std::fmt;
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


struct DeterministicDie {
    next: usize,
    times_rolled: usize,
}

struct GameState {
    die: DeterministicDie,
    positions: [usize;2],
    scores: [usize;2],
}


impl DeterministicDie {
    fn new() -> Self {
        DeterministicDie{next: 1, times_rolled: 0}
    }

    fn roll(&mut self) -> usize {
        let answer = self.next;
        self.next += 1;
        if self.next == 101 {
            self.next = 1;
        }
        self.times_rolled += 1;
        answer
    }
}

impl GameState {
    fn new(positions: [usize;2]) -> Self {
        GameState{die: DeterministicDie::new(), positions, scores: [0;2]}
    }

    fn take_turn(&mut self, player: usize) {
        let roll_sum = self.die.roll() + self.die.roll() + self.die.roll();
        self.positions[player] = (self.positions[player] + roll_sum - 1) % 10 + 1;
        self.scores[player] += self.positions[player];
    }

    fn game_not_over(&self) -> bool {
        self.scores[0] < 1000 && self.scores[1] < 1000
    }

    fn low_score(&self) -> usize {
        *self.scores.iter().min().unwrap()
    }
}


fn run() -> Result<(),InputError> {
    let starts = read_dice_game_file()?;
    println!("starts: ({},{})", starts[0], starts[1]);

    let mut game = GameState::new(starts);
    let mut player = 0;
    while game.game_not_over() {
        game.take_turn(player);
        player = (player + 1) % 2;
    }
    println!("After the game we had {} low score and {} rolls for a total of {}",
        game.low_score(),
        game.die.times_rolled,
        game.low_score() * game.die.times_rolled
    );

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
