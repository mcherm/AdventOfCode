mod eznom;

use std::fs;
use std::io;
use std::io::ErrorKind;
use std::collections::BTreeSet;
use itertools::Itertools;
use nom::multi::many0 as nom_many0;
use nom::character::complete::i32 as nom_value;
use nom::character::complete::alpha1 as nom_alpha1;
use nom::sequence::tuple as nom_tuple;
use nom::bytes::complete::tag as nom_tag;
use nom::branch::alt as nom_alt;
use nom::character::complete::newline as nom_newline;
use eznom::type_builder;



#[derive(Debug)]
struct HappinessStatement {
    person_1: String,
    person_2: String,
    gain: i32,
}

#[derive(Debug)]
struct HappinessGrid {
    people: Vec<String>,
    gains: Vec<i32>,
}


impl HappinessStatement {
    pub fn parse_list(input: &str) -> nom::IResult<&str, Vec<Self>> {
        nom_many0(Self::parse)(input)
    }

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_alpha1,
            nom_tag(" would "),
            nom_alt((
                nom_tag("gain"),
                nom_tag("lose"),
            )),
            nom_tag(" "),
            nom_value,
            nom_tag(" happiness units by sitting next to "),
            nom_alpha1,
            nom_tag("."),
            nom_newline,
        ))(s);
        let build =
            |
                (person_1,    _, gain_lose,    _, val,    _, person_2,    _,    _):
                (    &str, &str,      &str, &str, i32, &str,     &str, &str, char)
            |
            HappinessStatement{
                person_1: person_1.to_string(),
                person_2: person_2.to_string(),
                gain: val * (if gain_lose == "gain" {1} else {-1}),
            }
        ;
        type_builder(recognize, build)(input)
    }
}

const ME: &str = "me";

impl HappinessGrid {
    pub fn new(statements: &Vec<HappinessStatement>, add_me: bool) -> Result<Self, io::Error> {
        let mut people_set = BTreeSet::new();
        for statement in statements.iter() {
            people_set.insert(statement.person_1.clone());
        }
        if add_me {
            people_set.insert(ME.to_string());
        }
        let people: Vec<String> = people_set.into_iter().collect();
        let num_people: usize = people.len();
        let gains: Vec<i32> = vec![i32::MIN; num_people * num_people];
        let mut grid = HappinessGrid{people, gains};
        grid.apply_statements(statements, add_me)?;
        Ok(grid)
    }

    fn size(&self) -> usize {
        self.people.len()
    }

    fn person_index(&self, person: &str) -> Option<usize> {
        self.people.iter().position(|x| *x == person)
    }

    fn get_gain(&self, person_1: &str, person_2: &str) -> i32 {
        let i = self.person_index(person_1).unwrap();
        let j = self.person_index(person_2).unwrap();
        self.gains[i * self.size() + j]
    }

    /// This is called only during new(), in order to set the data from
    /// the statements into self.gains.
    fn apply_statements(&mut self, statements: &Vec<HappinessStatement>, add_me: bool) -> Result<(), io::Error> {
        let size = self.size();
        for statement in statements {
            let i = self.person_index(&statement.person_1).unwrap();
            let j = self.person_index(&statement.person_2).unwrap();
            self.gains[i * size + j] = statement.gain;
        }
        if add_me {
            let i = self.person_index(ME).unwrap();
            for person in self.people.iter() {
                let j = self.person_index(person).unwrap();
                self.gains[i * size + j] = 0;
                self.gains[j * size + i] = 0;
            }
        }
        for i in 0..size { // person next to self
            self.gains[i * size + i] = 0;
        }
        if self.gains.iter().any(|x| *x == i32::MIN) {
            return Err(io::Error::new(ErrorKind::Other, "Incomplete Rules"))
        }
        Ok(())
    }


    /// Given a permutation of the persons (and it MUST be a valid permutation),
    /// this returns the total happiness gain.
    fn eval(&self, ordering: &Vec<&String>) -> i32 {
        ordering.iter()
            .circular_tuple_windows()
            .map(|(p1, p2)| self.get_gain(p1, p2) + self.get_gain(p2, p1))
            .sum()
    }
}

fn input() -> Result<Vec<HappinessStatement>, io::Error> {
    let s = fs::read_to_string("input/2015/13/input.txt")?;
    match HappinessStatement::parse_list(&s) {
        Ok(("", statements)) => Ok(statements),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



fn part_a(statements: &Vec<HappinessStatement>) -> Result<(), io::Error> {
    let grid = HappinessGrid::new(&statements, false)?;
    let mut best_ordering: Option<Vec<&String>> = None;
    let mut best_gain: Option<i32> = None;
    for ordering in grid.people.iter().permutations(grid.size()) {
        let gain = grid.eval(&ordering);
        match best_gain {
            None => {
                best_gain = Some(gain);
                best_ordering = Some(ordering);
            },
            Some(b_gain) if b_gain < gain => {
                best_gain = Some(gain);
                best_ordering = Some(ordering);
            },
            _ => {},
        }
    }
    println!(
        "The best seating arrangement, with a gain of {}, is {:?}.",
        best_gain.unwrap(),
        best_ordering.unwrap()
    );
    Ok(())
}

fn part_b(statements: &Vec<HappinessStatement>) -> Result<(), io::Error> {
    let grid = HappinessGrid::new(&statements, true)?;
    let mut best_ordering: Option<Vec<&String>> = None;
    let mut best_gain: Option<i32> = None;
    for ordering in grid.people.iter().permutations(grid.size()) {
        let gain = grid.eval(&ordering);
        match best_gain {
            None => {
                best_gain = Some(gain);
                best_ordering = Some(ordering);
            },
            Some(b_gain) if b_gain < gain => {
                best_gain = Some(gain);
                best_ordering = Some(ordering);
            },
            _ => {},
        }
    }
    println!(
        "The best seating arrangement with me, with a gain of {}, is {:?}.",
        best_gain.unwrap(),
        best_ordering.unwrap()
    );
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
