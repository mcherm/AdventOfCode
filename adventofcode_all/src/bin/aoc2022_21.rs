
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fmt::{Debug, Display, Formatter};
    use std::fs;
    use itertools::Itertools;
    use nom;
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::tag,
        combinator::map,
        character::complete::{line_ending, satisfy, i64 as nom_Num},
        sequence::tuple,
        multi::{count, many0},
    };
    use nom::sequence::terminated;


    pub fn input() -> Result<Vec<Monkey>, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_21.txt")?;
        match Monkey::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    // FIXME: Verify that division is legal
    pub type Num = i64;

    /// Stores 4-character (lowercase letter) names efficiently.
    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    pub struct Name {
        code: u32
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Job {
        Const(Num),
        Plus(Name, Name),
        Minus(Name, Name),
        Times(Name, Name),
        Divide(Name, Name),
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Monkey {
        pub name: Name,
        pub job: Job,
    }


    /// Convert between lowercase ascii letters and u32 in the range 0..26 with panics if out of range.
    fn letter_to_num(c: char) -> u32 {
        (c as u32) - ('a' as u32)
    }

    /// Convert between lowercase ascii letters and u32 in the range 0..26 with panics if out of range.
    fn num_to_letter(n: u32) -> char {
        char::from_u32(n + ('a' as u32)).unwrap()
    }


    impl Name {
        /// Create a new Name from a string. Panics if the string isn't perfectly valid.
        pub fn new(s: &str) -> Name {
            let (extra, name) = Name::parse(s).unwrap();
            if extra != "" {
                panic!("Extra characters in name string");
            }
            name
        }

        /// Construct Self or panic if given bad data.
        pub fn from_vec(chars: Vec<char>) -> Self {
            assert!(chars.len() == 4);
            assert!(chars.iter().all(|c| c.is_ascii_lowercase()));
            let mut code = 0;
            for c in chars.iter() {
                code *= 26;
                code += letter_to_num(*c);
            }
            Self{code}
        }

        fn parse(input: &str) -> IResult<&str, Self> {
            map(
                count( satisfy(|c| c >= 'a' && c <= 'z'), 4 ),
                Name::from_vec
            )(input)
        }

        fn to_chars(&self) -> [char;4] {
            let mut code = self.code;
            let d = num_to_letter(code % 26);
            code = code / 26;
            let c = num_to_letter(code % 26);
            code = code / 26;
            let b = num_to_letter(code % 26);
            code = code / 26;
            let a = num_to_letter(code % 26);
            [a,b,c,d]
        }
    }

    impl Debug for Name {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self)
        }
    }

    impl Display for Name {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.to_chars().iter().join(""))
        }
    }


    impl Job {
        fn parse(input: &str) -> IResult<&str, Self> {
            alt((
                map(
                    nom_Num,
                    |x| Job::Const(x)
                ),
                map(
                    tuple((
                        Name::parse,
                        alt((
                            tag(" + "),
                            tag(" - "),
                            tag(" * "),
                            tag(" / "),
                        )),
                        Name::parse,
                    )),
                    |(name_1, operation, name_2)| {
                        match operation {
                            " + " => Job::Plus(name_1, name_2),
                            " - " => Job::Minus(name_1, name_2),
                            " * " => Job::Times(name_1, name_2),
                            " / " => Job::Divide(name_1, name_2),
                            _ => panic!(),
                        }
                    }
                )
            ))(input)
        }
    }

    impl Monkey {
        fn parse(input: &str) -> IResult<&str, Self> {
            map(
                tuple((
                    Name::parse,
                    tag(": "),
                    Job::parse,
                )),
                |(name, _, job)| Monkey{name, job}
            )(input)
        }

        /// Parses a newline-terminated list of Blueprints
        fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
            many0(terminated(Monkey::parse, line_ending))(input)
        }
    }

}



// ======= Part 1 Compute =======

mod compute_1 {
    use crate::parse::{Num, Name, Job, Monkey};
    use std::collections::HashMap;


