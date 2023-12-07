use anyhow;


// ======= Parsing =======

type Num = u32;

#[derive(Debug)]
pub struct Race {
    time: Num,
    dist: Num,
}



mod parse {
    use std::fs;
    use std::iter::zip;
    use itertools::Itertools;
    use super::{Race, Num};
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
    };
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Vec<Race>, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_06.txt")?;
        match Race::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    impl Race {
        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            nom::combinator::map(
                nom::sequence::tuple((
                    tag("Time:"),
                    nom::multi::many1(
                        nom::sequence::preceded(
                            nom::multi::many1(tag(" ")),
                            nom_num
                        )
                    ),
                    nom::character::complete::line_ending,
                    tag("Distance:"),
                    nom::multi::many1(
                        nom::sequence::preceded(
                            nom::multi::many1(tag(" ")),
                            nom_num
                        )
                    ),
                    nom::character::complete::line_ending,
                )),
                |(_, times, _, _, dists, _)| {
                    zip(times, dists)
                        .map(|(time, dist): (Num,Num)| Race{time, dist})
                        .collect_vec()
                }
            )(input)
        }
    }

}


// ======= Compute =======

mod compute {
    use super::Race;

    impl Race {

        /// This counts the number of different ways to win. This particular verion isn't
        /// smart, it just uses brute force to try them all.
        pub fn brute_force_ways_to_win(&self) -> usize {
            (0 ..= self.time)
                .map(|hold_time| hold_time * (self.time - hold_time))
                .filter(|dist| *dist > self.dist)
                .count()
        }

    }
}


// ======= main() =======


fn part_a(data: &Vec<Race>) {
    println!("\nPart a:");
    let answer: usize = data.iter()
        .map(|race| race.brute_force_ways_to_win())
        .product();
    println!("Total ways to win: {}", answer);
}


fn part_b(_data: &Vec<Race>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
