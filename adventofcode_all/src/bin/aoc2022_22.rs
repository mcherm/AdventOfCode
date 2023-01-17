
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fmt::Debug;
    use std::fs;
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::tag,
        combinator::{value, map},
        character::complete::{line_ending, u8 as nom_num},
        sequence::{terminated, tuple},
        multi::{many1},
    };


    pub fn input() -> Result<InputData, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_22.txt")?;
        match InputData::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    #[derive(Debug, Copy, Clone)]
    pub enum GridElem {
        Wall,
        Open,
        Blank,
    }

    #[derive(Debug)]
    pub struct MapOfBoard {
        width: usize,
        height: usize,
        data: Vec<GridElem>,
    }


    #[derive(Debug, Copy, Clone)]
    pub enum TurnDir {
        Left, Right
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Step {
        Move(usize),
        Turn(TurnDir),
    }

    /// Stores 4-character (lowercase letter) names efficiently.
    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    pub struct Name {
        code: u32
    }

    #[derive(Debug)]
    pub struct InputData {
        pub map: MapOfBoard,
        pub steps: Vec<Step>,
    }



    impl GridElem {
        /// Parses a single GridElem
        fn parse(input: &str) -> IResult<&str, Self> {
            alt((
                value(GridElem::Wall, tag("#")),
                value(GridElem::Open, tag(".")),
                value(GridElem::Blank, tag(" ")),
            ))(input)
        }
    }

    impl MapOfBoard {
        /// Create an instance from what we read.
        fn from_rows(rows: Vec<Vec<GridElem>>) -> Self {
            let height = rows.len();
            let width = rows.iter().map(|row| row.len()).max().unwrap();
            let mut data = Vec::new();
            for y in 0..height {
                let row = &rows[y];
                for x in 0..width {
                    let grid_elem = row.get(x).unwrap_or(&GridElem::Blank);
                    data.push(*grid_elem);
                }
            }
            MapOfBoard{width, height, data}
        }

        /// Accessor for width
        pub fn width(&self) -> usize {
            self.width
        }

        /// Accessor for height
        pub fn height(&self) -> usize {
            self.height
        }

        /// Retrieve from an (x,y) location
        pub fn get_at(&self, x: usize, y: usize) -> GridElem {
            assert!(x < self.width);
            assert!(y < self.height);
            self.data[y * self.width + x]
        }

        /// Parses the whole MapOfBoard
        fn parse(input: &str) -> IResult<&str, Self> {
            map(
                many1(
                    terminated(
                        many1( GridElem::parse ),
                        line_ending
                    )
                ),
                |rows| MapOfBoard::from_rows(rows)
            )(input)
        }
    }

    impl Step {
        /// Parses a single Step
        fn parse(input: &str) -> IResult<&str, Self> {
            alt((
                map(
                    nom_num,
                    |x| Step::Move( x as usize )
                ),
                value(Step::Turn(TurnDir::Left), tag("L")),
                value(Step::Turn(TurnDir::Right), tag("R")),
            ))(input)
        }


        /// Parses a newline-terminated list of Steps. This does not attempt to enforce
        /// that the steps alternate between moves and turns. But I don't think that NEEDS
        /// to be enforced.
        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
            many1(Step::parse)(input)
        }
    }

    impl InputData {
        /// Parses the entire input data
        fn parse(input: &str) -> IResult<&str, Self> {
            map(
                tuple((
                    MapOfBoard::parse,
                    line_ending,
                    Step::parse_list,
                    line_ending,
                )),
                |(map, _, steps, _)| InputData{map, steps}
            )(input)
        }
    }

}



// ======= Part 1 Compute =======

mod compute {
    use crate::parse::{GridElem, MapOfBoard, Step, TurnDir};


    /// An (x,y) coordinate in the grid
    #[derive(Debug, Copy, Clone)]
    pub struct Coord(usize, usize);

    /// A facing
    #[derive(Debug, Copy, Clone)]
    pub enum Facing {
        Right = 0,
        Down = 1,
        Left = 2,
        Up = 3,
    }

