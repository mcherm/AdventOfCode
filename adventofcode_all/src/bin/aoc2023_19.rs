use std::fmt::{Debug, Display, Formatter};
use anyhow;
use im::HashMap;
use advent_lib::asciienum::AsciiEnum;


// ======= Constants =======


// ======= Parsing =======

type Num = u32;



AsciiEnum!{
    enum Rating { X('x'), M('m'), A('a'), S('s') }
}


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Part {
    x: Num,
    m: Num,
    a: Num,
    s: Num,
}

AsciiEnum!{
    enum CompareOp { Less('<'), More('>'), }
}

#[allow(dead_code)] // FIXME: Remove once I use it
#[derive(Debug)]
pub struct Rule {
    rating: Rating,
    compare_op: CompareOp,
    value: Num,
    target: String,
}

#[allow(dead_code)] // FIXME: Remove once I use it
#[derive(Debug)]
pub struct Workflow {
    name: String,
    rules: Vec<Rule>,
    default_target: String,
}

#[allow(dead_code)] // FIXME: Remove once I use it
#[derive(Debug)]
pub struct ProblemSet {
    workflows: Vec<Workflow>,
    parts: Vec<Part>,
}

type Input = ProblemSet;

#[derive(Debug)]
struct InvalidRatingError(char);



impl Display for Part {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Part[{},{},{},{}]", self.x, self.m, self.a, self.s)
    }
}

impl Rule {
    fn new<T: ToString>(rating: Rating, compare_op: CompareOp, value: Num, target: T) -> Self {
        Self{rating, compare_op, value, target: target.to_string()}
    }
}

impl Workflow {
    fn new<T1: ToString, T2: ToString>(name: T1, rules: Vec<Rule>, default_target: T2) -> Self {
        Self{name: name.to_string(), rules, default_target: default_target.to_string()}
    }
}



mod parse {
    use super::{Input, Part, Rating, CompareOp, Rule, Workflow, ProblemSet};
    use std::fs;
    use nom;
    use nom::IResult;
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_19.txt")?;
        match Input::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    impl Rule {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    Rating::parse,
                    CompareOp::parse,
                    nom_num,
                    nom::bytes::complete::tag(":"),
                    nom::character::complete::alpha1,
                )),
                |(rating, compare_op, value, _, target)| Rule::new(rating, compare_op, value, target)
            )(input)
        }
    }

    impl Workflow {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom::character::complete::alpha1,
                    nom::bytes::complete::tag("{"),
                    nom::multi::separated_list1(
                        nom::bytes::complete::tag(","),
                        Rule::parse,
                    ),
                    nom::bytes::complete::tag(","),
                    nom::character::complete::alpha1,
                    nom::bytes::complete::tag("}"),
                )),
                |(name, _, rules, _, default_target, _)| {
                    Self::new(name, rules, default_target)
                }
            )(input)
        }

        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
            nom::multi::many1(
                nom::sequence::terminated(
                    Self::parse,
                    nom::character::complete::line_ending,
                )
            )(input)
        }
    }

    impl Part {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom::bytes::complete::tag("{x="),
                    nom_num,
                    nom::bytes::complete::tag(",m="),
                    nom_num,
                    nom::bytes::complete::tag(",a="),
                    nom_num,
                    nom::bytes::complete::tag(",s="),
                    nom_num,
                    nom::bytes::complete::tag("}"),
                )),
                |(_, x, _, m, _, a, _, s, _)| {
                    Part{x, m, a, s}
                }
            )(input)
        }

        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
            nom::multi::many1(
                nom::sequence::terminated(
                    Self::parse,
                    nom::character::complete::line_ending,
                )
            )(input)
        }

    }

    impl ProblemSet {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    Workflow::parse_list,
                    nom::character::complete::line_ending,
                    Part::parse_list,
                )),
                |(workflows, _, parts)| {
                    Self{workflows, parts}
                }
            )(input)
        }
    }

}


// ======= Compute =======

impl Part {
    /// Returns the rating this part has for the given Rating.
    fn get_rating(&self, r: Rating) -> Num {
        use Rating::*;
        match r {
            X => self.x,
            M => self.m,
            A => self.a,
            S => self.s,
        }
    }
}

impl CompareOp {
    /// Returns true if a is compare_op of b.
    fn is(&self, a: Num, b: Num) -> bool {
        use CompareOp::*;
        match self {
            Less => a < b,
            More => a > b,
        }
    }
}

impl Rule {
    /// Applies this rule to the given Part. Returns None if this rule doesn't say what to
    /// do, or a reference to a str if it should go somewhere.
    fn apply(&self, part: Part) -> Option<&str> {
        let r = part.get_rating(self.rating);
        if self.compare_op.is(r, self.value) {
            Some(&self.target)
        } else {
            None
        }
    }
}

impl Workflow {
    /// Returns the name as an &str.
    fn name(&self) -> &str {
        &self.name
    }

    /// Applies this workflow to the given Part and returns the name of the destination
    /// (which could be a Workflow or could be "R" or "A").
    fn apply(&self, part: Part) -> &str {
        for rule in self.rules.iter() {
            if let Some(destination) = rule.apply(part) {
                return destination;
            }
        }
        &self.default_target
    }
}

#[derive(Debug)]
struct Workshop<'a> {
    workflow_map: HashMap<&'a str, &'a Workflow>,
}

#[derive(Debug)]
enum Outcome { Accept, Reject }

impl<'a> Workshop<'a> {
    fn new(workflows: &'a Vec<Workflow>) -> Self {
        let mut workflow_map = HashMap::new();
        for workflow in workflows.iter() {
            workflow_map.insert(workflow.name(), workflow);
        }
        Self{workflow_map}
    }

    fn process_part(&self, part: Part) -> Outcome {
        let mut current: &str = "in";
        loop {
            current = self.workflow_map.get(current).expect("workflow name not found").apply(part);
            match current {
                "A" => return Outcome::Accept,
                "R" => return Outcome::Reject,
                _ => {},
            }
        }
    }
}



// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let workshop = Workshop::new(&input.workflows);
    let accepted_sum: Num = input.parts.iter()
        .filter(|part| matches!(workshop.process_part(**part), Outcome::Accept))
        .map(|part| part.x + part.m + part.a + part.s)
        .sum();
    println!("accepted_sum: {}", accepted_sum);
}


fn part_b(_input: &Input) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
