use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::learn_list::List;
use std::cmp::min;


const PRINT_LEVEL: PrintLevel = PrintLevel::NewBestValue;
const MAX_SIZE: Option<usize> = Some(20); // cut things off at this size


/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    NoData,
    NotSquare,
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
            InputError::IoError(err) => write!(f, "{}", err),
            InputError::BadInt(err) => write!(f, "{}", err),
            InputError::NoData => write!(f, "No data."),
            InputError::NotSquare => write!(f, "It is not square."),
        }
    }
}


/// Read in the input file.
fn read_grid_file() -> Result<Vec<Vec<u8>>, InputError> {
    let filename = "data/2021/day/15/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    let mut map_size: Option<usize> = None;
    let mut grid: Grid = Vec::new();
    for line in lines {
        let text: String = line?;
        let mut row = Vec::new();
        for char in text.chars() {
            let val: u8 = char.to_string().parse()?;
            row.push(val);
        }
        match map_size {
            None => map_size = Some(row.len()),
            Some(size) => if size != row.len() {
                return Err(InputError::NotSquare)
            }
        }
        grid.push(row);
    }
    match map_size {
        None => return Err(InputError::NoData),
        Some(size) => if size != grid.len() {
            return Err(InputError::NotSquare)
        }
    }

    Ok(grid)
}


type EntryCost = u8;
type Grid = Vec<Vec<EntryCost>>;
type PathCost = u32;
type Coord = (usize,usize);



fn max_size(grid: &Grid) -> usize {
    match MAX_SIZE {
        None => grid.len() - 1,
        Some(max_size) => min(max_size, grid.len()) - 1
    }
}

// Returns the list of valid neighbors to a location
fn neighbors(grid: &Grid, from: &Coord) -> Vec<Coord> {
    let (x,y): Coord = *from;
    let mut result = Vec::new();
    if x < max_size(grid) {
        result.push((x+1, y))
    }
    if y < max_size(grid) {
        result.push((x, y+1))
    }
    if x > 0 {
        result.push((x-1,y))
    }
    if y > 0 {
        result.push((x,y-1))
    }
    result
}


fn contains<T: Eq>(list: &List<T>, item: &T) -> bool {
    match list.head() {
        None => false,
        Some(head) => {
            if head == item {
                true
            } else {
                contains(&list.tail(), item)
            }
        }
    }
}

impl fmt::Display for List<Coord> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn recurse(f: &mut fmt::Formatter, list: List<Coord>) -> fmt::Result {
            match list.head() {
                None => write!(f, "]"),
                Some(head) => {
                    write!(f, ", ({},{})", head.0, head.1)?;
                    recurse(f, list.tail())
                },
            }
        }

        match self.head() {
            None => write!(f, "[]"),
            Some(head) => {
                write!(f, "[")?;
                write!(f, "({},{})", head.0, head.1)?;
                recurse(f, self.tail())?;
                Ok(())
            },
        }
    }
}


fn show_cost(cost: &Option<PathCost>) -> String {
    match cost {
        None => "None".to_string(),
        Some(cost) => cost.to_string()
    }
}



#[allow(dead_code)]
enum PrintLevel {
    Nothing,
    NewBestValue,
    AllDetails,
}
impl PrintLevel {
    fn all(&self) -> bool {
        match self {
            PrintLevel::Nothing => false,
            PrintLevel::NewBestValue => false,
            PrintLevel::AllDetails => true,
        }
    }

    fn best_val(&self) -> bool {
        match self {
            PrintLevel::Nothing => false,
            PrintLevel::NewBestValue => true,
            PrintLevel::AllDetails => true,
        }
    }
}


