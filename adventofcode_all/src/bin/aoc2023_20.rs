use std::fmt::{Debug};
use anyhow;


// ======= Constants =======


// ======= Parsing =======

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ModuleKind {
    Broadcaster, FlipFlop, Conjunction, Button, Output
}

#[derive(Debug)]
#[allow(dead_code)] // FIXME: Remove once it is written
pub struct Module {
    name: String,
    kind: ModuleKind,
    destinations: Vec<String>,
}

#[derive(Debug)]
#[allow(dead_code)] // FIXME: Remove once it is written
pub struct Machine {
    modules: Vec<Module>,
}

type Input = Machine;



impl Module {
    fn new<T1: ToString, T2: ToString>(name: T1, kind: ModuleKind, destinations: Vec<T2>) -> Self {
        Module{name: name.to_string(), kind, destinations: destinations.iter().map(|x| x.to_string()).collect()}
    }
}




mod parse {
    use super::{Input, Module, ModuleKind};
    use std::fs;
    use nom;
    use nom::IResult;
    use crate::Machine;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_20.txt")?;
        match Input::parse(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    impl Module {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    nom::branch::alt(( // this will return a (ModuleKind, name) tuple
                        nom::combinator::map(
                            nom::bytes::complete::tag("broadcaster"),
                            |s| (ModuleKind::Broadcaster, s)
                        ),
                        nom::combinator::map(
                            nom::sequence::tuple((
                                nom::bytes::complete::tag("%"),
                                nom::character::complete::alpha1,
                            )),
                            |(_, name)| (ModuleKind::FlipFlop, name)
                        ),
                        nom::combinator::map(
                            nom::sequence::tuple((
                                nom::bytes::complete::tag("&"),
                                nom::character::complete::alpha1,
                            )),
                            |(_, name)| (ModuleKind::Conjunction, name)
                        ),
                    )),
                    nom::bytes::complete::tag(" -> "),
                    nom::multi::separated_list1(
                        nom::bytes::complete::tag(", "),
                        nom::character::complete::alpha1
                    ),
                )),
                |((kind, name), _, destinations)| Module::new(name, kind, destinations)
            )(input)
        }
    }

    impl Machine {
        fn parse(input: &str) -> IResult<&str, Self> {
            nom::combinator::map(
                nom::multi::many1(
                    nom::sequence::terminated(
                        Module::parse,
                        nom::character::complete::line_ending,
                    )
                ),
                |modules| Machine{modules}
            )(input)
        }
    }

}


// ======= Compute =======


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    println!("input: {:?}", input);
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


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn none() {
    }
}
