
extern crate anyhow;


// ======= Constants =======


// ======= Parsing =======

mod parse {

    use std::fmt::Debug;
    use std::fs;
    use std::ops::Add;
    use std::collections::HashMap;
    use itertools::Itertools;
    use itertools::iproduct;
    use nom::{
        IResult,
        branch::alt,
        combinator::{value, map},
        character::complete::{char, line_ending},
        sequence::{delimited, tuple},
        multi::{many0, many1},
    };


    pub fn input() -> Result<Grove, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_24.txt")?;
        match Grove::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    pub type Num = u16;

    #[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
    pub struct Coord(pub Num, pub Num);

    #[derive(Debug, Copy, Clone)]
    pub enum Orientation {
        Horizontal, Vertical
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Direction {
        N, S, E, W,
    }

    #[derive(Debug, Clone)] // FIXME: Remove the Clone
    pub struct Blizzard {
        pub dir: Direction,
        loc: Coord,
    }

    #[derive(Debug)]
    pub struct Grove {
        pub size: Coord, // width and height of inside. (0,0) is the lower left
        start_x: Num,
        goal_x: Num,
        h_blizzards: HashMap<Num, Vec<Blizzard>>,
        v_blizzards: HashMap<Num, Vec<Blizzard>>,
    }



    impl Orientation {
        /// Take a Coord and split it into "with" and "against" fields.
        pub fn split(&self, coord: Coord) -> (Num, Num) {
            match self {
                Orientation::Horizontal => (coord.1, coord.0),
                Orientation::Vertical => (coord.0, coord.1),
            }
        }

        /// Take "with" and "against" fields and combine them into a Coord.
        pub fn join(&self, with: Num, against: Num) -> Coord {
            match self {
                Orientation::Horizontal => Coord(against, with),
                Orientation::Vertical => Coord(with, against),
            }
        }
    }

    impl Direction {
        pub fn orientation(&self) -> Orientation {
            match self {
                Direction::N => Orientation::Vertical,
                Direction::S => Orientation::Vertical,
                Direction::E => Orientation::Horizontal,
                Direction::W => Orientation::Horizontal,
            }
        }
    }

    impl Coord {
        /// Returns the (taxicab) distance between this and other.
        pub fn taxi_dist(&self, other: Coord) -> Num {
            self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
        }

        /// Returns the neighbors of this Coord, excluding any that would be outside
        /// of the given bounds.
        pub fn bounded_neighbors(&self, bounds: Coord) -> Vec<Coord> {
            let mut answer = Vec::with_capacity(4);
            let north = *self + Direction::N;
            if north.1 < bounds.1 {
                answer.push(north);
            }
            if self.1 > 0 {
                answer.push(*self + Direction::S);
            }
            let east = *self + Direction::E;
            if east.0 < bounds.0 {
                answer.push(east);
            }
            if self.0 > 0 {
                answer.push(*self + Direction::W);
            }
            answer
        }
    }

    /// Implement adding a Direction to a Coord (stepping one step in that direction). If it
    /// brings the value below 0, this will panic.
    impl Add<Direction> for Coord {
        type Output = Self;

        fn add(self, rhs: Direction) -> Self::Output {
            match rhs {
                Direction::N => Coord(self.0, self.1 + 1),
                Direction::S => Coord(self.0, self.1 - 1), // can panic
                Direction::E => Coord(self.0 + 1, self.1),
                Direction::W => Coord(self.0 - 1, self.1), // can panic
            }
        }
    }

    impl Blizzard {
        /// Returns the orientation of this Blizzard.
        pub fn orientation(&self) -> Orientation {
            self.dir.orientation()
        }

        /// Returns the position of this Blizzard at time 'time' if embedded in a grove
        /// with size 'size'.
        pub fn future_pos(&self, time: Num, size: Coord) -> Coord {
            let orientation = self.orientation();
            let (fixed_val, start_val) = orientation.split(self.loc);
            let (_, bound) = orientation.split(size);
            let moving_dest = match self.dir {
                Direction::N | Direction::E => (start_val + time) % bound,
                Direction::S | Direction::W => (start_val + bound - (time % bound)) % bound,
            };
            orientation.join(fixed_val, moving_dest)
        }
    }

