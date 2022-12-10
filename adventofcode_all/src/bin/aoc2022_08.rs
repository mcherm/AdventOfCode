
extern crate anyhow;

use std::fs;
use anyhow::anyhow;
use std;
use std::fmt::{Display, Formatter};


// ======= Parsing =======

fn input() -> Result<TreeGrid, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_08.txt")?;
    TreeGrid::parse(&s)
}


type TreeHeight = u8;
const MAX_TREE_HEIGHT: TreeHeight = 9;


#[derive(Debug, Clone)]
struct TreeGrid {
    rows: usize,
    cols: usize,
    data: Vec<TreeHeight>,
}

#[derive(Debug)]
struct BoolGrid {
    cols: usize,
    rows: usize,
    data: Vec<bool>,
    count: usize, // a count of how many are set to true
}


impl TreeGrid {
    /// Retrieves the TreeHeight at location (x,y). X and Y are zero-based, and if x >= cols
    /// or y >= rows then this will panic.
    fn get(&self, x: usize, y: usize) -> TreeHeight {
        assert!(x < self.cols);
        assert!(y < self.rows);
        *self.data.get(self.cols * y + x).unwrap()
    }

    /// Parse the input to create a TreeGrid.
    fn parse<'a>(input: &'a str) -> Result<Self, anyhow::Error> {
        let mut data = Vec::new();
        let mut cols_opt = None;
        let mut rows = 0;
        for line in input.lines() {
            rows += 1;
            if line.len() != cols_opt.unwrap_or_else(|| line.len()) {
                return Err(anyhow!("Lines do not all have the same length."));
            }
            cols_opt = Some(line.len());
            for c in line.chars() {
                let num = c.to_digit(10).ok_or(anyhow!("Not a base-10 digit."))?;
                data.push( TreeHeight::try_from(num)? );
            }
        }
        if rows == 0 {
            return Err(anyhow!("No rows in grid."));
        }
        let cols = cols_opt.unwrap();
        Ok(TreeGrid{rows, cols, data})
    }
}

impl BoolGrid {
    fn new(cols: usize, rows: usize) -> Self {
        let data = vec![false; rows * cols];
        let count = 0;
        BoolGrid{cols, rows, data, count}
    }

    fn get(&self, x: usize, y: usize) -> bool {
        assert!(x < self.cols);
        assert!(y < self.rows);
        let idx = self.cols * y + x;
        return self.data[idx]
    }

    /// Sets the value at (x,y) to true.
    fn set(&mut self, x: usize, y: usize) {
        assert!(x < self.cols);
        assert!(y < self.rows);
        let idx = self.cols * y + x;
        if !self.data[idx] {
            self.data[idx] = true;
            self.count += 1;
        }
    }

    fn get_count(&self) -> usize {
        self.count
    }
}

impl Display for BoolGrid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.rows {
            for x in 0..self.cols {
                let c = if self.get(x,y) {'*'} else {'.'};
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}



/// This is given a TreeGrid and a particular column. It runs down that column
/// and finds the first location of at least that particular height. It then
/// returns a Vec of the unique values.
fn find_first_of_each_height(tree_grid: &TreeGrid, x: usize, backward: bool, swap_args: bool) -> Vec<usize> {
    let mut answer = Vec::new();
    let mut next_height = 0;
    let lim_y = if swap_args {tree_grid.cols} else {tree_grid.rows};
    for y_index in 0..lim_y {
        let y = if backward {lim_y - y_index - 1} else {y_index};
        let h = if swap_args {tree_grid.get(y,x)} else {tree_grid.get(x,y)};
        if h >= next_height {
            answer.push(y);
            if h < MAX_TREE_HEIGHT {
                next_height = h + 1
            } else {
                break;
            }
        }
    }
    answer
}

/// Finds a BoolGrid of all the visible trees.
///
/// NOTE: I have tried to optimize the algorithm here. I am NOT
/// checking every cell in the grid -- in fact,  I think the approach
/// here is nearly optimal.
fn find_visible(tree_grid: &TreeGrid) -> BoolGrid {
    let mut visible = BoolGrid::new(tree_grid.cols, tree_grid.rows);
    for x in 0..tree_grid.cols {
        find_first_of_each_height(tree_grid, x, false, false).iter()
            .for_each(|y| visible.set(x,*y));
        find_first_of_each_height(tree_grid, x, true, false).iter()
            .for_each(|y| visible.set(x,*y));
    }
    for y in 0..tree_grid.rows {
        find_first_of_each_height(tree_grid, y, false, true).iter()
            .for_each(|x| visible.set(*x,y));
        find_first_of_each_height(tree_grid, y, true, true).iter()
            .for_each(|x| visible.set(*x,y));
    }
    visible
}

enum Direction { N, S, E, W }

impl Direction {
    /// Given x, returns the new x if stepping in direction self. Panics if out of range.
    fn step_x(&self, x: usize) -> usize {
        match self { Direction::N => x, Direction::S => x, Direction::E => x + 1, Direction::W => x - 1 }
    }

    /// Given x, returns the new x if stepping in direction self. Panics if out of range.
    fn step_y(&self, y: usize) -> usize {
        match self { Direction::N => y - 1, Direction::S => y + 1, Direction::E => y, Direction::W => y }
    }

    /// If moving in direction self, on tree_grid and you are now at (legal) position (x,y), then
    /// you can't take any more steps or you'll run off the grid.
    fn stop_here(&self, tree_grid: &TreeGrid, x: usize, y: usize) -> bool {
        match self {
            Direction::N => y == 0,
            Direction::S => y == tree_grid.rows - 1,
            Direction::E => x == tree_grid.cols - 1,
            Direction::W => x == 0,
        }
    }
}

/// Finds the view_distance from (start_x, start_y) in the given direction.
fn find_view_distance(tree_grid: &TreeGrid, start_x: usize, start_y: usize, dir: Direction) -> usize {
    let (mut x, mut y) = (start_x, start_y);
    let h = tree_grid.get(x,y);
    let mut steps = 0;
    while ! dir.stop_here(tree_grid, x, y) {
        steps += 1;
        x = dir.step_x(x);
        y = dir.step_y(y);
        if tree_grid.get(x,y) >= h {
            return steps
        }
    }
    steps
}

/// Finds the "scenic score" for the tree at location (x,y).
///
/// NOTE: Unlike in part A, I don't have any cleverness in the algorithm;
/// just brute force.
fn find_scenic_score(tree_grid: &TreeGrid, x: usize, y: usize) -> usize {
    let mut score = 1;
    for dir in [Direction::N, Direction::S, Direction::E, Direction::W] {
        score *= find_view_distance(tree_grid, x, y, dir);
    }
    score
}


// ======= main() =======

fn part_a(tree_grid: &TreeGrid) {
    println!("\nPart a:");
    let visible = find_visible(tree_grid);
    println!("Visible: \n{}", visible);
    println!("The count of visible trees is {}", visible.get_count());
}


fn part_b(tree_grid: &TreeGrid) {
    println!("\nPart b:");
    let mut max_score = 0;
    for y in 0..tree_grid.rows {
        for x in 0..tree_grid.cols {
            let score = find_scenic_score(tree_grid, x, y);
            if score > max_score {
                max_score = score;
            }
            print!("{:5} ", score);
        }
        println!();
    }
    println!("The max score is {}", max_score);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
