use std::fmt::{Display, Formatter};
use std::collections::{HashSet, HashMap};
use std::cmp::{max, min};
use std::hash::Hash;
use anyhow;
use itertools::Itertools;
use advent_lib::grid::{Direction, Coord, Grid};


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct DigDir(Direction);

#[derive(Debug, Copy, Clone)]
pub struct DigStep {
    dig_dir: DigDir,
    dist: u32,
}


impl DigDir {
    /// Creates a DigDir from the given character, or panics if it's an invalid character.
    fn from_char(c: char) -> Self {
        use Direction::*;
        DigDir(match c {
            'R' => East,
            'D' => South,
            'L' => West,
            'U' => North,
            _ => panic!("invalid DigDir '{}'", c)
        })
    }
}

impl Display for DigDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for DigStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.dig_dir, self.dist)
    }
}


type Input = Vec<(DigStep,DigStep)>;



mod parse {
    use super::{Input, DigDir, DigStep};
    use std::fs;
    use nom;
    use nom::IResult;
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_18.txt")?;
        match DigStep::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    impl DigDir {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::character::complete::one_of("UDLR"),
                |c: char| DigDir::from_char(c)
            )(input)
        }

        /// Reads the coded version, using 0..3
        fn parse_coded(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::character::complete::one_of("0123"),
                |c: char| DigDir::from_char(match c {
                    '0' => 'R', '1' => 'D', '2' => 'L', '3' => 'U', _ => panic!()
                })
            )(input)
        }
    }

    impl DigStep {
        fn parse_normal(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    DigDir::parse,
                    nom::bytes::complete::tag(" "),
                    nom_num
                )),
                |(dig_dir, _, dist)| {
                    DigStep{dig_dir, dist}
                }
            )(input)
        }

        fn parse_coded(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom::bytes::complete::tag("(#"),
                    nom::bytes::complete::take(5usize),
                    DigDir::parse_coded,
                    nom::bytes::complete::tag(")"),
                )),
                |(_, hex, dig_dir, _)| {
                    let dist = u32::from_str_radix(hex, 16).expect("invalid hex chars");
                    DigStep{dig_dir, dist}
                }
            )(input)
        }

        fn parse_pair(input: &str) -> IResult<&str, (Self, Self)> {
            nom::combinator::map(
                nom::sequence::tuple((
                    Self::parse_normal,
                    nom::bytes::complete::tag(" "),
                    Self::parse_coded,
                )),
                |(first, _, second)| {
                    (first, second)
                }
            )(input)
        }

        fn parse_list(input: &str) -> IResult<&str, Vec<(Self,Self)>> {
            nom::multi::many1(
                nom::sequence::terminated(
                    Self::parse_pair,
                    nom::character::complete::line_ending,
                )
            )(input)
        }

    }

}


// ======= Compute =======

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum DigType {
    Trench, Middle, Edge
}

#[derive(Debug)]
struct DigGrid {
    grid: Grid<DigType>,
}


impl Default for DigType {
    fn default() -> Self { DigType::Edge }
}

impl Display for DigType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DigType::Trench => '#',
            DigType::Middle => '@',
            DigType::Edge => '.',
        })
    }
}


impl DigGrid {
    /// Given the input, finds the bounds of the grid. It returns 2 coords -- the first is
    /// the offset at which the starting location must be placed to ensure that we never
    /// go off the left and top edges, and the second is the bounds of the rectangle we
    /// will fill.
    fn find_bounds(steps: &Vec<DigStep>) -> (Coord, Coord) {
        let mut x: i64 = 0;
        let mut y: i64 = 0;
        let mut min_x: i64 = 0;
        let mut min_y: i64 = 0;
        let mut max_x: i64 = 0;
        let mut max_y: i64 = 0;
        for step in steps {
            match step.dig_dir.0 {
                Direction::East => x += step.dist as i64,
                Direction::South => y += step.dist as i64,
                Direction::West => x -= step.dist as i64,
                Direction::North => y -= step.dist as i64,
            }
            min_x = min(x, min_x);
            min_y = min(y, min_y);
            max_x = max(x, max_x);
            max_y = max(y, max_y);
        }
        assert!(min_x <= 0 && min_y <= 0);
        let offset_x: usize = (-1 * min_x) as usize;
        let offset_y: usize = (-1 * min_y) as usize;
        let bound_x: usize = (max_x - min_x) as usize + 1;
        let bound_y: usize = (max_y - min_y) as usize + 1;
        (Coord(offset_x, offset_y), Coord(bound_x, bound_y))
    }

