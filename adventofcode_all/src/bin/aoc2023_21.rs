use std::fmt::{Debug, Display, Formatter};
use anyhow;
use std::collections::HashSet;
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

impl Spot {
    /// True if you can go there.
    fn passable(&self) -> bool {
        match self {
            Spot::Open => true,
            Spot::Rock => false,
        }
    }
}


/// A calculation of the number of steps needed to reach the locations in the grid from a
/// given start location. If a location cannot be reached then it stores None.
#[derive(Debug)]
struct DistanceGrid {
    #[allow(dead_code)]
    start: Coord,
    dist: Grid<Option<usize>>,
}


impl DistanceGrid {
    /// Construct a DistanceGrid for a given start location for a given set of spots.
    fn from_spots(spots: &Grid<Spot>, start: Coord) -> Self {
        let bound = spots.bound();
        let mut dist: Grid<Option<usize>> = Grid::new_default(bound);
        let mut steps = 0;
        let mut next_sites: HashSet<Coord> = HashSet::new();
        if spots.get(start).passable() {
            next_sites.insert(start);
        }
        while !next_sites.is_empty() {
            let sites: Vec<Coord> = next_sites.drain().collect();
            for site in sites {
                if *spots.get(site) == Spot::Open {
                    let d = dist.get_mut(site);
                    if d.is_none() {
                        *d = Some(steps);
                    }
                    for neighbor in site.neighbors(bound) {
                        // if it isn't a rock and we haven't been there, it's in the next step
                        if spots.get(neighbor).passable() && dist.get(neighbor).is_none() {
                            next_sites.insert(neighbor);
                        }
                    }
                }
            }
            steps += 1;
        }
        DistanceGrid{start, dist}
    }
}


impl Display for DistanceGrid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.dist.bound().y() {
            writeln!(f)?;
            for x in 0..self.dist.bound().x() {
                match self.dist.get(Coord(x,y)) {
                    None => write!(f, "|###")?,
                    Some(n) => write!(f, "|{:3}", n)?,
                };
            }
        }
        writeln!(f)
    }
}


/// This solves part 1 by counting the number of sites in that Garden that can be reached in
/// exactly the given number of steps.
fn count_reachable_sites(garden: &Garden, steps: usize) -> usize {
    let dist = DistanceGrid::from_spots(&garden.grid, garden.start);
    dist.dist.into_iter()
        .map(|d: Option<usize>| match d {
            None => 0,
            Some(s) => {
                if s <= steps && s % 2 == steps % 2 {
                    1
                } else {
                    0
                }
            }
        })
        .sum()
}


/// Represents an infinitely repeating garden and the calculations required to deal with
/// such a thing.
#[derive(Debug)]
struct MegaGarden<'a> {
    garden: &'a Garden,
}

impl<'a> MegaGarden<'a> {
    fn new(garden: &'a Garden) -> Self {
        Self{garden}
    }
}


impl<'a> Display for MegaGarden<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const PLOT_RADIUS: i32 = 1;

        for _plot_y in -PLOT_RADIUS ..= PLOT_RADIUS {
            for y in 0..self.garden.grid.bound().y() {
                for _plot_x in -PLOT_RADIUS ..= PLOT_RADIUS {
                    for x in 0..self.garden.grid.bound().x() {
                        write!(f, "{}", self.garden.grid.get(Coord(x,y)))?;
                    }
                }
                writeln!(f)?;
            }
        }
        writeln!(f)
    }
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let steps = 64;
    let count = count_reachable_sites(input, steps);
    println!("The elf can reach {} sites in exactly {} steps.", count, steps);
}


fn part_b(input: &Input) {
    println!("\nPart b:");
    let dist = DistanceGrid::from_spots(&input.grid, input.start);
    let start = input.start;
    let dist_if_empty = DistanceGrid{
        start: start,
        dist: Grid::from_function(input.grid.bound(), |coord| {
            let natural_dist = coord.x().abs_diff(start.x()) + coord.y().abs_diff(start.y());
            match dist.dist.get(coord) {
                None => None,
                Some(actual_dist) => {
                    if *actual_dist == natural_dist {None} else {Some(*actual_dist)}
                }
            }
        })
    };
    println!("{}", dist);
    println!("----------------------");
    println!("{}", dist_if_empty);
    println!("----------------------");
    let mega = MegaGarden::new(input);
    println!("{}", mega);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
