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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum AmphipodType {
    Amber,
    Bronze,
    Copper,
    Desert,
}


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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

#[allow(dead_code)]
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

    /// Returns a vector of all legal moves from this position.
    fn legal_moves(&self) -> Vec<Move> {
        let mut answer = Vec::new();

        // -- Moves out of a nook --
        for a in AmphipodType::ALL {
            let front = Location::FRONT_SLOTS[a.nook()];
            let back = Location::BACK_SLOTS[a.nook()];
            let from_opt: Option<Location> = match self.at(front) {
                Some(_) => Some(front),
                None => match self.at(back) {
                    Some(_) => Some(back),
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
                    Some(_) => Some(front),
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

        // -- Return answer --
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


// ======== Functions ========


// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let mut position: Position = read_maze_file()?;
    println!("{}", position);
    let legal_moves = position.legal_moves();
    for mv in legal_moves.iter() {
        println!("Move: {:?}", mv);
    }

    position = position.perform(legal_moves[0]);
    println!();
    println!("{}", position);
    let legal_moves = position.legal_moves();
    for mv in legal_moves.iter() {
        println!("Move: {:?}", mv);
    }

    position = position.perform(legal_moves[0]);
    println!();
    println!("{}", position);
    let legal_moves = position.legal_moves();
    for mv in legal_moves.iter() {
        println!("Move: {:?}", mv);
    }

    position = position.perform(legal_moves[0]);
    println!();
    println!("{}", position);
    let legal_moves = position.legal_moves();
    for mv in legal_moves.iter() {
        println!("Move: {:?}", mv);
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
                Move{amph: Bronze, from: FrontOfC, to: Hall4},
                Move{amph: Bronze, from: FrontOfC, to: Hall5},
                Move{amph: Bronze, from: FrontOfC, to: Hall6},
                Move{amph: Desert, from: FrontOfD, to: Hall4},
                Move{amph: Desert, from: FrontOfD, to: Hall5},
                Move{amph: Desert, from: FrontOfD, to: Hall6},
                Move{amph: Amber, from: Hall2, to: BackOfA},
            ],
            position.legal_moves()
        );
    }
}
