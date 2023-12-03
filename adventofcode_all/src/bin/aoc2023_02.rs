
use anyhow;


// ======= Parsing =======

#[derive(Debug)]
struct CubeAndColor<'a> {
    num: u32,
    color: &'a str,
}

#[derive(Debug)]
struct CubeSet {
    red: u32,
    green: u32,
    blue: u32,
}

#[derive(Debug)]
pub struct Game {
    game_num: u32,
    pulls: Vec<CubeSet>
}


mod parse {
    use std::fs;
    use super::{CubeSet, Game};
    use nom;
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::tag,
        character::complete::line_ending,
    };
    use nom::character::complete::u32 as nom_num;
    use crate::CubeAndColor;


    pub fn input<'a>() -> Result<Vec<Game>, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_02.txt")?;
        match Game::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    impl<'a> CubeAndColor<'a> {
        fn parse<'b: 'a>(input: &'b str) -> IResult<&'b str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom_num,
                    tag(" "),
                    alt((
                        tag("red"),
                        tag("blue"),
                        tag("green"),
                    ))
                )),
                |(num, _, color)| CubeAndColor{num, color}
            )(input)
        }
    }

    impl CubeSet {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::multi::separated_list1( tag(", "), CubeAndColor::parse),
                |cube_and_colors| {
                    let mut red = 0;
                    let mut green = 0;
                    let mut blue = 0;
                    for cube_and_color in cube_and_colors {
                        match cube_and_color.color {
                            "red" => red += cube_and_color.num,
                            "green" => green +=  cube_and_color.num,
                            "blue" => blue += cube_and_color.num,
                            _ => panic!("invalid color"),
                        }
                    }
                    CubeSet{red, green, blue}
                }
            )(input)
        }
    }

    impl Game {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    tag("Game "),
                    nom_num,
                    tag(": "),
                    nom::multi::separated_list1( tag("; "), CubeSet::parse)
                )),
                |(_, game_num, _, pulls)| {
                    Game{game_num, pulls}
                }
            )(input)
        }

        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            nom::multi::many1( nom::sequence::terminated(Self::parse, line_ending) )(input)
        }

    }
}


// ======= Compute =======

impl Game {
    fn power(&self) -> u32 {
        let max_red = self.pulls.iter()
            .map(|x| x.red)
            .max()
            .unwrap();
        let max_blue = self.pulls.iter()
            .map(|x| x.blue)
            .max()
            .unwrap();
        let max_green = self.pulls.iter()
            .map(|x| x.green)
            .max()
            .unwrap();
        max_red * max_blue * max_green
    }
}

// ======= main() =======

const MAX_RED: u32 = 12;
const MAX_GREEN: u32 = 13;
const MAX_BLUE: u32 = 14;

fn part_a(data: &Vec<Game>) {
    println!("\nPart a:");
    let sum: u32 = data.iter()
        .filter(|game| {
            game.pulls.iter().all(|cube_set| {
                cube_set.red <= MAX_RED &&
                cube_set.green <= MAX_GREEN &&
                cube_set.blue <= MAX_BLUE
            })
        })
        .map(|game| game.game_num)
        .sum();
    println!("SUM: {:?}", sum);
}


fn part_b(data: &Vec<Game>) {
    println!("\nPart b:");
    let sum: u32 = data.iter()
        .map(|game| game.power())
        .sum();
    println!("Sum of power: {:?}", sum);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