    impl Grove {
        /// Create an instance from what we read.
        fn from_data(data: (Num, Vec<Vec<Option<Direction>>>, Num)) -> Self {
            let (start_x, rows, goal_x) = data;
            let height = rows.len();
            if height == 0 {
                panic!("ElfPlaces must have at least one row.");
            }
            let width = rows[0].len();
            if width == 0 {
                panic!("ElfPlaces must be at least 1 column wide.");
            }
            if ! rows.iter().map(|row| row.len()).all_equal() {
                panic!("ElfGrid must be rectangular.");
            }
            let size = Coord(width.try_into().unwrap(), height.try_into().unwrap());
            let xs = 0..width;
            let ys = 0..height;
            let all_blizzards = iproduct!(xs, ys).filter_map(|(x,y)| {
                match rows.get(y).unwrap().get(x).unwrap() {
                    None => None,
                    Some(dir) => {
                        let x = x.try_into().unwrap();
                        let y = (height - y - 1).try_into().unwrap(); // reverse so (0,0) is bottom left
                        let dir = (*dir).clone();
                        Some(Blizzard{dir, loc: Coord(x,y)})
                    },
                }
            });
            let mut h_blizzards: HashMap<Num, Vec<Blizzard>> = HashMap::new();
            let mut v_blizzards: HashMap<Num, Vec<Blizzard>> = HashMap::new();
            for blizzard in all_blizzards {
                let orientation = blizzard.orientation();
                let (fixed_val, _start_val) = orientation.split(blizzard.loc);
                let hash_map = match orientation {
                    Orientation::Horizontal => &mut h_blizzards,
                    Orientation::Vertical => &mut v_blizzards,
                };
                hash_map.entry(fixed_val).or_insert(Vec::new()).push(blizzard.clone());
            }
            Grove{size, start_x, goal_x, h_blizzards, v_blizzards}
        }


        /// Parses the top or bottom row of a blizzard basin and returns the x coordinate
        /// of the gap.
        pub fn parse_exit_row(input: &str) -> IResult<&str, Num> {
            map(
                tuple((
                    char('#'), // left wall
                    many0(char('#')), // leading wall
                    char('.'), // open space
                    many0(char('#')), // trailing wall
                    line_ending, // end of line
                )),
                |(_, leading, _, _, _)| leading.len().try_into().unwrap()
            )(input)
        }

        pub fn parse_body_row(input: &str) -> IResult<&str, Vec<Option<Direction>>> {
            delimited(
                char('#'),
                many1(
                    alt((
                        value(None, char('.')),
                        value(Some(Direction::N), char('^')),
                        value(Some(Direction::S), char('v')),
                        value(Some(Direction::E), char('>')),
                        value(Some(Direction::W), char('<')),
                    ))
                ),
                tuple((char('#'), line_ending)),
            )(input)
        }

        /// Parses the whole ElfPlaces
        pub fn parse(input: &str) -> IResult<&str, Self> {
            map(
                tuple((
                    Self::parse_exit_row,
                    many1( Self::parse_body_row ),
                    Self::parse_exit_row,
                )),
                |data| Self::from_data(data)
            )(input)
        }

        /// Returns the Coord where one starts out
        pub fn start_coord(&self) -> Coord {
            Coord(self.start_x, self.size.1)
        }

        /// Returns the Coord that is 1 move away from winning (does NOT include the final move)
        pub fn goal_coord(&self) -> Coord {
            Coord(self.goal_x, 0)
        }

        /// Returns an iterator that iterates through all the blizzards.
        pub fn all_blizzards(&self) -> impl Iterator<Item=&Blizzard> {
            let h_blizzards = self.h_blizzards.values().flatten();
            let v_blizzards = self.v_blizzards.values().flatten();
            h_blizzards.chain(v_blizzards)
        }

