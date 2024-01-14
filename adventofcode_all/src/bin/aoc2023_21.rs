use std::fmt::{Debug, Display, Formatter};
use anyhow;
use std::collections::HashSet;
use advent_lib::grid::{Coord, Grid};
use advent_lib::asciienum::AsciiEnum;
use crate::outer::PlotPosition;


// ======= Constants =======
const PRINT_WORK: bool = false; // Set this to true and we print out the blocks we looked at and the counts of them.


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

/// Gives half of n, rounded down.
fn floor_half(n: usize) -> usize {
    n / 2
}

/// Gives half of n, rounded up.
fn ciel_half(n: usize) -> usize {
    (n + 1) / 2
}

/// Sqquare a number.
fn squared(n: usize) -> usize {
    n * n
}


/// There are two different "layouts" we might need to deal with -- an "Outer" layout
/// where all of the rm plots other than the outermost are totally filled because we
/// go at least half-way into the outermost corners, and an "Inner" layout where we
/// go less than half-way into the outermost corners, so the second-outermost plot
/// is ALSO not full. We need to create different small version of these to solve
/// the two cases -- they have different PlotPositions and everything. Oh, and there's
/// one more Layout for anything which is too small to do as one of the other layots.
/// This enum is for keeping track of these three cases.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Layout {
    Outer, Inner, TooSmall,
}


mod outer {

    /// An enum representing the various patterns of being filled in that we will see
    /// in various plots (assuming travel is unimpeded).
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum PlotPosition { // FIXME: Make 2 separate enums!
        NCorner, ECorner, SCorner, WCorner,
        NRipped, ERipped, SRipped, WRipped,
        NEOuter, NEInner, SEOuter, SEInner,
        SWOuter, SWInner, NWOuter, NWInner,
        PerfectCenter, OffsetCenter,
    }

}


impl Layout {
    /// Called with a garden that is square, centered, and unimpeded, this will select the
    /// appropriate Layout to use.
    fn select(garden: &Garden, num_steps: usize) -> Self {
        assert!(garden.is_square() && garden.is_centered() && garden.is_unimpeded());
        let garden_size = garden.grid.bound().x();
        let (_rc, rm, re) = garden.find_radii(num_steps);
        let big_enough = rm >= 1;
        let large_outer = re >= (garden_size + 1) / 2;
        if !big_enough {
            Layout::TooSmall
        } else if large_outer {
            Layout::Outer
        } else {
            Layout::Inner
        }
    }

    /// Returns the dimensions of the grid we'll build to create samples of each PlotPosition.
    /// It is different for different Layouts -- but they're 5x5 or 7x7 so we'll just return
    /// the multiplier.
    fn large_plot_dimensions(&self) -> usize {
        match self {
            Layout::Outer => 5,
            Layout::Inner => 7,
            Layout::TooSmall => panic!(),
        }
    }

    /// Returns the amount rm will be when we use the large layout.
    fn rm_when_reduced(&self) -> usize {
        (self.large_plot_dimensions() / 2) - 1
    }

    /// Returns the collection of PlotPositions that are actually used for the given layout.
    fn plot_positions(&self) -> Vec<PlotPosition> {
        use PlotPosition::*;
        match self {
            Layout::Outer => vec![
                NCorner, ECorner, SCorner, WCorner,
                NEOuter, NEInner, SEOuter, SEInner,
                SWOuter, SWInner, NWOuter, NWInner,
                PerfectCenter, OffsetCenter,
            ],
            Layout::Inner => vec![
                NCorner, ECorner, SCorner, WCorner,
                NRipped, ERipped, SRipped, WRipped,
                NEOuter, NEInner, SEOuter, SEInner,
                SWOuter, SWInner, NWOuter, NWInner,
                PerfectCenter, OffsetCenter,
            ],
            Layout::TooSmall => panic!(),
        }
    }

    /// Returns the position (offset from the center) of the plot we want to look at as a
    /// typical example of the given plot position. If given a PlotPosition not appropriate
    /// for this instance it will panic. // FIXME: There's a design here with a per-instance type of some sort. Maybe try to learn it? The panics are ugly
    fn standard_position(&self, pp: PlotPosition) -> (i32,i32) {
        use PlotPosition::*;
        match self {
            Layout::Outer => {
                match pp {
                    NCorner => (0,-2), ECorner => (2,0), SCorner => (0,2), WCorner => (-2,0),
                    NEOuter => (1,-2), SEOuter => (2,1), SWOuter => (-1,2), NWOuter => (-2,-1),
                    NEInner => (1,-1), SEInner => (1,1), SWInner => (-1,1), NWInner => (-1,-1),
                    PerfectCenter => (0, 0), OffsetCenter => (1, 0),
                    _ => panic!(),
                }
            }
            Layout::Inner => {
                match pp {
                    NCorner => (0,-3), ECorner => (3,0),  SCorner => (0,3), WCorner => (-3,0),
                    NRipped => (0,-2), ERipped => (2,0),  SRipped => (0,2), WRipped => (-2,0),
                    NEOuter => (1,-2), SEOuter => (2,1), SWOuter => (-1,2), NWOuter => (-2,-1),
                    NEInner => (1,-1), SEInner => (1,1), SWInner => (-1,1), NWInner => (-1,-1),
                    PerfectCenter => (0, 0), OffsetCenter => (1, 0),
                }
            }
            Layout::TooSmall => panic!(),
        }
    }

