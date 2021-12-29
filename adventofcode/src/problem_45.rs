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

#[derive(Debug, Eq, PartialEq, Clone)]
struct Positions {
    slot1: [Option<AmphipodType>; 2],
    slot2: [Option<AmphipodType>; 2],
    slot3: [Option<AmphipodType>; 2],
    slot4: [Option<AmphipodType>; 2],
    hallway: [Option<AmphipodType>; 11],
}


// ======== Implementations ========

impl AmphipodType {
    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            nom_tag("A"),
            nom_tag("B"),
            nom_tag("C"),
            nom_tag("D"),
        ))(input).map(|(rest, res)| (rest, match res {
            "A" => AmphipodType::Amber,
            "B" => AmphipodType::Bronze,
            "C" => AmphipodType::Copper,
            "D" => AmphipodType::Desert,
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

impl Positions {
    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        nom_tuple((
            nom_tag("#############\n#...........#\n###"),
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
        ))(input).map(|(rest, (_, s10, _, s20, _, s30, _, s40, _, s11, _, s21, _, s31, _, s41, _))| {
            (
                rest,
                Positions{
                    slot1: [Some(s10), Some(s11)],
                    slot2: [Some(s20), Some(s21)],
                    slot3: [Some(s30), Some(s31)],
                    slot4: [Some(s40), Some(s41)],
                    hallway: [None; 11],
                }
            )
        })
    }
}

impl Display for Positions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\
#############
#{}{}{}{}{}{}{}{}{}{}{}#
###{}#{}#{}#{}###
  #{}#{}#{}#{}#
  #########
",
            show_opt_amphipod(self.hallway[0]),
            show_opt_amphipod(self.hallway[1]),
            show_opt_amphipod(self.hallway[2]),
            show_opt_amphipod(self.hallway[3]),
            show_opt_amphipod(self.hallway[4]),
            show_opt_amphipod(self.hallway[5]),
            show_opt_amphipod(self.hallway[6]),
            show_opt_amphipod(self.hallway[7]),
            show_opt_amphipod(self.hallway[8]),
            show_opt_amphipod(self.hallway[9]),
            show_opt_amphipod(self.hallway[10]),
            show_opt_amphipod(self.slot1[0]),
            show_opt_amphipod(self.slot2[0]),
            show_opt_amphipod(self.slot3[0]),
            show_opt_amphipod(self.slot4[0]),
            show_opt_amphipod(self.slot1[1]),
            show_opt_amphipod(self.slot2[1]),
            show_opt_amphipod(self.slot3[1]),
            show_opt_amphipod(self.slot4[1]),
        )
    }
}


// ======== Functions ========

fn show_opt_amphipod(oa: Option<AmphipodType>) -> &'static str {
    match oa {
        None => ".",
        Some(a) => a.to_str(),
    }
}

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
