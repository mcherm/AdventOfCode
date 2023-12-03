
use anyhow;


// ======= Parsing =======

type Grid = Vec<Vec<char>>;

mod parse {
    use std::fs;
    use super::Grid;

    pub fn input<'a>() -> Result<Grid, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_03.txt")?;
        Ok(s.lines()
            .map(|line| line.chars().collect::<Vec<char>>())
            .collect::<Grid>())
    }
}

// ======= Compute =======

use std::iter::once;
use std::collections::HashMap;
use itertools::Itertools;


#[allow(dead_code)] // because it's only used for debugging
fn print_grid(grid: &Grid) {
    for row in grid {
        for val in row {
            print!("{}", val);
        }
        println!();
    }
}

fn surround_with_dots(orig: &Grid) -> Grid {
    let row_len = orig.first().unwrap().len() + 1;
    fn make_row(old_row: &Vec<char>) -> Vec<char> {
        once('.')
            .chain(old_row.iter().map(|x| *x))
            .chain(once('.')).collect()
    }
    once(vec!['.'; row_len])
        .chain(orig.iter().map(make_row))
        .chain(once(vec!['.'; row_len]))
        .collect()
}


#[derive(Debug, Clone)]
struct GridNum {
    row: usize,
    start_col: usize,
    end_col: usize, // one MORE than the actual end
    val: u32,
}


/// Given any Grid, this finds all the GridNums in it.
fn find_nums(grid: &Grid) -> Vec<GridNum> {
    let mut answer = Vec::new();
    for (row_num, row) in grid.iter().enumerate() {
        let mut col = 0;
        loop {
            if row[col].is_digit(10) {
                let start_col = col;
                loop {
                    col += 1;
                    if !row[col].is_digit(10) {
                        break;
                    }
                }
                let end_col = col;
                let val: u32 = row[start_col..end_col].iter().collect::<String>().parse().unwrap();
                answer.push(GridNum{row: row_num, start_col, end_col, val})
            }
            col += 1;
            if col >= row.len() {
                break;
            }
        }
    }
    answer
}


impl GridNum {
    /// This returns an iterator of (x,y) pairs for all the locations that are neighbors to
    /// a particular GridNum. It assumes that none of those locations are out of bounds
    /// for the grid, and if a GridNum has row=0 or start_col=0 this will panic.
    fn neighbors(&self) -> impl Iterator<Item=(usize,usize)> + '_ {
        assert!(self.row > 0);
        assert!(self.start_col > 0);
        let top =     (self.start_col..self.end_col).map(|x| (x, self.row - 1));
        let bottom =  (self.start_col..self.end_col).map(|x| (x, self.row + 1));
        let left =  ((self.row - 1)..=(self.row + 1)).map(|y| (self.start_col - 1, y));
        let right = ((self.row - 1)..=(self.row + 1)).map(|y| (self.end_col, y));
        top.chain(bottom).chain(left).chain(right)
    }
}

/// Returns true if this grid_num is a part number (is adjacent to any non-., non-digit) on
/// the given grid.
fn is_part_num(grid: &Grid, grid_num: &GridNum) -> bool {
    fn is_symbol(x: char) -> bool {
        (!x.is_digit(10)) && (x != '.')
    }
    let grid_has_symbol = |(x, y): (usize,usize)| is_symbol(grid[y][x]);
    for neighbor in grid_num.neighbors() {
        if grid_has_symbol(neighbor) {
            return true;
        }
    }
    false
}


/// Finds the GridNums in a grid, and returns it in the form of a Map to look them up by row.
fn nums_by_row(grid: &Grid) -> HashMap<usize,Vec<GridNum>> {
    // initialize the HashMap being empty
    let mut answer: HashMap<usize,Vec<GridNum>> =
        (0..grid.len()).map(|row| (row, Vec::new())).collect();
    for num in find_nums(grid) {
        // put each num in the right place in the HashMap
        answer.get_mut(&num.row).unwrap().push(num)
    }
    answer
}


/// Returns a list of (x,y) locations for the stars in a gird.
fn star_locs(grid: &Grid) -> Vec<(usize,usize)> {
    grid.iter().enumerate().map(|(y,row): (usize,&Vec<char>)| {
        row.iter().enumerate().filter_map(move |(x,val): (usize,&char)| {
            if *val == '*' {
                Some((x,y))
            } else {
                None
            }
        })
    }).flatten().collect_vec()
}


/// A Gear is 2 nums that surround a star
#[derive(Debug)]
struct Gear {
    #[allow(dead_code)] // the start_loc field isn't needed, but I'm tracking it anyhow
    star_loc: (usize, usize),
    nums: [GridNum; 2]
}


impl Gear {
    fn gear_ratio(&self) -> u32 {
        self.nums[0].val * self.nums[1].val
    }
}


fn get_gears(grid: &Grid) -> Vec<Gear> {
    let num_map = nums_by_row(grid);
    let mut answer: Vec<Gear> = Vec::new();
    for star_loc in star_locs(grid) {
        let mut adjacent_nums: Vec<&GridNum> = Vec::new();
        for row in (star_loc.1 - 1) ..= (star_loc.1 + 1) {
            for num in num_map[&row].iter() {
                for neighbor in num.neighbors() {
                    if neighbor == star_loc {
                        adjacent_nums.push(num);
                    }
                }
            }
        }
        if adjacent_nums.len() == 2 {
            // we found a gear!
            let nums = [adjacent_nums[0].clone(), adjacent_nums[1].clone()];
            answer.push(Gear{star_loc, nums})
        }
    }
    answer
}


// ======= main() =======


fn part_a(data: &Grid) {
    println!("\nPart a:");
    let safe_grid = surround_with_dots(data);
    // print_grid(&safe_grid); // NOTE: Restore this if we want to look at it
    let parts_sum: u32 = find_nums(&safe_grid)
        .iter()
        .filter(|grid_num| is_part_num(&safe_grid, grid_num))
        .map(|grid_num| grid_num.val)
        .sum();
    println!("parts sum: {}", parts_sum);
}


fn part_b(data: &Grid) {
    println!("\nPart b:");
    let safe_grid = surround_with_dots(data);
    let gears = get_gears(&safe_grid);
    let sum: u32 = gears.iter().map(|x| x.gear_ratio()).sum();
    println!("Gear Ratio Sum = {}", sum);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