    /// Used during construction to flood-fill the grid. This will find every spot in
    /// the grid that is labeled as "Middle" and will change all points touching it
    /// marked as "Edge" to "Middle" (without ever touching or crossing locations marked
    /// "Trench". It can't assume "Middle" is ACTUALLY "Middle", because we might have
    /// them wrong and need to swap later.
    fn flood_fill_grid(grid: &mut Grid<DigType>) {
        use DigType::*;

        // create a set of middle coords to consider
        let bound = grid.bound();
        let area = bound.x() * bound.y();
        let mut unprocessed_middle: HashSet<Coord> = HashSet::with_capacity(area);

        // populate the set with known middles
        for coord in grid.bound().range_by_rows() {
            if *grid.get(coord) == Middle {
                unprocessed_middle.insert(coord);
            }
        }

        // remove items from the set, adding any neighbors
        while let Some(&coord) = unprocessed_middle.iter().next() {
            unprocessed_middle.remove(&coord); // we're handling it now, so it's no longer unprocessed
            for neighbor in coord.neighbors(bound) {
                let spot = grid.get_mut(neighbor);
                if *spot == Edge {
                    *spot = Middle;
                    unprocessed_middle.insert(neighbor);
                }
            }
        }
    }

    /// Creates the grid from a given input. This will ONLY work if the length of the
    /// steps is small enough that we can actually create a grid.
    fn new(steps: &Vec<DigStep>) -> Self {

        // --- Find the bounds and make a grid ---
        let (offset, bounds) = Self::find_bounds(steps);
        let mut grid: Grid<DigType> = Grid::new_default(bounds);

        // --- Dig the trenches ---
        use DigType::*;
        let mut pos = offset;
        grid.set(pos, Trench); // dig the top-left corner

        // NOTE: We don't know whether we'll go around clockwise or counterclockwise, so
        //   we don't know which side is "inside" and which is "outside". That's OK.
        //   We will "guess" that we're going clockwise and mark everything to the right
        //   as being "Middle". Then we'll flood-fill... but all the time being careful
        //   not to run off the edge of the grid. We *will* however, keep track of whether
        //   we ever ran off the grid. If so, then after filling it all in (wrongly) we
        //   will have a final pass to swap all "Middle" for "Edge" and vice versa.
        let mut need_to_swap = false;

        // dig_dir is the direction we're digging the trench. pos is the location we're
        // sitting in at this moment. We're guessing that clockwise is "inward", but we
        // might be wrong and will set need_to_swap if we ever encounter the edge of the
        // grid. Which will certainly happen (we touch the edge of the board in at least
        // 4 places since the grid was minimal size) if we picked wrong.
        fn fill_inward(grid: &mut Grid<DigType>, need_to_swap: &mut bool, dig_dir: DigDir, pos: Coord) {
            let inward_dir = dig_dir.0.clockwise();
            match pos.bounded_step(inward_dir, grid.bound()) {
                Some(inward_pos) => {
                    let inward_spot = grid.get_mut(inward_pos);
                    if *inward_spot == Edge {
                        *inward_spot = Middle;
                    }
                },
                None => {
                    *need_to_swap = true;
                }
            }
        }

        for step in steps {
            fill_inward(&mut grid, &mut need_to_swap, step.dig_dir, pos);
            for _ in 0..step.dist {
                pos = pos.safe_step(step.dig_dir.0);
                grid.set(pos, Trench);
                fill_inward(&mut grid, &mut need_to_swap, step.dig_dir, pos);
            }
        }
        assert_eq!(pos, offset); // Make sure the instructions bring us back to the start

        // --- Dig the middle ---
        Self::flood_fill_grid(&mut grid);

        // --- Swap "Middle" and "Edge" if needed ---
        if need_to_swap {
            for item in grid.iter_mut() {
                *item = match *item {
                    Trench => Trench,
                    Middle => Edge,
                    Edge => Middle,
                }
            }
        }

        // --- Return it ---
        Self{grid}
    }

    /// Returns the total area of the trench plus the middle.
    fn area(&self) -> usize {
        self.grid.iter().filter(|dig_type| **dig_type != DigType::Edge).count()
    }
}


impl Display for DigGrid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.grid)
    }
}


/// An object that supports mapping from indexes to items or items to indexes (efficiently
/// in either direction).
struct TwoWayIndexMap<T: Hash + Ord + Clone> {
    index_to_value: Vec<T>,
    value_to_index: HashMap<T,usize>,
}

impl<T: Hash + Ord + Clone> TwoWayIndexMap<T> {

    /// This is passed a set of values which can be sorted. It puts them in order, and numbers
    /// them 0, 1, 2, and so forth, then returns a TwoWayIndexMap which can efficiently convert
    /// either way between values and indexes.
    pub fn new(values: HashSet<T>) -> TwoWayIndexMap<T> {
        let index_to_value: Vec<T> = values.iter().sorted().cloned().collect();
        let value_to_index: HashMap<T,usize> = index_to_value.iter()
            .enumerate()
            .map(|(i,x)| (x.clone(),i))
            .collect();
        TwoWayIndexMap{index_to_value, value_to_index}
    }

    /// Convert index to value.
    pub fn to_value(&self, i: usize) -> Option<&T> {
        self.index_to_value.get(i)
    }

    /// Convert value to index.
    pub fn to_index(&self, value: &T) -> Option<usize> {
        self.value_to_index.get(value).map(|x| *x)
    }

