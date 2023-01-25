
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
            ElfGrid{elves, preferred_pdir}
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

        fn proposal(&self, elf: &Elf) -> Option<Coord> {
            // --- consider all neighbors ---
            if Direction::all().all(|dir| self.has_elf(elf.pos.step(dir))) {
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
                //   was proposed. I'm assuming it should propose to stay put.
                println!("Assumption made about ({},{})", elf.pos.0, elf.pos.1); // FIXME: What to do?
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

            for (elf, proposal) in zip(self.elves.iter(), proposals.iter()) {
                fn find_pdir(elf: &Elf, proposal: &Option<Coord>) -> Option<PrimaryDirection> {
                    if let Some(proposed_coord) = proposal {
                        let mut dir = PrimaryDirection::N;
                        loop {
                            if *proposed_coord == elf.pos.step(dir.into()) {
                                return Some(dir);
                            }
                            dir = dir.next();
                            if dir == PrimaryDirection::N {
                                panic!("Proposal wasn't a primary direction.");
                            }
                        }
                    } else {
                        return None;
                    }
                }
                println!("Elf at ({},{}) proposes to move {:?}", elf.pos.0, elf.pos.1, find_pdir(elf, proposal));
            }

            // --- second half: make moves ---
            for (elf, proposal) in zip(self.elves.iter_mut(), proposals) {
                if let Some(coord) = proposal {
                    if *proposal_count.get(&coord).unwrap() == 1 {
                        elf.pos = coord;
                    }
                }
            }

            // --- third step: rotate preferred_pdir ---
            self.preferred_pdir = self.preferred_pdir.next();
        }
    }

    impl Display for ElfGrid {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            assert!(!self.elves.is_empty());
            let min_x = self.elves.iter().map(|elf| elf.pos.0).min().unwrap();
            let max_x = self.elves.iter().map(|elf| elf.pos.0).max().unwrap();
            let min_y = self.elves.iter().map(|elf| elf.pos.1).min().unwrap();
            let max_y = self.elves.iter().map(|elf| elf.pos.1).max().unwrap();
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


fn part_a(input: &ElfPlaces) {
    println!("\nPart a:");
    let mut elf_grid = ElfGrid::new(input);
    println!("elf_grid:\n{}", elf_grid);
    elf_grid.perform_round();
    println!("after round 1, elf_grid:\n{}", elf_grid);
    elf_grid.perform_round();
    println!("after round 2, elf_grid:\n{}", elf_grid);
    elf_grid.perform_round();
    println!("after round 3, elf_grid:\n{}", elf_grid);
}


fn part_b(_input: &ElfPlaces) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
