use std::fmt::{Display, Formatter};
use anyhow;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Coord(usize,usize);

#[derive(Debug)]
pub struct Grid {
    bound: Coord,
    rows: Vec<Vec<bool>>,
    cols: Vec<Vec<bool>>,
}


impl Grid {
    fn new<I1, I2>(values: I1) -> Self
        where I1: IntoIterator<Item=I2>, I2: IntoIterator<Item=bool>
    {
        let mut width = 0;
        let mut rows = Vec::new();
        let mut first_row = true;
        for row_values in values {
            let row: Vec<bool> = row_values.into_iter().collect();
            if first_row {
                width = row.len();
                first_row = false;
            } else {
                if row.len() != width {
                    panic!("not all rows are the same length")
                }
            }
            rows.push(row);
        }
        assert!(width >= 1);
        let height = rows.len();
        assert!(height >= 1);
        let bound = Coord(width, height);
        let cols: Vec<Vec<bool>> = (0..width).map(|x| {
            (0..height).map(|y| {
                rows[y][x]
            }).collect()
        }).collect();
        Grid{bound, rows, cols}
    }

    fn value(&self, coord: Coord) -> bool {
        assert!(coord.0 < self.bound.0 && coord.1 < self.bound.1);
        self.cols[coord.0][coord.1]
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
    use super::{Input, Grid};
    use std::fs;
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
        fn parse(input: &str) -> IResult<&str, Self> {
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
                    let data_2 = rows.iter()
                        .map(|row| row.iter().map(|c| match c{
                            '.' => false,
                            '#' => true,
                            _ => panic!("invalid char '{}'", c),
                        }));
                    Grid::new(data_2)
                }
            )(input)
        }

        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
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
            Orient::Row => Coord(with, against),
            Orient::Col => Coord(against, with),
        }
    }

    #[allow(dead_code)]
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
    fn line_as_vec(&self, orient: Orient, a: usize) -> &Vec<bool> {
        assert!(a < orient.against(self.bound));
        match orient {
            Orient::Row => &self.rows[a],
            Orient::Col => &self.cols[a],
        }
    }

    /// Returns a particular row or column (depending on orient) from the
    /// Grid as a Vec of bool.
    fn line<'a>(&self, orient: Orient, offset: usize) -> Line {
        assert!(offset < orient.against(self.bound));
        let values = self.line_as_vec(orient, offset);
        Line{orient, values, offset}
    }

    /// Returns the x position just before the leftmost mirror column if there
    /// is a mirror column, or None if there isn't.
    fn find_mirror(&self, orient: Orient) -> Option<usize> {
        'pairs:
        for v0 in 0..(orient.against(self.bound) - 1) {
            let line0 = self.line_as_vec(orient, v0);
            let line1 = self.line_as_vec(orient, v0 + 1);
            if line0 == line1 {
                // we have a pair... check the rest for mirroring
                let mut k = 1;
                while v0 >= k && v0 + 1 + k < orient.against(self.bound) {
                    let line_left = self.line_as_vec(orient, v0 - k);
                    let line_right = self.line_as_vec(orient, v0 + 1 + k);
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


    /// Returns a list of all NearMirrors with the given orientation an no more than max_smudges
    /// smudges required.
    fn find_near_mirror(&self, orient: Orient, max_smudges: usize) -> Vec<NearMirror> {
        let against_bound = orient.against(self.bound);
        let mut answer = Vec::new();

        'pairs:
        for v0 in 0..(against_bound - 1) {
            let mut mismatches = Vec::new();

            let line0 = self.line(orient, v0);
            let line1 = self.line(orient, v0 + 1);
            let center_near_match = line0.near_match(&line1, max_smudges - mismatches.len());
            if let Some(center_mismatches) = center_near_match {
                mismatches.extend(center_mismatches);
                // we have a pair... check the rest for mirroring
                let mut k = 1; // k is "how far out from the center pair we are"
                while v0 >= k && v0 + 1 + k < against_bound {
                    let line_before = self.line(orient, v0 - k);
                    let line_after= self.line(orient, v0 + 1 + k);
                    let outer_near_match = line_before.near_match(&line_after, max_smudges - mismatches.len());
                    if let Some(outer_mismatches) = outer_near_match {
                        mismatches.extend(outer_mismatches);
                        // we extended it one more, and haven't reached the edge so we'll keep going
                    } else {
                        // It failed to mirror; go on and check the next pair
                        continue 'pairs;
                    }
                    k += 1;
                }
                // we got to an edge... that's a mirror!
                let near_mirror = NearMirror{orient, pos: v0, mismatches};
                answer.push(near_mirror);
            }
        }
        answer
    }

    /// Returns a tuple containing the perfect mirror and the single-mismatch mirror, or
    /// panics if there aren't exactly 1 of each.
    ///
    /// The problem statement (from part 1) guarantees that every Grid we are given will
    /// have exactly one mirror with no mismatches. It is my theory (but I'm not quite 100%
    /// sure of this) that any Grid which has a single, unambiguous "smudge" will necessarily
    /// have exactly one miss-by-one, which will be of the other orientation. I have not
    /// yet mathematically proved this must be the case, but I HAVE confirmed that every one
    /// of my inputs satisfies this requirement.
    ///
    /// What this function does is to verify that this is the case, then return the
    /// single-mismatch mirror. If it encounters something different it will panic with
    /// a message.
    fn find_single_flawed_mirror(&self) -> NearMirror {
        let mut no_mismatch_mirrors: Vec<NearMirror> = Vec::new();
        let mut one_mismatch_mirrors: Vec<NearMirror> = Vec::new();
        for orient in [Orient::Row, Orient::Col] {
            for near_mirror in self.find_near_mirror(orient, 1) {
                match near_mirror.mismatches.len() {
                    0 => no_mismatch_mirrors.push(near_mirror),
                    1 => one_mismatch_mirrors.push(near_mirror),
                    _ => panic!("near mirror has more mismatches than allowed"),
                }
            }
        }

        if no_mismatch_mirrors.len() != 1 {
            panic!("The input grid did not have exactly one mirror solution, as the problem guaranteed.");
        }
        if one_mismatch_mirrors.len() != 1 {
            panic!("The input grid had a pattern I thought was impossible. See find_mirror_and_near_miss() for notes.");
        }

        one_mismatch_mirrors[0].clone()
    }

    /// Given a mirror, this finds near_miss that we add by fixing the smudge. Or it panics if
    /// the Grid isn't one with a single fixable smudge.
    ///
    /// NOTE: Look, the example they provided in the problem WAS WRONG!! It is NOT the case that
    /// "the smudge is at (0,0)" -- it could equally well be at (0,6)! However, both have the
    /// same new line of reflection. So this returns the orientation and position of the new
    /// line of reflection.
    fn find_smudge(&self) -> (Orient, usize) {
        let flawed_mirror = self.find_single_flawed_mirror();
        assert_eq!(flawed_mirror.mismatches.len(), 1);
        (flawed_mirror.orient, flawed_mirror.pos)
    }

}


/// Represents a single Row or Column in a given Grid.
#[derive(Debug)]
struct Line<'a> {
    orient: Orient,
    values: &'a Vec<bool>,
    offset: usize, // the row/column number in the grid
}

/// When a possible line would be a reflection if only a location were toggled,
/// there are always actually TWO locations, either of which could be toggled to
/// make the reflection work. This data structure represents one such pair.
#[derive(Debug, Copy, Clone)]
struct Mismatch([Coord; 2]);


impl<'a> Line<'a> {
    /// Returns the length of the line.
    fn len(&self) -> usize {
        self.values.len()
    }

    /// This can be called to compare two Lines with the same orientation and the
    /// same length. It returns the list of Mismatches which would need to be
    /// toggled for these to match. However, it is also passed a max_mismatches and it
    /// will return None instead of a success if the number of mismatches exceeds that.
    fn near_match(&self, other: &Line, max_mismatches: usize) -> Option<Vec<Mismatch>> {
        assert_eq!(self.orient, other.orient);
        assert_eq!(self.len(), other.len());
        let mut mismatches = Vec::new();
        for v in 0..self.len() {
            if self.values[v] != other.values[v] {
                if mismatches.len() == max_mismatches {
                    return None;
                } else {
                    let p1 = self.orient.coord(v, self.offset);
                    let p2 = other.orient.coord(v, other.offset);
                    mismatches.push( Mismatch([p1 ,p2]) );
                }
            }
        }
        Some(mismatches)
    }
}

impl<'a> Display for Line<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.values.iter().map(|b| if *b {'#'} else {'.'}).collect::<String>())
    }
}