    /// Returns the number of items that are mapped.
    #[allow(dead_code)] // This is useful when printing stuff out and debugging
    pub fn len(&self) -> usize {
        self.index_to_value.len()
    }
}




/// This determines the area for a path where the steps are too big to solve using
/// just a simple grid (because it would take too much memory).
fn giant_size_area(steps: &Vec<DigStep>) -> usize {

    // --- Find the bounds ---
    let (offset, bounds) = DigGrid::find_bounds(steps);

    // --- Find interesting rows and columns ---
    // These are the rows/columns where we dig a trench AND those one away from it.
    // The idea is that everything can be compressed to a "big" grid storing only
    // these "interesting" rows and columns.
    use Direction::*;
    let mut special_rows: HashSet<usize> = HashSet::with_capacity(steps.len());
    let mut special_cols: HashSet<usize> = HashSet::with_capacity(steps.len());
    let mut x: usize = offset.0;
    let mut y: usize = offset.1;
    for step in steps {
        match step.dig_dir.0 {
            East => x += step.dist as usize,
            South => y += step.dist as usize,
            West => x -= step.dist as usize,
            North => y -= step.dist as usize,
        }
        match step.dig_dir.0 {
            East | West => {
                special_rows.insert(y);
                special_rows.insert(y + 1);
            },
            North | South => {
                special_cols.insert(x);
                special_cols.insert(x + 1);
            },
        }
    }
    // The boundaries should be included because they were one more than a boundary
    //   trench. Confirm that this is the case.
    assert!(special_rows.contains(&bounds.y()));
    assert!(special_cols.contains(&bounds.x()));

    // --- Build a 2-way map between "special rows" and "special cols" and small numbers ---
    let row_map = TwoWayIndexMap::new(special_rows);
    let col_map = TwoWayIndexMap::new(special_cols);

    // --- Construct an input based on the indexes, which WILL be small enough to solve ---
    let mut small_steps: Vec<DigStep> = Vec::with_capacity(steps.len());
    let mut big_x: usize = offset.0;
    let mut big_y: usize = offset.1;
    for big_step in steps {
        let dig_dir = big_step.dig_dir;
        let big_dist = big_step.dist;
        let small_dist = match dig_dir.0 {
            East => {
                let small_start = col_map.to_index(&big_x).expect("any value we encounter should be in the map");
                big_x += big_dist as usize;
                let small_end = col_map.to_index(&big_x).expect("any value we encounter should be in the map");
                let small_dist = small_end - small_start;
                small_dist as u32
            },
            South => {
                let small_start = row_map.to_index(&big_y).expect("any value we encounter should be in the map");
                big_y += big_dist as usize;
                let small_end = row_map.to_index(&big_y).expect("any value we encounter should be in the map");
                let small_dist = small_end - small_start;
                small_dist as u32
            },
            West => {
                let small_start = col_map.to_index(&big_x).expect("any value we encounter should be in the map");
                big_x -= big_dist as usize;
                let small_end = col_map.to_index(&big_x).expect("any value we encounter should be in the map");
                let small_dist = small_start - small_end;
                small_dist as u32
            },
            North => {
                let small_start = row_map.to_index(&big_y).expect("any value we encounter should be in the map");
                big_y -= big_dist as usize;
                let small_end = row_map.to_index(&big_y).expect("any value we encounter should be in the map");
                let small_dist = small_start - small_end;
                small_dist as u32
            },
        };
        let small_step = DigStep{dig_dir: dig_dir, dist: small_dist};
        small_steps.push(small_step);
    }

    // --- Solve the small one ---
    let small_dig_grid = DigGrid::new(&small_steps);

    // --- Find the area, but use bigger sizes ---
    let big_grid_area: usize = small_dig_grid.grid.bound().range_by_rows()
        .filter(|coord| *small_dig_grid.grid.get(*coord) != DigType::Edge)
        .map(|coord| {
            let big_x_min = col_map.to_value(coord.x()).expect("any value we encounter should be in the map");
            let big_x_max = col_map.to_value(coord.x() + 1).expect("any value we encounter should be in the map");
            let big_delta_x = big_x_max - big_x_min;
            let big_y_min = row_map.to_value(coord.y()).expect("any value we encounter should be in the map");
            let big_y_max = row_map.to_value(coord.y() + 1).expect("any value we encounter should be in the map");
            let big_delta_y = big_y_max - big_y_min;
            big_delta_x * big_delta_y
        })
        .sum();

    big_grid_area
}

// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let steps = input.into_iter().map(|(x,_)| x).copied().collect();
    let dig_grid = DigGrid::new(&steps);
    println!("The area it can hold is {}", dig_grid.area());
}


fn part_b(input: &Input) {
    println!("\nPart b:");
    let steps: Vec<DigStep> = input.into_iter().map(|(_,x)| x).copied().collect();
    let area = giant_size_area(&steps);
    println!("The area it can hold is {}", area);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
