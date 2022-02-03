mod eznom;

use std::fs;
use std::io;
use std::collections::HashSet;
use std::collections::BTreeSet;
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

const ELECTRON: &str = "e";



/// Finds the shortest way to synthesize the goal starting from ELECTRON. Returns
/// Some(count-of-steps-needed) or None if it's impossible.
fn synthesize(replacements: &Vec<Replacement>, goal: &String) -> Option<usize> {
    if goal == ELECTRON {
        Some(0)
    } else {
        // FIXME: I have some concern that there might be overlapping matches. But for the moment
        //   I'm somewhat convinced that it doesn't happen.
        // The subgoals are going to be in order. Because I'm trying them in order by
        //   size, the first one that succeeds MUST be the shortest solution. So if we
        //   find ANY solution, we return it.
        let subgoals = shrink_one_step(replacements, goal);
        for (_, subgoal) in subgoals {
            if let Some(n) = synthesize(replacements, &subgoal) {
                return Some(n + 1);
            }
        }
        return None;
    }
}

/// Returns the list (as (size,String) tuples so it will be sorted by order of least size first)
/// of sub-goals one step smaller than goal.
fn shrink_one_step(replacements: &Vec<Replacement>, goal: &String) -> BTreeSet<(usize, String)> {
    let mut answer: BTreeSet<(usize, String)> = BTreeSet::new();
    for replacement in replacements {
        for (pos, s) in goal.match_indices(&replacement.output) {
            let before = &goal[0..pos];
            let after = &goal[(pos + s.len())..];
            let subgoal: String = [before, &replacement.input, after].join("");
            answer.insert((subgoal.len(), subgoal));
        }
    }
    answer
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


fn part_b(replacement_problem: &ReplacementProblem) -> Result<(), io::Error> {
    let steps = synthesize(&replacement_problem.replacements, &replacement_problem.start);
    match steps {
        Some(n) => println!("The synthesis can be done in {} steps.", n),
        None => println!("The synthesis cannot be done."),
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
