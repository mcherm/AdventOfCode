use std::fs;
use std::io;
use std::fmt::{Display, Formatter};
use std::collections::HashMap;


/// Represents a square Life board.
#[derive(Debug, Clone)]
struct LifeBoard {
    size: usize,
    state: Vec<bool>,
    locked: HashMap<(usize,usize),bool>,
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
        let coord: (usize,usize) = (x,y);
        *self.locked.get(&coord).unwrap_or_else(
            || self.state.get(y * self.size + x).unwrap()
        )
    }

    /// Returns the number of on locations
    fn count(&self) -> usize {
        let mut answer = 0;
        for y in 0..self.size {
            for x in 0..self.size {
                if self.val(x,y) {
                    answer += 1;
                }
            }
        }
        answer
    }

    /// Makes the 4 corners locked on
    fn lock_corners_on(&mut self) {
        let max = self.size - 1;
        self.locked.insert((0,0), true);
        self.locked.insert((0,max), true);
        self.locked.insert((max,0), true);
        self.locked.insert((max,max), true);
    }

    /// Returns the neighbors of (x,y), with false for any
    /// neighbors that would be off the edge of the board.
    #[allow(dead_code)]
    fn neighbors(&self, x: usize, y: usize) -> [bool;8] {
        let max = self.size - 1;
        [
            if x==0 || y==0     {false} else {self.val(x-1, y-1)},
            if y==0             {false} else {self.val( x,  y-1)},
            if x==max || y==0   {false} else {self.val(x+1, y-1)},
            if x==0             {false} else {self.val(x-1,  y )},
            if x==max           {false} else {self.val(x+1,  y )},
            if x==0 || y==max   {false} else {self.val(x-1, y+1)},
            if y==max           {false} else {self.val( x,  y+1)},
            if x==max || y==max {false} else {self.val(x+1, y+1)},
        ]
    }

    /// Returns the neighbors of (x,y), with false for any
    /// neighbors that would be off the edge of the board.
    fn num_on_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut answer = 0;
        let max = self.size - 1;
        if x!=0   && y!=0   && self.val(x-1, y-1) {answer += 1};
        if           y!=0   && self.val( x,  y-1) {answer += 1};
        if x!=max && y!=0   && self.val(x+1, y-1) {answer += 1};
        if x!=0             && self.val(x-1,  y ) {answer += 1};
        if x!=max           && self.val(x+1,  y ) {answer += 1};
        if x!=0 && y!=max   && self.val(x-1, y+1) {answer += 1};
        if y!=max           && self.val( x,  y+1) {answer += 1};
        if x!=max && y!=max && self.val(x+1, y+1) {answer += 1};
        answer
    }

    /// Performs a single step of animation
    fn step(&mut self) {
        let mut new_state = Vec::with_capacity(self.state.len());
        for y in 0..self.size {
            for x in 0..self.size {
                let is_on = self.val(x,y);
                let neighbors_on = self.num_on_neighbors(x,y);
                new_state.push(
                    if is_on {
                        neighbors_on == 2 || neighbors_on == 3
                    } else {
                        neighbors_on == 3
                    }
                );
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
        let locked: HashMap<(usize,usize),bool> = HashMap::new();
        Ok(LifeBoard{size, state, locked})
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
    const NUM_STEPS: usize = 100;
    for _ in 0..NUM_STEPS {
        board.step();
    }
    println!("After {} steps, we have:", NUM_STEPS);
    println!("{}", board);
    println!();
    println!("Which has {} lights on.", board.count());
    Ok(())
}


fn part_b(life_board: &LifeBoard) -> Result<(), io::Error> {
    let mut board = life_board.clone();
    board.lock_corners_on();
    println!("Locking the corners, we start with:");
    println!("{}", board);
    println!();
    const NUM_STEPS: usize = 100;
    for _ in 0..NUM_STEPS {
        board.step();
    }
    println!("After {} steps, we have:", NUM_STEPS);
    println!("{}", board);
    println!();
    println!("Which has {} lights on.", board.count());
    Ok(())
}

fn main() -> Result<(), ReadError> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