    /// Tells how many times the given PlotPosition will occur in a MegaGrid where
    /// the "middle radius" ("rm") is the given value.
    fn times_appearing(&self, pp: PlotPosition, rm: usize) -> usize {
        match self {
            Layout::Outer => {
                use PlotPosition::*;
                match pp {
                    NCorner => 1, ECorner => 1, SCorner => 1, WCorner => 1,
                    NEOuter => rm + 1, SEOuter => rm + 1, SWOuter => rm + 1, NWOuter => rm + 1,
                    NEInner => rm, SEInner => rm, SWInner => rm, NWInner => rm,
                    PerfectCenter => squared(2 * floor_half(rm) + 1),
                    OffsetCenter => squared(2 * ciel_half(rm)),
                    _ => panic!(),
                }
            }
            Layout::Inner => {
                assert!(rm > 1); // I assume this -- better check it always holds, and adjust min sizes to enforce it
                use PlotPosition::*;
                match pp {
                    NCorner => 1, ECorner => 1, SCorner => 1, WCorner => 1,
                    NRipped => 1, ERipped => 1, SRipped => 1, WRipped => 1,
                    NEOuter => rm, SEOuter => rm, SWOuter => rm, NWOuter => rm,
                    NEInner => rm - 1, SEInner => rm - 1, SWInner => rm - 1, NWInner => rm - 1,
                    PerfectCenter => squared(2 * floor_half(rm - 1) + 1),
                    OffsetCenter => squared(2 * ciel_half(rm - 1)),
                }
            }
            Layout::TooSmall => panic!(),
        }
    }