// Recursively, tries all paths except those known to be worse than something
// already seen.
fn find_best_path_exhaustively(grid: &Grid) -> PathCost {

    // Returns the total cost for the cheapest path from here to the end that is cheaper than
    // best_cost OR None if there isn't one.
    //
    // grid is the grid we're navigating,
    // previous is the path BEFORE the last step,
    // last_step is the most recent step taken,
    // leading_cost is the cost of all steps up and including last_step
    //
    // It can return None because it isn't possible to get to the end from here or because all
    // such paths are more expensive than best_cost.
    fn best_path_from(print: &PrintLevel, grid: &Grid, best_cost: Option<PathCost>, previous: List<Coord>, last_coord: &Coord, leading_cost: PathCost, indent: &str) -> Option<PathCost> {
        if print.all() {println!("{}best_path_from(_, _, {}, ({},{}), {}, {})", indent, show_cost(&best_cost), last_coord.0, last_coord.1, previous, leading_cost);}
        if *last_coord == (max_size(grid), max_size(grid)) {
            if print.best_val() {println!("{}  cost is {} for path {}", indent, leading_cost, previous.prepend(*last_coord));}
            assert!(best_cost.is_none() || best_cost.unwrap() > leading_cost); // We shouldn't get here unless it's going to be better
            Some(leading_cost)
        } else {
            let mut best_known_cost: Option<PathCost> = best_cost;
            let mut best_neighbor_cost: Option<PathCost> = None;
            for neighbor in neighbors(grid, &last_coord) {
                if !contains(&previous, &neighbor) {
                    let cost_to_last_coord = leading_cost + (grid[neighbor.1][neighbor.0] as u32);
                    // recuse ONLY if it's got a chance of improving on what we know about
                    if best_known_cost.is_none() || best_known_cost.unwrap() > cost_to_last_coord {
                        let new_indent = format!("{}  ", indent);
                        let better_cost = best_path_from(print, grid, best_known_cost, previous.prepend(*last_coord), &neighbor, cost_to_last_coord, &new_indent);
                        assert!(best_cost.is_none() || better_cost.is_none() ||  better_cost.unwrap() < best_cost.unwrap()); // has to be better!
                        if better_cost.is_some() {
                            best_neighbor_cost = better_cost;
                            best_known_cost = better_cost
                        }
                    }
                } else {
                    if print.all() {println!("{}  ({},{}) already visited.", indent, neighbor.0, neighbor.1);}
                }
            }
            best_neighbor_cost
        }
    }

    let cost = best_path_from(&PRINT_LEVEL, &grid, None, List::new(), &(0,0), 0, "");
    assert!(cost.is_some()); // there must be SOME best path
    cost.unwrap()
}




fn run() -> Result<(),InputError> {
    let grid: Grid = read_grid_file()?;
    let result = find_best_path_exhaustively(&grid);
    println!("Result: {}", result);
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



#[cfg(test)]
mod test {
    use std::fmt::Write;
    use super::List;
    use super::Coord;
    use super::show_cost;

    #[test]
    fn display_list() {
        let list0: List<Coord> = List::new();
        let list1: List<Coord> = list0.prepend((3,5));
        let list2: List<Coord> = list1.prepend((7,7));
        let list3: List<Coord> = list2.prepend((12,14));

        let mut output0 = String::new();
        write!(&mut output0, "{}", list0).expect("Error writing to String.");
        assert_eq!(output0, "[]");

        let mut output1 = String::new();
        write!(&mut output1, "{}", list1).expect("Error writing to String.");
        assert_eq!(output1, "[(3,5)]");

        let mut output2 = String::new();
        write!(&mut output2, "{}", list2).expect("Error writing to String.");
        assert_eq!(output2, "[(7,7), (3,5)]");

        let mut output3 = String::new();
        write!(&mut output3, "{}", list3).expect("Error writing to String.");
        assert_eq!(output3, "[(12,14), (7,7), (3,5)]");
    }

    #[test]
    fn test_show_cost() {
        assert_eq!(show_cost(&None), "None");
        assert_eq!(show_cost(&Some(3)), "3");
    }
}
