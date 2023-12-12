use std::ops::Add;
use anyhow;
use itertools::Itertools;

// ======= Constants =======


// ======= Parsing =======

type Num = i32;

#[derive(Debug)]
pub struct Row(Vec<Num>);


mod parse {
    use std::fs;
    use super::Row;
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
    };
    use nom::character::complete::i32 as nom_num;


    pub fn input<'a>() -> Result<Vec<Row>, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_09.txt")?;
        match Row::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    impl Row {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::multi::separated_list1(
                    tag(" "),
                    nom_num
                ),
                Row
            )(input)
        }

        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            nom::multi::many1(
                nom::sequence::terminated(
                    Self::parse,
                    nom::character::complete::line_ending,
                ),
            )(input)
        }
    }

}


// ======= Compute =======


impl Row {

    fn find_subrow(&self) -> Row {
        Row(
            self.0.iter()
                .tuple_windows::<(_,_)>()
                .map(|(a,b)| b - a)
                .collect_vec()
        )
    }

    fn zeros(&self) -> bool {
        self.0.iter().all(|x| *x == 0)
    }

    /// Gets the next item in this Row.
    fn get_next(&self) -> Num {
        if self.zeros() {
            0
        } else {
            self.0.last().unwrap().add( self.find_subrow().get_next() )
        }
    }

    /// Gets the next item in this Row.
    fn get_prev(&self) -> Num {
        if self.zeros() {
            0
        } else {
            *self.0.first().unwrap() - self.find_subrow().get_prev()
        }
    }

}


// ======= main() =======


fn part_a(data: &Vec<Row>) {
    println!("\nPart a:");
    let answer: Num = data.iter().map(|row| row.get_next()).sum();
    println!("The sum is {}", answer);
}


fn part_b(data: &Vec<Row>) {
    println!("\nPart b:");
    let answer: Num = data.iter().map(|row| row.get_prev()).sum();
    println!("The sum is {}", answer);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
