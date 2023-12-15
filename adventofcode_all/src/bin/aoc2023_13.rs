use std::fmt::{Display, Formatter};
use anyhow;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Coord(usize,usize);

#[derive(Debug)]
pub struct Grid {
    bound: Coord,
    data: Vec<bool>,
}


impl Grid {
    fn value(&self, coord: Coord) -> bool {
        assert!(coord.0 < self.bound.0 && coord.1 < self.bound.1);
        self.data[self.bound.0 * coord.1 + coord.0]
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}


impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for y in 0..self.bound.1 {
            for x in 0..self.bound.0 {
                write!(f, "{}", if self.value(Coord(x,y)) {'#'} else {'.'})?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}



type Input = Vec<Grid>;



mod parse {
    use super::{Input, Grid, Coord};
    use std::fs;
    use itertools::Itertools;
    use nom;
    use nom::IResult;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_13.txt")?;
        match Grid::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    impl Grid {
        /// Parses the input. Assumes the grid is at least 1 row and at least one
        /// column or this will fail in various ways.
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
               nom::multi::many1(
                   nom::sequence::terminated(
                       nom::multi::many1(
                            nom::character::complete::one_of(".#")
                       ),
                       nom::character::complete::line_ending
                   )
               ),
               |rows: Vec<Vec<char>>| {
                    let height = rows.len();
                    assert!(height >= 1);
                    assert!(rows.iter().map(|x| x.len()).all_equal());
                    let width = rows[0].len();
                    assert!(width >= 1);
                    let bound = Coord(width, height);
                    let data = rows.iter()
                        .map(|row| {
                            row.iter()
                                .map(|c| match c {
                                    '.' => false,
                                    '#' => true,
                                    _ => panic!("invalid char '{}'", c),
                                })
                        })
                        .flatten()
                        .collect();
                    Grid{bound, data}
                }
            )(input)
        }

        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            nom::multi::separated_list1(
                nom::character::complete::line_ending,
                Self::parse
            )(input)
        }

    }

}


// ======= Compute =======

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Orient {
    Row, Col
}

impl Orient {
    fn coord(&self, with: usize, against: usize) -> Coord {
        match self {
            Orient::Row => Coord(against, with),
            Orient::Col => Coord(with, against),
        }
    }

    fn with(&self, coord: Coord) -> usize {
        match self {
            Orient::Row => coord.0,
            Orient::Col => coord.1,
        }
    }

    fn against(&self, coord: Coord) -> usize {
        match self {
            Orient::Row => coord.1,
            Orient::Col => coord.0,
        }
    }
}


impl Grid {

    /// Returns a particular row or column (depending on orient) from the
    /// Grid as a Vec of bool.
    fn line(&self, orient: Orient, v: usize) -> Vec<bool> {
        assert!(v < orient.against(self.bound));
        (0..orient.with(self.bound))
            .map(|w| self.value(orient.coord(v,w)))
            .collect()
    }

    /// Returns the x position just before the leftmost mirror column if there
    /// is a mirror column, or None if there isn't.
    fn find_mirror(&self, orient: Orient) -> Option<usize> {
        'pairs:
        for v0 in 0..(orient.against(self.bound) - 1) {
            let line0 = self.line(orient, v0);
            let line1 = self.line(orient, v0 + 1); // FIXME: could be more efficient if I kept them
            if line0 == line1 {
                // we have a pair... check the rest for mirroring
                let mut k = 1;
                while v0 >= k && v0 + 1 + k < orient.against(self.bound) {
                    let line_left = self.line(orient, v0 - k);
                    let line_right = self.line(orient, v0 + 1 + k);
                    if line_left != line_right {
                        // It failed to mirror; go on and check the next pair
                        continue 'pairs;
                    }
                    k += 1;
                }
                // we got to an edge... that's a mirror!
                return Some(v0);
            }
        }
        // tried all the pairs and didn't find a mirror
        None
    }
}


/// Returns the "summarize" score for this list of Grids.
fn summarize_grids(grids: &Vec<Grid>) -> usize {
    let mut summary = 0;
    for grid in grids {
        let mirror_row = grid.find_mirror(Orient::Row);
        let mirror_col = grid.find_mirror(Orient::Col);
        let value = match (mirror_row, mirror_col) {
            (None, None) => panic!("No mirror!"),
            (Some(_), Some(_)) => panic!("Mirror both ways!"),
            (Some(v), None) => (v + 1) * 100,
            (None, Some(v)) => v + 1,
        };
        summary += value;
    }
    summary
}

// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let summary = summarize_grids(input);
    println!("The summary is {}", summary);
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
