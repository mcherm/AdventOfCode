use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashSet;


/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    InvalidDot,
    NoFoldSection,
    InvalidFold,
    IllegalFoldLocation,
    NoFolds,
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
            InputError::InvalidDot => write!(f, "Invalid dot."),
            InputError::NoFoldSection => write!(f, "No fold section."),
            InputError::InvalidFold => write!(f, "Invalid fold instruction."),
            InputError::IllegalFoldLocation => write!(f, "Illegal location for fold."),
            InputError::NoFolds => write!(f, "No folds given."),
        }
    }
}


/// Read in the input file.
fn read_origami_file() -> Result<OrigamiData, InputError> {
    let filename = "data/2021/day/13/input.txt";
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    let mut dots: Vec<[usize;2]> = Vec::new();
    loop {
        let text: String = lines.next().ok_or(InputError::NoFoldSection)??;
        if text.len() == 0 {
            break
        } else {
            let mut split = text.split(",");
            let x: usize = split.next().ok_or(InputError::InvalidDot)?.parse()?;
            let y: usize = split.next().ok_or(InputError::InvalidDot)?.parse()?;
            if split.next().is_some() {
                return Err(InputError::InvalidDot);
            }
            dots.push([x,y]);
        }
    }

    fn read_fold_instruction(text: String) -> Result<FoldInstruction,InputError> {
        lazy_static! {
            static ref FOLD_REGEX: Regex = Regex::new(
                r"^fold along (x|y)=(\d+)$"
            ).unwrap();
        }
        let captures: Captures = FOLD_REGEX.captures(&text).ok_or(InputError::InvalidFold)?;
        let position: usize = captures.get(2).unwrap().as_str().parse::<usize>()?;
        let direction: &str = captures.get(1).unwrap().as_str();
        match direction {
            "x" => {
                Ok(FoldInstruction::LeftAlongX(position))
            },
            "y" => {
                Ok(FoldInstruction::UpAlongY(position))
            },
            _ => panic!("Regex failed."),
        }
    }

    let mut folds: Vec<FoldInstruction> = Vec::new();
    while let Some(line) = lines.next() {
        folds.push(read_fold_instruction(line?)?)
    }
    if folds.len() == 0 {
        return Err(InputError::NoFolds);
    }

    Ok(OrigamiData{dots, folds})
}


#[derive(Debug, Copy, Clone)]
enum FoldInstruction {
    UpAlongY(usize),
    LeftAlongX(usize),
}


#[derive(Debug)]
struct OrigamiData {
    dots: Vec<[usize;2]>,
    folds: Vec<FoldInstruction>,
}



#[derive(Debug)]
struct FoldedPaper {
    grid: Vec<Vec<bool>>,
    x_size: usize,
    y_size: usize,
}

impl FoldedPaper {
    fn new(origami_data: &OrigamiData) -> Self {
        assert!(origami_data.dots.len() > 0);
        let x_size = origami_data.dots.iter().map(|d| d[0]).max().unwrap() + 1;
        let y_size = origami_data.dots.iter().map(|d| d[1]).max().unwrap() + 1;
        let mut grid = vec![vec![false; x_size]; y_size];
        for dot in origami_data.dots.iter() {
            grid[dot[1]][dot[0]] = true;
        }
        FoldedPaper{grid, x_size, y_size}
    }

    fn dot_at(&self, x: usize, y: usize) -> bool {
        self.grid[y][x]
    }

    fn _count_dots(&self) -> usize {
        self.grid.iter().map(
            |row| row.iter().map(
                |b| if *b {1} else {0}
            ).sum::<usize>()
        ).sum::<usize>()
    }

    fn fold_up(&self, position: usize) -> Result<Self,InputError> {
        println!("Folding up on {}", position); // FIXME: Remove
        if position < self.y_size / 2 {
            return Err(InputError::IllegalFoldLocation)
        };
        let x_size = self.x_size;
        let y_size = position;
        let mut grid = vec![vec![false; x_size]; y_size];
        for y in 0..y_size {
            for x in 0..x_size {
                grid[y][x] = self.grid[y][x] || self.grid[2 * position - y][x];
            }
        }
        Ok(FoldedPaper{grid, x_size, y_size})
    }

    fn fold_left(&self, position: usize) -> Result<Self,InputError> {
        println!("Folding left on {}", position); // FIXME: Remove
        if position < self.x_size / 2 {
            return Err(InputError::IllegalFoldLocation)
        };
        let x_size = position;
        let y_size = self.y_size;
        let mut grid = vec![vec![false; x_size]; y_size];
        for y in 0..y_size {
            for x in 0..x_size {
                grid[y][x] = self.grid[y][x] || self.grid[y][2 * position - x];
            }
        }
        Ok(FoldedPaper{grid, x_size, y_size})
    }

    fn fold(&self, fold_instruction: FoldInstruction) -> Result<Self,InputError> {
        match fold_instruction {
            FoldInstruction::UpAlongY(position) => {
                self.fold_up(position)
            },
            FoldInstruction::LeftAlongX(position) => {
                self.fold_left(position)
            },
        }
    }
}

impl fmt::Display for FoldedPaper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.y_size {
            write!(f, "\n")?;
            for x in 0..self.x_size {
                let char = match self.dot_at(x, y) {
                    true => "#",
                    false => ".",
                };
                write!(f, "{}", char)?;
            }
        }
        Ok(())
    }
}



#[derive(Debug)]
struct FoldedPaper2 {
    dots: HashSet<[usize;2]>,
    folds: Vec<FoldInstruction>,
}

impl FoldedPaper2 {
    fn new(origami_data: &OrigamiData) -> Self {
        let mut dots = HashSet::new();
        for dot in origami_data.dots {
            dots.insert(dot)
        }
        let folds = origami_data.folds.clone(); // This would fold it fully
        let folds = Vec::new(); // This leaves it unfolded
        FoldedPaper2{dots, folds}
    }
}

impl fmt::Display for FoldedPaper2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.y_size {
            write!(f, "\n")?;
            for x in 0..self.x_size {
                let char = match self.dot_at(x, y) {
                    true => "#",
                    false => ".",
                };
                write!(f, "{}", char)?;
            }
        }
        Ok(())
    }
}




fn run() -> Result<(), InputError> {
    let origami_data =  read_origami_file()?;
    let mut folded_paper = FoldedPaper::new(&origami_data);
    for fold_instruction in origami_data.folds {
        folded_paper = folded_paper.fold(fold_instruction)?;
    }
    println!("After folding here is the image:\n{}", folded_paper);
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
