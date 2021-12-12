use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
// use std::collections::HashMap;
use multimap::MultiMap;


/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    InvalidPassage,
    InvalidCavern(String),
    NoStart,
    NoEnd,
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl From<std::num::ParseIntError> for InputError {
    fn from(error: std::num::ParseIntError) -> Self {
        InputError::BadInt(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err)   => write!(f, "{}", err),
            InputError::BadInt(err)    => write!(f, "{}", err),
            InputError::InvalidPassage => write!(f, "Invalid passage"),
            InputError::InvalidCavern(s) => write!(f, "Invalid cavern: '{}'", s),
            InputError::NoStart => write!(f, "No start"),
            InputError::NoEnd => write!(f, "No end"),
        }
    }
}


/// Read in the input file.
fn read_cavernmap_file() -> Result<CavernMap, InputError> {
    let filename = "data/2021/day/12/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    let mut passages: Vec<[Cavern;2]> = Vec::new();
    for line in lines {
        let text = line?;
        let mut split = text.split("-");
        let name_1 = split.next().ok_or(InputError::InvalidPassage)?.to_string();
        let name_2 = split.next().ok_or(InputError::InvalidPassage)?.to_string();
        if split.next().is_some() {
            return Err(InputError::InvalidPassage);
        }
        let cavern_1 = Cavern::new(name_1)?;
        let cavern_2 = Cavern::new(name_2)?;
        passages.push([cavern_1, cavern_2]);
    }
    let cavern_map = CavernMap::new(passages)?;
    Ok(cavern_map)
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Cavern {
    name: String,
    big: bool,
}

impl Cavern {
    fn new(name: String) -> Result<Cavern,InputError> {
        let up = name.to_ascii_uppercase();
        let down = name.to_ascii_lowercase();
        match (up==name, down==name) {
            (true,true) | (false,false) => Err(InputError::InvalidCavern(name)),
            (true,false) => Ok(Cavern{name, big:true}),
            (false,true) => Ok(Cavern{name, big:false}),
        }
    }

    fn is_start(&self) -> bool {
        self.name == "start"
    }

    fn is_end(&self) -> bool {
        self.name == "end"
    }

    fn is_big(&self) -> bool {
        self.big
    }
}


#[derive(Debug)]
struct CavernMap {
    neighbors: MultiMap<Cavern,Cavern>,
    start: Cavern,
    end: Cavern,
}

impl CavernMap {
    fn new(passages: Vec<[Cavern; 2]>) -> Result<Self,InputError> {
        let mut neighbors: MultiMap<Cavern,Cavern> = MultiMap::new();
        let mut start_opt: Option<Cavern> = None;
        let mut end_opt: Option<Cavern> = None;
        for passage in passages {
            for cavern in passage.iter() {
                if start_opt.is_none() && cavern.is_start() {
                    start_opt = Some(cavern.clone());
                }
                if end_opt.is_none() && cavern.is_end() {
                    end_opt = Some(cavern.clone());
                }
            }
            neighbors.insert(passage[0].clone(), passage[1].clone());
            neighbors.insert(passage[1].clone(), passage[0].clone());
        }
        let start = start_opt.ok_or(InputError::NoStart)?;
        let end = end_opt.ok_or(InputError::NoEnd)?;
        Ok(CavernMap{neighbors, start, end})
    }
}



pub fn main() {
    match read_cavernmap_file() {
        Ok(cavern_map) => {
            println!("Cavern Map: {:#?}", cavern_map);
        },
        Err(err) => println!("Error: {}", err),
    }
}
