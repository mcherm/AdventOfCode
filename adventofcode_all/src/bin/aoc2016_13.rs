
extern crate anyhow;

use std::fs;
use anyhow::Error;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::collections::HashMap;


fn input() -> Result<usize, Error> {
    let s = fs::read_to_string("input/2016/input_13.txt")?;
    Ok(s.parse()?)
}

const PRINT_WORK: bool = true;
const START: (usize, usize) = (1,1);
const DESTINATION: (usize,usize) = (31,39);

fn is_open(fav: usize, x: usize, y: usize) -> bool {
    (x * x + 3 * x + 2 * x * y + y + y * y + fav).count_ones() % 2 == 0
}

/// Given an iterable group of (usize,usize) points, this returns (x_max, y_max), the
/// largest first-coord and largest second-coord.
fn max_point<I>(points: I) -> (usize,usize)
    where I: IntoIterator<Item = (usize,usize)> + Clone
{
    (
        points.clone().into_iter().map(|p| p.0).max().unwrap(),
        points.into_iter().map(|p| p.1).max().unwrap()
    )
}

/// Returns a vec of the open points neighboring p.
fn get_neighbors(fav: usize, p: (usize,usize)) -> Vec<(usize,usize)> {
    let mut answer = Vec::with_capacity(4);
    let (x,y) = p;
    if y > 0 && is_open(fav, x, y-1) {
        answer.push((x, y-1));
    }
    if x > 0 && is_open(fav, x-1, y) {
        answer.push((x-1, y));
    }
    if is_open(fav, x+1, y) {
        answer.push((x+1, y));
    }
    if is_open(fav, x, y+1) {
        answer.push((x, y+1));
    }
    answer
}

struct Path {
    points: Vec<(usize,usize)>,
}

impl Path {
    fn new() -> Self {
        Path{points: Vec::new()}
    }

    fn push(&mut self, p: (usize, usize)) {
        self.points.push(p);
    }

    fn pop(&mut self) {
        self.points.pop();
    }

    fn contains(&self, p: &(usize,usize)) -> bool {
        self.points.contains(p)
    }

    fn steps(&self) -> usize {
        self.points.len() - 1
    }

    /// Returns the (max_x, max_y), the largest x and y coordinates in the path.
    fn max_point(&self) -> (usize,usize) {
        if self.points.is_empty() {
            (0,0)
        } else {
            max_point(self.points.clone())
        }
    }
}


/// This prints out the grid (to "just large enough" with a little padding).
fn print_grid(fav: usize, path: &Path) {
    const PADDING: usize = 3;
    let (max_x, max_y) = max_point([START, DESTINATION, path.max_point()]);
    for y in 0..(max_y + PADDING) {
        for x in 0..(max_x + PADDING) {
            let c = if (x,y) == DESTINATION  {
                "X"
            } else if (x,y) == START {
                "O"
            } else if path.contains(&(x,y)) {
                "*"
            } else if is_open(fav, x, y) {
                "."
            } else {
                "#"
            };
            print!("{}", c);
        }
        println!();
    }
}


/// This will find a solution if there is one. But it is NOT guaranteed to find the SHORTEST
/// solution. If no solution is found it probably runs forever, but MIGHT panic instead.
fn explore_grid(fav: usize) -> Path {

    /// The recursive portion. It either returns true and modifies path to a successful path that
    /// goes goes to tip, then continues on to START OR it returns false and leaves path unchanged
    /// to indicate that doing so isn't possible. It's doing a depth-first search which isn't
    /// guaranteed to find the best answer.
    fn explore_from_point(fav: usize, visited: &mut HashSet<(usize,usize)>, path: &mut Path, p: (usize,usize)) -> bool {
        if PRINT_WORK {
            print_grid(fav, &path);
            println!();
        }
        path.push(p);
        visited.insert(p);
        if p == START {
            return true; // return success
        }
        for neighbor in get_neighbors(fav, p) {
            if ! visited.contains(&neighbor) {
                if explore_from_point(fav, visited, path, neighbor) {
                    return true;
                }
            }
        }
        path.pop();
        return false;
    }

    let mut visited: HashSet<(usize,usize)> = HashSet::new();
    let mut path = Path::new();
    let found_path = explore_from_point(fav, &mut visited, &mut path, DESTINATION);
    if ! found_path {
        panic!("Could not find a path!");
    }
    path
}

type Point = (usize,usize);

#[derive(Copy, Clone)]
struct SolutionStep {
    steps: usize,
    prev: Option<Point>
}

impl SolutionStep {
    fn make_first() -> Self {
        SolutionStep{steps: 0, prev: None}
    }

    fn make(steps: usize, prev_p: Point) -> Self {
        SolutionStep{steps, prev: Some(prev_p)}
    }
}

/// This will find (one of) the shortest solution(s) if there is one. If there is no solution
/// it will either run forever or panic.
fn explore_grid_optimal(fav: usize) -> Path {
    let mut visited: HashMap<Point,SolutionStep> = HashMap::new(); // p -> (shortest_steps, prev_p)
    let mut tips: VecDeque<(Point, SolutionStep)> = VecDeque::new(); // queue of (p, prev_p)
    let mut best_solution: Option<SolutionStep> = None;

    tips.push_back((DESTINATION, SolutionStep::make_first()));
    loop {
        let (p, prev_step) = match tips.pop_front() {
            None => break, // We got through everything worth looking at, so we can exit the loop
            Some(next_tip) => next_tip,
        };
        let new_steps = prev_step.steps + 1;
        if best_solution.is_some() && best_solution.unwrap().steps <= new_steps {
            continue; // this can't beat the best we've found so don't bother to look at it
        }
        if let Some(visited_step) = visited.get(&p) {
            if visited_step.steps <= new_steps {
                continue; // already been here in = or < steps, so don't bother to look at it
            }
        }
        // -- we have NOT been here in this few steps before, so we'll try it --
        visited.insert(p, prev_step);
        // -- Maybe we've solved it --
        if p == START {
            match best_solution {
                None => {
                    // -- this is the first solution to be discovered --
                    best_solution = Some(SolutionStep::make(new_steps, p))
                },
                Some(SolutionStep{steps, ..}) if new_steps < steps => {
                    // -- this is better than the previous best solution --
                    best_solution = Some(SolutionStep::make(new_steps, p))
                },
                _ => {}, // It's NOT a new best solution
            }
        } else {
            // -- not solved, let's go ahead and consider the neighbors of p
            for neighbor in get_neighbors(fav, p) {
                tips.push_back((neighbor, SolutionStep::make(new_steps, p)));
            }
        }
    }

    match best_solution {
        None => panic!("No solution can be found."),
        Some(SolutionStep{steps, prev}) => {
            // -- collect the points --
            let mut path = Path::new();
            let mut opt_p = prev;
            while opt_p.is_some() {
                path.push(opt_p.unwrap());
                opt_p = visited.get(&opt_p.unwrap()).unwrap().prev;
            }
            path
        }
    }
}




fn part_a(fav: &usize) {
    println!("\nPart a:");

    let path = explore_grid_optimal(*fav);
    print_grid(*fav, &path);
    println!();
    println!("It takes at least {} steps to complete it.", path.steps())
}


fn part_b(_fav: &usize) {
    println!("\nPart b:");
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
