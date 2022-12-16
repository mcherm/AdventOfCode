
extern crate anyhow;


use std::fs;
use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use std::collections::HashMap;
use advent_lib::astar::{
    solve_with_astar, State,
    grid::{Coord, GridVec, GridMove, taxicab_dist, moves_from}
};


// ======= Switches =======

const PRINT_EVERY_N_MOVES: usize = 0;


// ======= Parsing =======

fn input() -> Result<InputGrid, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_12.txt")?;
    InputGrid::parse(&s)
}

struct InputGrid {
    values: Vec<Vec<char>>
}

impl InputGrid {
    fn parse<'a>(input: &'a str) -> Result<Self, anyhow::Error> {
        let values: Vec<Vec<char>> = input.lines().map(|line| line.chars().collect::<Vec<char>>() ).collect();
        if values.len() == 0 {
            return Err(anyhow!("No rows in the map."));
        }
        let row_len = values[0].len();
        if ! values.iter().all(|x| x.len() == row_len) {
            return Err(anyhow!("Rows not all the same length."));
        }
        Ok(InputGrid{values})
    }
}

// ======= Calculations =======

type Height = char;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct HeightMap {
    spots: GridVec<Height>,
    start: Coord,
    end: Coord,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct WanderState<'a> {
    height_map: &'a HeightMap,
    pos: Coord,
}


impl HeightMap {
    fn new(input: &InputGrid) -> Result<Self, anyhow::Error> {
        let height = input.values.len();
        let width = input.values.first().unwrap().len();
        let mut start_opt: Option<(usize,usize)> = None;
        let mut end_opt: Option<(usize,usize)> = None;
        let mut chars: Vec<Vec<char>> = vec![vec!['.'; width]; height];
        for (y, row) in input.values.iter().enumerate() {
            for (x, ch) in row.iter().enumerate() {
                let height: Height = match ch {
                    'S' => {
                        match start_opt {
                            Some(_) => return Err(anyhow!("Multiple start locations.")),
                            None => start_opt = Some((x,y)),
                        };
                        'a'
                    }
                    'E' => {
                        match end_opt {
                            Some(_) => return Err(anyhow!("Multiple end locations.")),
                            None => end_opt = Some((x,y)),
                        };
                        'z'
                    }
                    'a' ..= 'z' => {*ch}
                    _ => return Err(anyhow!("Invalid character.")),
                };
                chars[y][x] = height;
            }
        }
        let start = start_opt.ok_or(anyhow!("No starting location."))?;
        let end = end_opt.ok_or(anyhow!("No ending location."))?;
        let spots = GridVec::from_vec2d(&chars);
        Ok(HeightMap{spots, start, end})
    }
}


/// This is a helper used for Display of HeightMap and WanderState. It is passed a
/// GridVec<Height> and prints it out, except that it is ALSO passed a dictionary
/// of Coord -> character mappings which override the values in the GridVec.
fn fmt_letters_with_overlaps(f: &mut Formatter<'_>, grid: &GridVec<Height>, fixed: &HashMap<Coord, char>) -> std::fmt::Result {
    let width = grid.size().0;
    for (i, ch) in grid.iter().enumerate() {
        let x = i % width;
        let y = i / width;
        let display_ch = fixed.get(&(x,y)).unwrap_or(&ch);
        write!(f, "{}", display_ch)?;
        if (i + 1) % width == 0 {
            writeln!(f)?;
        }
    }
    Ok(())
}


impl Display for HeightMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_letters_with_overlaps(f, &self.spots, &HashMap::from([
            (self.start, 'S'),
            (self.end, 'E'),
        ]))
    }
}

impl<'a> Display for WanderState<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        fmt_letters_with_overlaps(f, &self.height_map.spots, &HashMap::from([
            (self.height_map.start, 'S'),
            (self.height_map.end, 'E'),
            (self.pos, '*'),
        ]))
    }
}



impl<'a> State for WanderState<'a> {
    type TMove = GridMove;

    fn is_winning(&self) -> bool {
        self.pos == self.height_map.end
    }

    fn min_moves_to_win(&self) -> usize {
        taxicab_dist(self.pos, self.height_map.end)
    }

    fn avail_moves(&self) -> Vec<Self::TMove> {
        moves_from(self.pos, self.height_map.spots.size()).into_iter()
            .filter(|mv| {
                let from_height = self.height_map.spots.get(&mv.from());
                let to_height = self.height_map.spots.get(&mv.to());
                (i64::from(u32::from(*to_height))) - (i64::from(u32::from(*from_height))) <= 1 // not more than 1 step up
            })
            .collect()
    }

    fn enact_move(&self, mv: &Self::TMove) -> Self {
        WanderState{ height_map: self.height_map, pos: mv.to() }
    }
}

/// This is just a wrapper allowing us to define Display for a pair of things.
struct DisplayPath<'a>(&'a HeightMap, &'a Vec<GridMove>);

impl<'a> Display for DisplayPath<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut hash_map = HashMap::from([
            (self.0.end, 'E'),
            (self.0.start, 'S'),
        ]);
        for mv in self.1.iter() {
            hash_map.insert(mv.from(), mv.direction_to_ascii_picture());
        }
        fmt_letters_with_overlaps(f, &self.0.spots, &hash_map)
    }
}



// ======= main() =======

fn part_a(input: &InputGrid) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    let height_map = &HeightMap::new(input)?;
    let pos = height_map.start.clone();
    let initial_state = WanderState{height_map, pos};
    if let Some(solution) = solve_with_astar(&initial_state, PRINT_EVERY_N_MOVES) {
        println!("It was solved");
        println!("{}", DisplayPath(height_map, &solution));
        println!();
        println!("That took {} steps.", solution.len());
    } else {
        Err(anyhow!("No path found."))?;
    }
    Ok(())
}


fn part_b(_input: &InputGrid) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
