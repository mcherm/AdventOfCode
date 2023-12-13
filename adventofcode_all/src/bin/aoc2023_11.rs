use std::fmt::{Display, Formatter};
use anyhow;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Coord(usize,usize);

#[derive(Debug)]
pub struct SpaceImage {
    bound: Coord,
    data: Vec<bool>,
    empty_cols: Vec<usize>,
    empty_rows: Vec<usize>,
}

type Input = SpaceImage;


impl SpaceImage {
    /// Returns true if the given location is a galaxy. The coord must be in bounds.
    fn is_galaxy(&self, coord: Coord) -> bool {
        assert!(coord.0 < self.bound.0 && coord.1 < self.bound.1);
        self.data[self.bound.0 * coord.1 + coord.0]
    }
}


impl Display for SpaceImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for y in 0..self.bound.1 {
            for x in 0..self.bound.0 {
                write!(f, "{}", if self.is_galaxy(Coord(x,y)) {'#'} else {'.'})?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}


mod parse {
    use std::fs;
    use std::collections::HashSet;
    use super::{Input, SpaceImage, Coord};


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_11.txt")?;
        SpaceImage::parse(&s)
    }


    impl SpaceImage {
        /// Parses the input. Assumes the grid is at least 1 row and at least one
        /// column or this will fail in various ways.
        fn parse(input: &str) -> Result<Self, anyhow::Error> {
            let mut width = 0;
            let mut height = 0;
            let mut data: Vec<bool> = Vec::new();
            let mut occupied_cols: HashSet<usize> = HashSet::new();
            let mut empty_rows: Vec<usize> = Vec::new();
            for (y,line) in input.lines().enumerate() {
                height += 1;
                assert!(y + 1 == height);
                let mut line_width = 0;
                let mut row_is_empty = true;
                for (x,c) in line.chars().enumerate() {
                    line_width += 1;
                    assert!(x + 1 == line_width);
                    let is_galaxy = match c {
                        '.' => false,
                        '#' => true,
                        c => panic!("unexpected character, '{}'", c),
                    };
                    if is_galaxy {
                        row_is_empty = false;
                        occupied_cols.insert(x);
                    }
                    data.push(is_galaxy);
                }
                anyhow::ensure!(y == 0 || line_width == width, "uneven lines");
                width = line_width;
                if row_is_empty {
                    empty_rows.push(y);
                }
            }
            let empty_cols = (0..width).filter(|x| !occupied_cols.contains(x)).collect();
            assert!(width >0 && height > 0);
            let bound = Coord(width, height);

            Ok(SpaceImage{bound, data, empty_cols, empty_rows})
        }

    }

}


// ======= Compute =======


impl SpaceImage {
    /// Given two coords that are within the SpaceImage and that are both galaxies,
    /// this returns that taxicab distance between them after expansion. It panics
    /// on any other input.
    fn expanded_distance(&self, expansion: usize, a: Coord, b: Coord) -> usize {
        assert!(a.0 < self.bound.0 && a.1 < self.bound.1);
        assert!(b.0 < self.bound.0 && b.1 < self.bound.1);
        assert!(self.is_galaxy(a) && self.is_galaxy(b));
        let base_dist = a.0.abs_diff(b.0) + a.1.abs_diff(b.1);
        let expanded_x = self.empty_cols.iter()
            .filter(|x| **x > a.0.min(b.0) && **x < a.0.max(b.0))
            .count();
        let expanded_y = self.empty_rows.iter()
            .filter(|y| **y > a.1.min(b.1) && **y < a.1.max(b.1))
            .count();
        base_dist + (expansion - 1) * (expanded_x + expanded_y)
    }

    /// Returns the sum of the distance between all pairs of galaxies.
    fn expanded_distance_all_pairs(&self, expansion: usize) -> usize {
        let mut galaxies: Vec<Coord> = Vec::new();
        for y in 0..self.bound.1 {
            for x in 0..self.bound.0 {
                let coord = Coord(x,y);
                if self.is_galaxy(coord) {
                    galaxies.push(coord)
                }
            }
        }
        let mut answer: usize = 0;
        for (i, gal_1) in galaxies.iter().enumerate() {
            for gal_2 in galaxies.iter().skip(i + 1) {
                answer += self.expanded_distance(expansion, *gal_1, *gal_2);
            }
        }
        answer
    }
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    println!("Dist between all galaxy pairs is {}", input.expanded_distance_all_pairs(2));
}


fn part_b(input: &Input) {
    println!("\nPart b:");
    println!("Dist between all galaxy pairs is {}", input.expanded_distance_all_pairs(1000000));
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
