mod eznom;

use std::fs;
use std::io;
use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use nom::multi::many0 as nom_many0;
use nom::sequence::tuple as nom_tuple;
use nom::character::complete::alpha1 as nom_alpha1;
use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::u32 as nom_value;
use nom::character::complete::newline as nom_newline;
use eznom::type_builder;


#[derive(Debug)]
struct Road {
    locations: (String, String),
    distance: u32,
}

struct Map {
    travel: HashMap<(String,String), u32>,
    sites: Vec<String>,
}


impl Road {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_alpha1,
            nom_tag(" to "),
            nom_alpha1,
            nom_tag(" = "),
            nom_value,
            nom_newline,
        ))(s);
        let build = |(loc1, _, loc2, _, distance, _): (&str, &str, &str, &str, u32, char)| Road {
            locations: (loc1.to_string(), loc2.to_string()),
            distance
        };
        type_builder(recognize, build)(input)
    }
}


impl Map {
    fn new(roads: &Vec<Road>) -> Self {
        let mut travel = HashMap::new();
        let mut site_set = HashSet::new();
        for route in roads {
            let path_1 = route.locations.clone();
            let path_2 = (path_1.1.clone(), path_1.0.clone());
            site_set.insert(path_1.0.clone());
            site_set.insert(path_1.1.clone());
            travel.insert(path_1, route.distance);
            travel.insert(path_2, route.distance);
        }
        let sites = site_set.into_iter().sorted().collect();
        Map{travel, sites}
    }

    fn num_sites(&self) -> usize {
        self.sites.len()
    }

    fn cost(&self, route: &Vec<&String>) -> u32 {
        assert!(route.len() > 0);
        let mut route_it = route.iter();
        let mut answer = 0;
        let mut prev_site = route_it.next().unwrap();
        loop {
            match route_it.next() {
                None => break,
                Some(site) => {
                    answer += self.travel.get(&((*prev_site).clone(), (*site).clone())).unwrap();
                    prev_site = site;
                },
            }
        }
        answer
    }
}


fn parse_routes(input: &str) -> nom::IResult<&str, Vec<Road>> {
    nom_many0(Road::parse)(input)
}


fn input() -> Result<Vec<Road>, io::Error> {
    let s = fs::read_to_string("input/2015/09/input.txt")?;
    match parse_routes(&s) {
        Ok(("", routes)) => Ok(routes),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}

fn part_a(routes: &Vec<Road>) -> Result<(), io::Error> {
    let map = Map::new(routes);
    let mut routes = map.sites.iter()
        .permutations(map.num_sites())
        .map(|route| {
            let cost = map.cost(&route);
            (cost, route)
        })
        .sorted();
    let (best_cost, best_route) = routes.next().unwrap();
    println!("The shortest route is {:?} costing {}.", best_route, best_cost);
    Ok(())
}

fn part_b(routes: &Vec<Road>) -> Result<(), io::Error> {
    let map = Map::new(routes);
    let mut routes = map.sites.iter()
        .permutations(map.num_sites())
        .map(|route| {
            let cost = map.cost(&route);
            (cost, route)
        })
        .sorted()
        .rev();
    let (best_cost, best_route) = routes.next().unwrap();
    println!("The longest route is {:?} costing {}.", best_route, best_cost);
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
