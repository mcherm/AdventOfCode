use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::fmt::Write;
use rect::{Rect, intersect};
use nom::sequence::tuple as nom_tuple;
use nom::bytes::complete::tag as nom_tag;
use nom::multi::many0 as nom_many0;
use nom::character::complete::newline as nom_newline;
use nom::branch::alt as nom_alt;


mod rect {
    use std::fmt::{Display, Formatter};
    use std::cmp::{max, min};
    use nom::sequence::tuple as nom_tuple;
    use nom::character::complete::i32 as nom_value;
    use nom::bytes::complete::tag as nom_tag;

    pub type Coord = i32;

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub struct Rect {
        pub x0: Coord,
        pub y0: Coord,
        pub x1: Coord,
        pub y1: Coord,
    }

    impl Rect {
        pub fn new(x0: Coord, y0: Coord, x1: Coord, y1: Coord) -> Self {
            assert!(x0 <= x1 && y0 <= y1);
            Rect{x0, y0, x1: x1 + 1, y1: y1 + 1}
        }

        pub fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_value,
                nom_tag(","),
                nom_value,
                nom_tag(" through "),
                nom_value,
                nom_tag(","),
                nom_value,
            ))(input).map(|(rest, (x0, _, y0, _, x1, _, y1))| (rest, Rect::new(x0, y0, x1, y1)))
        }

        /// Returns true if the given point is contained in this rect
        pub fn contains(&self, point: Pair) -> bool {
            self.x0 <= point.0 && point.0 < self.x1 && self.y0 <= point.1 && point.1 < self.y1
        }

        /// Returns true if the other intersects this
        pub fn intersects(&self, other: &Rect) -> bool {
            self.x0 < other.x1 && self.x1 > other.x0 && self.y0 < other.y1 && self.y1 > other.y0
        }

        /// Returns the area of the rect
        pub fn area(&self) -> i32 {
            (self.x1 - self.x0) * (self.y1 - self.y0)
        }
    }

    impl Display for Rect {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}, {}, {}, {}]", self.x0, self.y0, self.x1, self.y1)
        }
    }


    type Pair = (Coord, Coord);

    #[derive(Debug, Eq, PartialEq)]
    pub struct OverlapSet<T> {
        pub a_only: Vec<T>,
        pub both: Vec<T>,
        pub b_only: Vec<T>,
    }


    /// Given two pairs, (x0, x1) for some coordinate x this will figure
    /// how to break up along this axis and return an OverlapSet of Pairs.
    fn overlay_axis(a: Pair, b: Pair) -> OverlapSet<Pair> {
        // There are 7 sections that CAN exist... at most 3 WILL exist.
        // The sections are just-a, just-b, b-left, b-right, a-left, a-right, and overlap
        let mut answer = OverlapSet{a_only: Vec::new(), b_only: Vec::new(), both: Vec::new()};
        if a.1 <= b.0 || b.1 <= a.0 {
            // they don't overlap; we need just-a and just-b
            answer.a_only.push(a);
            answer.b_only.push(b);
        } else {
            // If they overlap, we need a section of overlap
            answer.both.push((max(a.0, b.0), min(a.1, b.1)));
            if b.0 < a.0 { // we need a section of b to the left of a
                answer.b_only.push((b.0, a.0));
            }
            if a.1 < b.1 { // we need a section of b to the right of a
                answer.b_only.push((a.1, b.1));
            }
            if a.0 < b.0 { // we need a section of a to the left of b
                answer.a_only.push((a.0, b.0));
            }
            if b.1 < a.1 { // we need a section of a to the right of b
                answer.a_only.push((b.1, a.1));
            }
        }
        answer
    }


    /// Given two rectangles that may intersect, this returns several lists of
    /// rectangles: a list that are in both, a list that are only in r1, and
    /// a list that are only in r2. None of the rectangles on the lists intersect
    /// each other, and those in list 0 and 1 together form r1 while those in list
    /// 0 and 2 together form r2.
    pub fn intersect(a: &Rect, b: &Rect) -> OverlapSet<Rect> {
        let mut answer = OverlapSet{a_only: Vec::new(), b_only: Vec::new(), both: Vec::new()};
        // --- First, we'll split along the x axis ---
        let x_splits = overlay_axis((a.x0, a.x1), (b.x0, b.x1));
        for (x0, x1) in x_splits.a_only {
            answer.a_only.push(Rect{x0, x1, y0: a.y0, y1: a.y1});
        }
        for (x0, x1) in x_splits.b_only {
            answer.b_only.push(Rect{x0, x1, y0: b.y0, y1: b.y1});
        }
        for (x0, x1) in x_splits.both {
            // Where the rectangles overlap we need to find the overlaps in the Y direction
            // --- now split what's in both along the y axis ---
            let y_splits = overlay_axis((a.y0, a.y1), (b.y0, b.y1));
            for (y0, y1) in y_splits.a_only {
                answer.a_only.push(Rect{x0, x1, y0, y1});
            }
            for (y0, y1) in y_splits.b_only {
                answer.b_only.push(Rect{x0, x1, y0, y1});
            }
            for (y0, y1) in y_splits.both {
                answer.both.push(Rect{x0, x1, y0, y1});
            }
        }
        // --- Return the result ---
        answer
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_overlay_axis() {
            assert_eq!(
                OverlapSet{
                    a_only: vec![(5,10), (15,20)],
                    b_only: vec![],
                    both:   vec![(10,15)],
                },
                overlay_axis((5,20), (10,15)),
            );
        }

        #[test]
        fn test_intersect_no_overlap() {
            assert_eq!(
                OverlapSet{
                    a_only: vec![Rect::new(0,0,4,4)],
                    b_only: vec![Rect::new(10,0,14,4)],
                    both:   vec![],
                },
                intersect(&Rect::new(0,0,4,4), &Rect::new(10,0,14,4))
            );
        }

        #[test]
        fn test_intersect_corners() {
            assert_eq!(
                OverlapSet{
                    a_only: vec![Rect::new(0,0,4,9), Rect::new(5,0,9,4)],
                    b_only: vec![Rect::new(10,5,14,14), Rect::new(5,10,9,14)],
                    both:   vec![Rect::new(5,5,9,9)],
                },
                intersect(&Rect::new(0,0,9,9), &Rect::new(5,5,14,14))
            );
        }

        #[test]
        fn test_intersect_hotdog() {
            assert_eq!(
                OverlapSet{
                    a_only: vec![Rect::new(0,5,4,9), Rect::new(10,5,14,9)],
                    b_only: vec![Rect::new(5,0,9,4), Rect::new(5,10,9,14)],
                    both:   vec![Rect::new(5,5,9,9)],
                },
                intersect(&Rect::new(0,5,14,9), &Rect::new(5,0,9,14))
            );
        }

        #[test]
        fn test_intersect_b_inside() {
            assert_eq!(
                OverlapSet{
                    a_only: vec![Rect::new(0,0,4,14), Rect::new(10,0,14,14), Rect::new(5,0,9,4), Rect::new(5,10,9,14)],
                    b_only: vec![],
                    both:   vec![Rect::new(5,5,9,9)],
                },
                intersect(&Rect::new(0,0,14,14), &Rect::new(5,5,9,9))
            );
        }
    }
}


