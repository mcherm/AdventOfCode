use std::fmt::{Display, Formatter};
use std::collections::HashSet;
use std::cmp::{max, min};
use anyhow;
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


type Input = Vec<DigStep>;



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
    }

    impl DigStep {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    DigDir::parse,
                    nom::bytes::complete::tag(" "),
                    nom_num,
                    nom::bytes::complete::tag(" ("),
                    nom::bytes::complete::is_not(")"),
                    nom::bytes::complete::tag(")"),
                )),
                |(dig_dir, _, dist, _, _, _)| {
                    DigStep{dig_dir, dist}
                }
            )(input)
        }

        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
            nom::multi::many1(
                nom::sequence::terminated(
                    Self::parse,
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
    fn find_bounds(input: &Input) -> (Coord, Coord) {
        let mut x: i64 = 0;
        let mut y: i64 = 0;
        let mut min_x: i64 = 0;
        let mut min_y: i64 = 0;
        let mut max_x: i64 = 0;
        let mut max_y: i64 = 0;
        for step in input {
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

    /// Creates the grid from a given input.
    fn new(input: &Input) -> Self {

        // --- Find the bounds and make a grid ---
        let (offset, bounds) = Self::find_bounds(input);
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

        for step in input {
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

// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let dig_grid = DigGrid::new(input);
    println!("The area it can hold is {}", dig_grid.area());
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
