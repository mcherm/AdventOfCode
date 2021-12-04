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

#[derive(Debug)]
struct BingoCard {
    cells: [[u8; BOARD_SIZE]; BOARD_SIZE],
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
        cards.push(BingoCard{cells});
    }

    // --- Return Result ---
    Ok((draws, cards))
}



pub fn main() {
    match read_bingo_file() {
        Ok((draws, cards)) => {
            println!("Draws: {:#?}", draws);
            println!("Cards: {:#?}", cards);
        },
        Err(err) => println!("Error: {}", err),
    }
}
