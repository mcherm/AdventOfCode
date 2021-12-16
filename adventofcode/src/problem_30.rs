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


const PRINT_LEVEL: PrintLevel = PrintLevel::Nothing;
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


/// Returns the size of largest coordinate (one less than the size of the grid).
fn max_coord(grid: &Grid) -> usize {
    match MAX_SIZE {
        None => grid.len() - 1,
        Some(max_size) => min(max_size, grid.len()) - 1
    }
}

// Returns the list of valid neighbors to a location
fn neighbors(grid: &Grid, from: &Coord) -> Vec<Coord> {
    let (x,y): Coord = *from;
    let mut result = Vec::new();
    if x < max_coord(grid) {
        result.push((x+1, y))
    }
    if y < max_coord(grid) {
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
                assert!(best_cost <= memo_data.best_cost);
                if arrival_cost >= memo_data.arrival_cost {
                    // We know this path must be worse
                    return None;
                }
                let arrival_delta = memo_data.arrival_cost - arrival_cost;
                match memo_data.resulting_best {
                    None => {
                    },
                    Some(memo_result) => {
                        let new_result = memo_result - arrival_delta;
                        if new_result < best_cost {
                            if PRINT_LEVEL.best_val() {println!("{}  cost is {}", indent, new_result);}
                            return Some(new_result)
                        } else {
                            return None;
                        }
                    },
                }
            }
            None => {},
        }

        let resulting_best: Option<PathCost>;
        if *start_coord == (max_coord(grid), max_coord(grid)) {
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

        memo_cache.insert(*start_coord, MemoData{best_cost, arrival_cost, resulting_best});

        resulting_best
    }


    let mut memo_cache: MemoCache = HashMap::new();
    let cost = best_path_from(&grid, &mut memo_cache, WORST_PATH_COST, &(0,0), 0, "");
    assert!(cost.is_some()); // there must be SOME best path
    cost.unwrap()
}


// produces coordinates, but in the order we want to traverse them (starting from
// the biggest coordinates and sweeping diagonally
fn coords_in_tail_order(grid: &Grid) -> Vec<Coord> {
    let max_c = max_coord(grid);
    let mut result = Vec::new();
    let mut sum_this_diagonal = max_c * 2;
    loop {
        let mut x = if sum_this_diagonal > max_c {sum_this_diagonal - max_c} else {0};
        loop {
            let y = sum_this_diagonal - x;
            result.push((x, y));
            if x == max_c || x == sum_this_diagonal {
                break;
            } else {
                x += 1;
            }
        }
        if sum_this_diagonal == 0 {
            break;
        } else {
            sum_this_diagonal -= 1;
        }
    }
    result
}



fn find_best_path_more_quickly(grid: &Grid) -> PathCost {

    type CostToEnd = Vec<Vec<Option<PathCost>>>;
    let mut cost_to_end: CostToEnd = grid.iter().map(|row| {
        row.iter().map(|_| {
            None
        }).collect()
    }).collect();

    fn print_known(cost_to_end: &CostToEnd) {
        for y in 0..cost_to_end.len() {
            for x in 0..cost_to_end.len() {
                match cost_to_end[y][x] {
                    None => print!("(*),"),
                    Some(x) => print!("{:3},", x),
                };
            }
            println!();
        }
    }

    /// This updates cost_to_end for a given coord, using only the data already populated
    /// It ALSO (recursively) updates any neighbors who now have a better path because
    /// of this one.
    fn find_cost(grid: &Grid, cost_to_end: &mut CostToEnd, coord: &Coord) -> () {
        if PRINT_LEVEL.best_val() {println!("\nfind_cost({},{}): ", coord.0, coord.1);}
        let max_c = max_coord(grid);
        cost_to_end[coord.1][coord.0] = if *coord == (max_c, max_c) {
            Some(0) // there's no cost to get from the end to the end!
        } else {
            let mut new_cost_from_here_2: Option<PathCost> = None;
            for neighbor in neighbors(grid, coord) {
                match cost_to_end[neighbor.1][neighbor.0] {
                    None => {
                        // This neighbor isn't known. Skip them.
                        if PRINT_LEVEL.all() {println!("Neighbor: ({},{}) isn't known.",neighbor.0, neighbor.1);}
                    },
                    Some(neighbor_known_cost) => {
                        // This neighbor is known; consider them as an option
                        let neighbor_risk: EntryCost = grid[neighbor.1][neighbor.0];
                        let cost_via_neighbor = neighbor_known_cost + neighbor_risk as PathCost;
                        if PRINT_LEVEL.all() {println!("Neighbor: ({},{}) has cost {} and needs {} totaling {}",neighbor.0, neighbor.1, cost_via_neighbor, neighbor_risk, cost_via_neighbor);}
                        new_cost_from_here_2 = match new_cost_from_here_2 {
                            None => {
                                // This is the first usable neighbor. Use this one
                                Some(cost_via_neighbor)
                            }
                            Some(known_new_cost) => {
                                // This isn't the first usable neighbor. Use the better one
                                if cost_via_neighbor < known_new_cost {
                                    Some(cost_via_neighbor)
                                } else {
                                    Some(known_new_cost)
                                }
                            }
                        };
                    },
                }
            }
            assert!(new_cost_from_here_2.is_some()); // Given how we walk the grid, there's always SOME path
            new_cost_from_here_2
        };


        /// Recursively update folks because of a better path
        ///
        /// coord: the location that will get reworked. This MUST have a known cost.
        /// better_path_cost: the new (better) path cost
        fn rework(grid: &Grid, cost_to_end: &mut CostToEnd, coord: &Coord) {
            let my_risk: EntryCost = grid[coord.1][coord.0];
            let my_cost: PathCost = cost_to_end[coord.1][coord.0].unwrap();
            let cost_to_get_there_via_me: PathCost = my_cost + (my_risk as PathCost);
            for neighbor in neighbors(grid, coord) {
                match cost_to_end[neighbor.1][neighbor.0] {
                    None => {}, // Neighbor isn't populated yet
                    Some(neighbor_current_cost) => {
                        // Neighbor IS populated... is going via us better?
                        if PRINT_LEVEL.all() {println!("Considering neighbor ({},{}): its cost is {} and going via me is {}", neighbor.0, neighbor.1, neighbor_current_cost, cost_to_get_there_via_me);}
                        if cost_to_get_there_via_me < neighbor_current_cost {
                            if PRINT_LEVEL.all() {println!("Should definitely rework neighbor ({},{}). It used {} but going via me is only {}", neighbor.0, neighbor.1, neighbor_current_cost, cost_to_get_there_via_me);}
                            cost_to_end[neighbor.1][neighbor.0] = Some(cost_to_get_there_via_me);
                            // Recurse because neighbor changed
                            rework(grid, cost_to_end, &neighbor);
                        }
                    },
                }
            }
        }
        rework(grid, cost_to_end, coord);

    }

    if PRINT_LEVEL.all() {println!("BEFORE:");}
    if PRINT_LEVEL.all() {print_known(&cost_to_end);}

    for coord in coords_in_tail_order(&grid) {
        find_cost(&grid, &mut cost_to_end, &coord);
        if PRINT_LEVEL.all() {print_known(&cost_to_end);}
    }

    cost_to_end[0][0].unwrap() // Return the answer in the start location
}



