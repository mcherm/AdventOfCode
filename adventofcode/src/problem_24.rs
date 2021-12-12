use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
    small: bool,
}

impl Cavern {
    fn new(name: String) -> Result<Cavern,InputError> {
        let up = name.to_ascii_uppercase();
        let down = name.to_ascii_lowercase();
        match (up==name, down==name) {
            (true,true) | (false,false) => Err(InputError::InvalidCavern(name)),
            (true,false) => Ok(Cavern{name, small:false}),
            (false,true) => Ok(Cavern{name, small:true}),
        }
    }

    fn is_start(&self) -> bool {
        self.name == "start"
    }

    fn is_end(&self) -> bool {
        self.name == "end"
    }

    fn is_small(&self) -> bool {
        self.small
    }
}

impl fmt::Display for Cavern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}


// An immutable LinkedList of Cavern objects
enum LinkedListOfCavern<'a> {
    Start {
        data: Cavern
    },
    Node {
        data: Cavern,
        next: &'a LinkedListOfCavern<'a>
    },
}
impl<'a> LinkedListOfCavern<'a> {
    fn new(c: &Cavern) -> Self {
        LinkedListOfCavern::Start{data: c.clone()}
    }

    fn append(&'a self, c: &Cavern) -> Self {
        LinkedListOfCavern::Node{data: c.clone(), next: self}
    }

    fn contains(&'a self, c: &Cavern) -> bool {
        match self {
            LinkedListOfCavern::Start{data} => {
                data == c
            },
            LinkedListOfCavern::Node{data, next} => {
                data == c || next.contains(c)
            }
        }
    }

    fn last(&'a self) -> &'a Cavern {
        match self {
            LinkedListOfCavern::Start{data} => {
                data
            },
            LinkedListOfCavern::Node{data, next: _} => {
                data
            }
        }
    }
}
impl<'a> fmt::Display for LinkedListOfCavern<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinkedListOfCavern::Start{data} => {
                write!(f, "{}", data)
            },
            LinkedListOfCavern::Node{data,next} => {
                write!(f, "{}", next)?;
                write!(f, "->{}", data)
            },
        }
    }
}


struct Path {
    steps: Vec<Cavern>,
    small_cave_revisits: usize,
}

impl Path {
//impl<'a> Path<'a> {
    // Creates a new Path. The argument must be the start cavern.
    fn new(start: &Cavern) -> Self {
        assert!(start.is_start());
        Path{steps: Vec::from([start.clone()]), small_cave_revisits: 0}
        // let caverns: LinkedListOfCavern = LinkedListOfCavern::Start{data: start.clone()};
        // Path{caverns, small_cave_revisits: 0}
    }

    // Attempt to create a new Path by adding the given Cavern onto this path.
    // Returns None if such a Path would be illegal.
    fn add(&self, c: &Cavern) -> Option<Path> {
        if c.is_start() {
            return None;
        }
        let small_cave_revisits;
        if c.is_small() && self.steps.contains(c) {
            if c.is_end() {
                return None;
            }
            small_cave_revisits = self.small_cave_revisits + 1;
        } else {
            small_cave_revisits = self.small_cave_revisits;
        }
        if small_cave_revisits > 1 {
            return None;
        }
        let mut steps = self.steps.clone();
        steps.push(c.clone());
        let result = Path{steps, small_cave_revisits};
        Some(result)
    }

    // Returns the last Cavern in the path. Since paths always include
    // at least one Cavern, this returns a Cavern, not an Option<Cavern>.
    fn last(&self) -> &Cavern {
        &self.steps.last().unwrap()
    }

    // Returns true if the Path ends at an "end" cavern.
    fn terminates(&self) -> bool {
        self.last().is_end()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.steps.len() == 0 {
            return write!(f, "[]")
        }
        let mut steps_iter = self.steps.iter();
        write!(f, "{}", steps_iter.next().unwrap())?;
        for step in steps_iter {
            write!(f, "->{}", step)?;
        }
        Ok(())
    }
}


struct Path2<'a> {
    caverns: LinkedListOfCavern<'a>,
    small_cave_revisits: usize,
}

