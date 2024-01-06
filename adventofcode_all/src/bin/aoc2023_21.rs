use std::fmt::Debug;
use anyhow;
use advent_lib::grid::{Coord, Grid};
use advent_lib::asciienum::AsciiEnum;


// ======= Constants =======


// ======= Parsing =======

// FIXME: I really wanted to just combine advent_lib::grid::Grid with
//   advent_lib::asciienum::AsciiEnum and build a simple Grid of AsciiEnums. But they don't
//   work. It stems from a complaint that anyhow::Error (which asciiEnum returns when you
//   fail to convert a char into it) does not implement std::error::Error (and WHY NOT?!!).
//   I spent some time and failed to be able to fix that. So I'm writing the parsing more
//   by-hand than I should.


AsciiEnum!{
    enum Spot {
        Open('.'),
        Rock('#'),
    }
}

#[derive(Debug)]
pub struct Garden {
    grid: Grid<Spot>,
    start: Coord,
}

type Input = Garden;



mod parse {
    use super::{Input, Garden, Spot, Coord, Grid};
    use std::fs;
    use anyhow::anyhow;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_21.txt")?;
        // FIXME: It's inefficient to create square_grid then throw it away. But I'm doing it.
        assert!(s.len() > 1);
        let mut square_grid: Vec<Vec<Spot>> = Vec::new();
        let mut start_opt: Option<Coord> = None;
        for (y, line) in s.lines().enumerate() {
            let mut grid_row: Vec<Spot> = Vec::new();
            for (x, c) in line.chars().enumerate() {
                let item = match c {
                    '.' => Spot::Open,
                    '#' => Spot::Rock,
                    'S' => {
                        if start_opt.is_some() {
                            return Err(anyhow!("multiple start locations"));
                        }
                        start_opt = Some(Coord(x,y));
                        Spot::Open
                    }
                    _ => panic!("unexpected character '{}'", c),
                };
                grid_row.push(item);
            }
            square_grid.push(grid_row)
        }

        if start_opt.is_none() {
            return Err(anyhow!("No starting location"));
        }
        let start = start_opt.unwrap();
        let grid: Grid<Spot> = (square_grid).try_into()?;
        Ok(Garden{grid, start})
    }

}


// ======= Compute =======



// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    println!("Input was {:?}", input);
}


fn part_b(_input: &Input) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
