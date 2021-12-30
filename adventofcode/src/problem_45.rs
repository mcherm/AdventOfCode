use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs;
use nom::bytes::complete::tag as nom_tag;
use nom::sequence::tuple as nom_tuple;
use nom::branch::alt as nom_alt;
use Location::{
    Hall0, Hall1, Hall2, Hall3, Hall4, Hall5, Hall6,
    FrontOfA, FrontOfB, FrontOfC, FrontOfD,
    BackOfA, BackOfB, BackOfC, BackOfD
};
use AmphipodType::{Amber, Bronze, Copper, Desert};


// ======== Reading Input ========

/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    InvalidMazeFile,
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err) => write!(f, "{}", err),
            InputError::InvalidMazeFile => write!(f, "Invalid maze file."),
        }
    }
}

/// Read in the input file.
fn read_maze_file() -> Result<Position, InputError> {
    // --- read file ---
    let filename = "data/2021/day/23/input.txt";
    let contents = fs::read_to_string(filename)?;
    // NOTE: I should raise an error, not unwrap but I don't know how.
    let (rest, positions) = Position::parse_nom(&contents).unwrap();
    if rest != "" {
        return Err(InputError::InvalidMazeFile);
    }

    // --- return result ---
    Ok(positions)
}



// ======== Types ========

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
enum AmphipodType {
    Amber,
    Bronze,
    Copper,
    Desert,
}


#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
enum Location {
    Hall0 = 0,
    Hall1 = 1,
    Hall2 = 2,
    Hall3 = 3,
    Hall4 = 4,
    Hall5 = 5,
    Hall6 = 6,
    FrontOfA = 7,
    FrontOfB = 8,
    FrontOfC = 9,
    FrontOfD = 10,
    BackOfA = 11,
    BackOfB = 12,
    BackOfC = 13,
    BackOfD = 14,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Position {
    slots: [Option<AmphipodType>; Location::NUM_VALUES],
}

type Cost = u32;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Move {
    amph: AmphipodType,
    from: Location,
    to: Location,
}

// ======== Implementations ========

impl AmphipodType {
    const NUM_VALUES: usize = 4;
    const ALL: [AmphipodType; AmphipodType::NUM_VALUES] = [Amber, Bronze, Copper, Desert];

    /// Reads a field which could be an AmphipodType or a "." for None.
    fn parse_nom(input: &str) -> nom::IResult<&str, Option<Self>> {
        nom_alt((
            nom_tag("A"),
            nom_tag("B"),
            nom_tag("C"),
            nom_tag("D"),
            nom_tag("."),
        ))(input).map(|(rest, res)| (rest, match res {
            "A" => Some(Amber),
            "B" => Some(Bronze),
            "C" => Some(Copper),
            "D" => Some(Desert),
            "." => None,
            _ => panic!("should never happen")
        }))
    }

    fn to_str(&self) -> &'static str {
        match self {
            Amber => &"A",
            Bronze => &"B",
            Copper => &"C",
            Desert => &"D",
        }
    }

    /// Returns the nook index for a given AmphipodType
    fn nook(&self) -> usize {
        match self {
            Amber => 0,
            Bronze => 1,
            Copper => 2,
            Desert => 3,
        }
    }

    /// Returns the cost per step for this AmphipodType
    fn step_cost(&self) -> Cost {
        match self {
            Amber => 1,
            Bronze => 10,
            Copper => 100,
            Desert => 1000,
        }
    }
}

impl Display for AmphipodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}


impl Location {
    const NUM_VALUES: usize = 15;
    const FRONT_SLOTS: [Location; 4] = [FrontOfA, FrontOfB, FrontOfC, FrontOfD];
    const BACK_SLOTS: [Location; 4] = [BackOfA, BackOfB, BackOfC, BackOfD];
    #[allow(dead_code)]
    const HALL_SLOTS: [Location; 7] = [Hall0, Hall1, Hall2, Hall3, Hall4, Hall5, Hall6];