        // FIXME: It would be a lot better if we only looked at the ones in the right row.
        //   Doing that would require a data structure change.
        /// Returns true if the given coord is blocked at the given time.
        pub fn is_unblocked(&self, coord: Coord, time: Num) -> bool {
            let h_blizzards = self.h_blizzards.get(&coord.1).into_iter().flatten();
            let v_blizzards = self.v_blizzards.get(&coord.0).into_iter().flatten();
            h_blizzards.chain(v_blizzards).all(|blizzard| {
                blizzard.future_pos(time, self.size) != coord
            })
        }
    }

}


// ======= Part 1 Compute =======

mod compute {
    use std;
    use std::fmt::{Display, Formatter};
    use std::hash::{Hash, Hasher};
    use crate::parse::{Grove, Coord, Num, Direction};
    use advent_lib::astar;

    const PRINT_DETAILS: bool = false;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Step {
        Wait,
        MoveTo(Coord),
    }

    /// A state of the traversal.
    #[derive(Debug, Clone)]
    struct GroveState<'a> {
        grove: &'a Grove, // immutable reference
        time: Num, // 0 is sitting on the start location before we go anywhere
        loc: Coord, // current location of the traveler
    }

    /// The function that solves it.
    pub fn solve(grove: &Grove) -> Option<Vec<Step>> {
        let initial_state = GroveState{grove, time: 0, loc: grove.start_coord()};
        let print_every_n_moves = 1;
        astar::solve_with_astar(&initial_state, print_every_n_moves)
    }



    impl<'a> GroveState<'a> {
        /// Returns true if the given coord is blocked at the given time.
        fn is_unblocked(&self, coord: Coord, time: Num) -> bool {
            self.grove.is_unblocked(coord, time)
        }

        /// This can be used to get a grid of chars which tell where the blizzards at
        /// a time that's extra_t in the future.
        fn chars_for_blizzards(&self, extra_time: Num) -> Vec<Vec<char>> {
            let mut chars: Vec<Vec<char>> = vec![vec!['.'; self.grove.size.0.into()]; self.grove.size.1.into()];
            for blizzard in self.grove.all_blizzards() {
                let coord = blizzard.future_pos(self.time + extra_time, self.grove.size);
                let c_ref: &mut char = chars.get_mut(coord.1 as usize).unwrap().get_mut(coord.0 as usize).unwrap();
                *c_ref = match *c_ref {
                    '.' => match blizzard.dir {
                        Direction::N => '^',
                        Direction::S => 'v',
                        Direction::E => '>',
                        Direction::W => '<',
                    },
                    '^' | 'v' | '>' | '<' => '2',
                    '2' => '3',
                    '3' => '4',
                    '4' => '5',
                    '5' => '6',
                    '6' => '7',
                    '7' => '8',
                    '8' => '9',
                    '9' => '+',
                    '+' => '+',
                    _ => panic!("Unexpected value"),
                };
            }
            chars
        }

        fn chars_for_header(&self) -> Vec<char> {
            let mut header_chars: Vec<char> = vec!['#'; (self.grove.size.0 + 2) as usize];
            header_chars[(self.grove.start_coord().0 + 1) as usize] = '.';
            header_chars
        }

        fn chars_for_footer(&self) -> Vec<char> {
            let mut footer_chars: Vec<char> = vec!['#'; (self.grove.size.0 + 2) as usize];
            footer_chars[(self.grove.goal_coord().0 + 1) as usize] = '.';
            footer_chars
        }