    /// A group of monkeys
    #[derive(Debug)]
    pub struct MonkeyTroop {
        monkeys: HashMap<Name, Monkey>,
    }


    impl MonkeyTroop {
        pub fn new(input: &Vec<Monkey>) -> Self {
            let monkeys = input.iter().map(|m| (m.name, *m)).collect();
            Self{monkeys}
        }

        /// Evaluates the Monkey with the given name.
        pub fn eval(&self, name: Name) -> Num {
            match self.monkeys.get(&name).unwrap().job {
                Job::Const(x) => x,
                Job::Plus(n1, n2) => self.eval(n1) + self.eval(n2),
                Job::Minus(n1, n2) => self.eval(n1) - self.eval(n2),
                Job::Times(n1, n2) => self.eval(n1) * self.eval(n2),
                Job::Divide(n1, n2) => {
                    let n1 = self.eval(n1);
                    let n2 = self.eval(n2);
                    assert!(n1 % n2 == 0); // make sure divisions are exact
                    n1 / n2
                },
            }
        }
    }
}


// ======= Part 2 Compute =======

mod compute_2 {
    use crate::parse::{Num, Name, Job, Monkey};
    use std::collections::HashMap;

    // from eval
    enum Outcome {
        WillShout(Num),
        HasHuman,
    }
    use Outcome::WillShout;
    use Outcome::HasHuman;


    /// A participant in the cooperation of monkeys that answer part 2
    #[derive(Debug)]
    pub enum RequestMonkey {
        Root(Name, Name),
        Human,
        NiceMonkey(Monkey),
    }

    /// A group of monkeys
    #[derive(Debug)]
    pub struct RequestMonkeyTroop {
        request_monkeys: HashMap<Name, RequestMonkey>,
    }


    impl RequestMonkey {
        /// Create an entry into the HashMap of RequestMonkeys.
        fn make_entry(monkey: &Monkey) -> (Name, RequestMonkey) {
            if monkey.name == Name::new("root") {
                let (n1, n2) = match monkey.job {
                    Job::Const(_) => panic!("Root must not be a constant!"),
                    Job::Plus(n1, n2) => (n1, n2),
                    Job::Minus(n1, n2) => (n1, n2),
                    Job::Times(n1, n2) => (n1, n2),
                    Job::Divide(n1, n2) => (n1, n2),
                };
                (monkey.name, RequestMonkey::Root(n1, n2))
            } else if monkey.name == Name::new("humn") {
                (monkey.name, RequestMonkey::Human)
            } else {
                (monkey.name, RequestMonkey::NiceMonkey(monkey.clone()))
            }
        }
    }

    impl RequestMonkeyTroop {
        pub fn new(input: &Vec<Monkey>) -> Self {
            let request_monkeys = input.iter().map(RequestMonkey::make_entry).collect();
            Self{request_monkeys}
        }

        /// Finds out the answer to part 2.
        pub fn what_to_yell(&self) -> Num {
            if let RequestMonkey::Root(n1, n2) = self.request_monkeys.get(&Name::new("root")).unwrap() {
                if let WillShout(v1) = self.outcome(*n1) {
                    self.request(*n2, v1)
                } else if let WillShout(v2) = self.outcome(*n1) {
                    self.request(*n1, v2)
                } else {
                    panic!("Both sides have a human");
                }
            } else {
                panic!("Root monkey wasn't a Root.");
            }
        }

