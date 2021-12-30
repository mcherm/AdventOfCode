use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs;
use nom::bytes::complete::tag as nom_tag;
use nom::sequence::tuple as nom_tuple;
use nom::branch::alt as nom_alt;


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
fn read_maze_file() -> Result<Positions, InputError> {
    // --- read file ---
    let filename = "data/2021/day/23/input.txt";
    let contents = fs::read_to_string(filename)?;
    // NOTE: I should raise an error, not unwrap but I don't know how.
    let (rest, positions) = Positions::parse_nom(&contents).unwrap();
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
struct Positions {
    slots: [Option<AmphipodType>; Location::NUM_VALUES],
}



// ======== Implementations ========

impl AmphipodType {
    /// Reads a field which could be an AmphipodType or a "." for None.
    fn parse_nom(input: &str) -> nom::IResult<&str, Option<Self>> {
        nom_alt((
            nom_tag("A"),
            nom_tag("B"),
            nom_tag("C"),
            nom_tag("D"),
            nom_tag("."),
        ))(input).map(|(rest, res)| (rest, match res {
            "A" => Some(AmphipodType::Amber),
            "B" => Some(AmphipodType::Bronze),
            "C" => Some(AmphipodType::Copper),
            "D" => Some(AmphipodType::Desert),
            "." => None,
            _ => panic!("should never happen")
        }))
    }

    fn to_str(&self) -> &'static str {
        match self {
            AmphipodType::Amber => &"A",
            AmphipodType::Bronze => &"B",
            AmphipodType::Copper => &"C",
            AmphipodType::Desert => &"D",
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
}


impl Positions {
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
                Positions{
                    slots: [h0, h1, h2, h3, h4, h5, h6, fa, fb, fc, fd, ba, bb, bc, bd],
                }
            )}
        )
    }
    
    fn show_opt_amphipod_at(&self, loc: Location) -> &'static str {
        match self.slots[loc as usize] {
            None => ".",
            Some(a) => a.to_str(),
        }
    }
}

impl Display for Positions {
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
            self.show_opt_amphipod_at(Location::Hall0),
            self.show_opt_amphipod_at(Location::Hall1),
            self.show_opt_amphipod_at(Location::Hall2),
            self.show_opt_amphipod_at(Location::Hall3),
            self.show_opt_amphipod_at(Location::Hall4),
            self.show_opt_amphipod_at(Location::Hall5),
            self.show_opt_amphipod_at(Location::Hall6),
            self.show_opt_amphipod_at(Location::FrontOfA),
            self.show_opt_amphipod_at(Location::FrontOfB),
            self.show_opt_amphipod_at(Location::FrontOfC),
            self.show_opt_amphipod_at(Location::FrontOfD),
            self.show_opt_amphipod_at(Location::BackOfA),
            self.show_opt_amphipod_at(Location::BackOfB),
            self.show_opt_amphipod_at(Location::BackOfC),
            self.show_opt_amphipod_at(Location::BackOfD),
        )
    }
}


// ======== Functions ========


// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let positions: Positions = read_maze_file()?;
    println!("{}", positions);
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

}
