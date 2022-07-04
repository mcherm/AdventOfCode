mod eznom;

use std::fs;
use std::io;

use mod_exp::mod_exp;

use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::u64 as nom_num;
use nom::character::complete::newline as nom_newline;
use nom::sequence::tuple as nom_tuple;
use eznom::type_builder;


type Num = u64;

static START: Num = 20151125;
static MULTIPLICAND: Num = 252533;
static MODULO: Num = 33554393;


#[derive(Debug)]
enum Error {
    Io(io::Error),
}
impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Error::Io(e) } }


fn input() -> Result<(Num, Num), Error> {
    let s = fs::read_to_string("input/2015/25/input.txt")?;
    match parse_input(&s) {
        Ok(("", tuple)) => {
            Ok(tuple)
        },
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}




fn parse_input(input: &str) -> nom::IResult<&str, (Num,Num)> {
    let recognize = |s| nom_tuple((
        nom_tag("To continue, please consult the code grid in the manual.  Enter the code at row "),
        nom_num,
        nom_tag(", column "),
        nom_num,
        nom_tag("."),
        nom_newline,
    ))(s);
    let build = |(_, y, _, x, _, _): (&str, Num, &str, Num, &str, char)| (x,y);
    type_builder(recognize, build)(input)
}


fn get_sequence_num(x: Num, y: Num) -> Num {
    ((x + y) * (x + y + 1) / 2) + x
}

fn get_code(seq_num: Num) -> Num {
    (START * mod_exp(MULTIPLICAND, seq_num, MODULO)) % MODULO
}


fn part_a((x,y): &(Num, Num)) {
    println!("---- Part A ----");
    // println!("({}, {})", x, y);
    // let seq = get_sequence_num(*x,*y);
    // println!("seq: {}", seq);
    // for k in 0..16 {
    //     println!("{}", get_code(k));
    // }
    let code = get_code(get_sequence_num(*x - 1, *y - 1));
    println!("The code is {}", code);
}


fn part_b((_x,_y): &(Num, Num)) {
    println!("---- Part B ----");
}

fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
