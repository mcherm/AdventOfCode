use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashSet;

#[allow(dead_code)]
mod single_linked_list {
    // Source: https://rust-unofficial.github.io/too-many-lists/third-final.html
    use std::rc::Rc;

    pub struct List<T> {
        head: Link<T>,
    }

    type Link<T> = Option<Rc<Node<T>>>;

    struct Node<T> {
        elem: T,
        next: Link<T>,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            List { head: None }
        }

        pub fn is_empty(&self) -> bool {
            match self.head {
                None => true,
                Some(_) => false,
            }
        }

        pub fn prepend(&self, elem: T) -> List<T> {
            List { head: Some(Rc::new(Node {
                elem: elem,
                next: self.head.clone(),
            }))}
        }

        pub fn tail(&self) -> List<T> {
            List { head: self.head.as_ref().and_then(|node| node.next.clone()) }
        }

        pub fn head(&self) -> Option<&T> {
            self.head.as_ref().map(|node| &node.elem)
        }

        pub fn iter(&self) -> Iter<'_, T> {
            Iter { next: self.head.as_deref() }
        }
    }

    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            let mut head = self.head.take();
            while let Some(node) = head {
                if let Ok(mut node) = Rc::try_unwrap(node) {
                    head = node.next.take();
                } else {
                    break;
                }
            }
        }
    }

    pub struct Iter<'a, T> {
        next: Option<&'a Node<T>>,
    }

    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            self.next.map(|node| {
                self.next = node.next.as_deref();
                &node.elem
            })
        }
    }
}


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



struct FoldedPaper2 {
    dots: HashSet<[usize;2]>,
    folds: single_linked_list::List<FoldInstruction>, // folds, with most recent fold FIRST
    orig_x_size: usize,
    orig_y_size: usize,
}

impl FoldedPaper2 {
    fn new(origami_data: &OrigamiData) -> Self {
        let mut dots: HashSet<[usize;2]> = HashSet::new();
        for dot in &origami_data.dots {
            dots.insert(*dot);
        }
        let orig_x_size = origami_data.dots.iter().map(|d| d[0]).max().unwrap() + 1;
        let orig_y_size = origami_data.dots.iter().map(|d| d[1]).max().unwrap() + 1;

        let mut folds: single_linked_list::List<FoldInstruction> = single_linked_list::List::new();
        for fold_instruction in &origami_data.folds {
            folds = folds.prepend(*fold_instruction)
        }
        FoldedPaper2{dots, folds, orig_x_size, orig_y_size}
    }

    fn x_size(&self) -> usize {
        for fold_instruction in self.folds.iter() {
            if let FoldInstruction::LeftAlongX(pos) = fold_instruction {
                return *pos;
            }
        }
        self.orig_x_size
    }

    fn y_size(&self) -> usize {
        for fold_instruction in self.folds.iter() {
            if let FoldInstruction::UpAlongY(pos) = fold_instruction {
                return *pos;
            }
        }
        self.orig_y_size
    }

    fn dot_at(&self, x: usize, y: usize) -> bool {

        fn has_dot(dots: &HashSet<[usize;2]>, point: &[usize;2], folds: &single_linked_list::List<FoldInstruction>) -> bool {
            match folds.head() {
                None => dots.contains(point),
                Some(fold_instruction) => {
                    let other_point: [usize;2] = match fold_instruction {
                        FoldInstruction::LeftAlongX(fold_pos) => {
                            [2 * fold_pos - point[0],point[1]]
                        },
                        FoldInstruction::UpAlongY(fold_pos) => {
                            [point[0],2 * fold_pos - point[1]]
                        },
                    };
                    let tail = folds.tail();
                    has_dot(dots, point, &tail) || has_dot(dots, &other_point, &tail)
                },
            }
        }

        has_dot(&self.dots, &[x,y], &self.folds)
    }
}

impl fmt::Display for FoldedPaper2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.y_size() {
            write!(f, "\n")?;
            for x in 0..self.x_size() {
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
    let origami_data_1 =  read_origami_file()?;
    let origami_data_2 =  read_origami_file()?;

    println!("Beginning original code...");
    let start = std::time::Instant::now();
    let mut folded_paper = FoldedPaper::new(&origami_data_1);
    for fold_instruction in origami_data_1.folds {
        folded_paper = folded_paper.fold(fold_instruction)?;
    }
    let duration = start.elapsed();
    println!("After folding here is the image:\n{}", folded_paper);
    println!("time: {:?}", duration);

    println!("\nBeginning updated algorithm...");
    let start = std::time::Instant::now();
    let folded_paper = FoldedPaper2::new(&origami_data_2);
    let duration = start.elapsed();
    println!("After folding here is the image:\n{}", folded_paper);
    println!("time: {:?}", duration);

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
