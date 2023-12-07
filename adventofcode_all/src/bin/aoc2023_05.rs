use anyhow;


// ======= Parsing =======

type Num = u32;

#[derive(Debug)]
pub struct AlmanacMapRow(Num, Num, Num);

#[derive(Debug)]
pub struct AlmanacMap {
    source: String,
    dest: String,
    rows: Vec<AlmanacMapRow>,
}

#[derive(Debug)]
pub struct Almanac {
    seeds: Vec<Num>,
    maps: Vec<AlmanacMap>
}


mod parse {
    use std::fs;
    use super::{Almanac, AlmanacMap, AlmanacMapRow};
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
    };
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Almanac, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_05.txt")?;
        match Almanac::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    impl Almanac {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    tag("seeds: "),
                    nom::multi::separated_list1(
                        tag(" "),
                        nom_num,
                    ),
                    nom::character::complete::line_ending,
                    nom::character::complete::line_ending,
                    nom::multi::separated_list1(
                        nom::character::complete::line_ending,
                        AlmanacMap::parse,
                    ),
                )),
                |(_, seeds, _, _, maps)| {
                    Almanac{seeds, maps: maps}
                }
            )(input)
        }
    }

    impl AlmanacMap {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom::character::complete::alpha1::<&'a str, _>,
                    tag("-to-"),
                    nom::character::complete::alpha1::<&'a str, _>,
                    tag(" map:"),
                    nom::character::complete::line_ending,
                    nom::multi::many0(AlmanacMapRow::parse),
                )),
                |(source, _, dest, _, _, rows)| {
                    AlmanacMap{
                        source: source.to_string(),
                        dest: dest.to_string(),
                        rows,
                    }
                }
            )(input)
        }
    }


    impl AlmanacMapRow {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom_num,
                    tag(" "),
                    nom_num,
                    tag(" "),
                    nom_num,
                    nom::character::complete::line_ending,
                )),
                |(a, _, b, _, c, _)| {
                    AlmanacMapRow(a,b,c)
                }
            )(input)
        }
    }
}


// ======= Compute =======



// ======= main() =======


fn part_a(data: &Almanac) {
    println!("\nPart a:");
    println!("data: {:?}", data);
}


fn part_b(_data: &Almanac) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
