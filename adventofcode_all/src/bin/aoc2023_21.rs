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

impl Default for Spot {
    fn default() -> Self {
        Spot::Open
    }
}


/// A calculation of the number of steps needed to reach the locations in the grid from a
/// given start location. If a location cannot be reached then it stores None.
#[derive(Debug)]
struct DistanceGrid {
    #[allow(dead_code)] // we only have it so it'll show up in the Display
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


/// Calculates Ciel(a/b) (assuming there is no overflow)
fn ceiling_divide(a: usize, b: usize) -> usize {
    (a + b - 1 ) / b
}


/// An enum representing the various patterns of being filled in that we will see
/// in various plots (assuming travel is unimpeded).
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum PlotPosition {
    NCorner, ECorner, SCorner, WCorner,
    NEOuter, NEInner, SEOuter, SEInner,
    SWOuter, SWInner, NWOuter, NWInner,
    PerfectCenter, OffsetCenter,
}

/// Gives half of n, rounded down.
fn floor_half(n: usize) -> usize {
    n / 2
}

/// Gives half of n, rounded up.
fn ciel_half(n: usize) -> usize {
    (n + 1) / 2
}

fn squared(n: usize) -> usize {
    n * n
}

use PlotPosition::*;
impl PlotPosition {
    /// a list of all PlotPositions (in a standard order).
    const ALL: [Self; 14] = [
        NCorner,
        ECorner,
        SCorner,
        WCorner,
        NEOuter,
        NEInner,
        SEOuter,
        SEInner,
        SWOuter,
        SWInner,
        NWOuter,
        NWInner,
        PerfectCenter,
        OffsetCenter,
    ];

    /// Converts a PlotPosition to an index.
    fn idx(&self) -> usize {
        Self::ALL.iter().position(|pp| pp == self).unwrap()
    }

    /// Converts an index into a PlotPosition.
    fn from_idx(n: usize) -> Option<Self> {
        Self::ALL.get(n).copied()
    }

    /// Gives a canonical location in a (-2 ..= +2) range of plots where we can find
    /// an example of a plot for each PlotPosition.
    fn place_in_5x5(&self) -> (i32, i32) {
        match self {
            NCorner => (0,-2),
            ECorner => (2,0),
            SCorner => (0,2),
            WCorner => (-2,0),
            NEOuter => (1,-2),
            NEInner => (1,-1),
            SEOuter => (2,1),
            SEInner => (1,1),
            SWOuter => (-1,2),
            SWInner => (-1,1),
            NWOuter => (-2,-1),
            NWInner => (-1,-1),
            PerfectCenter => (0, 0),
            OffsetCenter => (1, 0),
        }
    }

    /// Tells how many times the given PlotPosition will occur in a MegaGrid where
    /// the "middle radius" ("rm") is the given value.
    fn times_appearing(&self, rm: usize) -> usize {
        match self {
            NCorner => 1,
            ECorner => 1,
            SCorner => 1,
            WCorner => 1,
            NEOuter => rm + 1,
            NEInner => rm,
            SEOuter => rm + 1,
            SEInner => rm,
            SWOuter => rm + 1,
            SWInner => rm,
            NWOuter => rm + 1,
            NWInner => rm,
            PerfectCenter => squared(2 * floor_half(rm) + 1),
            OffsetCenter => squared(2 * ciel_half(rm)),
        }
    }

    /// Returns true for the center ones, false for all edges.
    fn is_center(&self) -> bool {
        match self {
            NCorner | ECorner | SCorner | WCorner |
            NEOuter | NEInner | SEOuter | SEInner |
            SWOuter | SWInner | NWOuter | NWInner => false,
            PerfectCenter |  OffsetCenter => true,
        }
    }
}

struct PlotPositionCounts([usize; PlotPosition::ALL.len()]);

impl PlotPositionCounts {
    /// Get the count for a PlotPosition.
    fn get(&self, pp: PlotPosition) -> usize {
        *self.0.get(pp.idx()).unwrap()
    }

    /// Constructor, from a function that returns the count.
    fn new<F: FnMut(PlotPosition) -> usize>(mut f: F) -> Self {
        Self(core::array::from_fn(|i| f(PlotPosition::from_idx(i).unwrap())))
    }
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

