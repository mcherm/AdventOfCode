use std::fmt::{Debug, Display, Formatter};
use anyhow;
use std::collections::HashMap;
use advent_lib::asciienum::AsciiEnum;


// ======= Constants =======


// ======= Parsing =======

type Num = u64;



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
    enum CompareOp { Less('<'), More('>') }
}

#[derive(Debug)]
pub struct Rule {
    rating: Rating,
    compare_op: CompareOp,
    value: Num,
    target: String,
}

#[derive(Debug)]
pub struct Workflow {
    name: String,
    rules: Vec<Rule>,
    default_target: String,
}

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
    use nom::character::complete::u64 as nom_num;


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
        pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
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


/// Represents any value in the range from .0 to .1 - 1. If .0 == .1 then it has
/// size 0 but is still valid. It will always be true that .0 <= .1.
#[derive(Debug, Copy, Clone)]
pub struct ValueRange(Num,Num);

/// A range of parts
#[derive(Copy, Clone)]
pub struct PartRange {
    x: ValueRange,
    m: ValueRange,
    a: ValueRange,
    s: ValueRange,
}

impl ValueRange {
    const NONE: Self = ValueRange(0,0);

    /// Returns the size of this range.
    fn len(&self) -> Num {
        self.1 - self.0
    }

    /// Splits this ValueRange into 2 different ValueRanges -- the first with the subrange
    /// that satisfies [ values compare_op value ], and the second with the subrange that
    /// does NOT satisfy it. Either range could be empty.
    fn split_with(&self, compare_op: CompareOp, value: Num) -> (ValueRange, ValueRange) {
        use CompareOp::*;
        match compare_op {
            Less => if value <= self.0 {
                (ValueRange::NONE, *self)
            } else if value >= self.1 {
                (*self, ValueRange::NONE)
            } else {
                (ValueRange(self.0, value), ValueRange(value, self.1))
            },
            More => if value + 1 <= self.0 {
                (*self, ValueRange::NONE)
            } else if value + 1 >= self.1 {
                (ValueRange::NONE, *self)
            } else {
                (ValueRange(value + 1, self.1), ValueRange(self.0, value + 1))
            },
        }
    }

    /// Returns true if this range overlaps the other range.
    fn overlaps(&self, other: &Self) -> bool {
        !(self.1 <= other.0 || self.0 >= other.1)
    }
}

impl PartialEq for ValueRange {
    fn eq(&self, other: &Self) -> bool {
        self.len() == 0 && other.len() == 0 ||
            self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for ValueRange {}

impl Display for ValueRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}..{})", self.0, self.1)
    }
}


impl PartRange {
    /// Returns a PartRange that covers ALL possible parts.
    fn all_parts() -> Self {
        Self{
            x: ValueRange(1,4001),
            m: ValueRange(1,4001),
            a: ValueRange(1,4001),
            s: ValueRange(1,4001),
        }
    }

    /// Returns the number of points in this PartRange.
    fn size(&self) -> Num {
        self.x.len() * self.m.len() * self.a.len() * self.s.len()
    }

    /// Returns true if this PartRange has no points in it.
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn get_rating_range(&self, r: Rating) -> ValueRange {
        use Rating::*;
        match r {
            X => self.x,
            M => self.m,
            A => self.a,
            S => self.s,
        }
    }

    /// This return  a new PartRange which is the same except that one of the ratings
    /// (indicated by r) has been replaced with the given range.
    fn replace_rating(&self, r: Rating, new_range: ValueRange) -> Self {
        use Rating::*;
        match r {
            X => Self{x: new_range, ..*self},
            M => Self{m: new_range, ..*self},
            A => Self{a: new_range, ..*self},
            S => Self{s: new_range, ..*self},
        }
    }

    /// Returns true if this range overlaps the other range.
    #[allow(dead_code)]
    fn overlaps(&self, other: &Self) -> bool {
        self.x.overlaps(&other.x) && self.m.overlaps(&other.m) && self.a.overlaps(&other.a) && self.s.overlaps(&other.s)
    }
}

impl Debug for PartRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PartRange[{}, {}, {}, {}]", self.x, self.m, self.a, self.s)
    }
}

