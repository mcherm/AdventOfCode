
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fs;
    use nom;
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::tag,
        combinator::{opt, map, success},
        character::complete::{multispace1, line_ending},
        sequence::{tuple, terminated},
        multi::many0,
    };
    use nom::character::complete::u32 as nom_Num;


    pub fn input() -> Result<Vec<Blueprint>, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_19.txt")?;
        match Blueprint::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    type Num = u32;


    #[derive(Debug, Copy, Clone)]
    pub struct Blueprint {
        id: Num,
        ore_robot_ore: Num,
        clay_robot_ore: Num,
        obsidian_robot_ore: Num,
        obsidian_robot_clay: Num,
        geode_robot_ore: Num,
        geode_robot_obsidian: Num,
    }

    impl Blueprint {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            map(
                tuple((
                    tuple((
                        tag("Blueprint "),
                        nom_Num,
                        tag(": "),
                    )),
                    tuple((
                        tag("Each ore robot costs "),
                        nom_Num,
                        tag(" ore. "),
                    )),
                    tuple((
                        tag("Each clay robot costs "),
                        nom_Num,
                        tag(" ore. "),
                    )),
                    tuple((
                        tag("Each obsidian robot costs "),
                        nom_Num,
                        tag(" ore and "),
                        nom_Num,
                        tag(" clay. "),
                    )),
                    tuple((
                        tag("Each geode robot costs "),
                        nom_Num,
                        tag(" ore and "),
                        nom_Num,
                        tag(" obsidian."),
                    )),
                )),
                |(
                    (_, id, _,),
                    (_, ore_robot_ore, _,),
                    (_, clay_robot_ore, _,),
                    (_, obsidian_robot_ore, _, obsidian_robot_clay, _,),
                    (_, geode_robot_ore, _, geode_robot_obsidian, _,),
                 )| Blueprint{
                    id,
                    ore_robot_ore,
                    clay_robot_ore,
                    obsidian_robot_ore,
                    obsidian_robot_clay,
                    geode_robot_ore,
                    geode_robot_obsidian,
                }
            )(input)
        }

        /// Parses a newline-terminated list of LineSpecs
        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            many0(terminated(Self::parse, line_ending))(input)
        }
    }

}



// ======= Part 1 Compute =======

mod compute {
}




// ======= main() =======

use crate::parse::{Blueprint, input};



fn part_a(input: &Vec<Blueprint>) {
    println!("\nPart a:");
    for bp in input {
        println!("{:?}", bp);
    }
}


fn part_b(_input: &Vec<Blueprint>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}