    /// Given a number of steps, this determines the exact answer the HARD way -- by simply
    /// counting them. It won't scale to the full-size problem, but it IS useful for testing
    /// my better algorithms.
    fn slow_solve(&self, num_steps: usize) -> usize {
        let garden_x = self.garden.grid.bound().x();
        let garden_y = self.garden.grid.bound().y();
        let plot_x_radius = ceiling_divide(num_steps, garden_x);
        let plot_y_radius = ceiling_divide(num_steps, garden_y);
        let giant_x_bound = garden_x * (plot_x_radius * 2 + 1);
        let giant_y_bound = garden_y * (plot_y_radius * 2 + 1);
        let giant_bound = Coord(giant_x_bound, giant_y_bound);
        let giant_spots: Grid<Spot> = Grid::from_function(giant_bound, |c| {
            *self.garden.grid.get(Coord(c.x() % garden_x, c.y() % garden_y))
        });
        let giant_start_x = (garden_x * plot_x_radius) + self.garden.start.x();
        let giant_start_y = (garden_y * plot_y_radius) + self.garden.start.y();
        let giant_start = Coord(giant_start_x, giant_start_y);
        let giant_garden = Garden{grid: giant_spots, start: giant_start};
        count_reachable_sites(&giant_garden, num_steps)
    }


    /// Given a number of steps, this finds the Plot Position Counts for them.
    fn plot_count(&self, num_steps: usize) -> PlotPositionCounts {
        let garden_size = self.garden.grid.bound().x();
        let rc = self.garden.start.x() + 1;
        let rm = (num_steps - rc) / garden_size;
        let giant_bound = Coord(garden_size * 5, garden_size * 5);
        let giant_spots: Grid<Spot> = Grid::from_function(giant_bound, |c| {
            *self.garden.grid.get(Coord(c.x() % garden_size, c.y() % garden_size))
        });
        let giant_start_num = (garden_size * 2) + self.garden.start.x();
        let giant_start = Coord(giant_start_num, giant_start_num);
        let giant_garden = Garden{grid: giant_spots, start: giant_start};
        let giant_dist = DistanceGrid::from_spots(&giant_garden.grid, giant_garden.start);
        let count_plot = |pp: PlotPosition| {
            let (plot_x, plot_y) = pp.place_in_5x5();
            let reduced_steps = if pp.is_center() { // for edges, bring them in. For center, don't.
                num_steps
            } else {
                num_steps - (rm - 1) * garden_size
            };
            let mut count: usize = 0;
            for y in 0..garden_size {
                for x in 0..garden_size {
                    let giant_x = ((plot_x + 2) as usize) * garden_size + x;
                    let giant_y = ((plot_y + 2) as usize) * garden_size + y;
                    let giant_coord = Coord(giant_x, giant_y);
                    let is_reachable = match giant_dist.dist.get(giant_coord) {
                        None => false,
                        Some(n) => {
                            *n % 2 == reduced_steps % 2 && *n <= reduced_steps
                        }
                    };
                    if is_reachable {
                        count += 1;
                    }
                }
            }
            count
        };
        PlotPositionCounts::new(count_plot)
    }

    /// This solves the problem using the "fast" method. It does NOT check to see whether
    /// the "fast" method is applicable.
    ///
    /// The "fast" method is this: find the number of blocks going north, south, east, or
    /// west, outside the central one that we pass FULLY through. Call that "rm", the "middle
    /// radius".
    fn fast_solve(&self, num_steps: usize) -> usize {
        assert!(self.garden.is_square() && self.garden.is_centered());
        let garden_size = self.garden.grid.bound().x();
        let rc = self.garden.start.x() + 1;
        let rm = (num_steps - rc) / garden_size;
        let re = num_steps - rc - rm * garden_size + 1;
        // println!("rc: {}, rm: {}, re: {}", rc, rm, re); // FIXME: Remove
        assert!(rm >= 1);
        assert!(re >= (garden_size + 1) / 2);
        let plot_position_counts = self.plot_count(num_steps);
        PlotPosition::ALL.iter().map(|pp| {
            // println!("    {:?} appears {} times and has {} count", pp, pp.times_appearing(rm), plot_position_counts.get(*pp)); // FIXME: Remove
            pp.times_appearing(rm) * plot_position_counts.get(*pp)
        }).sum()
    }

