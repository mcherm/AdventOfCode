use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use rect::Rect;
use nom::sequence::tuple as nom_tuple;
use nom::bytes::complete::tag as nom_tag;
use nom::multi::many0 as nom_many0;
use nom::character::complete::newline as nom_newline;
use nom::branch::alt as nom_alt;


mod rect {
    use std::fmt::{Display, Formatter};
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

    }

    impl Display for Rect {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}, {}, {}, {}]", self.x0, self.y0, self.x1, self.y1)
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct OverlapSet<T> {
        pub a_only: Vec<T>,
        pub both: Vec<T>,
        pub b_only: Vec<T>,
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


struct SimpleLightGrid {
    on: [[bool; 1000]; 1000]
}

struct BrightnessLightGrid {
    bright: [[u8; 1000]; 1000]
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

impl BrightnessLightGrid {
    fn new() -> Self {
        BrightnessLightGrid{bright: [[0; 1000]; 1000]}
    }

    fn brightness(&self) -> i32 {
        let mut sum: i32 = 0;
        for x in 0..1000 {
            for y in 0..1000 {
                sum += self.bright[y as usize][x as usize] as i32;
            }
        }
        sum
    }

    fn apply(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::TurnOn(rect) => {
                for x in rect.x0..rect.x1 {
                    for y in rect.y0..rect.y1 {
                        self.bright[y as usize][x as usize] += 1;
                    }
                }
            },
            Instruction::TurnOff(rect) => {
                for x in rect.x0..rect.x1 {
                    for y in rect.y0..rect.y1 {
                        if self.bright[y as usize][x as usize] > 0 {
                            self.bright[y as usize][x as usize] -= 1;
                        }
                    }
                }
            },
            Instruction::Toggle(rect) => {
                for x in rect.x0..rect.x1 {
                    for y in rect.y0..rect.y1 {
                        self.bright[y as usize][x as usize] += 2;
                    }
                }
            },
        }
    }
}


fn part_a(instructions: &Vec<Instruction>) -> Result<(), io::Error> {
    let mut grid: SimpleLightGrid = SimpleLightGrid::new();
    for instruction in instructions {
        grid.apply(instruction);
    }
    println!("The area is {}", grid.area());
    Ok(())
}

fn part_b(instructions: &Vec<Instruction>) -> Result<(), io::Error> {
    let mut grid: BrightnessLightGrid = BrightnessLightGrid::new();
    for instruction in instructions {
        grid.apply(instruction);
    }
    println!("The brightness is {}", grid.brightness());
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