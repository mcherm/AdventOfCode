
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
    use itertools;
    use std::collections::{HashMap, HashSet};
    use std::fmt::{Display, Formatter};
    use itertools::iproduct;
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

    #[derive(Debug)]
    pub struct ElfGrid {
        all_elves: HashSet<Coord>,
        active_elves: HashSet<Coord>,
        preferred_pdir: PrimaryDirection,
        prev_moves: usize,
    }

    /// The values that can be returned for an elf deciding what to do.
    #[derive(Debug, Clone)]
    enum MovementChoice {
        Alone,
        TooCrowded,
        Propose{dest: Coord, activates: [Coord; 3]},
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
            let xs = 0..elf_places.width();
            let ys = 0..elf_places.height();
            let all_elves: HashSet<Coord> = iproduct!(xs, ys).filter_map(|(x,y)| {
                if elf_places.get_at(x,y) {
                    Some(Coord(x as i16, y as i16))
                } else {
                    None
                }
            }).collect();
            let active_elves = all_elves.clone();
            let preferred_pdir = PrimaryDirection::N;
            let prev_moves = all_elves.len();
            ElfGrid{all_elves, active_elves, preferred_pdir, prev_moves}
        }

        /// Returns true if there's an elf at this location; false if not.
        fn has_elf(&self, coord: Coord) -> bool {
            self.all_elves.contains(&coord)
        }

        /// Returns the bounding rectangle: [min_x, max_x, min_y, max_y]
        fn get_bounds(&self) -> [i16; 4] {
            assert!(! self.all_elves.is_empty());
            [
                self.all_elves.iter().map(|x| x.0).min().unwrap(),
                self.all_elves.iter().map(|x| x.0).max().unwrap(),
                self.all_elves.iter().map(|x| x.1).min().unwrap(),
                self.all_elves.iter().map(|x| x.1).max().unwrap(),
            ]
        }

        fn proposal(&self, elf: Coord) -> MovementChoice {
            // --- consider all neighbors ---
            if Direction::all().all(|dir| ! self.has_elf(elf.step(dir))) {
                // --- if there are no neighbors, go nowhere ---
                MovementChoice::Alone
            } else {
                // --- if there are no neighbors, consider each primary direction in order ---
                let mut pdir = self.preferred_pdir;
                for _ in 0..4 {
                    if pdir.leaning().iter().all(|dir| ! self.has_elf(elf.step(*dir))) {
                        // --- if all of them are empty, go that way ---
                        let dest = elf.step(pdir.into());
                        // it could affect things one more step in that general direction
                        let activates = pdir.leaning().map(|dir| dest.step(dir));
                        return MovementChoice::Propose{dest, activates};
                    }
                    pdir = pdir.next();
                }
                MovementChoice::TooCrowded
            }
        }

        /// Performs one round of position updates.
        pub fn perform_round(&mut self) {
            let mut proposals: HashMap<Coord, MovementChoice> = HashMap::with_capacity(self.active_elves.len());
            let mut proposal_count: HashMap<Coord, usize> = HashMap::new();

            // --- first half: get proposals ---
            for elf in self.active_elves.iter() {
                let proposal = self.proposal(*elf);
                if let MovementChoice::Propose{dest, ..} = proposal {
                    *proposal_count.entry(dest).or_insert(0) += 1;
                }
                proposals.insert(*elf, proposal);
            }

            // --- second half: make moves ---
            let mut activated: HashSet<Coord> = HashSet::new();
            let mut move_count = 0;
            for (elf, movement_choice) in proposals.iter() {
                match movement_choice {
                    MovementChoice::Alone => {
                        self.active_elves.remove(elf); // it's no longer active
                    }
                    MovementChoice::TooCrowded => {
                        // NOTE: The problem didn't say what to do if it had neighbors, but no direction
                        //   was proposed. I'm assuming it should propose to stay put. That seems to
                        //   match the examples.
                        // it's not going anywhere this time, but maybe from another direction, so it stays active
                    }
                    MovementChoice::Propose{dest, activates} => {
                        if *proposal_count.get(&dest).unwrap() == 1 {
                            // it actually moved
                            self.all_elves.remove(elf); // remove from old location
                            self.all_elves.insert(*dest); // add at new location
                            self.active_elves.remove(elf); // remove from old location
                            self.active_elves.insert(*dest); // its new location is active
                            activated.extend(activates.into_iter());
                            move_count += 1;
                        } else {
                            // multiple elves want to go to the same place; they don't move
                        }
                    }
                }
            }

            // --- activate the activated ones ---
            self.active_elves.extend(activated.into_iter().filter(|elf| self.all_elves.contains(elf)));

            // --- third step: update other fields of self ---
            self.preferred_pdir = self.preferred_pdir.next();
            self.prev_moves = move_count;
        }

        /// Performs rounds until a round where no on moves. Returns the number of rounds.
        pub fn perform_until_no_moves(&mut self) -> usize {
            let mut count = 0;
            loop {
                self.perform_round();
                count += 1;
                if self.prev_moves == 0 {
                    return count;
                }
            }
        }

        /// Returns the number of empty ground spaces within the bounding rectangle.
        pub fn empty_ground(&self) -> usize {
            let [min_x, max_x, min_y, max_y] = self.get_bounds();
            let area: usize = ((1 + max_x - min_x) as usize) * ((1 + max_y - min_y) as usize);
            area - self.all_elves.len()
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