    /// The grid on the map
    #[derive(Debug)]
    pub struct Grid<'a> {
        map: &'a MapOfBoard,
        pos: Coord,
        facing: Facing,
    }




    impl Coord {
        /// Modifies this coord by 1 in the direction of Facing, wrapping according to
        /// width and height.
        fn increment(&self, facing: Facing, width: usize, height: usize) -> Coord {
            let (mut x, mut y) = (self.0, self.1);
            match facing {
                Facing::Right => {
                    x += 1;
                    if x == width {
                        x = 0;
                    }
                },
                Facing::Down => {
                    y += 1;
                    if y == height {
                        y = 0;
                    }
                },
                Facing::Left => {
                    if x == 0 {
                        x = width - 1;
                    } else {
                        x -= 1;
                    }
                },
                Facing::Up => {
                    if y == 0 {
                        y = height - 1;
                    } else {
                        y -= 1;
                    }
                },
            }
            Coord(x,y)
        }
    }

    impl Facing {
        /// Convert a u8 (wrapping around) to a Facing
        fn from_u8(x: u8) -> Self {
            match x % 4 {
                0 => Facing::Right,
                1 => Facing::Down,
                2 => Facing::Left,
                3 => Facing::Up,
                _ => panic!("Mod failed us!"),
            }
        }

        /// Return the facing you get by rotating this one step in turn_dir.
        fn turn(&self, turn_dir: TurnDir) -> Facing {
            match turn_dir {
                TurnDir::Right => Facing::from_u8((*self as u8) + 1),
                TurnDir::Left => Facing::from_u8((*self as u8) + 3),
            }
        }
    }

    /// Finds the correct starting place on a given map.
    fn start_pos(map: &MapOfBoard) -> Coord {
        let y = 0;
        for x in 0..map.width() {
            match map.get_at(x, y) {
                GridElem::Open => return Coord(x,y),
                _ => {},
            }
        }
        panic!("No blanks in the first row.");
    }

    impl<'a> Grid<'a> {
        /// Construct a new Grid.
        pub fn new(map: &'a MapOfBoard) -> Self {
            let pos = start_pos(map);
            let facing = Facing::Right;
            Grid{map, pos, facing}
        }

        /// Given a coordinate, returns the GridElem at that spot.
        fn grid_elem(&self, coord: Coord) -> GridElem {
            self.map.get_at(coord.0, coord.1)
        }


        /// This executes a single step (moving or turning).
        fn apply(&mut self, step: Step) {
            match step {
                Step::Move(dist) => {
                    let mut steps_taken = 0;
                    let mut valid_pos = self.pos;
                    let mut probe_pos = valid_pos;
                    loop {
                        probe_pos = probe_pos.increment(self.facing, self.map.width(), self.map.height());
                        match self.grid_elem(probe_pos) {
                            GridElem::Wall => {
                                // we've been blocked; the move is over
                                break;
                            },
                            GridElem::Open => {
                                // OK, we've done one more step
                                valid_pos = probe_pos;
                                steps_taken += 1;
                                if steps_taken == dist {
                                    // We've done ALL the steps
                                    break;
                                }
                            }
                            GridElem::Blank => {
                                // Need to keep going through the loop until we find open space or a wall
                            }
                        }
                    }
                    // We finished moving
                    self.pos = valid_pos;
                }
                Step::Turn(turn_dir) => {
                    self.facing = self.facing.turn(turn_dir);
                }
            }
        }

        /// This executes a list of steps.
        pub fn apply_steps(&mut self, steps: &Vec<Step>) {
            for step in steps {
                self.apply(*step);
            }
        }

        /// Calculate the password.
        pub fn password(&self) -> usize {
            let col = self.pos.0 + 1;
            let row = self.pos.1 + 1;
            let facing = self.facing as usize;
            1000 * row + col * 4 + facing
        }
    }

}


// ======= Part 2 Compute =======



// ======= main() =======

use crate::parse::{input, InputData};
use crate::compute::Grid;


fn part_a(input: &InputData) {
    println!("\nPart a:");
    let mut grid = Grid::new(&input.map);
    grid.apply_steps(&input.steps);
    println!("Password = {}", grid.password());
}


fn part_b(_input: &InputData) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}