fn input() -> Result<Vec<Instruction>, io::Error> {
    let s = fs::read_to_string("input/2015/06/input.txt")?;
    match parse_instructions(&s) {
        Ok(("", instructions)) => Ok(instructions),
        Ok((_, _)) => panic!("Extra input"),
        Err(_) => panic!("Invalid input"),
    }
}



#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Instruction {
    TurnOn(Rect),
    TurnOff(Rect),
    Toggle(Rect),
}


#[derive(Debug)]
struct LightGrid {
    on_regions: Vec<Rect>
}


struct SimpleLightGrid {
    on: [[bool; 1000]; 1000]
}


impl Instruction {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        nom_alt((
            nom_tuple((
                nom_tag("turn on "),
                Rect::parse,
                nom_newline,
            )),
            nom_tuple((
                nom_tag("turn off "),
                Rect::parse,
                nom_newline,
            )),
            nom_tuple((
                nom_tag("toggle "),
                Rect::parse,
                nom_newline,
            )),
        ))(input).map(|(rest, (tag, rect, _))|
            (rest, match tag {
                "turn on " => Instruction::TurnOn(rect),
                "turn off " => Instruction::TurnOff(rect),
                "toggle " => Instruction::Toggle(rect),
                _ => panic!("invalid match")
            })
        )
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::TurnOn(rect) => write!(f, "turn on {}", rect),
            Instruction::TurnOff(rect) => write!(f, "turn off {}", rect),
            Instruction::Toggle(rect) => write!(f, "toggle {}", rect),
        }
    }
}


