
extern crate anyhow;
extern crate core;


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
        pub fn parse(input: &str) -> IResult<&str, Self> {
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
    use std::fmt::Debug;
    use crate::parse::{GridElem, MapOfBoard, Step, TurnDir};


    /// An (x,y) coordinate in the grid
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Coord(pub usize, pub usize);

    /// A facing
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Facing {
        Right = 0,
        Down = 1,
        Left = 2,
        Up = 3,
    }

    /// This implements the TemplateMethod pattern for a Grid -- it consists of the code
    /// to run when following the instructions takes one onto a blank location.
    pub trait WrapAroundBehavior: Debug {
        /// This is called when we are on a specific start_pos and are attempting
        /// to take a single-space move (not a whole "step") in the direction of
        /// facing. It will ONLY be called if attempting to do so leads to a
        /// location that is off the map. The method should return a location
        /// which IS on the map and the new facing.
        fn wrap_around(&self, start_pos: Coord, facing: Facing) -> (Coord, Facing);
    }

    /// The grid on the map
    #[derive(Debug)]
    pub struct Grid<'a> {
        map: &'a MapOfBoard,
        pos: Coord,
        facing: Facing,
        wrap_around_behavior: &'a dyn WrapAroundBehavior,
    }




    impl Coord {
        /// Modifies this coord by 1 in the direction of Facing, returning the new position
        /// if it is within (0..width, 0..height) and None if is outside that.
        fn increment(&self, facing: Facing, width: usize, height: usize) -> Option<Coord> {
            let (mut x, mut y) = (self.0, self.1);
            match facing {
                Facing::Right => {
                    x += 1;
                    if x == width {
                        return None;
                    }
                },
                Facing::Down => {
                    y += 1;
                    if y == height {
                        return None;
                    }
                },
                Facing::Left => {
                    if x == 0 {
                        return None;
                    } else {
                        x -= 1;
                    }
                },
                Facing::Up => {
                    if y == 0 {
                        return None;
                    } else {
                        y -= 1;
                    }
                },
            }
            Some(Coord(x,y))
        }

        /// This performs something equivalent to the div_mod operation -- it returns
        /// two Coords, the first one giving the position of the original in a grid
        /// of size divisor and the second giving the position within a cell of that
        /// grid. All coordinates measured from zero.
        pub fn div_mod(&self, divisor: usize) -> (Coord, Coord) {
            (Coord(self.0 / divisor, self.1 / divisor), Coord(self.0 % divisor, self.1 % divisor))
        }

        /// Given a divisor and the output of a div_mod(), this returns the original coordinate.
        pub fn from_div_mod(divisor: usize, div: Coord, mod_: Coord) -> Self {
            Coord(div.0 * divisor + mod_.0, div.1 * divisor + mod_.1)
        }
    }

    impl Facing {
        /// Every facing is either sideways (left or right) or non-sideways (up or down).
        pub fn goes_sideways(&self) -> bool {
            match self {
                Facing::Right => true,
                Facing::Down => false,
                Facing::Left => true,
                Facing::Up => false,
            }
        }

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

        /// Points in the opposite direction
        pub fn invert(&self) -> Facing {
            Facing::from_u8((*self as u8) + 2)
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


    #[derive(Debug)]
    pub struct KeepGoingBehavior<'a> {
        map: &'a MapOfBoard,
    }


    impl<'a> KeepGoingBehavior<'a> {
        pub fn new(map: &'a MapOfBoard) -> Self {
            KeepGoingBehavior{map}
        }
    }

    impl<'a> WrapAroundBehavior for KeepGoingBehavior<'a> {
        /// Our strategy will be to keep going in the direction of Facing, wrapping
        /// around at the outer bounds of the Map until we find open space or a wall
        /// (anything that isn't Blank).
        fn wrap_around(&self, start_pos: Coord, facing: Facing) -> (Coord, Facing) {
            let mut pos = start_pos;
            // Design note: the loop is guaranteed to exit because we must eventually
            //   wrap around back to start_pos, and we know THAT at least is non-blank.
            loop {
                let width = self.map.width();
                let height = self.map.height();
                match pos.increment(facing, width, height) {
                    Some(p) => pos = p,
                    None => {
                        pos = match facing {
                            Facing::Right => Coord(0, pos.1),
                            Facing::Down => Coord(pos.0, 0),
                            Facing::Left => Coord(width - 1, pos.1),
                            Facing::Up => Coord(pos.0, height - 1),
                        }
                    }
                }
                if ! matches!(self.map.get_at(pos.0, pos.1), GridElem::Blank) {
                    return (pos, facing);
                }
            }
        }
    }

    impl<'a> Grid<'a> {
        /// Construct a new Grid.
        pub fn new(map: &'a MapOfBoard, wrap_around_behavior: &'a dyn WrapAroundBehavior) -> Self {
            let pos = start_pos(map);
            let facing = Facing::Right;
            Grid{map, pos, facing, wrap_around_behavior}
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
                    let mut valid_pos = self.pos; // most recent position that was Open
                    let mut valid_facing = self.facing; // most recent facing
                    let mut probe_pos = valid_pos; // position we are now probing
                    let mut probe_facing = valid_facing; // facing we are now probing
                    loop {
                        // try incrementing
                        let probe_pos_opt = probe_pos.increment(valid_facing, self.map.width(), self.map.height());

                        // see if we wrapped, either way update probe_pos
                        if probe_pos_opt.is_none() || matches!(self.grid_elem(probe_pos_opt.unwrap()), GridElem::Blank) {
                            (probe_pos, probe_facing) = self.wrap_around_behavior.wrap_around(valid_pos, self.facing);
                        } else {
                            probe_pos = probe_pos_opt.unwrap()
                        }

                        // find out what's at that probe_pos
                        match self.grid_elem(probe_pos) {
                            GridElem::Wall => {
                                // we've been blocked; the move is over
                                break;
                            },
                            GridElem::Open => {
                                // OK, we've done one more step
                                valid_pos = probe_pos;
                                valid_facing = probe_facing;
                                steps_taken += 1;
                                if steps_taken == dist {
                                    // We've done ALL the steps
                                    break;
                                }
                            }
                            GridElem::Blank => {
                                panic!("The wrap_around_behavior should have ensured this was NOT blank.");
                            }
                        }
                    }
                    // We finished moving
                    self.pos = valid_pos;
                    self.facing = valid_facing;
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

/// A module for the CubeLayout object that represents a specific way a cube can be unfolded.
mod cubelayout {
    use std::collections::HashMap;
    use itertools::Itertools;
    use once_cell::sync::Lazy;
    use gcd::Gcd;
    use crate::compute::{Coord, Facing};



    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum FaceNum {F0, F1, F2, F3, F4, F5}

    /// A particular edge of a face which can be oriented next to a different one
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    struct Edge(FaceNum, Facing);

    /// Defines the connection between two particular sides.
    #[derive(Debug, Clone)]
    struct Wrap {
        edges: [Edge; 2],
        reverse: bool, // if true, then we map reverse the sense of the indexes
    }

    #[derive(Debug, Clone)]
    pub struct CubeLayout {
        bounds: Coord,
        filled: Vec<Vec<Option<FaceNum>>>,
        wraps: Vec<Wrap>, // NOTE: Will always have length 7
    }

    /// These are the aspect ratios that unfolded cubes can take on. (Names represent x by y.)
    #[derive(Debug, Eq, PartialEq, Hash)]
    pub enum Ratio {
        Ratio3By4,
        Ratio4By3,
        Ratio2By5,
        Ratio5By2,
    }


    impl FaceNum {
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

    impl Edge {
        /// Construct an Edge from a string like "3R"
        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::pair(FaceNum::parse, Facing::parse),
                |(face_num, facing)| Edge(face_num, facing)
            )(input)
        }

        /// If this Edge is in a Wrap in a CubeLayout which is flipped on the x axis, this
        /// returns the new Edge to use.
        fn flip_x(&self) -> Self {
            let new_facing = match self.1 {
                Facing::Right => Facing::Left,
                Facing::Down => Facing::Down,
                Facing::Left => Facing::Right,
                Facing::Up => Facing::Up,
            };
            Edge(self.0, new_facing)
        }

        /// If this Edge is in a Wrap in a CubeLayout which is flipped on the x axis, this
        /// returns the new Edge to use.
        fn flip_y(&self) -> Self {
            let new_facing = match self.1 {
                Facing::Right => Facing::Right,
                Facing::Down => Facing::Up,
                Facing::Left => Facing::Left,
                Facing::Up => Facing::Down,
            };
            Edge(self.0, new_facing)
        }

        /// If this Edge is in a Wrap in a CubeLayout which is transposed (x and y swapped),
        /// this returns the new Edge to use. Because of the way we transpose, transposing
        /// never requires a change in the reversal.
        fn transpose(&self) -> Self {
            match self {
                Edge(face_num, Facing::Right) => Edge(*face_num, Facing::Down ),
                Edge(face_num, Facing::Down)  => Edge(*face_num, Facing::Right),
                Edge(face_num, Facing::Left)  => Edge(*face_num, Facing::Up   ),
                Edge(face_num, Facing::Up)    => Edge(*face_num, Facing::Left ),
            }
        }
    }

    impl Wrap {
        /// Construct a Wrap from a string like "0R=2U"
        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    Edge::parse,
                    nom::branch::alt((
                        nom::bytes::complete::tag("=="),
                        nom::bytes::complete::tag("=!"),
                    )),
                    Edge::parse
                )),
                |(edge_1, rev, edge_2)| Wrap{edges: [edge_1, edge_2], reverse: rev == "=!"}
            )(input)
        }

        /// Parses a list of Wraps.
        fn parse_list(input: &str) -> nom::IResult<&str, Vec<Self>> {
            nom::multi::separated_list1(
                nom::bytes::complete::tag(" "),
                Wrap::parse,
            )(input)
        }

        /// Transforms this Wrap if we've flipped over the x axis.
        fn flip_x(&self) -> Self {
            let edges = self.edges.map(|x| x.flip_x());
            let mut reverse = self.reverse;
            if !self.edges[0].1.goes_sideways() {
                reverse = ! reverse;
            }
            if !self.edges[1].1.goes_sideways() {
                reverse = ! reverse;
            }
            Wrap{edges, reverse}
        }

        /// Transforms this Wrap if we've flipped over the y axis.
        fn flip_y(&self) -> Self {
            let edges = self.edges.map(|x| x.flip_y());
            let mut reverse = self.reverse;
            if self.edges[0].1.goes_sideways() {
                reverse = ! reverse;
            }
            if self.edges[1].1.goes_sideways() {
                reverse = ! reverse;
            }
            Wrap{edges, reverse}
        }

        /// Transposes this Wrap, returning the new Wrap.
        fn transpose(&self) -> Self {
            let edges = self.edges.map(|x| x.transpose());
            let reverse = self.reverse;
            Wrap{edges, reverse}
        }
    }

    impl PartialEq for Wrap {
        fn eq(&self, other: &Self) -> bool {
            if self.reverse == other.reverse {
                if self.edges[0] == other.edges[0] && self.edges[1] == other.edges[1] {
                    true
                } else if self.edges[1] == other.edges[1] && self.edges[1] == other.edges[0] {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
    }

    impl Eq for Wrap {
    }

    impl CubeLayout {
        /// Creates a CubeLayout from input. Since we intend to use this on hard-coded
        /// inputs, it panics if there is any syntax issue.
        pub fn from_visual(visual: &str, wrap_instructions: &str) -> Self {
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
                    nom::character::complete::line_ending // and each row is followed by a newline.
                )
            )(input)
        }


        /// This is given a rectangular grid of booleans, where true means "filled in" and
        /// false means "blank". It returns true if that grid matches this Fold and false
        /// if it doesn't.
        pub fn matches(&self, fill: &Vec<Vec<bool>>) -> bool {
            assert!(fill.len() > 0);
            assert!(fill[0].len() > 0);
            assert!(fill.iter().map(|x| x.len()).all_equal());
            let is_filled_grid: Vec<Vec<bool>> = self.filled.iter()
                .map(|row| {
                    row.iter().map(|x| x.is_some()).collect()
                })
                .collect();
            is_filled_grid == *fill
        }

        /// Returns a CubeLayout which is this one but flipped in the x direction.
        fn flip_x(&self) -> Self {
            let bounds = self.bounds;
            let filled = self.filled.iter()
                .map(|row| {
                    row.iter().rev().copied().collect()
                })
                .collect();
            let wraps = self.wraps.iter().map(|x| x.flip_x()).collect();
            CubeLayout{bounds, filled, wraps}
        }

        /// Returns a CubeLayout which is this one but flipped in the y direction.
        fn flip_y(&self) -> Self {
            let bounds = self.bounds;
            let filled = self.filled.iter().rev().cloned().collect();
            let wraps = self.wraps.iter().map(|x| x.flip_y()).collect();
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
            let wraps = self.wraps.iter().map(|x| x.transpose()).collect();
            CubeLayout{bounds, filled, wraps}
        }

        /// This is given a Coord (an (x,y) pair) where the numbers count from zero
        /// and have 1 for each face. It returns the face number that represents
        /// (or panics if it isn't a valid face).
        fn coord_to_face(&self, coord: Coord) -> FaceNum {
            self.filled[coord.1][coord.0].unwrap()
        }

        /// This is given a FaceNum and it returns a Coord (an (x,y) pair) where the numbers
        /// count from zero and have 1 for each face.
        fn face_to_coord(&self, face_num: FaceNum) -> Coord {
            for y in 0..self.bounds.1 {
                for x in 0..self.bounds.0 {
                    if self.filled[y][x] == Some(face_num) {
                        return Coord(x,y)
                    }
                }
            }
            panic!("The face {:?} wasn't in the mapping.", face_num);
        }



        /// This is given an edge we are exiting over and returns the edge we are entering
        /// on, along with a bool telling whether the direction is reversed. It panics if
        /// the edge it is given isn't a valid one.
        fn matching_edge(&self, from_edge: Edge) -> (Edge, bool) {
            for wrap in self.wraps.iter() {
                if wrap.edges[0] == from_edge {
                    return (wrap.edges[1], wrap.reverse)
                }
                if wrap.edges[1] == from_edge {
                    return (wrap.edges[0], wrap.reverse)
                }
            }
            panic!("No such edge found");
        }

        // FIXME: Document this.
        // FIXME: There are steps here that break down and reassemble. I should break them out. Make it work first.
        pub fn wrap_around(&self, divisor: usize, start_pos: Coord, facing: Facing) -> (Coord, Facing) {
            println!("    wrapping around divisor={}, start {:?}, face {:?}", divisor, start_pos, facing); // FIXME: Remove
            let (pos_of_face, pos_in_face) = start_pos.div_mod(divisor);
            let face_num = self.coord_to_face(pos_of_face);
            let dist_along_face = match facing {
                Facing::Right => {
                    assert_eq!(pos_in_face.0, divisor - 1);
                    pos_in_face.1
                },
                Facing::Down => {
                    assert_eq!(pos_in_face.1, divisor - 1);
                    pos_in_face.0
                },
                Facing::Left => {
                    assert_eq!(pos_in_face.0, 0);
                    pos_in_face.1
                },
                Facing::Up => {
                    assert_eq!(pos_in_face.1, 0);
                    pos_in_face.0
                },
            };
            let (Edge(new_face_num, new_edge_facing), reverse) = self.matching_edge(Edge(face_num, facing));
            let new_facing = new_edge_facing.invert();
            let new_dist_along_face = match reverse {
                true => divisor - dist_along_face - 1,
                false => dist_along_face
            };
            let new_pos_in_face = match new_facing {
                Facing::Right => Coord(0, new_dist_along_face),
                Facing::Down => Coord(new_dist_along_face, 0),
                Facing::Left => Coord(divisor - 1, new_dist_along_face),
                Facing::Up => Coord(new_dist_along_face, divisor - 1),
            };
            let new_pos_of_face = self.face_to_coord(new_face_num);
            let new_pos = Coord::from_div_mod(divisor, new_pos_of_face, new_pos_in_face);
            (new_pos, new_facing)
        }
    }

    impl PartialEq for CubeLayout {
        fn eq(&self, other: &Self) -> bool {
            if self.bounds != other.bounds {
                false
            } else if self.filled != other.filled {
                false
            } else {
                // compare wraps without order mattering
                if self.wraps.len() != other.wraps.len() {
                    false
                } else {
                    // assuming here that all of self.wraps are unique
                    self.wraps.iter().all(|wrap| other.wraps.contains(wrap))
                }
            }
        }
    }

    impl Eq for CubeLayout {}

    impl Ratio {
        /// Returns the (x,y) size of this Ratio.
        pub fn extent(&self) -> Coord {
            match self {
                Ratio::Ratio3By4 => Coord(3,4),
                Ratio::Ratio4By3 => Coord(4,3),
                Ratio::Ratio2By5 => Coord(2,5),
                Ratio::Ratio5By2 => Coord(5,2),
            }
        }

        /// Given an (x,y) size, this returns the greatest common divisor for the numbers.
        pub fn find_divisor(coord: Coord) -> usize {
            coord.0.gcd(coord.1)
        }

        /// Returns the SupportedRatio for a given (x,y) size (not necessarily reduced to
        /// least common terms), or returns None is this does not reduce to a supported ratio.
        pub fn find_ratio(coord: Coord) -> Option<Ratio> {
            let divisor = Ratio::find_divisor(coord);
            match (coord.0 / divisor, coord.1 / divisor) {
                (3,4) => Some(Ratio::Ratio3By4),
                (4,3) => Some(Ratio::Ratio4By3),
                (2,5) => Some(Ratio::Ratio2By5),
                (5,2) => Some(Ratio::Ratio5By2),
                _ => None
            }
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

    pub static ALL_LAYOUTS: Lazy<HashMap<Ratio, Vec<CubeLayout>>> = Lazy::new(|| {
        fn all_flips_of<T>(layouts: T) -> Vec<CubeLayout>
            where T: IntoIterator<Item=CubeLayout>
        {
            layouts.into_iter()
                .flat_map(|layout| [layout.clone(), layout.flip_x()].into_iter()) // flip each in the x direction
                .flat_map(|layout| [layout.clone(), layout.flip_y()].into_iter()) // flip each in the y direction
                .collect()
        }

        HashMap::from([
            (Ratio::Ratio4By3, all_flips_of(LAYOUTS_4_BY_3.iter().cloned())),
            (Ratio::Ratio3By4, all_flips_of(LAYOUTS_4_BY_3.iter().map(|x| x.transpose()))),
            (Ratio::Ratio5By2, all_flips_of(LAYOUTS_5_BY_2.iter().cloned())),
            (Ratio::Ratio2By5, all_flips_of(LAYOUTS_5_BY_2.iter().map(|x| x.transpose()))),
        ])
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
            assert_eq!(true, fold.matches(&vec![
                vec![true , false, false, false],
                vec![true , true , true , true ],
                vec![true , false, false, false],
            ]));
            assert_eq!(false, fold.matches(&vec![
                vec![true , true, false, false],
                vec![true , true , true , true ],
                vec![true , false, false, false],
            ]));
            assert_eq!(false, fold.matches(&vec![
                vec![true , false, false, false],
                vec![true , true , false, true ],
                vec![true , false, false, false],
            ]));
            assert_eq!(false, fold.matches(&vec![
                vec![true , false, false, false],
                vec![true , true , true , true ],
                vec![true , false, false, false],
                vec![false, false, false, false],
            ]));
            let fold = &LAYOUTS_5_BY_2[0];
            assert_eq!(Coord(5,2), fold.bounds);
            assert_eq!(true, fold.matches(&vec![
                vec![false, false, true , true , true ],
                vec![true , true , true , false, false],
            ]));
        }

        #[test]
        fn test_fold_flip_x() {
            let fold = &LAYOUTS_4_BY_3[0].flip_x();
            assert_eq!(Coord(4,3), fold.bounds);
            assert_eq!(true, fold.matches(&vec![
                vec![false, false, false, true ],
                vec![true , true , true , true ],
                vec![false, false, false, true ],
            ]));
        }

        #[test]
        fn test_fold_flip_y() {
            let fold = &LAYOUTS_4_BY_3[1].flip_y();
            assert_eq!(Coord(4,3), fold.bounds);
            assert_eq!(true, fold.matches(&vec![
                vec![true , false, false, false],
                vec![true , true , true , true ],
                vec![false, true, false, false ],
            ]));
        }

        #[test]
        fn test_fold_transpose() {
            let fold = &LAYOUTS_4_BY_3[0].transpose();
            assert_eq!(Coord(3,4), fold.bounds);
            assert_eq!(true, fold.matches(&vec![
                vec![true , true , true ],
                vec![false, true , false],
                vec![false, true , false],
                vec![false, true , false],
            ]));
        }

        /// A test of one that's been y-swapped and x-swapped.
        #[test]
        fn test_find_swapped() {
            let target = vec![
                vec![false, false, false, true ],
                vec![true , true , true , true ],
                vec![false, true , false, false],
            ];
            for (i, layout) in ALL_LAYOUTS.get(&Ratio::Ratio4By3).unwrap().iter().enumerate() {
                if layout.matches(&target) {
                    assert_eq!(i, 11); // should be the 11th pattern in the list
                    let expect_layout = CubeLayout::from_visual("\
                        ...5\n\
                        4321\n\
                        .0..\n\
                    ", "0D=!1D 0L=!4D 0R==2D 1R==4L 2U==5L 3U=!5U 4U=!5R");
                    assert_eq!(*layout, expect_layout);
                    return;
                }
            }
            panic!("didn't find a match");
        }

        #[test]
        fn test_transpose() {
            let layout = CubeLayout::from_visual("\
                .0..\n\
                .123\n\
                45..\n\
            ", "0R=!2U 0L=!4L 0U=!3U 1L==4U 2D==5R 3R==4D 3D=!5D");
            let expect_layout = CubeLayout::from_visual("\
                ..4\n\
                015\n\
                .2.\n\
                .3.\n\
            ", "0D=!2L 0U=!4U 0L=!3L 1U==4L 2R==5D 3D==4R 3R=!5R");
            assert_eq!(layout.transpose(), expect_layout);
        }

        #[test]
        fn test_find_in_list() {
            let target = vec![
                vec![false, false, true ],
                vec![true , true , true ],
                vec![false, true , false],
                vec![false, true , false],
            ];
            for (i, layout) in ALL_LAYOUTS.get(&Ratio::Ratio3By4).unwrap().iter().enumerate() {
                if layout.matches(&target) {
                    assert_eq!(24, i); // should find as the 24th pattern in the list
                    let expect_layout = CubeLayout::from_visual("\
                        ..4\n\
                        015\n\
                        .2.\n\
                        .3.\n\
                    ", "0D=!2L 0U=!4U 0L=!3L 1U==4L 2R==5D 3D==4R 3R=!5R");
                    assert_eq!(*layout, expect_layout);
                    return;
                }
            }
            panic!("didn't find a match");
        }
    }
}



/// A module for taking a Grid and supporting movement on it according to a CubeLayout.
mod cubewrap {
    use anyhow::anyhow;
    use crate::compute::{Coord, WrapAroundBehavior, Facing};
    use crate::parse::{GridElem, MapOfBoard};
    use crate::cubelayout::{CubeLayout, Ratio, ALL_LAYOUTS};


    // FIXME: Remove this -- move the fields into WrapAroundCubeBehavior and return a tuple.
    /// This contains the CubeLayout along with the sizing that fits a specific
    /// MapOfBoard.
    #[derive(Debug)]
    struct LayoutFit {
        layout: &'static CubeLayout,
        divisor: usize,
    }

    #[derive(Debug)]
    pub struct WrapAroundCubeBehavior {
        layout_fit: LayoutFit,
    }



    /// Given a MapOfBoard, this finds the CubeLayout that matches it along with the
    /// scaling (a LayoutFit), or returns an Error.
    fn find_layout_fit(map_of_board: &MapOfBoard) -> Result<LayoutFit, anyhow::Error> {
        let dims = Coord(map_of_board.width(), map_of_board.height());
        let ratio: Ratio = Ratio::find_ratio(dims).ok_or(anyhow!("Not a valid ratio"))?; // error if we don't find one
        let divisor = Ratio::find_divisor(dims);
        let extent = ratio.extent();
        let fills: Vec<Vec<bool>> = (0..extent.1).map(|y| {
            (0..extent.0).map(|x| {
                !matches!(map_of_board.get_at(divisor * x, divisor * y), GridElem::Blank)
            }).collect()
        }).collect();
        for layout in ALL_LAYOUTS.get(&ratio).unwrap() {
            if layout.matches(&fills) {
                return Ok(LayoutFit{layout, divisor})
            }
        }
        Err(anyhow!("Did not match any layout"))
    }


    impl<'a> WrapAroundCubeBehavior {
        pub fn new(map: &'a MapOfBoard) -> Result<Self, anyhow::Error> {
            let layout_fit = find_layout_fit(map)?;
            Ok(WrapAroundCubeBehavior{layout_fit})
        }
    }


    impl<'a> WrapAroundBehavior for WrapAroundCubeBehavior {
        fn wrap_around(&self, start_pos: Coord, facing: Facing) -> (Coord, Facing) {
            self.layout_fit.layout.wrap_around(self.layout_fit.divisor, start_pos, facing)
        }
    }



    #[cfg(test)]
    mod tests {
        use super::*;
        use itertools::Itertools;


        /// Takes as input a literal from the code where the start of each line is marked with
        /// "|" and the end of each line is marked with "\n\", and cleans it up. Needed because
        /// we want to write indented literal blocks where leading space is significant. Is
        /// intended to be used on source code literals, so it panics if it encounters
        /// invalid stuff.
        fn read_block_indent(input: &str) -> String {
            input.lines()
                .map(|x| x.trim_start().strip_prefix("|").unwrap())
                .join("\n")
        }


        #[test]
        fn test_read_block_indent() {
            let literal = "\
                |ABC
                |DEF
                |   GHI
                |
                |JKL
                |";
            let expect = "ABC\nDEF\n   GHI\n\nJKL\n";
            assert_eq!(read_block_indent(literal), expect);
        }

        #[test]
        fn test_error_for_bad_map() {
            let map_data = read_block_indent("\
                |    .#
                |    .#
                |.#...#
                |..#...
                |    .#..
                |");
            let (rest, map_of_board) = MapOfBoard::parse(&map_data).unwrap();
            assert_eq!(rest, "");
            let layout_or_err = find_layout(&map_of_board);
            assert!(matches!(layout_or_err, Err(_)));
        }

        #[test]
        fn test_with_sample_map() -> Result<(), anyhow::Error>{
            let map_data = read_block_indent("\
                |        ...#
                |        .#..
                |        #...
                |        ....
                |...#.......#
                |........#...
                |..#....#....
                |..........#.
                |        ...#....
                |        .....#..
                |        .#......
                |        ......#.
                |");
            let (rest, map_of_board) = MapOfBoard::parse(&map_data).unwrap();
            assert_eq!(rest, "");
            let layout = find_layout(&map_of_board)?;
            let expect_layout = CubeLayout::from_visual("\
                ..0.\n\
                321.\n\
                ..54\n\
            ", "0L==2U 0R=!4R 0U=!3U 1R=!4U 2D=!5L 3L=!4D 3D=!5D");
            assert_eq!(*layout, expect_layout);
            Ok(())
        }

    }
}

// ======= main() =======

use crate::parse::{input, InputData};
use crate::compute::{Grid, KeepGoingBehavior};
use crate::cubewrap::WrapAroundCubeBehavior;


fn part_a(input: &InputData) {
    println!("\nPart a:");
    let wrap_around_behavior = KeepGoingBehavior::new(&input.map);
    let mut grid = Grid::new(&input.map, &wrap_around_behavior);
    grid.apply_steps(&input.steps);
    println!("Password = {}", grid.password());
}


fn part_b(input: &InputData) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    let wrap_around_behavior = WrapAroundCubeBehavior::new(&input.map)?;
    let mut grid = Grid::new(&input.map, &wrap_around_behavior);
    grid.apply_steps(&input.steps);
    println!("Password = {}", grid.password());
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data)?;
    Ok(())
}

