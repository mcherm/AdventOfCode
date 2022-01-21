mod eznom;

use std::fs;
use std::io;
use itertools::Itertools;
use nom::multi::many0 as nom_many0;
use nom::character::complete::u32 as nom_value;
use nom::character::complete::alpha1 as nom_alpha1;
use nom::sequence::tuple as nom_tuple;
use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::newline as nom_newline;
use eznom::type_builder;


const RACE_STEPS: usize = 2503;


#[derive(Debug, Clone)]
struct ReindeerCapability {
    name: String,
    speed: u32,
    endurance: u32,
    recovery: u32,
}

#[derive(Debug)]
struct Reindeer {
    capability: ReindeerCapability,
    is_resting: bool,
    time_in_current_state: u32,
    distance_traveled: u32,
    points: u32,
}



impl ReindeerCapability {
    pub fn parse_list(input: &str) -> nom::IResult<&str, Vec<Self>> {
        nom_many0(Self::parse)(input)
    }

    //Dasher can fly 11 km/s for 12 seconds, but then must rest for 125 seconds.
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_alpha1,
            nom_tag(" can fly "),
            nom_value,
            nom_tag(" km/s for "),
            nom_value,
            nom_tag(" seconds, but then must rest for "),
            nom_value,
            nom_tag(" seconds."),
            nom_newline,
        ))(s);
        let build =
            |
                (name,    _, speed,    _, endurance,    _, recovery,    _,    _):
                (&str, &str,   u32, &str,       u32, &str,      u32, &str, char)
            |
                ReindeerCapability{
                    name: name.to_string(),
                    speed,
                    endurance,
                    recovery,
                }
            ;
        type_builder(recognize, build)(input)
    }
}


impl Reindeer {
    fn new(capability: &ReindeerCapability) -> Self {
        Reindeer{
            capability: capability.clone(),
            is_resting: false,
            time_in_current_state: 0,
            distance_traveled: 0,
            points: 0,
        }
    }

    fn step(&mut self) {
        if !self.is_resting {
            self.distance_traveled += self.capability.speed;
        }
        self.time_in_current_state += 1;
        let max_time_in_state = if self.is_resting {self.capability.recovery} else {self.capability.endurance};
        if self.time_in_current_state >= max_time_in_state {
            self.is_resting = ! self.is_resting;
            self.time_in_current_state = 0;
        }
    }

    fn steps(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }
}


fn input() -> Result<Vec<ReindeerCapability>, io::Error> {
    let s = fs::read_to_string("input/2015/14/input.txt")?;
    match ReindeerCapability::parse_list(&s) {
        Ok(("", capabilities)) => Ok(capabilities),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}



fn part_a(capabilities: &Vec<ReindeerCapability>) -> Result<(), io::Error> {
    assert!(capabilities.len() > 0);
    let herd = capabilities.iter().map(|x| Reindeer::new(x)).collect_vec();
    let mut winning_distance = None;
    let mut winning_name = None;
    for mut deer in herd {
        deer.steps(RACE_STEPS);
        if winning_distance.is_none() || deer.distance_traveled > winning_distance.unwrap() {
            winning_distance = Some(deer.distance_traveled);
            winning_name = Some(deer.capability.name);
        }
    }
    println!("The winning reindeer, {}, went {} km.", winning_name.unwrap(), winning_distance.unwrap());
    Ok(())
}


fn part_b(capabilities: &Vec<ReindeerCapability>) -> Result<(), io::Error> {
    assert!(capabilities.len() > 0);
    let mut herd = capabilities.iter().map(|x| Reindeer::new(x)).collect_vec();
    for _ in 0..RACE_STEPS {
        for deer in herd.iter_mut() {
            deer.step();
        }
        let lead_distance = herd.iter().map(|x| x.distance_traveled).max().unwrap();
        for deer in herd.iter_mut() {
            if deer.distance_traveled == lead_distance {
                deer.points += 1;
            }
        }
    }
    let (winning_points, winning_name) = herd.iter()
        .map(|deer| (deer.points, &deer.capability.name))
        .max().unwrap();
    println!("The winning deer, {}, had {} points.", winning_name, winning_points);
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
