
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
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Coord(pub usize, pub usize);

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

        pub fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom::branch::alt((
                nom::combinator::value(Facing::Right, nom::bytes::complete::tag("R")),
                nom::combinator::value(Facing::Down, nom::bytes::complete::tag("D")),
                nom::combinator::value(Facing::Left, nom::bytes::complete::tag("L")),
                nom::combinator::value(Facing::Up, nom::bytes::complete::tag("U")),
            ))(input)
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

/// A module for identifying an unfolded cube and dealing with it.
mod cubefold {
    use itertools::Itertools;
    use nom::character::complete::line_ending;
    use crate::compute::{Coord, Facing};
    use once_cell::sync::Lazy;



    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum FaceNum {F0, F1, F2, F3, F4, F5}

    /// Defines the connection between two particular sides.
    #[derive(Debug, Clone)]
    struct Wrap {
        edges: [(FaceNum, Facing); 2],
        reverse: bool, // if true, then we map reverse the sense of the indexes
    }

    #[derive(Debug, Clone)]
    struct CubeLayout {
        bounds: Coord,
        filled: Vec<Vec<Option<FaceNum>>>,
        wraps: Vec<Wrap>, // NOTE: Will always have length 7
    }



    impl FaceNum {
        // FIXME: Remove if not used
        #[allow(dead_code)]
        fn from_num(n: u8) -> Self {
            match n {
                0 => FaceNum::F0,
                1 => FaceNum::F1,
                2 => FaceNum::F2,
                3 => FaceNum::F3,
                4 => FaceNum::F4,
                5 => FaceNum::F5,
                _ => panic!()
            }
        }