    fn to_str(&self) -> &'static str {
        match self {
            Hall0 => "Hall0",
            Hall1 => "Hall1",
            Hall2 => "Hall2",
            Hall3 => "Hall3",
            Hall4 => "Hall4",
            Hall5 => "Hall5",
            Hall6 => "Hall6",
            FrontOfA => "FrontOfA",
            FrontOfB => "FrontOfB",
            FrontOfC => "FrontOfC",
            FrontOfD => "FrontOfD",
            BackOfA => "BackOfA",
            BackOfB => "BackOfB",
            BackOfC => "BackOfC",
            BackOfD => "BackOfD",
        }
    }

    /// Pass this a nook number and it will return two vectors of Hall
    /// locations: the first is those reachable by turning left, the
    /// second is those reachable by turning right. They are in the
    /// order in which the locations need to be entered.
    fn hall_from(nook: AmphipodType) -> (Vec<Location>, Vec<Location>) {
        match nook {
            Amber => (
                vec![Hall1, Hall0],
                vec![Hall2, Hall3, Hall4, Hall5, Hall6]
            ),
            Bronze => (
                vec![Hall2, Hall1, Hall0],
                vec![Hall3, Hall4, Hall5, Hall6]
            ),
            Copper => (
                vec![Hall3, Hall2, Hall1, Hall0],
                vec![Hall4, Hall5, Hall6]
            ),
            Desert => (
                vec![Hall4, Hall3, Hall2, Hall1, Hall0],
                vec![Hall5, Hall6]
            ),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}



impl Position {
    /// Parses a position, assuming it is valid and panicking if it isn't.
    #[allow(dead_code)]
    fn parse_good(input: &str) -> Self {
        let (rest, answer) = Self::parse_nom(input).unwrap();
        assert!(rest.len() == 0);
        answer
    }

    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        // This tuple was so long we had to break it into 2 separate tuples
        nom_tuple((
            nom_tuple((
                nom_tag("#############\n#"),
                AmphipodType::parse_nom,
                AmphipodType::parse_nom,
                nom_tag("."),
                AmphipodType::parse_nom,
                nom_tag("."),
                AmphipodType::parse_nom,
                nom_tag("."),
                AmphipodType::parse_nom,
                nom_tag("."),
                AmphipodType::parse_nom,
                AmphipodType::parse_nom,
                nom_tag("#\n"),
            )),
            nom_tuple((
                nom_tag("###"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("###\n  #"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#\n  #########\n"),
            )),
        ))(input).map(
            |(rest, (
                (_, h0, h1, _, h2, _, h3, _, h4, _, h5, h6, _),
                (_, fa, _, fb, _, fc, _, fd, _, ba, _, bb, _, bc, _, bd, _)
            ))| {(
                rest,
                Position {
                    slots: [h0, h1, h2, h3, h4, h5, h6, fa, fb, fc, fd, ba, bb, bc, bd],
                }
            )}
        )
    }

    fn at(&self, loc: Location) -> Option<AmphipodType> {
        self.slots[loc as usize]
    }
    
    fn show_opt_amphipod_at(&self, loc: Location) -> &'static str {
        match self.slots[loc as usize] {
            None => ".",
            Some(a) => a.to_str(),
        }
    }

    /// Returns true if the problem is solved.
    fn is_complete(&self) -> bool {
        *self == FINAL_POSITION
    }

    /// Returns a vector of all legal moves from this position. They will be sorted
    /// in order from most expensive to least.
    fn legal_moves(&self) -> Vec<Move> {
        let mut answer = Vec::new();

        // -- Moves out of a nook (if it's not YOUR nook or if you are blocking someone) --
        for a in AmphipodType::ALL {
            let front = Location::FRONT_SLOTS[a.nook()];
            let back = Location::BACK_SLOTS[a.nook()];
            let from_opt: Option<Location> = match self.at(front) {
                Some(amph) => {
                    if amph == a { // it's my row; I can only move if I'm blocking someone
                        match self.at(back) {
                            Some(x) if x != a => Some(front),
                            _ => None
                        }
                    } else {
                        Some(front)
                    }
                },
                None => match self.at(back) {
                    Some(amph) => {
                        if amph == a { // it's my row
                            None
                        } else { // not my row; I'm allowed to leave
                            Some(back)
                        }
                    },
                    None => None
                },
            };
            if let Some(from) = from_opt {
                let amph: AmphipodType = self.at(from).unwrap();
                let (left_hall, right_hall) = Location::hall_from(a);
                for to in left_hall {
                    match self.at(to) {
                        Some(_) => break, // no more space on the left
                        None => answer.push(Move{amph, from, to}),
                    }
                }
                for to in right_hall {
                    match self.at(to) {
                        Some(_) => break, // no more space on the right
                        None => answer.push(Move{amph, from, to}),
                    }
                }
            }
        }

        // -- Moves into a nook --
        for a in AmphipodType::ALL {
            let front = Location::FRONT_SLOTS[a.nook()];
            let back = Location::BACK_SLOTS[a.nook()];
            let to_opt: Option<Location> = match self.at(front) {
                Some(_) => None,
                None => match self.at(back) {
                    Some(amph) => {
                        if amph == a { // back is filled in properly
                            Some(front)
                        } else { // back has someone else; we can't go in yet
                            None
                        }
                    },
                    None => Some(back),
                },
            };
            if let Some(to) = to_opt {
                let (left_hall, right_hall) = Location::hall_from(a);
                'left_hall:
                for from in left_hall {
                    match self.at(from) {
                        Some(amph) => {
                            if amph == a {
                                answer.push(Move{amph, from, to});
                            }
                            break 'left_hall; // no more to the left
                        },
                        None => {}, // keep looking to the left
                    }
                }
                for from in right_hall {
                    match self.at(from) {
                        Some(amph) => {
                            if amph == a {
                                answer.push(Move{amph, from, to});
                            }
                            break; // no more to the right
                        },
                        None => {}, // keep looking to the right
                    }
                }
            }
        }

        // -- Sort and return answer --
        answer.sort();
        answer
    }


    fn perform(&self, mv: Move) -> Position {
        let mut slots = self.slots.clone();
        slots[mv.from as usize] = None;
        slots[mv.to as usize] = Some(mv.amph);
        Position{slots}
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\
#############
#{}{}.{}.{}.{}.{}{}#
###{}#{}#{}#{}###
  #{}#{}#{}#{}#
  #########
",
            self.show_opt_amphipod_at(Hall0),
            self.show_opt_amphipod_at(Hall1),
            self.show_opt_amphipod_at(Hall2),
            self.show_opt_amphipod_at(Hall3),
            self.show_opt_amphipod_at(Hall4),
            self.show_opt_amphipod_at(Hall5),
            self.show_opt_amphipod_at(Hall6),
            self.show_opt_amphipod_at(FrontOfA),
            self.show_opt_amphipod_at(FrontOfB),
            self.show_opt_amphipod_at(FrontOfC),
            self.show_opt_amphipod_at(FrontOfD),
            self.show_opt_amphipod_at(BackOfA),
            self.show_opt_amphipod_at(BackOfB),
            self.show_opt_amphipod_at(BackOfC),
            self.show_opt_amphipod_at(BackOfD),
        )
    }
}

