use anyhow;


// ======= Parsing =======

#[derive(Debug)]
pub struct Card {
    winning: Vec<u32>,
    have: Vec<u32>,
}


mod parse {
    use std::fs;
    use super::{Card};
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
        character::complete::line_ending,
    };
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Vec<Card>, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_04.txt")?;
        match Card::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    impl Card {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    tag("Card"),
                    nom::multi::many1(tag(" ")),
                    nom_num,
                    tag(": "),
                    nom::multi::separated_list1(
                        tag(" "),
                        nom::sequence::preceded(
                            nom::combinator::opt( tag(" ") ), // optional leading space we ignore
                            nom_num, // a number
                        )
                    ),
                    tag(" | "),
                    nom::multi::separated_list1(
                        tag(" "),
                        nom::sequence::preceded(
                            nom::combinator::opt( tag(" ") ), // optional leading space we ignore
                            nom_num, // a number
                        )
                    ),
                )),
                |(_, _, _, _, winning, _, have)| {
                    Card{winning, have}
                }
            )(input)
        }

        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            nom::multi::many1( nom::sequence::terminated(Self::parse, line_ending) )(input)
        }

    }
}


// ======= Compute =======

use std::collections::HashSet;

impl Card {
    fn points(&self) -> u32 {
        let have_set: HashSet<u32> = self.have.iter().copied().collect();
        assert!(have_set.len() == self.have.len());
        let win_count = self.winning.iter().filter(|x| have_set.contains(x)).count();
        if win_count == 0 {
            0
        } else {
            1 << (win_count - 1)
        }
    }
}

// ======= main() =======


fn part_a(data: &Vec<Card>) {
    println!("\nPart a:");
    let sum: u32 = data.iter().map(|x| x.points()).sum();
    println!("Sum of cards: {:?}", sum);
}


fn part_b(_data: &Vec<Card>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
