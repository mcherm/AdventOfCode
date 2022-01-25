mod eznom;

#[macro_use]
extern crate lazy_static;

use std::fs;
use std::io;
use std::collections::HashMap;
use nom::multi::many0 as nom_many0;
use nom::character::complete::u32 as nom_value;
use nom::character::complete::alpha1 as nom_alpha1;
use nom::sequence::tuple as nom_tuple;
use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::newline as nom_newline;
use nom::multi::separated_list0 as nom_separated_list0;
use eznom::type_builder;

type Tags = HashMap<String,u32>;

#[derive(Debug, Clone)]
struct Aunt {
    aunt_num: u32,
    tags: Tags,
}



impl Aunt {

    /// Returns true if all fields present in self.tags match against
    /// a field in expect.
    pub fn matches(&self, expect: &Tags) -> bool {
        for (key, val) in self.tags.iter() {
            if expect.get(key).unwrap() != val {
                return false;
            }
        }
        true
    }

    /// Performs a match based on the special rules of part 2.
    pub fn matches_2(&self, expect: &Tags) -> bool {
        for (key, val) in self.tags.iter() {
            let v = expect.get(key).unwrap();
            match key.as_str() {
                "cats" | "trees" => if val <= v {
                    return false;
                },
                "pomeranians" | "goldfish" => if val >= v {
                    return false;
                },
                _ => if val != v {
                    return false;
                },

            }
        }
        true
    }

    pub fn parse_list(input: &str) -> nom::IResult<&str, Vec<Self>> {
        nom_many0(Self::parse)(input)
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("Sue "),
            nom_value,
            nom_tag(": "),
            nom_separated_list0(
                nom_tag(", "),
                nom_tuple((
                    nom_alpha1,
                    nom_tag(": "),
                    nom_value,
                ))
            ),
            nom_newline,
        ))(s);
        let build =
            |(_, aunt_num, _, tag_vec, _): (&str, u32, &str, Vec<(&str, &str, u32)>, char)| {
                let tags = tag_vec.into_iter().map(|(tag,_,val)| (tag.to_string(),val)).collect();
                Aunt{aunt_num, tags}
            };
        type_builder(recognize, build)(input)
    }
}


lazy_static!{
    static ref ANALYSIS: Tags = HashMap::from([
        ("children", 3),
        ("cats", 7),
        ("samoyeds", 2),
        ("pomeranians", 3),
        ("akitas", 0),
        ("vizslas", 0),
        ("goldfish", 5),
        ("trees", 3),
        ("cars", 2),
        ("perfumes", 1),
    ].map(|(k,v)| (k.to_string(), v)));
}


fn input() -> Result<Vec<Aunt>, io::Error> {
    let s = fs::read_to_string("input/2015/16/input.txt")?;
    match Aunt::parse_list(&s) {
        Ok(("", aunts)) => Ok(aunts),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}




fn part_a(aunts: &Vec<Aunt>) -> Result<(), io::Error> {
    for aunt in aunts {
        if aunt.matches(&ANALYSIS) {
            println!("Sue {} matches the analysis.", aunt.aunt_num);
        }
    }
    Ok(())
}


fn part_b(aunts: &Vec<Aunt>) -> Result<(), io::Error> {
    for aunt in aunts {
        if aunt.matches_2(&ANALYSIS) {
            println!("In the second case, Sue {} matches the analysis.", aunt.aunt_num);
        }
    }
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
