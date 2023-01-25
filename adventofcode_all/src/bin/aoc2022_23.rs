
extern crate anyhow;


// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fmt::Debug;
    use std::fs;
    use anyhow::anyhow;
    use itertools::Itertools;
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::tag,
        combinator::{value, map},
        character::complete::line_ending,
        sequence::terminated,
        multi::many1,
    };


    pub fn input() -> Result<ElfPlaces, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_23.txt")?;
        match ElfPlaces::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    #[derive(Debug)]
    pub struct ElfPlaces {
        data: Vec<Vec<bool>>,
    }



    impl ElfPlaces {
        /// Create an instance from what we read.
        fn from_rows(data: Vec<Vec<bool>>) -> Result<Self, anyhow::Error> {
            let height = data.len();
            if height == 0 {
                return Err(anyhow!("ElfGrid must have at least one row."));
            }
            let width = data[0].len();
            if width == 0 {
                return Err(anyhow!("ElfGrid must be at least 1 column wide."));
            }
            if ! data.iter().map(|row| row.len()).all_equal() {
                return Err(anyhow!("ElfGrid must be rectangular."));
            }
            Ok(ElfPlaces{data})
        }

        /// Accessor for width
        pub fn width(&self) -> usize {
            self.data.first().unwrap().len()
        }

        /// Accessor for height
        pub fn height(&self) -> usize {
            self.data.len()
        }

        /// Retrieve from an (x,y) location
        pub fn get_at(&self, x: usize, y: usize) -> bool {
            self.data[y][x]
        }

        /// Parses the whole MapOfBoard
        pub fn parse(input: &str) -> IResult<&str, Self> {
            map(
                many1(
                    terminated(
                        many1(
                            alt((
                                value(true, tag("#")),
                                value(false, tag(".")),
                            ))
                        ),
                        line_ending
                    )
                ),
                |rows| {
                    // try it, but panic on errors because I don't know enough to return a parse error
                    match ElfPlaces::from_rows(rows) {
                        Err(e) => panic!("{}", e),
                        Ok(elf_places) => elf_places
                    }
                }
            )(input)
        }
    }

}


// ======= Part 1 Compute =======

