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
    use super::{Almanac, AlmanacMap, AlmanacRange, Num};
    use std::collections::HashMap;
    use itertools::Itertools;


    /// This is a structure, made from an AlmanacMap, that performs the actual mapping.
    #[derive(Debug, Clone)]
    struct AlmanacMapper<'a>(&'a AlmanacMap);

    /// This is performs mapping overall.
    #[derive(Debug)]
    pub struct AlmanacSolver<'a> {
        seeds: &'a Vec<Num>,
        maps_by_dest: HashMap<&'a str, AlmanacMapper<'a>>,
    }

    /// Represents a range of numbers, with min, top, and len.
    #[derive(Debug)]
    struct Range {
        min: Num,
        top: Num, // one more than the top number
    }

    /// This represents a few ranges of numbers.
    type Ranges = Vec<Range>;


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

        /// Given a range, this runs it through the map and returns a new range or
        /// ranges.
        fn map_range(&self, input_ranges: Ranges) -> Ranges {
            let mut input_ranges: Ranges = input_ranges;
            for row in self.0.rows.iter() {
                input_ranges = input_ranges.iter()
                    .map(|in_range| map_range_to_one_row(in_range, row))
                    .flatten()
                    .collect();
            }
            let output_ranges: Ranges = input_ranges.iter()
                .map(|in_range| Range::new(
                    self.map(in_range.min()),
                    self.map(in_range.max()) + 1, // convert largest, then add 1 to get top not max.
                ))
                .collect();
            output_ranges
        }
    }

    fn map_range_to_one_row(input_range: &Range, row: &AlmanacRange) -> Ranges {
        let a = row.source_start;
        let first_break = if a > input_range.min && a < input_range.top() {
            Some(a)
        } else {
            None
        };
        let b = row.source_start + row.len;
        assert!(b > a);
        let second_break = if b > input_range.min && b < input_range.top() {
            Some(b)
        } else {
            None
        };
        let x = input_range.min();
        let y = input_range.top();
        match (first_break, second_break) {
            (None, None) => vec![Range::new(x, y)],
            (Some(a), None) => vec![Range::new(x, a), Range::new(a, y)],
            (None, Some(b)) => vec![Range::new(x, b), Range::new(b, y)],
            (Some(a), Some(b)) => vec![Range::new(x, a), Range::new(a, b), Range::new(b, y)],
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

        pub fn solve_part_b(&self) -> Num {
            let path = self.path_from_seed();
            let seed_ranges = self.seeds.chunks(2)
                .map(|chunk| {
                    let min = *chunk.get(0).unwrap();
                    let len = *chunk.get(1).unwrap();
                    Range::new(min, min + len)
                })
                .collect_vec();
            let mut ranges = seed_ranges;
            for name in path.iter().skip(1) {
                ranges = self.maps_by_dest.get(name).unwrap().map_range(ranges);
            }
            ranges.iter()
                .map(|range| range.min())
                .min()
                .unwrap()
        }
    }

    impl Range {
        fn new(min: Num, top: Num) -> Self {
            assert!(top > min);
            Range{min, top}
        }

        fn min(&self) -> Num {
            self.min
        }

        /// Returns the largest one
        fn max(&self) -> Num {
            self.top - 1
        }

        /// Returns the upper bound (which is NOT in the range)
        fn top(&self) -> Num {
            self.top
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


fn part_b(data: &Almanac) {
    println!("\nPart b:");
    let solver = AlmanacSolver::new(data);
    let lowest_location = solver.solve_part_b();
    println!("Lowest Location (using ranges): {}", lowest_location);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
