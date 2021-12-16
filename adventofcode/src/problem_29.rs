//
// NOTES
//
// OK, I learned a few things:
//   * I need to memoize my function.
//   * I might not need to avoid going back to items I've used before, because if I go
//       down-and-to-the-right first I'll find at least one solution at first and after
//       that looping paths will get pruned due to having worse scores.
//


use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::cmp::min;
use std::collections::HashMap;


const PRINT_LEVEL: PrintLevel = PrintLevel::NewBestValue;
const MAX_SIZE: Option<usize> = None; // cut things off at this size
const ALLOW_BACKTRACKING: bool = true;


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
const WORST_PATH_COST: PathCost = u32::MAX;
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
    if ALLOW_BACKTRACKING {
        if x > 0 {
            result.push((x-1,y))
        }
        if y > 0 {
            result.push((x,y-1))
        }
    }
    result
}



#[allow(dead_code)]
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

    struct MemoData {
        best_cost: PathCost,
        arrival_cost: PathCost,
        resulting_best: Option<PathCost>,
    }
    type MemoCache = HashMap<Coord, MemoData>;

    // Returns the cost for the cheapest path from here to the end is less than max_cost, or
    // None if there isn't one.
    //
    // grid is the grid we're navigating. It doesn't change as we recurse.
    // memo_cache is the data we're using to memoize the function. The object is modified as we pass it around.
    // best_cost is the best overall PathCost we have seen, or None if we haven't seen one ever.
    // start_coord is the location we are starting from.
    // arrival_cost is the cost of all steps up and including arriving on start_coord.
    //
    // This can return None because it isn't possible to get to the end from here or because all
    // such paths are more expensive than best_cost.
    //
    fn best_path_from(
        grid: &Grid,
        memo_cache: &mut MemoCache,
        best_cost: PathCost,
        start_coord: &Coord,
        arrival_cost: PathCost,
        indent: &str
    ) -> Option<PathCost> {
        if PRINT_LEVEL.all() {println!("{}best_path_from(_, _, best:{}, at:({},{}), arrival:{})", indent, best_cost, start_coord.0, start_coord.1, arrival_cost);}

        match memo_cache.get(start_coord) {
            Some(memo_data) => {
//                println!("{}  YES found in cache: with bound of {} and arrival_cost of {} we had {}", indent, memo_data.best_cost, memo_data.arrival_cost, show_cost(&memo_data.resulting_best));
                assert!(best_cost <= memo_data.best_cost);
                if arrival_cost >= memo_data.arrival_cost {
                    // We know this path must be worse
                    return None;
                }
                let arrival_delta = memo_data.arrival_cost - arrival_cost;
//                let best_delta = memo_data.best_cost - best_cost;
//                println!("{}  MEMO_ANALYZE: arrival_delta = {}; best_delta = {}", indent, arrival_delta, best_delta);
//                if arrival_delta > best_delta {
//                    println!("{}  * Have to do it over as arival_delta > best_delta", indent);
//                } else {
//                    println!("{}  * We should be able to use the memo.", indent);
//                }
                match memo_data.resulting_best {
                    None => {
//                        println!("{}  MEMO-FAIL: The memo couldn't find an answer, but this time we have a lower target.", indent);
                    },
                    Some(memo_result) => {
                        let new_result = memo_result - arrival_delta;
                        if new_result < best_cost {
                            if PRINT_LEVEL.best_val() {println!("{}  cost is {}", indent, new_result);}
                            return Some(new_result)
                        } else {
//                            println!("{}  MEMO: Quick return it's no better.", indent); // FIXME: Debug
                            return None;
                        }
                    },
                }
            }
            None => {
//                println!("{}  Not found in cache.", indent);
            },
        }

        let resulting_best: Option<PathCost>;
        if *start_coord == (max_size(grid), max_size(grid)) {
            if PRINT_LEVEL.best_val() {println!("{}  cost is {}", indent, arrival_cost);}
            assert!(best_cost > arrival_cost); // We shouldn't get here unless it's going to be better
            resulting_best = Some(arrival_cost);
        } else {
            let mut best_known_cost: PathCost = best_cost;
            let mut neighbor_cost_beating_best_known: Option<PathCost> = None;
            for neighbor in neighbors(grid, &start_coord) {
                let cost_to_last_coord = arrival_cost + (grid[neighbor.1][neighbor.0] as u32);
                // recuse ONLY if it's got a chance of improving on what we know about
                if best_known_cost > cost_to_last_coord {
                    let new_indent = format!("{}  ", indent);
                    let better_cost = best_path_from(grid, memo_cache, best_known_cost, &neighbor, cost_to_last_coord, &new_indent);
                    assert!(better_cost.is_none() ||  better_cost.unwrap() < best_cost); // has to be better!
                    // FIXME: This assignment could me made simpler
                    if better_cost.is_some() {
                        neighbor_cost_beating_best_known = better_cost;
                        best_known_cost = better_cost.unwrap();
                    }
                } else {
                    if PRINT_LEVEL.all() {println!("{}  ({},{}) is no better.", indent, neighbor.0, neighbor.1);}
                }
            }
            resulting_best = neighbor_cost_beating_best_known;
        }

//        println!("{}  Memoizing ({},{}) -> best:{}, arrive:{}, result:{}", indent, start_coord.0, start_coord.1, best_cost, arrival_cost, show_cost(&resulting_best));
        memo_cache.insert(*start_coord, MemoData{best_cost, arrival_cost, resulting_best});

        resulting_best
    }


    let mut memo_cache: MemoCache = HashMap::new();
    let cost = best_path_from(&grid, &mut memo_cache, WORST_PATH_COST, &(0,0), 0, "");
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
    use super::show_cost;

    #[test]
    fn test_show_cost() {
        assert_eq!(show_cost(&None), "None");
        assert_eq!(show_cost(&Some(3)), "3");
    }
}
