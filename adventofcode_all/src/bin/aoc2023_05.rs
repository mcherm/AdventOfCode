use anyhow;


// ======= Parsing =======

type Num = u64;

#[derive(Debug)]
pub struct AlmanacRange {
    dest_start: Num,
    source_start: Num,
    len: Num,
}

#[derive(Debug)]
pub struct AlmanacMap {
    source: String,
    dest: String,
    rows: Vec<AlmanacRange>,
}

#[derive(Debug)]
pub struct Almanac {
    seeds: Vec<Num>,
    maps: Vec<AlmanacMap>
}


mod parse {
    use std::fs;
    use super::{Almanac, AlmanacMap, AlmanacRange};
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
    };
    use nom::character::complete::u64 as nom_num;


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
                    nom::multi::many0(AlmanacRange::parse),
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


    impl AlmanacRange {
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
                |(dest_start, _, source_start, _, len, _)| {
                    AlmanacRange{dest_start, source_start, len}
                }
            )(input)
        }
    }
}


// ======= Compute =======

mod compute {
    use im::HashMap;
    use crate::{Almanac, AlmanacMap};
    use super::Num;


    /// This is a structure, made from an AlmanacMap, that performs the actual mapping.
    #[derive(Debug, Clone)]
    struct AlmanacMapper<'a>(&'a AlmanacMap);

    /// This is performs mapping overall.
    #[derive(Debug)]
    pub struct AlmanacSolver<'a> {
        seeds: &'a Vec<Num>,
        maps_by_dest: HashMap<&'a str, AlmanacMapper<'a>>,
    }

    impl<'a> AlmanacMapper<'a> {
        /// Takes a source number and produces the right dest value.
        fn map(&self, num: Num) -> Num {
            for range in self.0.rows.iter() {
                if num >= range.source_start && num < range.source_start + range.len {
                    return range.dest_start + num - range.source_start;
                }
            }
            num
        }
    }

    impl<'a> AlmanacSolver<'a> {
        pub fn new(almanac: &'a Almanac) -> Self {
            let seeds = &almanac.seeds;
            let maps_by_dest = almanac.maps.iter()
                .map(|m| (
                    m.dest.as_str(),
                    AlmanacMapper(m)
                ))
                .collect();
            AlmanacSolver{seeds, maps_by_dest}
        }

        /// This finds the list of names we need to traverse, from seed up through
        /// location (in that order).
        fn path_from_seed(&self) -> Vec<&'a str> {
            let mut answer = Vec::new();
            let mut name = "location";
            loop {
                answer.push(name);
                if name == "seed" {
                    break;
                }
                name = self.maps_by_dest.get(name).unwrap().0.source.as_str();
            }
            answer.reverse();
            answer
        }

        /// Given a seed number and the path from seed, this finds the corresponding location.
        fn find_location(&self, path: &Vec<&'a str>, seed: Num) -> Num {
            let mut n = seed;
            for name in path.iter().skip(1) {
                n = self.maps_by_dest.get(name).unwrap().map(n);
            }
            n
        }

        /// Solves part a.
        pub fn find_lowest_location(&self) -> Num {
            let path = self.path_from_seed();
            self.seeds.iter()
                .map(|seed| self.find_location(&path, *seed))
                .min()
                .unwrap()
        }

    }
}


// ======= main() =======

use compute::AlmanacSolver;

fn part_a(data: &Almanac) {
    println!("\nPart a:");
    let solver = AlmanacSolver::new(data);
    let lowest_location = solver.find_lowest_location();
    println!("Lowest Location: {}", lowest_location);
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
