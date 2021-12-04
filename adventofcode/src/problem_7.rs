use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt;
use std::num::ParseIntError;
use std::convert::TryInto;



const BOARD_SIZE: usize = 5;


/// An error that we can encounter when reading the input.
pub enum InputError {
    IoError(std::io::Error),
    MissingDrawsLine,
    BadInt(ParseIntError),
    MissingBlankLine,
    WrongBoardSize,
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl From<ParseIntError> for InputError {
    fn from(error: ParseIntError) -> Self {
        InputError::BadInt(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err)     => write!(f, "{}", err),
            InputError::MissingDrawsLine => write!(f, "Missing draws line"),
            InputError::BadInt(err)      => write!(f, "{}", err),
            InputError::MissingBlankLine => write!(f, "Missing a blank line"),
            InputError::WrongBoardSize   => write!(f, "Wrong board size"),
        }
    }
}


type Draws = Vec<u8>;

#[derive(Debug, Copy, Clone)]
struct BingoCard {
    cells: [[u8; BOARD_SIZE]; BOARD_SIZE],
    marks: [[bool; BOARD_SIZE]; BOARD_SIZE],
    last_marked: Option<u8>,
}

impl BingoCard {

    fn new(cells: [[u8; BOARD_SIZE];BOARD_SIZE]) -> BingoCard {
        let marks = [[false; BOARD_SIZE]; BOARD_SIZE];
        let last_marked = None;
        BingoCard{cells, marks, last_marked}
    }

    fn is_winner(&self) -> bool {
        for i in 0..BOARD_SIZE {
            if self.marks[i].iter().all(|x| *x) {
                return true;
            }
            if self.marks.iter().all(|row| row[i]) {
                return true;
            }
        }
        false
    }

    fn mark(&mut self, value: u8) {
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                if self.cells[i][j] == value {
                    self.marks[i][j] = true;
                    self.last_marked = Some(value);
                }
            }
        }
    }

    fn score(&self) -> Option<u32> {
        if let Some(last_marked) = self.last_marked {
            let mut sum: u32 = 0;
            for i in 0..BOARD_SIZE {
                for j in 0..BOARD_SIZE {
                    if !self.marks[i][j] {
                        sum += self.cells[i][j] as u32;
                    }
                }
            }
            Some(sum * (last_marked as u32))
        } else {
            None
        }
    }
}


/// This reads the file containing a diagnostic report and returns it as a vector of Strings,
/// or returns an error.
fn read_bingo_file() -> Result<(Draws, Vec<BingoCard>), InputError>  {
    let filename = "data/2021/day/4/input.txt";
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    // --- Read Draws ---
    let draws_line: String = lines.next().ok_or(InputError::MissingDrawsLine)??;
    let draws_result: Result<Vec<u8>,ParseIntError> = draws_line
        .split(",")
        .map(|s| s.parse::<u8>())
        .collect();
    let draws: Vec<u8> = draws_result?;

    // --- Read Boards ---
    let mut cards = Vec::new();
    loop {
        match lines.next() {
            Some(blank_line) => {
                if blank_line? != "" {
                    return Err(InputError::MissingBlankLine)
                }
            },
            None => break,
        }
        let mut rows_vec: Vec<[u8; BOARD_SIZE]> = Vec::new();
        for _i in 0..BOARD_SIZE {
            let board_line: String = lines.next().ok_or(InputError::WrongBoardSize)??;
            let row_vec_result: Result<Vec<u8>,ParseIntError> = board_line.split_whitespace().map(|s| s.parse::<u8>()).collect();
            let row_vec: Vec<u8> = row_vec_result?;
            if row_vec.len() != BOARD_SIZE {
                return Err(InputError::WrongBoardSize)
            }
            let row: [u8; BOARD_SIZE] = row_vec.try_into().unwrap();
            rows_vec.push(row);
        }
        if rows_vec.len() != BOARD_SIZE {
            return Err(InputError::WrongBoardSize)
        }
        let cells = rows_vec.try_into().unwrap();
        cards.push(BingoCard::new(cells));
    }

    // --- Return Result ---
    Ok((draws, cards))
}


/// Marks draws from draws onto the cards until there are one or more
/// winners, then returns the winners. If there are no winners after
/// all the draws it returns an empty vector.
fn score_bingo_cards(draws: Draws, mut cards: Vec<BingoCard>) -> Vec<BingoCard> {
    fn get_winners(cards: &Vec<BingoCard>) -> Option<Vec<BingoCard>> {
        let mut winners: Vec<BingoCard> = Vec::new();
        for card in cards {
            if card.is_winner() {
                winners.push(*card);
            }
        }
        if winners.len() > 0 {
            return Some(winners);
        } else {
            return None;
        }
    }

    for value in draws {
        if let Some(winners) = get_winners(&cards) {
            return winners;
        }
        for card in &mut cards {
            card.mark(value);
        }
    }
    return Vec::new(); // no winners!
}



pub fn main() {
    match read_bingo_file() {
        Ok((draws, cards)) => {
            let winners = score_bingo_cards(draws, cards);
            if winners.len() == 0 {
                println!("There were no winners.");
            } else if winners.len() > 1 {
                println!("There were multiple winners.");
            } else {
                println!("There was a single winner with score {}", winners[0].score().unwrap());
            }
        },
        Err(err) => println!("Error: {}", err),
    }
}