        /// Writes out this GroveState in picture form.
        fn write_flex_picture(&self, f: &mut Formatter<'_>, extra_time: Num, show_expedition: bool) -> std::fmt::Result {
            // --- blank line ---
            writeln!(f)?;

            // --- header ---
            let mut header_chars = self.chars_for_header();
            if show_expedition {
                if self.loc.1 == self.grove.size.1 {
                    header_chars[(self.grove.start_coord().0 + 1) as usize] = 'E';
                }
            }
            for c in header_chars {
                write!(f, "{}", c)?;
            }
            writeln!(f)?;

            // --- body ---
            let mut chars = self.chars_for_blizzards(extra_time);
            if show_expedition {
                if self.loc.1 < self.grove.size.1 {
                    assert!(chars[self.loc.1 as usize][self.loc.0 as usize] == '.');
                    chars[self.loc.1 as usize][self.loc.0 as usize] = 'E';
                }
            }
            for row in chars.iter().rev() {
                write!(f, "#")?;
                for c in row {
                    write!(f, "{}", c)?;
                }
                write!(f, "#\n")?;
            }

            // --- footer ---
            let footer_chars = self.chars_for_footer();
            for c in footer_chars {
                write!(f, "{}", c)?;
            }
            writeln!(f)?;

            // --- blank line ---
            writeln!(f)
        }

        /// Writes out this GroveState in picture form.
        fn write_picture(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            self.write_flex_picture(f, 0, true)
        }


        /// Writes out the blizzard positions for this GroveState at extra_time steps in
        /// the future.
        fn write_blizzards(&self, f: &mut Formatter<'_>, extra_time: Num) -> std::fmt::Result {
            self.write_flex_picture(f, extra_time, false)
        }
    }

    impl<'a> Display for GroveState<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "At {} in ({},{})", self.time, self.loc.0, self.loc.1)?;
            if PRINT_DETAILS {
                self.write_picture(f)?;
                write!(f, "NEXT:")?;
                self.write_blizzards(f, 1)?;
            }
            Ok(())
        }
    }

    impl<'a> Hash for GroveState<'a> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.time.hash(state);
            self.loc.hash(state);
        }
    }

    impl<'a> PartialEq<Self> for GroveState<'a> {
        fn eq(&self, other: &Self) -> bool {
            assert!(std::ptr::eq(self.grove, other.grove));
            self.time == other.time && self.loc == other.loc
        }
    }

    impl<'a> Eq for GroveState<'a> {}


    impl<'a> astar::State for GroveState<'a> {
        type TMove = Step;

        fn is_winning(&self) -> bool {
            self.loc == self.grove.goal_coord()
        }

        fn min_moves_to_win(&self) -> usize {
            self.loc.taxi_dist(self.grove.goal_coord()).into()
        }

        fn avail_moves(&self) -> Vec<Self::TMove> {
            let mut answer = Vec::with_capacity(5);
            if self.loc == self.grove.start_coord() {
                // In the entry location is a special case; it can only move South
                let coord = self.loc + Direction::S;
                if self.is_unblocked(coord, self.time + 1) {
                    answer.push(Step::MoveTo(coord));
                }
            } else {
                // Anywhere else it can move to any space that will be free
                for coord in self.loc.bounded_neighbors(self.grove.size) {
                    if self.is_unblocked(coord, self.time + 1) {
                        answer.push(Step::MoveTo(coord));
                    }
                }
            }
            if self.loc == self.grove.start_coord() + Direction::S {
                // Being below the entry location is another special case; it can ALSO move to entry
                answer.push(Step::MoveTo(self.grove.start_coord()));
            }
            if self.is_unblocked(self.loc, self.time + 1) {
                answer.push(Step::Wait);
            }
            if PRINT_DETAILS {
                println!("The available moves from this state are: {:?}", answer);
            }
            answer
        }

        fn enact_move(&self, mv: &Self::TMove) -> Self {
            let grove = self.grove;
            let time = self.time + 1;
            let loc = match mv {
                Step::Wait => self.loc,
                Step::MoveTo(coord) => *coord,
            };
            GroveState{grove, time, loc}
        }
    }

}


// ======= Part 2 Compute =======


// ======= main() =======

use parse::{input, Grove};
use compute::solve;



fn part_a(grove: &Grove) {
    println!("\nPart a:");
    let solution = solve(grove);
    match solution {
        None => println!("There is no solution."),
        Some(path) => println!("Solved in {} steps: {:?} then down.", path.len() + 1, path),
    }
}


fn part_b(_input: &Grove) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
