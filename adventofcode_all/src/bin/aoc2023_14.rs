use std::fmt::{Display, Formatter};
use anyhow;
use itertools::Itertools;
use std::ops::Range;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Coord(usize,usize);


#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
enum Rock {
    Round, Square, Empty
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Grid {
    data: Vec<Vec<Rock>>,
}


impl Grid {
    fn new(data: Vec<Vec<Rock>>) -> Self {
        assert!(data.len() > 0);
        assert!(data[0].len() > 0);
        assert!(data.iter().map(|row| row.len()).all_equal());
        Grid{data}
    }

    fn width(&self) -> usize {
        self.data[0].len()
    }

    fn height(&self) -> usize {
        self.data.len()
    }

    fn value(&self, coord: Coord) -> Rock {
        assert!(coord.0 < self.width() && coord.1 < self.height());
        self.data[coord.1][coord.0]
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

impl Display for Rock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Rock::Round => 'O',
            Rock::Square => '#',
            Rock::Empty => '.',
        })
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for y in 0..self.height() {
            for x in 0..self.width() {
                write!(f, "{}", self.value(Coord(x,y)))?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}



type Input = Grid;



mod parse {
    use super::{Input, Grid, Rock};
    use std::fs;
    use anyhow::anyhow;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_14.txt")?;
        let data_result: Result<Vec<Vec<Rock>>, anyhow::Error> = s.lines().map(|line| {
            line.chars().map(|c| match c {
                'O' => Ok(Rock::Round),
                '#' => Ok(Rock::Square),
                '.' => Ok(Rock::Empty),
                _ => Err(anyhow!("invalid character in grid")),
            }).collect()
        }).collect();
        Ok(Grid::new(data_result?))
    }

}


// ======= Compute =======


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Direction {
    North, West, South, East
}


impl Direction {
    /// Returns true if this goes against the natural counting order, or false if it doesn't.
    fn is_reversed(&self) -> bool {
        match self {
            Direction::North => true,
            Direction::West => true,
            Direction::South => false,
            Direction::East => false,
        }
    }

    /// Given a coordinate parallel to the Direction and one perpindicular to it (in that
    /// order), this returns the Coord.
    fn coord(&self, parallel: usize, perp: usize) -> Coord {
        match self {
            Direction::North => Coord(perp, parallel),
            Direction::West  => Coord(parallel, perp),
            Direction::South => Coord(perp, parallel),
            Direction::East  => Coord(parallel, perp),
        }
    }
}


/// Sadly, the built-in std::ops::Range can be reversed efficiently, but doing so creates a
/// new type. To be able to return a single type that may or may not be revertsed, I have
/// to build my own.
#[derive(Debug)]
struct ReversibleRange {
    start: usize, // inclusive, always the lower value
    end: usize, // exclusive, always the upper value
    reversed: bool, // true means we go from (end-1) down through (start)
    pos: usize, // Goes from 0..(end + 1). 0 means we haven't started iterating,
                // end means we finished iterating, and any other value means
                // our next value will be the pos'th one.
}

impl ReversibleRange {
    fn new(range: Range<usize>, reversed: bool) -> Self {
        ReversibleRange{
            start: range.start,
            end: range.end,
            reversed,
            pos: 0,
        }
    }
}

impl Iterator for ReversibleRange {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.end {
            None
        } else {
            let answer = if self.reversed {
                self.end - self.pos - 1
            } else {
                self.start + self.pos
            };
            self.pos += 1;
            Some(answer)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.end - self.start - self.pos;
        (size, Some(size))
    }
}


impl Grid {
    /// Returns self.width() or self.height() parallel to the direction given.
    fn para_limit(&self, dir: Direction) -> usize {
        match dir {
            Direction::North => self.height(),
            Direction::West => self.width(),
            Direction::South => self.height(),
            Direction::East => self.width(),
        }
    }

    /// Returns self.width() or self.height() perpendicular to the direction given.
    fn perp_limit(&self, dir: Direction) -> usize {
        match dir {
            Direction::North => self.width(),
            Direction::West => self.height(),
            Direction::South => self.width(),
            Direction::East => self.height(),
        }
    }