impl<'a> Path2<'a> {
    // Creates a new Path. The argument must be the start cavern.
    fn new(start: &Cavern) -> Self {
        assert!(start.is_start());
        let caverns: LinkedListOfCavern = LinkedListOfCavern::Start{data: start.clone()};
        Path2{caverns, small_cave_revisits: 0}
    }

    // Attempt to create a new Path by adding the given Cavern onto this path.
    // Returns None if such a Path would be illegal.
    fn add(&'a self, c: &Cavern) -> Option<Path2<'a>> {
        if c.is_start() {
            return None;
        }
        let small_cave_revisits;
        if c.is_small() && self.caverns.contains(c) {
            if c.is_end() {
                return None;
            }
            small_cave_revisits = self.small_cave_revisits + 1;
        } else {
            small_cave_revisits = self.small_cave_revisits;
        }
        if small_cave_revisits > 1 {
            return None;
        }
        let caverns: LinkedListOfCavern<'a> = self.caverns.append(&c);
        let result: Path2 = Path2{caverns, small_cave_revisits};
        Some(result)
    }

    // Returns the last Cavern in the path. Since paths always include
    // at least one Cavern, this returns a Cavern, not an Option<Cavern>.
    fn last(&self) -> Cavern {
        self.caverns.last().clone()
    }

    // Returns true if the Path ends at an "end" cavern.
    fn terminates(&self) -> bool {
        self.last().is_end()
    }
}

impl<'a> fmt::Display for Path2<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.caverns)
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

    fn wander_from(&self, initial_path: &Path) -> u32 {
        let mut path_count: u32 = 0;
        let legal_paths = self.neighbors.get_vec(initial_path.last())
            .unwrap()
            .iter()
            .filter_map(|x| initial_path.add(x));
        for path in legal_paths {
            if path.terminates() {
                path_count += 1;
            } else {
                path_count += self.wander_from(&path); // Recurse
            }
        }
        path_count
    }

    fn wander(&self) -> u32 {
        let path: Path = Path::new(&self.start);
        self.wander_from(&path)
    }

    fn wander_from2(&self, initial_path: &Path2) -> u32 {
        let mut path_count: u32 = 0;
        let legal_paths = self.neighbors.get_vec(&initial_path.last())
            .unwrap()
            .iter()
            .filter_map(|x| initial_path.add(x));
        for path in legal_paths {
            if path.terminates() {
                path_count += 1;
            } else {
                path_count += self.wander_from2(&path); // Recurse
            }
        }
        path_count
    }

    fn wander2(&self) -> u32 {
        let path: Path2 = Path2::new(&self.start);
        self.wander_from2(&path)
    }
}




fn run_test() -> Result<(),InputError> {
    let start = Cavern::new("start".to_string())?;
    let llc: LinkedListOfCavern = LinkedListOfCavern::new(&start);
    let c1 = Cavern::new("ab".to_string())?;
    let llc: LinkedListOfCavern = llc.append(&c1);
    let c2 = Cavern::new("RR".to_string())?;
    let llc: LinkedListOfCavern = llc.append(&c2);
    let last = llc.last();
    println!("llc = {} and contains ab is {} and last is {}", llc, llc.contains(&c1), last);

    let p1 = Path2::new(&start);
    println!("p1 = {} ", p1);
    let p2 = p1.add(&c1).unwrap();
    println!("p2 = {} ", p2);
    let p3 = p2.add(&c2).unwrap();
    println!("p3 = {} last={} terminates={}", p3, p3.last(), p3.terminates());
    Ok(())
}

pub fn main() {
    match run_test() {
        Ok(_) => {
            println!("Ok");
        },
        Err(_) => {
            println!("Err");
        },
    }


    match read_cavernmap_file() {
        Ok(cavern_map) => {
            let start = std::time::Instant::now();
            let path_count = cavern_map.wander();
            let duration = start.elapsed();
            println!("path count = {} which took {:?}", path_count, duration);

            let start = std::time::Instant::now();
            let path_count = cavern_map.wander2();
            let duration = start.elapsed();
            println!("path count = {} which took {:?}", path_count, duration);

        },
        Err(err) => println!("Error: {}", err),
    }
}

// path count = 128506 which took 2.793934083s
// path count = 128506 which took 1.000018196s