const FINAL_POSITION: Position = Position{slots: [
    None, None, None, None, None, None, None,
    Some(Amber), Some(Bronze), Some(Copper), Some(Desert),
    Some(Amber), Some(Bronze), Some(Copper), Some(Desert),
]};



impl Move {
    /// Calculate the cost of this move
    fn cost(&self) -> Cost {
        distance(self.from, self.to) * self.amph.step_cost()
    }

    /// Calculate the "value" of this move -- which a heuristic for the order I want
    /// to try things
    fn value(&self) -> Cost {
        // FIXME: I want things like putting away valuable thing before getting out other stuff.
        //   But for now it's just pretty much "use cheapest".
        self.cost()
    }

    fn sort_tuple(&self) -> (Cost, Location, Location, AmphipodType) {
        (self.value(), self.to, self.from, self.amph)
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_tuple().cmp(&other.sort_tuple())
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


// ======== Functions ========

const DISTANCE_MAP: [[Cost; Location::NUM_VALUES]; Location::NUM_VALUES] = [
    //h0 h1 h2 h3 h4 h5 h6 fa fb fc fd ba bb bc bd
    [  0, 1, 3, 5, 7, 9,10, 3, 5, 7, 9, 4, 6, 8,10],
    [  1, 0, 2, 4, 6, 8, 9, 2, 4, 6, 8, 3, 5, 7, 9],
    [  3, 2, 0, 2, 4, 6, 7, 2, 2, 4, 6, 3, 3, 5, 7],
    [  5, 4, 2, 0, 2, 4, 5, 4, 2, 2, 4, 5, 3, 3, 5],
    [  7, 6, 4, 2, 0, 2, 3, 6, 4, 2, 2, 7, 5, 3, 3],
    [  9, 8, 6, 4, 2, 0, 1, 8, 6, 4, 2, 9, 7, 5, 3],
    [ 10, 9, 7, 5, 3, 1, 0, 9, 7, 5, 3,10, 8, 6, 4],
    [  3, 2, 2, 4, 6, 8, 9, 0, 4, 6, 8, 1, 5, 7, 9],
    [  5, 4, 2, 2, 4, 6, 7, 4, 0, 4, 6, 5, 1, 5, 7],
    [  7, 6, 4, 2, 2, 4, 5, 6, 4, 0, 4, 7, 5, 1, 5],
    [  9, 8, 6, 4, 2, 2, 3, 8, 6, 4, 0, 9, 7, 5, 1],
    [  4, 3, 3, 5, 7, 9,10, 1, 5, 7, 9, 0, 6, 8,10],
    [  6, 5, 3, 3, 5, 7, 8, 5, 1, 5, 7, 6, 0, 6, 8],
    [  8, 7, 5, 3, 3, 5, 6, 7, 5, 1, 5, 8, 6, 0, 6],
    [ 10, 9, 7, 5, 3, 3, 4, 9, 7, 5, 1,10, 7, 6, 0],
];

/// Returns the number of "steps" between 2 locations.
fn distance(loc1: Location, loc2: Location) -> Cost {
    DISTANCE_MAP[loc1 as usize][loc2 as usize]
}


/// Returns some Vec<Move> that will "solve" this position or None if it
/// is unsolvable.
fn solve(position: Position) -> Option<Vec<Move>> {
    println!("solve()");
    if position.is_complete() {
        Some(vec![])
    } else {
        for mv in position.legal_moves() {
            println!("  mv: {:?}", mv);
            let recurse = solve(position.perform(mv));
            if let Some(path) = recurse {
                let mut answer = Vec::with_capacity(path.len() + 1);
                answer.push(mv);
                answer.extend(path);
                return Some(answer);
            }
        }
        None
    }
}

// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let position: Position = read_maze_file()?;

    let path_opt = solve(position);
    match path_opt {
        None => println!("There were no solutions."),
        Some(path) => {
            println!("Solution:");
            for mv in path {
                println!("    {:?}", mv);
            }
        }
    }

    Ok(())
}


pub fn main() {
    match run() {
        Ok(()) => {
            println!("Done");
        },
        Err(err) => println!("Error: {}", err),
    }
}

// ======== Tests ========

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file() {
        let _ = read_maze_file().unwrap();
    }

