use std::fs;
use std::io;
use std::fmt::{Display, Formatter};


/// Represents a square Life board.
#[derive(Debug, Clone)]
struct LifeBoard {
    size: usize,
    state: Vec<bool>,
}


#[derive(Debug)]
enum ReadError {
    IOErr(io::Error),
    InvalidCharacter,
    NoNewline,
    UnevenLineLength,
    NotSquare,
    FileEndsInLine,
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::IOErr(e)
    }
}

impl LifeBoard {

    /// Return the value at (x,y)
    fn val(&self, x: usize, y: usize) -> bool {
        assert!(x < self.size);
        assert!(y < self.size);
        *self.state.get(y * self.size + x).unwrap()
    }

    /// Performs a single step of animation
    fn step(&mut self) {
        let mut new_state = Vec::with_capacity(self.state.len());
        for i in 0..self.size {
            for j in 0..self.size {
                new_state.push(false);
            }
        }
        self.state = new_state;
    }

    fn parse_board(input: &str) -> Result<Self, ReadError> {
        let mut first_row: Vec<bool> = Vec::new();
        let mut chars = input.chars();
        'first_row:
        loop {
            match chars.next() {
                None => return Err(ReadError::NoNewline),
                Some(c) => match c {
                    '#' => first_row.push(true),
                    '.' => first_row.push(false),
                    '\n' => break 'first_row,
                    _   => return Err(ReadError::InvalidCharacter),
                },
            }
        }
        let size: usize = first_row.len();
        let mut state: Vec<bool> = Vec::with_capacity(size * size);
        state.extend(first_row);
        let mut row_len = 0;
        let mut row_count = 1;
        'other_rows:
        loop {
            row_len += 1;
            match chars.next() {
                None => {
                    if row_len == 1 {
                        break 'other_rows;
                    } else {
                        return Err(ReadError::FileEndsInLine);
                    }
                },
                Some(c) => match c {
                    '#' => state.push(true),
                    '.' => state.push(false),
                    '\n' => {
                        if row_len == size + 1 {
                            row_count += 1;
                            row_len = 0;
                        } else {
                            return Err(ReadError::UnevenLineLength);
                        }
                    },
                    _   => return Err(ReadError::InvalidCharacter),
                },
            }
        }
        if row_count != size {
            return Err(ReadError::NotSquare);
        }
        Ok(LifeBoard{size, state})
    }

}

impl Display for LifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.size {
            for j in 0..self.size {
                write!(f, "{}", if self.val(j,i) {'#'} else {'.'})?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}



fn input() -> Result<LifeBoard, ReadError> {
    let s = fs::read_to_string("input/2015/18/input.txt")?;
    Ok(LifeBoard::parse_board(&s)?)
}




fn part_a(life_board: &LifeBoard) -> Result<(), io::Error> {
    let mut board = life_board.clone();
    println!("We start with:");
    println!("{}", board);
    println!();
    board.step();
    println!("After one step, we have:");
    println!("{}", board);
    Ok(())
}


fn part_b(_life_board: &LifeBoard) -> Result<(), io::Error> {
    Ok(())
}

fn main() -> Result<(), ReadError> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