    /// Gives a Range of the positions in the Grid going in the given direction.
    fn range<'a>(&self, dir: Direction) -> ReversibleRange {
        match dir {
            Direction::North => ReversibleRange::new(0..self.height(), true),
            Direction::West  => ReversibleRange::new(0..self.width(),  true),
            Direction::South => ReversibleRange::new(0..self.height(), false),
            Direction::East  => ReversibleRange::new(0..self.width(),  false),
        }
    }

    /// Gives the overall load for the given column.
    fn column_load(&self, x: usize) -> usize {
        assert!(x < self.width());
        let mut load = 0;
        let mut floating_rounds = 0;
        for y in (0..self.height()).rev() {
            match self.value(Coord(x,y)) {
                Rock::Round => {
                    floating_rounds += 1;
                }
                Rock::Square => {
                    let min = self.height() - y - floating_rounds;
                    let max = self.height() - y;
                    load += (min..max).sum::<usize>();
                    floating_rounds = 0;
                }
                Rock::Empty => {}
            }
        }
        // -- handle remaining floating rounds --
        let min = self.height() + 1 - floating_rounds;
        let max = self.height() + 1;
        load += (min..max).sum::<usize>();

        load
    }

    /// Find the total load (part 1).
    fn load(&self) -> usize {
        (0..self.width()).map(|y| self.column_load(y)).sum()
    }


    /// Finds a single column of tilting north
    fn tilt_slice(&self, dir: Direction, perp: usize) -> Vec<Rock> {
        assert!(perp < self.perp_limit(dir));
        let mut answer: Vec<Rock> = Vec::with_capacity(self.para_limit(dir));
        let mut floating_spaces: usize = 0;
        let mut floating_rounds: usize = 0;
        for para in self.range(dir) {
            match self.value(dir.coord(para, perp)) {
                Rock::Empty => {
                    floating_spaces += 1;
                }
                Rock::Round => {
                    floating_rounds += 1;
                }
                Rock::Square => {
                    for _ in 0..floating_spaces {
                        answer.push(Rock::Empty);
                    }
                    for _ in 0..floating_rounds {
                        answer.push(Rock::Round);
                    }
                    answer.push(Rock::Square);
                    floating_spaces = 0;
                    floating_rounds = 0;
                }
            }
        }
        for _ in 0..floating_spaces {
            answer.push(Rock::Empty);
        }
        for _ in 0..floating_rounds {
            answer.push(Rock::Round);
        }
        if dir.is_reversed() {
            // FIXME: I could avoid the reversal by letting this return it wrong-way-round and
            //   then when I transpose it in the caller, dealing with it there. I'll save that
            //   idea for later.
            answer.reverse(); // we built it backward of the "natural" direction that's needed. Reverse it.
        }
        answer
    }


    /// Find the new grid created by tilting in the given direction.
    fn tilt(&self, dir: Direction) -> Grid {
        let mut slices: Vec<Vec<Rock>> = Vec::with_capacity(self.perp_limit(dir));
        for w in 0..self.perp_limit(dir) {
            slices.push(self.tilt_slice(dir, w));
        }
        let new_data: Vec<Vec<Rock>> = (0..self.height()).map(|y| {
            (0..self.width()).map(|x| {
                match dir {
                    Direction::North => slices[x][y],
                    Direction::West => slices[y][x],
                    Direction::South => slices[x][y],
                    Direction::East => slices[y][x],
                }
            }).collect()
        }).collect();
        Grid::new(new_data)
    }

    /// Find the new grid created by performing one whole cycle.
    fn cycle(&self) -> Grid {
        let grid = self.tilt(Direction::North);
        let grid = grid.tilt(Direction::West);
        let grid = grid.tilt(Direction::South);
        let grid = grid.tilt(Direction::East);
        grid
    }
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let load = input.load();
    println!("The load was {}", load);
}


fn part_b(input: &Input) {
    println!("\nPart b:");
    println!("Input is {}", input);
    let grid = input;
    let grid = grid.cycle();
    println!("After one cycle it is {}", grid);
    let grid = grid.cycle();
    println!("After two cycles it is {}", grid);
    let grid = grid.cycle();
    println!("After three cycles it is {}", grid);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