    #[test]
    fn test_legal_moves_1() {
        let position = Position::parse_good("#############
#B..A.C.....#
###.#.#B#D###
  #.#D#C#A#
  #########\n");
        assert_eq!(
            vec![
                Move{amph: Amber, from: Hall2, to: BackOfA},
                Move{amph: Bronze, from: FrontOfC, to: Hall4},
                Move{amph: Bronze, from: FrontOfC, to: Hall5},
                Move{amph: Bronze, from: FrontOfC, to: Hall6},
                Move{amph: Desert, from: FrontOfD, to: Hall4},
                Move{amph: Desert, from: FrontOfD, to: Hall5},
                Move{amph: Desert, from: FrontOfD, to: Hall6},
            ],
            position.legal_moves()
        );
    }

    #[test]
    fn test_legal_moves_2() {
        let position = Position::parse_good("#############
#A..........#
###.#B#C#D###
  #A#B#C#D#
  #########\n");
        assert_eq!(
            vec![
                Move{amph: Amber, from: Hall0, to: FrontOfA},
            ],
            position.legal_moves()
        );
    }

    #[test]
    fn test_legal_moves_3() {
        let position = Position::parse_good("#############
#A..........#
###.#A#C#D###
  #B#B#C#D#
  #########\n");
        assert_eq!(
            vec![
                Move{amph: Amber, from: FrontOfB, to: Hall2},
                Move{amph: Amber, from: FrontOfB, to: Hall3},
                Move{amph: Amber, from: FrontOfB, to: Hall1},
                Move{amph: Amber, from: FrontOfB, to: Hall4},
                Move{amph: Amber, from: FrontOfB, to: Hall5},
                Move{amph: Amber, from: FrontOfB, to: Hall6},
                Move{amph: Bronze, from: BackOfA, to: Hall1},
                Move{amph: Bronze, from: BackOfA, to: Hall2},
                Move{amph: Bronze, from: BackOfA, to: Hall3},
                Move{amph: Bronze, from: BackOfA, to: Hall4},
                Move{amph: Bronze, from: BackOfA, to: Hall5},
                Move{amph: Bronze, from: BackOfA, to: Hall6},
            ],
            position.legal_moves()
        );
    }
}