/// A type for returning a near miss. It will have 0 or more smudges, which are individual
/// coords on the mirror that need to be modified to make the reflection work.
#[derive(Debug, Clone)]
struct NearMirror {
    orient: Orient,
    pos: usize,
    mismatches: Vec<Mismatch>,
}



/// Returns the "summarize" score for this list of Grids.
fn summarize_grids(grids: &Vec<Grid>) -> usize {
    let mut summary = 0;
    for grid in grids {
        let mirror_row = grid.find_mirror(Orient::Row);
        let mirror_col = grid.find_mirror(Orient::Col);
        let value = match (mirror_row, mirror_col) {
            (None, None) => panic!("No mirror!"),
            (Some(_), Some(_)) => panic!("Mirror both ways, at row {} and col {}!", mirror_row.unwrap(), mirror_col.unwrap()),
            (Some(v), None) => (v + 1) * 100,
            (None, Some(v)) => v + 1,
        };
        summary += value;
    }
    summary
}


fn summarize_smudged_grids(grids: &Vec<Grid>) -> usize {
    grids.iter()
        .map(|grid| {
            let (orient, pos) = grid.find_smudge();
            match orient {
                Orient::Row => (pos + 1) * 100,
                Orient::Col => pos + 1,
            }
        })
        .sum()
}

// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let summary = summarize_grids(input);
    println!("The summary is {}", summary);
}


fn part_b(input: &Input) {
    println!("\nPart b:");
    let summary = summarize_smudged_grids(input);
    println!("The summary is {}", summary);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