    /// This performs a "fast_solve" if possible and a "slow_solve" if the "fast_solve" isn't
    /// possible.
    fn smart_solve(&self, num_steps: usize) -> usize {
        // FIXME: It might be possible to improve the fast_solve() so it would work in cases
        //   where the garen wasn't square or wasn't centered, but those would add so much
        //   complexity to the logic that I'm not going to bother to support them. (In practice,
        //   the grids given out are MUCH simpler and can be solved without everything we do
        //   here to handle possible rocks on the "edges".)
        let satisfies = self.garden.is_square() && self.garden.is_centered() && self.garden.is_unimpeded();
        let garden_size = self.garden.grid.bound().x();
        let rc = self.garden.start.x() + 1;
        let rm = (num_steps - rc) / garden_size;
        let re = num_steps - rc - rm * garden_size + 1;
        let big_enough = rm >= 1;
        let outer_shaping = re >= (garden_size + 1) / 2;
        if satisfies && big_enough && outer_shaping { // FIXME: I don't want "outer_shaping" to be part of the condition
            self.fast_solve(num_steps)
        } else {
            self.slow_solve(num_steps)
        }
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

/// A wrapper struct to provide a fancier display.
struct MegaDist<'a>(&'a MegaGarden<'a>, usize);

impl<'a> Display for MegaDist<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const PLOT_RADIUS: usize = 2;
        const NUM_WIDTH: usize = 2; // the number of characters wide each number is printed

        // -- create a helper function for lines between plots --
        fn write_border_line(bound: Coord, f: &mut Formatter<'_>) -> std::fmt::Result {
            for _plot_x in 0 .. (PLOT_RADIUS * 2 + 1) {
                write!(f, "[]")?;
                for _x in 0..bound.x() {
                    write!(f, "{}", "X".repeat(NUM_WIDTH + 1))?;
                }
            }
            writeln!(f, "[]")
        }

        // -- create distances --
        let bound = self.0.garden.grid.bound();
        let garden_x = bound.x();
        let garden_y = bound.y();
        let giant_x_bound = garden_x * (PLOT_RADIUS * 2 + 1);
        let giant_y_bound = garden_y * (PLOT_RADIUS * 2 + 1);
        let giant_bound = Coord(giant_x_bound, giant_y_bound);
        let giant_spots: Grid<Spot> = Grid::from_function(giant_bound, |c| {
            *self.0.garden.grid.get(Coord(c.x() % garden_x, c.y() % garden_y))
        });
        let giant_start_x = (garden_x * PLOT_RADIUS) + self.0.garden.start.x();
        let giant_start_y = (garden_y * PLOT_RADIUS) + self.0.garden.start.y();
        let giant_start = Coord(giant_start_x, giant_start_y);
        let giant_dist = DistanceGrid::from_spots(&giant_spots, giant_start);

        // -- draw it all --
        let steps = self.1;
        for plot_y in 0 .. (PLOT_RADIUS * 2 + 1) {
            write_border_line(bound, f)?;
            for y in 0..bound.y() {
                for plot_x in 0 .. (PLOT_RADIUS * 2 + 1) {
                    write!(f, " X")?;
                    for x in 0..bound.x() {
                        let giant_coord = Coord(
                            plot_x * bound.x() + x,
                            plot_y * bound.y() + y,
                        );
                        match giant_dist.dist.get(giant_coord) {
                            None => write!(f, " {}", "#".repeat(NUM_WIDTH))?,
                            Some(n) => if n % 2 == steps % 2 {
                                write!(f, " {:1$}", n, NUM_WIDTH)?
                            } else {
                                write!(f, " {}", "-".repeat(NUM_WIDTH))?
                            },
                        };
                    }
                }
                writeln!(f, " X")?;
            }
        }
        write_border_line(bound, f)
    }
}


impl Garden {

    /// An "unimpeded" Garden is one where the distance from the center (rounded down) to
    /// each edge is the same as the distance would be if there were no rocks. I'm only
    /// going to produce a fast solution for unimpeded grids. (Mine happens to be unimpeded.)
    fn is_unimpeded(&self) -> bool {
        let bound = self.grid.bound();
        let start_row_empty  = (0..bound.x()).map(|x| Coord(x, self.start.y())).all(|c| self.grid.get(c).passable());
        let start_col_empty  = (0..bound.y()).map(|y| Coord(self.start.x(), y)).all(|c| self.grid.get(c).passable());
        let top_row_empty    = (0..bound.x()).map(|x| Coord(x, 0)             ).all(|c| self.grid.get(c).passable());
        let bottom_row_empty = (0..bound.x()).map(|x| Coord(x, bound.y() - 1) ).all(|c| self.grid.get(c).passable());
        let left_col_empty   = (0..bound.y()).map(|y| Coord(0, y)             ).all(|c| self.grid.get(c).passable());
        let right_col_empty  = (0..bound.y()).map(|y| Coord(bound.x() - 1, y) ).all(|c| self.grid.get(c).passable());
        start_row_empty && start_col_empty && top_row_empty && bottom_row_empty && left_col_empty && right_col_empty
    }

    /// Returns true if the start is in the exact center of the garden; false if not.
    fn is_centered(&self) -> bool {
        self.grid.bound().x() == self.start.x() * 2 + 1 &&
            self.grid.bound().y() == self.start.y() * 2 + 1
    }