fn parse_instructions(input: &str) -> nom::IResult<&str, Vec<Instruction>> {
    nom_many0(Instruction::parse)(input)
}

impl LightGrid {
    fn new() -> Self {
        LightGrid{on_regions: Vec::new()}
    }

    fn area(&self) -> i32 {
        self.on_regions.iter().map(|x| x.area()).sum()
    }

    fn turn_on(&mut self, rect: &Rect) {
        self.turn_off(rect);
        self.on_regions.push(rect.clone());
    }

    fn turn_off(&mut self, rect: &Rect) {
        let mut new_on_regions = vec![];
        self.on_regions.retain(|reg| {
            if reg.intersects(rect) {
                // -- It overlaps, so we'll keep just the shards of it not covered by rect --
                let overlaps = intersect(reg, rect);
                new_on_regions.extend(overlaps.a_only);
                false
            } else {
                // -- It doesn't overlap, so keep it --
                true
            }
        });
        self.on_regions.extend(new_on_regions);
    }

    fn toggle(&mut self, rect: &Rect) {
        let mut toggle_regions = vec![*rect];
        let mut new_on_regions = vec![];
        self.on_regions.retain(|reg| {
            let mut keep_reg = true;
            let mut new_toggle_regions = vec![];
            toggle_regions.retain(|toggle_reg| {
                if reg.intersects(toggle_reg) {
                    keep_reg = false;
                    let overlaps = intersect(reg, toggle_reg);
                    new_on_regions.extend(overlaps.a_only); // keep shards of reg not covered by toggle_rec
                    new_toggle_regions.extend(overlaps.b_only); // continue toggling shards of toggle_rec not intersecting
                    false // don't keep the toggle_region
                } else {
                    true // keep the toggle_region
                }
            });
            toggle_regions.extend(new_toggle_regions);
            keep_reg
        });
        self.on_regions.extend(new_on_regions);
        self.on_regions.extend(toggle_regions);
    }

    fn apply(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::TurnOn(rect) => self.turn_on(rect),
            Instruction::TurnOff(rect) => self.turn_off(rect),
            Instruction::Toggle(rect) => self.toggle(rect),
        }
    }

    #[allow(dead_code)]
    fn to_string(&self, size: i32) -> String {
        let mut output = String::new();
        for i in 0..size {
            for j in 0..size {
                let is_lit = self.on_regions.iter().any(|x| x.contains((i,j)));
                let c = if is_lit {'*'} else {'.'};
                write!(output, "{}", c).unwrap();
            }
            writeln!(output).unwrap();
        }
        output
    }
}

impl SimpleLightGrid {
    fn new() -> Self {
        SimpleLightGrid{on: [[false; 1000]; 1000]}
    }

    fn area(&self) -> i32 {
        let mut count = 0;
        for x in 0..1000 {
            for y in 0..1000 {
                if self.on[y][x] {
                    count += 1
                };
            }
        }
        count
    }

    fn apply(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::TurnOn(rect) => {
                for x in rect.x0..rect.x1 {
                    for y in rect.y0..rect.y1 {
                        self.on[y as usize][x as usize] = true;
                    }
                }
            },
            Instruction::TurnOff(rect) => {
                for x in rect.x0..rect.x1 {
                    for y in rect.y0..rect.y1 {
                        self.on[y as usize][x as usize] = false;
                    }
                }
            },
            Instruction::Toggle(rect) => {
                for x in rect.x0..rect.x1 {
                    for y in rect.y0..rect.y1 {
                        self.on[y as usize][x as usize] = !self.on[y as usize][x as usize];
                    }
                }
            },
        }
    }
}


fn part_a(instructions: &Vec<Instruction>) -> Result<(), io::Error> {
    let mut s_grid: SimpleLightGrid = SimpleLightGrid::new();
    let mut c_grid: LightGrid = LightGrid::new();
    for instruction in instructions {
        println!("Applying {}", instruction);
        s_grid.apply(instruction);
        c_grid.apply(instruction);
        assert_eq!(s_grid.area(), c_grid.area());
    }
    println!("The area is {}", c_grid.area());
    Ok(())
}

fn part_b(_instructions: &Vec<Instruction>) -> Result<(), io::Error> {
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}


// It is NOT 647473 (that's too high).
// Simple version says 377891