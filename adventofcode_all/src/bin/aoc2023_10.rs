use std::fmt::{Display, Formatter};
use anyhow;
use std::collections::{HashMap, HashSet};


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug)]
pub enum Item {
    PipeNS,
    PipeEW,
    PipeNE,
    PipeNW,
    PipeSW,
    PipeSE,
    Ground,
}


#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Coord(usize,usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Direction {
    North, South, East, West
}
use Direction::*;

#[derive(Debug)]
pub struct Grid {
    width: usize,
    height: usize,
    start_coord: Coord,
    start_dir: Direction, // direction we start going around
    items: HashMap<Coord,Item>
}

type Input = Grid;


impl Item {
    fn as_char(&self) -> char {
        match self {
            Item::PipeNS => '|',
            Item::PipeEW => '-',
            Item::PipeNE => 'L',
            Item::PipeNW => 'J',
            Item::PipeSW => '7',
            Item::PipeSE => 'F',
            Item::Ground => '.',
        }
    }

    /// Returns the set of directions this connects to.
    fn connections(&self) -> HashSet<Direction> {
        match self {
            Item::PipeNS => [North, South].into(),
            Item::PipeEW => [East, West].into(),
            Item::PipeNE => [North, East].into(),
            Item::PipeNW => [North, West].into(),
            Item::PipeSW => [South, West].into(),
            Item::PipeSE => [South, East].into(),
            Item::Ground => [].into(),
        }
    }

    /// Returns true if this connects to the indicated direction and false if not.
    fn connects_to(&self, dir: Direction) -> bool {
        self.connections().contains(&dir)
    }
}

impl Into<char> for Item {
    fn into(self) -> char {
        self.as_char()
    }
}



impl TryInto<Item> for char {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Item, Self::Error> {
        match self {
            '|' => Ok(Item::PipeNS),
            '-' => Ok(Item::PipeEW),
            'L' => Ok(Item::PipeNE),
            'J' => Ok(Item::PipeNW),
            '7' => Ok(Item::PipeSW),
            'F' => Ok(Item::PipeSE),
            '.' => Ok(Item::Ground),
            _ => Err(anyhow::anyhow!("invalid character '{}'", self))
        }
    }
}

impl TryInto<Item> for HashSet<Direction> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Item, Self::Error> {
        let has = |dir: Direction| self.contains(&dir);
        Ok(match (has(North), has(South), has(East), has(West)) {
            (true, true, false, false) => Item::PipeNS,
            (false, false, true, true) => Item::PipeEW,
            (true, false, true, false) => Item::PipeNE,
            (true, false, false, true) => Item::PipeNW,
            (false, true, false, true) => Item::PipeSW,
            (false, true, true, false) => Item::PipeSE,
            (false, false, false, false) => Item::Ground,
            _ => Err(anyhow::anyhow!("invalid connections for an item"))?
        })
    }
}


impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl Direction {
    /// Returns the opposite of this direction
    fn reverse(&self) -> Direction {
        match self {
            North => South,
            South => North,
            East => West,
            West => East,
        }
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{}", self.items.get(&Coord(x,y)).unwrap())?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}

impl Coord {
    /// Returns a list of (direction,neighbor_coord) pairs. It will not always be
    /// of length 4, but WILL always be in the grid.
    fn neighbors(&self,  width: usize, height: usize) -> Vec<(Direction,Coord)> {
        let mut answer = Vec::with_capacity(4);
        if self.1 > 0 {
            answer.push((Direction::North, Coord(self.0, self.1 - 1)))
        }
        if self.1 + 1 < height {
            answer.push((Direction::South, Coord(self.0, self.1 + 1)));
        }
        if self.0 + 1 < width {
            answer.push((Direction::East, Coord(self.0 + 1, self.1)));
        }
        if self.0 > 0 {
            answer.push((Direction::West, Coord(self.0 - 1, self.1)));
        }
        answer
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}


mod parse {
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use super::{Input, Grid, Coord, Direction, Item};


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_10.txt")?;
        Grid::parse(&s)
    }


    impl Grid {
        /// Parses the input. Assumes the grid is at least 1 row and at least one
        /// column or this will fail in various ways.
        fn parse(input: &str) -> Result<Self, anyhow::Error> {
            let mut width = 0;
            let mut height = 0;
            let mut items: HashMap<Coord,Item> = HashMap::new();
            let mut start_opt: Option<Coord> = None;

            for (y,line) in input.lines().enumerate() {
                height += 1;
                assert!(y + 1 == height);
                let mut line_width = 0;
                for (x,c) in line.chars().enumerate() {
                    line_width += 1;
                    assert!(x + 1 == line_width);
                    if c == 'S' {
                        anyhow::ensure!(start_opt.is_none(), "multiple start locations");
                        start_opt = Some(Coord(x,y));
                    } else {
                        items.insert(Coord(x,y), c.try_into()?);
                    }
                }
                anyhow::ensure!(y == 0 || line_width == width, "uneven lines");
                width = line_width;
            }

            if let Some(start_coord) = start_opt {
                // Need to set the start item
                let mut connects: HashSet<Direction> = HashSet::with_capacity(2);
                for (dir, coord) in start_coord.neighbors(width, height) {
                    if items.get(&coord).unwrap().connects_to(dir.reverse()) {
                        connects.insert(dir);
                    }
                }
                anyhow::ensure!(connects.len() == 2, "start does not connect to exactly 2 pipes");
                let start_dir: Direction = *connects.iter().next().unwrap(); // pick a direction to go around
                let start_item: Item = connects.try_into()?;
                items.insert(start_coord, start_item);

                // Now the grid is ready
                Ok(Grid{width, height, start_coord, start_dir, items})
            } else {
                Err(anyhow::anyhow!("no start location"))
            }
        }

    }

}


// ======= Compute =======

#[derive(Debug)]
struct PathStep {
    coord: Coord,
    going_to: Direction,
}

#[derive(Debug)]
struct Path {
    steps: Vec<PathStep>
}

impl Path {
    /// Gives the length of the Path.
    fn len(&self) -> usize {
        self.steps.len()
    }
}

impl Grid {
    /// Traces the path, from start_pos going until we get back
    /// to it, returned as a Path object. Panics if the path is
    /// found to not be a proper loop.
    fn trace_path(&self) -> Path {
        let mut steps = Vec::new();
        let mut step: PathStep = PathStep{
            coord: self.start_coord,
            going_to: self.start_dir,
        };
        loop {
            let next_coord = step.coord
                .neighbors(self.width, self.height)
                .into_iter()
                .filter_map(move |(dir, next_coord)| if dir == step.going_to {
                    Some(next_coord)
                } else {
                    None
                })
                .next()
                .expect(format!("path goes off the board at {}", step.coord).as_str());
            let next_item = self.items.get(&next_coord).unwrap();
            if matches!(next_item, Item::Ground) {
                panic!("path leads to the ground at {}", step.coord)
            }
            let mut where_we_could_go_next = next_item.connections();
            let connected_to_prev_step = where_we_could_go_next.remove(&step.going_to.reverse());
            if !connected_to_prev_step {
                panic!("path doesn't connect properly at {}", step.coord)
            }
            assert!(where_we_could_go_next.len() == 1);
            let next_going_to = where_we_could_go_next.into_iter().next().unwrap();

            // Now we can actually move there and create the new PathStep
            steps.push(step);
            step = PathStep{coord: next_coord, going_to: next_going_to};
            if step.coord == self.start_coord {
                // we completed the loop!
                break;
            }
        }
        Path{steps}
    }
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let len = input.trace_path().len();
    assert!(len % 2 == 0);
    let half_len = len / 2;
    println!("The midpoint is {} steps away", half_len);
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
