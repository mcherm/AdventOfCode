mod eznom;

use std::fs;
use std::io;
// use std::fmt::{Display, Formatter};
use std::collections::HashSet;
use nom::character::complete::alpha1 as nom_alpha1;
use nom::bytes::complete::tag as nom_tag;
use nom::sequence::tuple as nom_tuple;
use nom::character::complete::newline as nom_newline;
use nom::multi::many0 as nom_many0;
use eznom::type_builder;


#[derive(Debug)]
struct Replacement {
    input: String,
    output: String,
}

#[derive(Debug)]
struct ReplacementProblem {
    replacements: Vec<Replacement>,
    start: String,
}


impl Replacement {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_alpha1,
            nom_tag(" => "),
            nom_alpha1,
            nom_newline
        ))(s);
        let build = |(input, _, output, _): (&str, &str, &str, char)|
            Replacement{input: input.to_string(), output: output.to_string()};
        type_builder(recognize, build)(input)
    }

    pub fn parse_list(input: &str) -> nom::IResult<&str, Vec<Self>> {
        nom_many0(Self::parse)(input)
    }

}

impl ReplacementProblem {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            Replacement::parse_list,
            nom_newline,
            nom_alpha1,
            nom_newline,
        ))(s);
        let build = |(replacements, _, start, _): (Vec<Replacement>, char, &str, char)|
            ReplacementProblem{replacements, start: start.to_string()};
        type_builder(recognize, build)(input)
    }

    fn count_distinct_after_one_application(&self) -> usize {
        let mut outputs: HashSet<String> = HashSet::new();
        for replacement in &self.replacements {
            for (pos, _) in self.start.match_indices(&replacement.input) {
                let before = &self.start[0..pos];
                let after = &self.start[(pos + replacement.input.len())..];
                let output: String = [before, &replacement.output, after].join("");
                outputs.insert(output);
            }
        }
        outputs.len()
    }
}

fn input() -> Result<ReplacementProblem, io::Error> {
    let s = fs::read_to_string("input/2015/19/input.txt")?;
    match ReplacementProblem::parse(&s) {
        Ok(("", replacement_problem)) => Ok(replacement_problem),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



fn part_a(replacement_problem: &ReplacementProblem) -> Result<(), io::Error> {
    println!("After one replacement there are {} distinct strings.",
             replacement_problem.count_distinct_after_one_application()
    );
    Ok(())
}


fn part_b(_replacement_problem: &ReplacementProblem) -> Result<(), io::Error> {
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