    /// Returns true if the garden has the same width as height.
    fn is_square(&self) -> bool {
        self.grid.bound().x() == self.grid.bound().y()
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
    // let dist = DistanceGrid::from_spots(&input.grid, input.start);
    // let start = input.start;
    // let dist_if_empty = DistanceGrid{
    //     start: start,
    //     dist: Grid::from_function(input.grid.bound(), |coord| {
    //         let natural_dist = coord.x().abs_diff(start.x()) + coord.y().abs_diff(start.y());
    //         match dist.dist.get(coord) {
    //             None => None,
    //             Some(actual_dist) => {
    //                 if *actual_dist == natural_dist {None} else {Some(*actual_dist)}
    //             }
    //         }
    //     })
    // };
    // println!("{}", dist);
    // println!("----------------------");
    // println!("{}", dist_if_empty);
    // println!("----------------------");
    assert!(input.is_unimpeded());
    let steps = 28;
    let mega = MegaGarden::new(input);
    println!("MegaGarden:\n{}", MegaDist(&mega, steps));
    println!();
    let slow_count = mega.slow_solve(steps);
    let fast_count = mega.fast_solve(steps);
    println!("In exactly {} steps we can reach {} (slow count) or {} (fast count) positions.", steps, slow_count, fast_count);
    // FIXME: All we REALLY need is below this line:
    mega.smart_solve(steps);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}


#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;
    use rand::Rng;

    /// This takes a given Garden and num_steps and makes sure it gives the right answer,
    /// panicking if it doesn't.
    fn check_solution(garden: &Garden, num_steps: usize) {
        print!("Taking {} steps in a {}x{} garden", num_steps, garden.grid.bound().x(), garden.grid.bound().y());
        let mega = MegaGarden::new(garden);
        let slow_count = mega.slow_solve(num_steps);
        let fast_count = mega.fast_solve(num_steps);
        assert_eq!(slow_count, fast_count);
        println!(" gives {} locations.", slow_count);
    }

    /// This generates a random garden. It will be square, with side lengths of the size given
    /// (which must be odd). It will be centered and unimpeded. The other locations will have a
    /// probability density of being rocks (density should be from 0.0 to 1.0).
    fn random_garden(size: usize, density: f32) -> Garden {
        assert_eq!(size % 2, 1); // size must be odd
        let mut rng = rand::thread_rng();
        let bound = Coord(size,size);
        let start_val = (size - 1) / 2;
        let start = Coord(start_val, start_val);
        let rock_func = |c: Coord| {
            if c.x() == start_val || c.x() == 0 || c.x() + 1 == size {
                Spot::Open
            } else if c.y() == start_val || c.y() == 0 || c.y() + 1 == size {
                Spot::Open
            } else {
                if rng.gen::<f32>() < density {
                    Spot::Rock
                } else {
                    Spot::Open
                }
            }
        };
        let grid: Grid<Spot> = Grid::from_function(bound, rock_func);
        Garden{grid, start}
    }

    #[test]
    fn try_specific_pattern() {
        let grid: Grid<Spot> = vec![
            ".......",
            ".#.....",
            "..#....",
            ".......",
            ".##..#.",
            "..#.##.",
            ".......",
        ].iter()
            .map(|s| s.chars().map(|c| match c {'.' => Spot::Open, '#' => Spot::Rock, _ => panic!()}).collect_vec())
            .collect_vec()
            .try_into()
            .unwrap();
        assert!(grid.bound().x() == grid.bound().y());
        let start_pos = (grid.bound().x() - 1) / 2;
        let start = Coord(start_pos, start_pos);
        let garden = Garden{grid, start};
        check_solution(&garden, 21);
    }

    fn try_random_garden() {
        let mut rng = rand::thread_rng();
        let size = rng.gen_range(2..15) * 2 + 1;
        let num_steps = rng.gen_range((size * 3)..(size * 50));
        let garden = random_garden(size, 0.4);
        let outer_shaping = {
            let garden_size = garden.grid.bound().x();
            let rc = garden.start.x() + 1;
            let rm = (num_steps - rc) / garden_size;
            let re = num_steps - rc - rm * garden_size + 1;
            re >= (garden_size + 1) / 2
        };
        if outer_shaping {
            check_solution(&garden, num_steps);
        } else {
            println!("!!! Need to handle inner_shaping !!!");
        }
    }

    #[test]
    fn try_many_random_gardens() {
        let num_tests = 8;
        for _ in 0..num_tests {
            try_random_garden();
        }
    }
}
