use std::fmt::{Debug, Display, Formatter};
use anyhow;
use advent_lib::grid::{Coord, Grid, Direction};
use std::collections::HashSet;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Copy, Clone)]
pub enum BeamControl {
    Space, VSplitter, HSplitter, BackMirror, ForeMirror
}

pub type BeamGrid = Grid<BeamControl>;


/// An error type when reading a character which should be a BeamControl.
#[derive(Debug)]
pub struct InvalidBeamControlCharacter(char);


type Input = BeamGrid;


impl BeamControl {
    /// Returns a list of the directions you exit this BeamControl if you enter it from
    /// dir.
    pub fn split(&self, dir: Direction) -> Vec<Direction> {
        use BeamControl::*;
        use Direction::*;
        match (self, dir) {
            (Space, d) => vec![d],
            (VSplitter, North) => vec![North],
            (VSplitter, South) => vec![South],
            (VSplitter, _) => vec![North, South],
            (HSplitter, East) => vec![East],
            (HSplitter, West) => vec![West],
            (HSplitter, _) => vec![East, West],
            (BackMirror, North) => vec![West],
            (BackMirror, South) => vec![East],
            (BackMirror, East) => vec![South],
            (BackMirror, West) => vec![North],
            (ForeMirror, North) => vec![East],
            (ForeMirror, South) => vec![West],
            (ForeMirror, East) => vec![North],
            (ForeMirror, West) => vec![South],
        }
    }
}

impl Display for BeamControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use BeamControl::*;
        write!(f, "{}", match self {
            Space => '.',
            VSplitter => '|',
            HSplitter => '-',
            BackMirror => '\\',
            ForeMirror => '/',
        })
    }
}

impl TryFrom<char> for BeamControl {
    type Error = InvalidBeamControlCharacter;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use BeamControl::*;
        match value {
            '.' => Ok(Space),
            '|' => Ok(VSplitter),
            '-' => Ok(HSplitter),
            '\\' => Ok(BackMirror),
            '/' => Ok(ForeMirror),
            _ => Err(InvalidBeamControlCharacter(value)),
        }
    }
}

impl Display for InvalidBeamControlCharacter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for InvalidBeamControlCharacter {}




mod parse {
    use super::{Input, BeamGrid, BeamControl};
    use std::fs;

    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_16.txt")?;
        parse_beam_grid(&s)
    }

    impl BeamControl {
        pub fn parse(c: char) -> Self {
            use BeamControl::*;
            match c {
                '.' => Space,
                '|' => VSplitter,
                '-' => HSplitter,
                '\\' => BackMirror,
                '/' => ForeMirror,
                _ => panic!("invalid beam control, '{}'", c),
            }
        }
    }

    fn parse_beam_grid(input: &str) -> Result<BeamGrid, anyhow::Error> {
        Ok(BeamGrid::from_char_string(input)?)
    }

}


// ======= Compute =======


/// Represents the tip of a beam. It contains the spot the beam will go to and the direction
/// it is going.
#[derive(Debug)]
struct BeamTip {
    destination: Coord,
    dir: Direction,
}

#[derive(Debug)]
struct ActiveBeamTable<'a> {
    controls: &'a BeamGrid,
    beams: Vec<BeamTip>,
    illuminated: Grid<HashSet<Direction>>, // for each cell, which directions (if any) have had a beam
}

const STARTING_TIP: BeamTip = BeamTip{destination: Coord(0,0), dir: Direction::East};

impl<'a> ActiveBeamTable<'a> {
    fn new(controls: &'a BeamGrid) -> Self {
        let beams = vec![STARTING_TIP];
        let illuminated: Grid<HashSet<Direction>> = Grid::new_default(controls.bound());
        Self{controls, beams, illuminated}
    }

    /// Returns true if there is a beam that can advance, false if not.
    fn can_advance(&self) -> bool {
        !self.beams.is_empty()
    }

    /// Advances a single beam. If there are no beams to advance, this panics.
    fn advance_one(&mut self) {
        let tip = self.beams.pop().expect("invoked advance_one() when we couldn't advance");
        self.illuminated.get_mut(tip.destination).insert(tip.dir); // mark this has happened!

        let new_dirs = self.controls.get(tip.destination).split(tip.dir);
        for new_dir in new_dirs {
            match tip.destination.bounded_step(new_dir, self.controls.bound()) {
                None => {}
                Some(new_destination) => {
                    if !self.illuminated.get(new_destination).contains(&new_dir) {
                        self.beams.push(BeamTip{destination: new_destination, dir: new_dir});
                    }
                }
            }
        }
    }

    /// Advances everything all the way to the end.
    fn advance_all(&mut self) {
        while self.can_advance() {
            self.advance_one();
        }
    }

    /// Prints out the locations that are energized.
    #[allow(dead_code)]
    fn print_energy(&self) {
        println!();
        for y in 0..self.illuminated.bound().y() {
            for x in 0..self.illuminated.bound().x() {
                print!("{}", if self.illuminated.get(Coord(x,y)).is_empty() {'.'} else {'#'});
            }
            println!();
        }
    }

    fn count_energized(&self) -> usize {
        self.illuminated.iter()
            .map(|set| if set.is_empty() {0} else {1})
            .sum()
    }
}

// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let mut active = ActiveBeamTable::new(input);
    active.advance_all();
    let count = active.count_energized();
    println!("There are {} that are active.", count);
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
