
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


#[derive(Debug)]
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


/// Returns true if this grid_num is a part number (is adjacent to any non-., non-digit) on
/// the given grid.
fn is_part_num(grid: &Grid, grid_num: &GridNum) -> bool {
    fn is_symbol(x: char) -> bool {
        (!x.is_digit(10)) && (x != '.')
    }
    let grid_has_symbol = |x: usize, y: usize| is_symbol(grid[y][x]);
    for x in grid_num.start_col .. grid_num.end_col {
        if grid_has_symbol(x, grid_num.row - 1) {
            return true;
        }
        if grid_has_symbol(x, grid_num.row + 1) {
            return true;
        }
    }
    for x in [grid_num.start_col - 1, grid_num.end_col] {
        if grid_has_symbol(x, grid_num.row - 1) {
            return true;
        }
        if grid_has_symbol(x, grid_num.row) {
            return true;
        }
        if grid_has_symbol(x, grid_num.row + 1) {
            return true;
        }
    }
    false
}


// ======= main() =======


fn part_a(data: &Grid) {
    println!("\nPart a:");
    let safe_grid = surround_with_dots(data);
    print_grid(&safe_grid);
    let parts_sum: u32 = find_nums(&safe_grid)
        .iter()
        .filter(|grid_num| is_part_num(&safe_grid, grid_num))
        .map(|grid_num| grid_num.val)
        .sum();
    println!("parts sum: {}", parts_sum);
}


fn part_b(_data: &Grid) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
