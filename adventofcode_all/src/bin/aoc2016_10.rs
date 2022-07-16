
extern crate anyhow;

use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::fs;
use anyhow::Error;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::newline,
    combinator::map,
    multi::many0,
    sequence::tuple
};
use nom::character::complete::u32 as nom_u32;



fn input() -> Result<Vec<Instruction>, Error> {
    let s = fs::read_to_string("input/2016/input_10.txt")?;
    match Instruction::parse_vec(&s) {
        Ok(("", instructions)) => Ok(instructions),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Num = u32;


#[derive(Debug, Copy, Clone)]
pub enum Receiver {
    Bot(Num),
    Output(Num),
}

impl Receiver {
    fn parse_bot<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("bot "),
                nom_u32,
            )),
            |(_, val)| Self::Bot(val)
        )(input)
    }

    fn parse_output<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("output "),
                nom_u32,
            )),
            |(_, val)| Self::Output(val)
        )(input)
    }

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            Self::parse_bot,
            Self::parse_output,
        ))(input)
    }
}


#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    Assign{val: Num, dest: Receiver},
    Give{from_: Num, low: Receiver, high: Receiver},
}

impl Instruction {
    fn parse_assign<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("value "),
                nom_u32,
                tag(" goes to "),
                Receiver::parse,
                newline
            )),
            |(_, val, _, dest, _)| Self::Assign{val, dest}
        )(input)
    }

    fn parse_give<'a>(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("bot "),
                nom_u32,
                tag(" gives low to "),
                Receiver::parse,
                tag(" and high to "),
                Receiver::parse,
                newline
            )),
            |(_, from_, _, low, _, high, _)| Self::Give{from_, low, high}
        )(input)
    }

    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            Self::parse_assign,
            Self::parse_give,
        ))(input)
    }

    fn parse_vec<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0(Self::parse)(input)
    }
}

#[derive(Debug)]
struct Bot {
    chips: BinaryHeap<Num>
}

impl Bot {
    fn new() -> Self {
        Bot{chips: BinaryHeap::default()}
    }
}




#[derive(Debug)]
struct FactoryFloor {
    bots: HashMap<Num, Bot>,
    outputs: HashMap<Num, Vec<Num>>,
    to_do: HashMap<Num, (Receiver, Receiver)>,
}

impl FactoryFloor {
    fn give_to_bot(&mut self, bot_num: Num, val: Num) {
        let bot = self.bots.entry(bot_num).or_insert(Bot::new());
        bot.chips.push(val);
    }

    fn give_to_output(&mut self, output_num: Num, val: Num) {
        let output = self.outputs.entry(output_num).or_insert(Vec::new());
        output.push(val);
    }

    fn give_to_receiver(&mut self, receiver: Receiver, val: Num) {
        match receiver {
            Receiver::Bot(bot_num) => self.give_to_bot(bot_num, val),
            Receiver::Output(output_num) => self.give_to_output(output_num, val),
        }
    }

    fn initialize(instructions: &Vec<Instruction>) -> Self {
        let mut answer = FactoryFloor{
            bots: HashMap::new(),
            to_do: HashMap::new(),
            outputs: HashMap::new()
        };
        for instruction in instructions.iter() {
            match instruction {
                Instruction::Assign{val, dest: Receiver::Bot(bot_num)} => {
                    answer.give_to_bot(*bot_num, *val);
                },
                Instruction::Give{from_, low, high} => {
                    answer.to_do.insert(*from_, (*low, *high));
                },
                Instruction::Assign{dest: Receiver::Output(_), ..} => {
                    panic!("Not allowed to assign directly to output.");
                },
            }
        }
        answer
    }

    fn get_active_bot_nums(&self) -> Vec<Num> {
        self.bots.iter().filter(|x| x.1.chips.len() == 2).map(|x| *x.0).collect()
    }


    /// Passed a Bot (that must have 2 chips), this distributes them
    /// according to the instructions (and panics if there are no
    /// valid instructions or if the bot doesn't have 2 chips).
    fn perform_give(&mut self, bot_num: Num) {
        let (low_rcv, high_rcv) = self.to_do.get(&bot_num).expect("Bot {bot_num} lacks instructions.");
        let chips = &mut self.bots.get_mut(&bot_num).unwrap().chips;
        assert!(chips.len() == 2);
        let high_chip = chips.pop().unwrap();
        let low_chip = chips.pop().unwrap();
        assert!(chips.len() == 0);
        if (low_chip, high_chip) == (17, 61) {
            println!("The bot that compares 17 to 61 is bot {}", bot_num);
        }
        let (low_rcv, high_rcv) = (low_rcv.clone(), high_rcv.clone());
        self.give_to_receiver(high_rcv, high_chip);
        self.give_to_receiver(low_rcv, low_chip);
    }

    /// Instruct the bots to take one step. Returns true if there were any active bots.
    fn step_bots(&mut self) -> bool {
        let mut answer = false;
        for bot_num in self.get_active_bot_nums() {
            self.perform_give(bot_num);
            answer = true;
        }
        answer
    }

    fn run_to_completion(&mut self) {
        loop {
            let still_going = self.step_bots();
            if !still_going {
                break;
            }
        }
    }
}


fn part_a(instructions: &Vec<Instruction>) {
    println!("\nPart a:");
    let mut factory = FactoryFloor::initialize(instructions);
    factory.run_to_completion()
}


fn part_b(instructions: &Vec<Instruction>) {
    println!("\nPart b:");
    let mut factory = FactoryFloor::initialize(instructions);
    factory.run_to_completion();
    let get_sole_value = |output_num| {
        let v: &Vec<Num> = factory.outputs.get(&output_num).unwrap();
        assert!(v.len() == 1);
        v.get(0).unwrap()
    };
    let x = get_sole_value(0);
    let y = get_sole_value(1);
    let z = get_sole_value(2);
    println!("Multiply {} x {} x {} = {}", x, y, z, x*y*z);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