        // FIXME: Remove if not used
        #[allow(dead_code)]
        fn to_num(&self) -> u8 {
            match self {
                FaceNum::F0 => 0,
                FaceNum::F1 => 1,
                FaceNum::F2 => 2,
                FaceNum::F3 => 3,
                FaceNum::F4 => 4,
                FaceNum::F5 => 5,
            }
        }

        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom::branch::alt((
                nom::combinator::value(FaceNum::F0, nom::bytes::complete::tag("0")),
                nom::combinator::value(FaceNum::F1, nom::bytes::complete::tag("1")),
                nom::combinator::value(FaceNum::F2, nom::bytes::complete::tag("2")),
                nom::combinator::value(FaceNum::F3, nom::bytes::complete::tag("3")),
                nom::combinator::value(FaceNum::F4, nom::bytes::complete::tag("4")),
                nom::combinator::value(FaceNum::F5, nom::bytes::complete::tag("5")),
            ))(input)
        }
    }

    impl Wrap {
        /// Construct a Wrap from a string like "0R=2U"
        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    FaceNum::parse,
                    Facing::parse,
                    nom::branch::alt((
                        nom::bytes::complete::tag("=="),
                        nom::bytes::complete::tag("=!"),
                    )),
                    FaceNum::parse,
                    Facing::parse,
                )),
                |(fc1, fng1, rev, fc2, fng2)| Wrap{edges: [(fc1, fng1), (fc2, fng2)], reverse: rev == "=!"}
            )(input)
        }

        /// Parses a list of Wraps.
        fn parse_list(input: &str) -> nom::IResult<&str, Vec<Self>> {
            nom::multi::separated_list1(
                nom::bytes::complete::tag(" "),
                Wrap::parse,
            )(input)
        }
    }

    impl CubeLayout {
        /// Creates a CubeLayout from input. Since we intend to use this on hard-coded
        /// inputs, it panics if there is any syntax issue.
        fn from_visual(visual: &str, wrap_instructions: &str) -> Self {
            assert_eq!(48, wrap_instructions.len());
            if let Ok((leftover, filled)) = Self::parse_filled(visual) {
                assert_eq!(leftover, "");
                let bounds = Coord(filled[0].len(), filled.len());
                if let Ok((leftover, wraps)) = Wrap::parse_list(wrap_instructions) {
                    assert_eq!(wraps.len(), 7);
                    assert_eq!(leftover, "");
                    CubeLayout{bounds, filled, wraps}
                } else {
                    panic!("failed to parse wrap instructions")
                }
            } else {
                panic!("the 'filled' visual input is invalid.");
            }
        }

        /// Nom parser for the string that populates the "filled" field.
        fn parse_filled(input: &str) -> nom::IResult<&str, Vec<Vec<Option<FaceNum>>>> {
            nom::multi::many1( // multiple rows...
                nom::sequence::terminated(
                    nom::multi::many1( // each row has items
                        nom::branch::alt(( // each item is either...
                            nom::combinator::map( // (1) a FaceNum (treated as Some(face_num))
                                FaceNum::parse,
                                |face_num| Some(face_num)
                            ),
                            nom::combinator::value( // or (2) "." treated as None
                                None,
                                nom::bytes::complete::tag(".")
                            ),
                        )),
                    ),
                    line_ending // and each row is followed by a newline.
                )
            )(input)
        }


        /// This is given a rectangular grid of booleans, where true means "filled in" and
        /// false means "blank". It returns true if that grid matches this Fold and false
        /// if it doesn't.
        fn matches(&self, fill: Vec<Vec<bool>>) -> bool {
            assert!(fill.len() > 0);
            assert!(fill[0].len() > 0);
            assert!(fill.iter().map(|x| x.len()).all_equal());
            let is_filled_grid: Vec<Vec<bool>> = self.filled.iter()
                .map(|row| {
                    row.iter().map(|x| x.is_some()).collect()
                })
                .collect();
            is_filled_grid == fill
        }

        /// Returns a CubeLayout which is this one but flipped in the x direction.
        fn flip_x(&self) -> Self {
            let bounds = self.bounds;
            let filled = self.filled.iter()
                .map(|row| {
                    row.iter().rev().copied().collect()
                })
                .collect();
            let wraps = self.wraps.clone(); // FIXME: Must transform this
            CubeLayout{bounds, filled, wraps}
        }

        /// Returns a CubeLayout which is this one but flipped in the y direction.
        fn flip_y(&self) -> Self {
            let bounds = self.bounds;
            let filled = self.filled.iter().rev().cloned().collect();
            let wraps = self.wraps.clone(); // FIXME: Must transform this
            CubeLayout{bounds, filled, wraps}
        }

        /// Returns a CubeLayout which is this one but transposed (swapping x and y).
        fn transpose(&self) -> Self {
            let bounds = Coord(self.bounds.1, self.bounds.0);
            let mut filled = Vec::new();
            for y in 0..self.bounds.0 {
                let mut row = Vec::new();
                for x in 0..self.bounds.1 {
                    row.push(self.filled[x][y]);
                }
                filled.push(row);
            }
            let wraps = self.wraps.clone(); // FIXME: Must transform this
            CubeLayout{bounds, filled, wraps}
        }

    }

    static LAYOUTS_4_BY_3: Lazy<[CubeLayout; 10]> = Lazy::new(|| [
        CubeLayout::from_visual("\
            0...\n\
            1234\n\
            5...\n\
        ", "0R==2U 0L==4U 0U=!3U 1L==4R 2D==5R 3D=!5D 4D=!5L"),
        CubeLayout::from_visual("\
            .0..\n\
            1234\n\
            5...\n\
        ", "0L=!1U 0U=!4U 0R=!3U 1L==4R 2D==5R 3D=!5D 4D=!5L"),
        CubeLayout::from_visual("\
            ..0.\n\
            1234\n\
            5...\n\
        ", "0U=!1U 0R=!4U 0L==2U 1L==4R 2D==5R 3D=!5D 4D=!5L"),
        // rules:
        CubeLayout::from_visual("\
            ...0\n\
            1234\n\
            5...\n\
        ", "0R=!1U 0L==3U 0U=!2U 1L==4R 2D==5R 3D=!5D 4D=!5L"),
        CubeLayout::from_visual("\
            .0..\n\
            1234\n\
            .5..\n\
        ", "0L=!1U 0U=!4U 0R=!3U 1L==4R 1D=!5L 3D==5R 4D=!5D"),
        CubeLayout::from_visual("\
            ..0.\n\
            1234\n\
            .5..\n\
        ", "0U=!1U 0R=!4U 0L=!2U 1L==4R 1D=!5L 3D==5R 4D=!5D"),
        CubeLayout::from_visual("\
            .0..\n\
            .123\n\
            45..\n\
        ", "0R=!2U 0L=!4L 0U=!3U 1L==4U 2D==5R 3R==4D 3D=!5D"),
        CubeLayout::from_visual("\
            ..0.\n\
            .123\n\
            45..\n\
        ", "0R=!3U 0L==1U 0U==4L 1L==4U 2D==5R 3R==4D 3D=!5D"),
        CubeLayout::from_visual("\
            ...0\n\
            .123\n\
            45..\n\
        ", "0U=!1U 0R==4L 0L==2U 1L==4U 2D==5R 3R==4D 3D=!5D"),
        CubeLayout::from_visual("\
            ..01\n\
            .23.\n\
            45..\n\
        ", "0L==2U 0U==4L 1R==5D 1D==3R 1U==4D 2L==4U 3D==5R"),
    ]);

    static LAYOUTS_5_BY_2: Lazy<[CubeLayout; 1]> = Lazy::new(|| [
        CubeLayout::from_visual("\
            ..012\n\
            345..\n\
        ", "0L==4U 0U=!3U 1D==5R 1U==3L 2R==4D 2D=!5D 2U==3D"),
    ]);

    /// This contains every possible CubeLayout. Some of these are actually duplicates, because
    /// of symmetry. But that's OK, it still at least has every one of them.
    static ALL_POSSIBLE_LAYOUTS: Lazy<Vec<CubeLayout>> = Lazy::new(|| {
        let answer: Vec<CubeLayout> = LAYOUTS_4_BY_3.iter().chain(LAYOUTS_5_BY_2.iter())
            .flat_map(|layout| [layout.clone(), layout.flip_x()].into_iter()) // flip each in the x direction
            .flat_map(|layout| [layout.clone(), layout.flip_y()].into_iter()) // flip each in the y direction
            .flat_map(|layout| [layout.clone(), layout.transpose()].into_iter()) // transpose each one
            .collect();
        assert_eq!(answer.len(), 88);
        answer
    });


    #[cfg(test)]
    mod tests {
        use super::*;
        use FaceNum::*;

        #[test]
        fn test_indent_strings() {
            assert_eq!(LAYOUTS_4_BY_3[0].filled, vec![
                vec![Some(F0), None,     None,     None    ],
                vec![Some(F1), Some(F2), Some(F3), Some(F4)],
                vec![Some(F5), None,     None,     None    ],
            ]);
        }

        #[test]
        fn test_fold_matches() {
            let fold = &LAYOUTS_4_BY_3[0];
            assert_eq!(Coord(4,3), fold.bounds);
            assert_eq!(true, fold.matches(vec![
                vec![true , false, false, false],
                vec![true , true , true , true ],
                vec![true , false, false, false],
            ]));
            assert_eq!(false, fold.matches(vec![
                vec![true , true, false, false],
                vec![true , true , true , true ],
                vec![true , false, false, false],
            ]));
            assert_eq!(false, fold.matches(vec![
                vec![true , false, false, false],
                vec![true , true , false, true ],
                vec![true , false, false, false],
            ]));
            assert_eq!(false, fold.matches(vec![
                vec![true , false, false, false],
                vec![true , true , true , true ],
                vec![true , false, false, false],
                vec![false, false, false, false],
            ]));
            let fold = &LAYOUTS_5_BY_2[0];
            assert_eq!(Coord(5,2), fold.bounds);
            assert_eq!(true, fold.matches(vec![
                vec![false, false, true , true , true ],
                vec![true , true , true , false, false],
            ]));
        }

        #[test]
        fn test_fold_flip_x() {
            let fold = &LAYOUTS_4_BY_3[0].flip_x();
            assert_eq!(Coord(4,3), fold.bounds);
            assert_eq!(true, fold.matches(vec![
                vec![false, false, false, true ],
                vec![true , true , true , true ],
                vec![false, false, false, true ],
            ]));
        }

        #[test]
        fn test_fold_flip_y() {
            let fold = &LAYOUTS_4_BY_3[1].flip_y();
            assert_eq!(Coord(4,3), fold.bounds);
            assert_eq!(true, fold.matches(vec![
                vec![true , false, false, false],
                vec![true , true , true , true ],
                vec![false, true, false, false ],
            ]));
        }

        #[test]
        fn test_fold_transpose() {
            let fold = &LAYOUTS_4_BY_3[0].transpose();
            assert_eq!(Coord(3,4), fold.bounds);
            assert_eq!(true, fold.matches(vec![
                vec![true , true , true ],
                vec![false, true , false],
                vec![false, true , false],
                vec![false, true , false],
            ]));
        }
    }
}


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

