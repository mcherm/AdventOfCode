use std::fmt::{Display, Formatter};
use anyhow;
use itertools::Itertools;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Coord(usize,usize);


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Rock {
    Round, Square, Empty
}

#[derive(Debug)]
pub struct Grid {
    data: Vec<Vec<Rock>>,
}


impl Grid {
    fn new(data: Vec<Vec<Rock>>) -> Self {
        assert!(data.len() > 0);
        assert!(data[0].len() > 0);
        assert!(data.iter().map(|row| row.len()).all_equal());
        Grid{data}
    }

    fn width(&self) -> usize {
        self.data[0].len()
    }

    fn height(&self) -> usize {
        self.data.len()
    }

    fn value(&self, coord: Coord) -> Rock {
        assert!(coord.0 < self.width() && coord.1 < self.height());
        self.data[coord.1][coord.0]
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

impl Display for Rock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Rock::Round => 'O',
            Rock::Square => '#',
            Rock::Empty => '.',
        })
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for y in 0..self.height() {
            for x in 0..self.width() {
                write!(f, "{}", self.value(Coord(x,y)))?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}



type Input = Grid;



mod parse {
    use super::{Input, Grid, Rock};
    use std::fs;
    use anyhow::anyhow;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_14.txt")?;
        let data_result: Result<Vec<Vec<Rock>>, anyhow::Error> = s.lines().map(|line| {
            line.chars().map(|c| match c {
                'O' => Ok(Rock::Round),
                '#' => Ok(Rock::Square),
                '.' => Ok(Rock::Empty),
                _ => Err(anyhow!("invalid character in grid")),
            }).collect()
        }).collect();
        Ok(Grid::new(data_result?))
    }

}


// ======= Compute =======


impl Grid {
    /// Gives the overall load for the given column.
    fn column_load(&self, x: usize) -> usize {
        assert!(x < self.width());
        let mut load = 0;
        let mut floating_rounds = 0;
        for y in (0..self.height()).rev() {
            match self.value(Coord(x,y)) {
                Rock::Round => {
                    floating_rounds += 1;
                }
                Rock::Square => {
                    let min = self.height() - y - floating_rounds;
                    let max = self.height() - y;
                    load += (min..max).sum::<usize>();
                    floating_rounds = 0;
                }
                Rock::Empty => {}
            }
        }
        // -- handle remaining floating rounds --
        let min = self.height() + 1 - floating_rounds;
        let max = self.height() + 1;
        load += (min..max).sum::<usize>();

        load
    }

    /// Find the total load (part 1).
    fn load(&self) -> usize {
        (0..self.width()).map(|y| self.column_load(y)).sum()
    }
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let load = input.load();
    println!("The load was {}", load);
}


fn part_b(_input: &Input) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