    /// Returns true for the center ones, false for all edges.
    fn is_center(&self, pp: PlotPosition) -> bool {
        use PlotPosition::*;
        match pp {
            NCorner | ECorner | SCorner | WCorner |
            NRipped | ERipped | SRipped | WRipped |
            NEOuter | NEInner | SEOuter | SEInner |
            SWOuter | SWInner | NWOuter | NWInner => false,
            PerfectCenter |  OffsetCenter => true,
        }
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

    /// This solves the problem using the "fast" method. It does NOT check to see whether
    /// the "fast" method is applicable.
    ///
    /// The "fast" method is this: find the number of blocks going north, south, east, or
    /// west, outside the central one that we pass FULLY through. Call that "rm", the "middle
    /// radius".
    fn fast_solve(&self, num_steps: usize, layout: Layout) -> usize {
        if layout == Layout::TooSmall {
            self.slow_solve(num_steps) // it's small enough that "slow_solve" is fast
        } else {
            let garden_size = self.garden.grid.bound().x();
            let (_rc, rm, _re) = self.garden.find_radii(num_steps);
            let large_plot_size = garden_size * layout.large_plot_dimensions();
            let large_bound = Coord(large_plot_size, large_plot_size);
            let large_spots: Grid<Spot> = Grid::from_function(large_bound, |c| {
                *self.garden.grid.get(Coord(c.x() % garden_size, c.y() % garden_size))
            });
            let large_start_num = large_plot_size / 2;
            let large_start = Coord(large_start_num, large_start_num);
            let large_garden = Garden{grid: large_spots, start: large_start};
            let large_dist = DistanceGrid::from_spots(&large_garden.grid, large_garden.start);
            let count_locations_in_plot = |pp: PlotPosition| {
                let (plot_x, plot_y) = layout.standard_position(pp);
                let reduced_steps = if layout.is_center(pp) { // for edges, bring them in. For center, don't.
                    num_steps
                } else {
                    num_steps - (rm - layout.rm_when_reduced()) * garden_size
                };
                let mut count: usize = 0;
                for y in 0..garden_size {
                    for x in 0..garden_size {
                        let center_plot_pos = (layout.large_plot_dimensions() / 2) as i32;
                        let large_x = ((plot_x + center_plot_pos) as usize) * garden_size + x;
                        let large_y = ((plot_y + center_plot_pos) as usize) * garden_size + y;
                        let large_coord = Coord(large_x, large_y);
                        let is_reachable = match large_dist.dist.get(large_coord) {
                            None => false, // rocks aren't reachable
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
            layout.plot_positions().iter()
                .map(|pp| {
                    let count = count_locations_in_plot(*pp);
                    let times_appearing = layout.times_appearing(*pp, rm);
                    if PRINT_WORK {
                        println!("With layout {:?}, {:?} has a count of {} and appears {} times.", layout, pp, count, times_appearing);
                    }
                    count * times_appearing
                })
                .sum()
        }
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
        if satisfies {
            let layout = Layout::select(self.garden, num_steps);
            self.fast_solve(num_steps, layout)
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
struct MegaDist<'a> {
    mega_garden: MegaGarden<'a>,
    num_steps: usize, // needed only to check odds vs evens
    large_dimensions: usize
}

impl<'a> MegaDist<'a> {
    fn new(garden: &'a Garden, num_steps: usize, large_dimensions: usize) -> Self {
        let mega_garden = MegaGarden::new(garden);
        MegaDist{mega_garden: mega_garden, num_steps, large_dimensions}
    }
}


impl<'a> Display for MegaDist<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const NUM_WIDTH: usize = 2; // the number of characters wide each number is printed

        // -- create a helper function for lines between plots --
        fn write_border_line(mega_dist: &MegaDist, f: &mut Formatter<'_>) -> std::fmt::Result {
            let bound = mega_dist.mega_garden.garden.grid.bound();
            for _plot_x in 0 .. mega_dist.large_dimensions {
                write!(f, "[]")?;
                for _x in 0..bound.x() {
                    write!(f, "{}", "X".repeat(NUM_WIDTH + 1))?;
                }
            }
            writeln!(f, "[]")
        }

        // -- create distances --
        let bound = self.mega_garden.garden.grid.bound();
        let garden_x = bound.x();
        let garden_y = bound.y();
        let giant_x_bound = garden_x * self.large_dimensions;
        let giant_y_bound = garden_y * self.large_dimensions;
        let giant_bound = Coord(giant_x_bound, giant_y_bound);
        let giant_spots: Grid<Spot> = Grid::from_function(giant_bound, |c| {
            *self.mega_garden.garden.grid.get(Coord(c.x() % garden_x, c.y() % garden_y))
        });
        let giant_start_x = (garden_x * self.large_dimensions) / 2;
        let giant_start_y = (garden_y * self.large_dimensions) / 2;
        let giant_start = Coord(giant_start_x, giant_start_y);
        let giant_dist = DistanceGrid::from_spots(&giant_spots, giant_start);

        // -- draw it all --
        for plot_y in 0 .. self.large_dimensions {
            write_border_line(self, f)?;
            for y in 0..bound.y() {
                for plot_x in 0 .. self.large_dimensions {
                    write!(f, " X")?;
                    for x in 0..bound.x() {
                        let giant_coord = Coord(
                            plot_x * bound.x() + x,
                            plot_y * bound.y() + y,
                        );
                        match giant_dist.dist.get(giant_coord) {
                            None => write!(f, " {}", "#".repeat(NUM_WIDTH))?,
                            Some(n) => if n % 2 == self.num_steps % 2 {
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
        write_border_line(self, f)
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

    /// This returns the radii: rc, rm, and re (in that order). If you walk from the start
    /// point (in the center) to a far corner, then rc will be the number of locations
    /// (including the starting one) that are located in the starting, central garden;
    /// rm is the number of entire plots that we pass through, and re is the number of
    /// locations (including the ending one) that are located in the "edge" plot. Note
    /// that this equation holds: "rc + garden_size * rm + re = num_steps + 1".
    ///
    /// This ONLY works for square, centered gardens.
    fn find_radii(&self, num_steps: usize) -> (usize, usize, usize) {
        assert!(self.is_square() && self.is_centered());
        let garden_size = self.grid.bound().x();
        let rc = self.start.x() + 1;
        let rm = (num_steps - rc) / garden_size;
        let re = num_steps - rc - rm * garden_size + 1;
        (rc, rm, re)
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
    let steps = 28;
    let mega = MegaGarden::new(input);
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
        let layout = Layout::select(garden, num_steps);
        let fast_count = mega.fast_solve(num_steps, layout);
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
    fn try_specific_pattern_1() {
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
        let steps = 21;
        if PRINT_WORK {
            println!("{}", MegaDist::new(&garden, steps, 5));
        }
        check_solution(&garden, steps);
    }

    #[test]
    fn try_specific_pattern_2() {
        let grid: Grid<Spot> = vec![
            ".....",
            ".#.#.",
            ".....",
            "...#.",
            ".....",
        ].iter()
            .map(|s| s.chars().map(|c| match c {'.' => Spot::Open, '#' => Spot::Rock, _ => panic!()}).collect_vec())
            .collect_vec()
            .try_into()
            .unwrap();
        assert!(grid.bound().x() == grid.bound().y());
        let start_pos = grid.bound().x() / 2;
        let start = Coord(start_pos, start_pos);
        let garden = Garden{grid, start};
        let steps = 13;
        if PRINT_WORK {
            println!("{}", MegaDist::new(&garden, steps, 7));
        }
        check_solution(&garden, steps);
    }

    fn try_random_garden() {
        let mut rng = rand::thread_rng();
        let size = rng.gen_range(2..15) * 2 + 1; // odd numbers, 3 to 31
        let num_steps = rng.gen_range(size..(size * 50));
        let garden = random_garden(size, 0.4);
        check_solution(&garden, num_steps);
    }

    #[test]
    fn try_many_random_gardens() {
        let num_tests = 8;
        for _ in 0..num_tests {
            try_random_garden();
        }
    }
}