impl Display for PartRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PartRange[{}, {}, {}, {}]", self.x, self.m, self.a, self.s)
    }
}

impl Rule {
    /// Applies this rule to the given PartRange. Returns a PartRange that is unaffected, a
    /// destination, and a PartRange that goes to that destination. One or the other of
    /// those PartRanges might be empty.
    fn apply_ranged(&self, part_range: PartRange) -> (PartRange, &str, PartRange) {
        let value_range = part_range.get_rating_range(self.rating);
        let (success_value_range, failure_value_range) = value_range.split_with(self.compare_op, self.value);
        let success_part_range = part_range.replace_rating(self.rating, success_value_range);
        let failure_part_range = part_range.replace_rating(self.rating, failure_value_range);
        (failure_part_range, &self.target, success_part_range)
    }
}

impl Workflow {
    /// Applies this workflow to the given PartRange. Returns a map of the destinations that
    /// should receive non-empty PartRanges, and what PartRange (or ranges!) each should receive.
    /// All PartRanges in the HashMap will be non-empty.
    pub fn apply_ranged(&self, part_range: PartRange) -> HashMap<&str, Vec<PartRange>> {
        let mut answer: HashMap<&str, Vec<PartRange>> = HashMap::new();
        let mut remaining_range = part_range;
        for rule in self.rules.iter() {
            let (rest, destination, this_range) = rule.apply_ranged(remaining_range);
            if ! this_range.is_empty() {
                answer.entry(destination)
                    .or_default() // if there's no list yet, create an empty one
                    .push(this_range); // add this one to the list
            }
            remaining_range = rest;
        }
        if ! remaining_range.is_empty() {
            answer.entry(&self.default_target)
                .or_default()  // if there's no list yet, create an empty one
                .push(remaining_range); // add this one to the list
        }
        answer
    }
}


impl<'a> Workshop<'a> {

    /// This finds, out of all possible parts, the number which are accepted.
    ///
    /// FIXME: If the initial instructions include any infinite loops, then those ought to
    ///   count as not-accepted. But if that happens, this code runs infinitely. If that
    ///   is observed, then we need to modify this to check for loops and reject those. For
    ///   now I'm betting on the input not containing any such loops.
    fn count_successes(&self) -> Num {
        let mut successful_ranges: Vec<PartRange> = Vec::new();
        let mut queue: Vec<(&str, PartRange)> = vec![("in", PartRange::all_parts())];
        while let Some((name, part_range)) = queue.pop() {
            let next_steps = self.workflow_map.get(name).expect("workflow name not found").apply_ranged(part_range);
            for (name, part_ranges) in next_steps {
                for part_range in part_ranges {
                    match name {
                        "R" => {}, // we can ignore the rejects
                        "A" => successful_ranges.push(part_range), // save the successes
                        _ => queue.push((name, part_range)), // consider everything else
                    }
                }
            }
        }
        successful_ranges.iter()
            .map(|part_range| part_range.size())
            .sum()
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


fn part_b(input: &Input) {
    println!("\nPart b:");
    let workshop = Workshop::new(&input.workflows);
    let successes = workshop.count_successes();
    println!("There are {} parts that could be accepted.", successes);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn value_range_split_with() {
        use CompareOp::*;
        let r = ValueRange(3,6);
        assert_eq!(r.len(), 3);
        assert_eq!(r.split_with(Less, 3), (ValueRange::NONE, ValueRange(3,6) ) );
        assert_eq!(r.split_with(Less, 4), (ValueRange(3,4),  ValueRange(4,6) ) );
        assert_eq!(r.split_with(Less, 5), (ValueRange(3,5),  ValueRange(5,6) ) );
        assert_eq!(r.split_with(Less, 6), (ValueRange(3,6),  ValueRange::NONE) );
        assert_eq!(r.split_with(More, 2), (ValueRange(3,6),  ValueRange::NONE) );
        assert_eq!(r.split_with(More, 3), (ValueRange(4,6),  ValueRange(3,4) ) );
        assert_eq!(r.split_with(More, 4), (ValueRange(5,6),  ValueRange(3,5) ) );
        assert_eq!(r.split_with(More, 5), (ValueRange::NONE, ValueRange(3,6) ) );
    }
}