mod compute {
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};
    use std::iter::zip;
    use crate::parse::ElfPlaces;

    #[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
    enum PrimaryDirection {
        N, S, W, E
    }

    #[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
    enum Direction {
        NW, N, NE,
        W,     E,
        SW, S, SE,
    }

    #[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
    struct Coord(i16, i16);

    // FIXME: This could probably go away since it's just wrapping one thing.
    #[derive(Debug, Copy, Clone)]
    struct Elf {
        pos: Coord,
    }

    #[derive(Debug)]
    pub struct ElfGrid {
        elves: Vec<Elf>,
        preferred_pdir: PrimaryDirection,
        prev_moves: usize,
    }




    impl PrimaryDirection {
        /// Gives the next PrimaryDirection in the rotation order.
        fn next(&self) -> Self {
            match self {
                PrimaryDirection::N => PrimaryDirection::S,
                PrimaryDirection::S => PrimaryDirection::W,
                PrimaryDirection::W => PrimaryDirection::E,
                PrimaryDirection::E => PrimaryDirection::N,
            }
        }

        /// Converts a primary direction (N, S, E, or W) into 3 directions to check.
        fn leaning(&self) -> [Direction; 3] {
            match self {
                PrimaryDirection::N => [Direction::N, Direction::NE, Direction::NW],
                PrimaryDirection::S => [Direction::S, Direction::SE, Direction::SW],
                PrimaryDirection::W => [Direction::W, Direction::NW, Direction::SW],
                PrimaryDirection::E => [Direction::E, Direction::NE, Direction::SE],
            }
        }
    }


    impl Direction {
        fn all() -> impl Iterator<Item = Direction> {
            [
                Direction::NW, Direction::N, Direction::NE,
                Direction::W,                Direction::E,
                Direction::SW, Direction::S, Direction::SE,
            ].iter().copied()
        }
    }

    impl From<PrimaryDirection> for Direction {
        fn from(pdir: PrimaryDirection) -> Self {
            match pdir {
                PrimaryDirection::N => Direction::N,
                PrimaryDirection::S => Direction::S,
                PrimaryDirection::W => Direction::W,
                PrimaryDirection::E => Direction::E,
            }
        }
    }


    impl Coord {
        /// Returns the coord that is one step in a given direction.
        fn step(&self, dir: Direction) -> Coord {
            match dir {
                Direction::NW => Coord(self.0 - 1, self.1 - 1),
                Direction::N => Coord(self.0, self.1 - 1),
                Direction::NE => Coord(self.0 + 1, self.1 - 1),
                Direction::W => Coord(self.0 - 1, self.1),
                Direction::E => Coord(self.0 + 1, self.1),
                Direction::SW => Coord(self.0 - 1, self.1 + 1),
                Direction::S => Coord(self.0, self.1 + 1),
                Direction::SE => Coord(self.0 + 1, self.1 + 1),
            }
        }
    }


    impl ElfGrid {
        /// Creates an ElfGrid from an ElfPlaces.
        pub fn new(elf_places: &ElfPlaces) -> Self {
            let mut elves = Vec::new();
            for y in 0..elf_places.height() {
                for x in 0..elf_places.width() {
                    if elf_places.get_at(x,y) {
                        elves.push(Elf{ pos: Coord(x as i16, y as i16)});
                    }
                }
            }
            let preferred_pdir = PrimaryDirection::N;
            let prev_moves = elves.len();
            ElfGrid{elves, preferred_pdir, prev_moves}
        }

        /// Returns true if there's an elf at this location; false if not.
        fn has_elf(&self, coord: Coord) -> bool {
            // FIXME: If we have a lot of elves, it might be wise to change data structures so this is more efficient
            for elf in self.elves.iter() {
                if elf.pos == coord {
                    return true;
                }
            }
            false
        }

        /// Returns the bounding rectangle: [min_x, max_x, min_y, max_y]
        fn get_bounds(&self) -> [i16; 4] {
            assert!(! self.elves.is_empty());
            [
                self.elves.iter().map(|elf| elf.pos.0).min().unwrap(),
                self.elves.iter().map(|elf| elf.pos.0).max().unwrap(),
                self.elves.iter().map(|elf| elf.pos.1).min().unwrap(),
                self.elves.iter().map(|elf| elf.pos.1).max().unwrap(),
            ]
        }

        fn proposal(&self, elf: &Elf) -> Option<Coord> {
            // --- consider all neighbors ---
            if Direction::all().all(|dir| ! self.has_elf(elf.pos.step(dir))) {
                // --- if there are no neighbors, go nowhere ---
                None
            } else {
                // --- if there are no neighbors, consider each primary direction in order ---
                let mut pdir = self.preferred_pdir;
                for _ in 0..4 {
                    if pdir.leaning().iter().all(|dir| ! self.has_elf(elf.pos.step(*dir))) {
                        // --- if all of them are empty, go that way ---
                        return Some(elf.pos.step(pdir.into()));
                    }
                    pdir = pdir.next();
                }
                // NOTE: The problem didn't say what to do if it had neighbors, but no direction
                //   was proposed. I'm assuming it should propose to stay put. That seems to
                //   match the examples.
                return None;
            }
        }

        /// Performs one round of position updates.
        pub fn perform_round(&mut self) {
            let mut proposals: Vec<Option<Coord>> = Vec::with_capacity(self.elves.len());
            let mut proposal_count: HashMap<Coord, usize> = HashMap::new();

            // --- first half: get proposals ---
            for elf in self.elves.iter() {
                let proposal = self.proposal(elf);
                proposals.push(proposal);
                if let Some(coord) = proposal {
                    *proposal_count.entry(coord).or_insert(0) += 1;
                }
            }

            // --- second half: make moves ---
            let mut move_count = 0;
            for (elf, proposal) in zip(self.elves.iter_mut(), proposals) {
                if let Some(coord) = proposal {
                    if *proposal_count.get(&coord).unwrap() == 1 {
                        elf.pos = coord;
                        move_count += 1;
                    }
                }
            }

            // --- third step: rotate preferred_pdir ---
            self.preferred_pdir = self.preferred_pdir.next();
            self.prev_moves = move_count;
        }

        /// Performs rounds until a round where no on moves. Returns the number of rounds.
        pub fn perform_until_no_moves(&mut self) -> usize {
            let mut count = 0;
            println!("at round {count} there are {} open spaces and {} moved", self.empty_ground(), self.prev_moves); // FIXME: Keep this?
            loop {
                self.perform_round();
                count += 1;
                println!("after round {count} there are {} open spaces and {} moved", self.empty_ground(), self.prev_moves); // FIXME: Keep this?
                if self.prev_moves == 0 {
                    return count;
                }
            }
        }

        /// Returns the number of empty ground spaces within the bounding rectangle.
        pub fn empty_ground(&self) -> usize {
            let [min_x, max_x, min_y, max_y] = self.get_bounds();
            let area: usize = ((1 + max_x - min_x) as usize) * ((1 + max_y - min_y) as usize);
            area - self.elves.len()
        }
    }

    impl Display for ElfGrid {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let [min_x, max_x, min_y, max_y] = self.get_bounds();
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let c = if self.has_elf(Coord(x,y)) {'#'} else {'.'};
                    write!(f, "{}", c)?;
                }
                writeln!(f)?;
            }
            Ok(())
        }
    }
}


// ======= Part 2 Compute =======


// ======= main() =======

use parse::{input, ElfPlaces};
use compute::ElfGrid;

const PRINT_GRID: bool = false;


fn part_a(input: &ElfPlaces) {
    println!("\nPart a:");
    let mut elf_grid = ElfGrid::new(input);
    if PRINT_GRID { println!("elf_grid:\n{}", elf_grid); }
    for i in 1..=10 {
        elf_grid.perform_round();
        if PRINT_GRID { println!("{elf_grid}"); }
        println!("after round {i} there are {} open spaces", elf_grid.empty_ground());
    }
}


fn part_b(input: &ElfPlaces) {
    println!("\nPart b:");
    let mut elf_grid = ElfGrid::new(input);
    if PRINT_GRID { println!("elf_grid:\n{}", elf_grid); }
    let round_num = elf_grid.perform_until_no_moves();
    println!("It took until the {} round before no one moved.", round_num);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