fn make_big_grid(grid: &Grid) -> Grid {
    let mut big_grid: Grid = Vec::new();
    for big_y in 0..5 {
        for grid_row in grid {
            let mut big_grid_row = Vec::new();
            for big_x in 0..5 {
                for value in grid_row {
                    let new_value = ((value + big_x + big_y - 1) % 9) + 1;
                    assert!(new_value > 0 && new_value < 10);
                    big_grid_row.push(new_value);
                }
            }
            big_grid.push(big_grid_row);
        }
    }
    big_grid
}




fn run() -> Result<(),InputError> {
    let grid: Grid = read_grid_file()?;
    let orig_version = false;
    if orig_version {
        let big_grid: Grid = make_big_grid(&grid);
        let result = find_best_path_exhaustively(&big_grid);
        println!("Result: {}", result);
    } else {
        let big_grid: Grid = make_big_grid(&grid);
        let result = find_best_path_more_quickly(&big_grid);
        println!("Result: {}", result);
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



#[cfg(test)]
mod test {
    // use super::Coord;
    use super::coords_in_tail_order;
    use super::Grid;
    use super::show_cost;
    use super::find_best_path_exhaustively;
    use super::find_best_path_more_quickly;

    #[test]
    fn test_show_cost() {
        assert_eq!(show_cost(&None), "None");
        assert_eq!(show_cost(&Some(3)), "3");
    }

    #[test]
    fn test_tail_order() {
        let grid: Grid = vec![
            vec![1,2,3],
            vec![4,5,6],
            vec![7,8,9],
        ];
        let coords = coords_in_tail_order(&grid);
        assert_eq!(coords, vec![
            (2,2), (1,2), (2,1), (0,2), (1,1), (2,0), (0,1), (1,0), (0,0)
        ])
    }

    #[test]
    fn test_simple_grid() {
        let grid: Grid = vec![
            vec![1,2,3],
            vec![4,5,6],
            vec![9,8,7],
        ];
        let result_exh = find_best_path_exhaustively(&grid);
        assert_eq!(result_exh, 18);
        let result_qck = find_best_path_more_quickly(&grid);
        assert_eq!(result_qck, 18);
    }


    #[test]
    fn test_with_backtracking() {
        let grid: Grid = vec![
            vec![1,1,1,1,9],
            vec![9,9,9,1,9],
            vec![9,1,1,1,9],
            vec![9,1,9,9,9],
            vec![9,1,1,1,1],
        ];
        let result_exh = find_best_path_exhaustively(&grid);
        assert_eq!(result_exh, 12);
        let result_qck = find_best_path_more_quickly(&grid);
        assert_eq!(result_qck, 12);
    }

}