        /// Finds the outcome for the monkey with the given name.
        fn outcome(&self, name: Name) -> Outcome {
            match self.request_monkeys.get(&name).unwrap() {
                RequestMonkey::Root(_, _) => {
                    panic!("Should never ask for outcome from the root monkey.");
                }
                RequestMonkey::Human => HasHuman,
                RequestMonkey::NiceMonkey(monkey) => {
                    match monkey.job {
                        Job::Const(x) => WillShout(x),
                        Job::Plus(n1, n2) => match (self.outcome(n1), self.outcome(n2)) {
                            (WillShout(v1), WillShout(v2)) => WillShout(v1 + v2),
                            (_, _) => HasHuman
                        }
                        Job::Minus(n1, n2) => match (self.outcome(n1), self.outcome(n2)) {
                            (WillShout(v1), WillShout(v2)) => WillShout(v1 - v2),
                            (_, _) => HasHuman
                        }
                        Job::Times(n1, n2) => match (self.outcome(n1), self.outcome(n2)) {
                            (WillShout(v1), WillShout(v2)) => WillShout(v1 * v2),
                            (_, _) => HasHuman
                        }
                        Job::Divide(n1, n2) => match (self.outcome(n1), self.outcome(n2)) {
                            (WillShout(v1), WillShout(v2)) => WillShout(v1 / v2),
                            (_, _) => HasHuman
                        }
                    }
                }
            }
        }

        /// Requests the monkey with the given name to tell us what the human must should out for
        /// it to return the value v. Panics if called on a monkey that doesn't depend on a
        /// human.
        fn request(&self, name: Name, v: Num) -> Num {
            match self.request_monkeys.get(&name).unwrap() {
                RequestMonkey::Root(_, _) => {
                    panic!("Should never request a value from the root monkey.");
                },
                RequestMonkey::Human => v,
                RequestMonkey::NiceMonkey(monkey) => {
                    match monkey.job {
                        Job::Const(_) => {
                            panic!("Should never request a value from a const monkey.")
                        }
                        Job::Plus(n1, n2) => match (self.outcome(n1), self.outcome(n2)) {
                            (WillShout(_), WillShout(_)) => panic!("Should never request a value from a monkey not dependent on a human."),
                            (WillShout(v1), HasHuman) => self.request(n2, v - v1),
                            (HasHuman, WillShout(v2)) => self.request(n1, v - v2),
                            (HasHuman, HasHuman) => panic!("Darn - it's hard."),
                        }
                        Job::Minus(n1, n2) => match (self.outcome(n1), self.outcome(n2)) {
                            (WillShout(_), WillShout(_)) => panic!("Should never request a value from a monkey not dependent on a human."),
                            (WillShout(v1), HasHuman) => self.request(n2, v1 - v),
                            (HasHuman, WillShout(v2)) => self.request(n1, v + v2),
                            (HasHuman, HasHuman) => panic!("Darn - it's hard."),
                        }
                        Job::Times(n1, n2) => match (self.outcome(n1), self.outcome(n2)) {
                            (WillShout(_), WillShout(_)) => panic!("Should never request a value from a monkey not dependent on a human."),
                            (WillShout(v1), HasHuman) => self.request(n2, v / v1),
                            (HasHuman, WillShout(v2)) => self.request(n1, v / v2),
                            (HasHuman, HasHuman) => panic!("Darn - it's hard."),
                        }
                        Job::Divide(n1, n2) => match (self.outcome(n1), self.outcome(n2)) {
                            (WillShout(_), WillShout(_)) => panic!("Should never request a value from a monkey not dependent on a human."),
                            (WillShout(v1), HasHuman) => self.request(n2, v1 / v),
                            (HasHuman, WillShout(v2)) => self.request(n1, v * v2),
                            (HasHuman, HasHuman) => panic!("Darn - it's hard."),
                        }
                    }
                }
            }
        }
    }
}


// ======= main() =======

use crate::parse::{input, Monkey, Name};
use crate::compute_1::MonkeyTroop;
use crate::compute_2::RequestMonkeyTroop;


fn part_a(input: &Vec<Monkey>) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    let troop = MonkeyTroop::new(input);
    println!("The root monkey yells out {}", troop.eval(Name::new("root")));
    Ok(())
}


fn part_b(input: &Vec<Monkey>) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    let troop = RequestMonkeyTroop::new(input);
    println!("The human must yell out {}", troop.what_to_yell());
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}

