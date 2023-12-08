use anyhow;


// ======= Parsing =======

type Num = u32;

#[derive(Debug)]
struct Card {
    c: char,
}

#[derive(Debug)]
pub struct Hand {
    cards: [Card; 5],
    bid: Num,
}


impl Card {
    fn new(c: char) -> Self {
        assert!("AKQJT98765432".contains(c));
        Card{c}
    }
}



mod parse {
    use std::fs;
    use super::{Hand, Card};
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
    };
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Vec<Hand>, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_07.txt")?;
        match Hand::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    impl Card {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::character::complete::anychar,
                |c| Card::new(c)
            )(input)
        }
    }

    impl Hand {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    Card::parse,
                    Card::parse,
                    Card::parse,
                    Card::parse,
                    Card::parse,
                    tag(" "),
                    nom_num,
                )),
                |(a,b,c,d,e,_,bid)| Hand{cards: [a,b,c,d,e], bid}
            )(input)
        }


        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            nom::multi::many1(
                nom::sequence::terminated(
                    Self::parse,
                    nom::character::complete::line_ending
                )
            )(input)
        }
    }

}


// ======= Compute =======




// ======= main() =======


fn part_a(data: &Vec<Hand>) {
    println!("\nPart a:");
    println!("Hands: {:?}", data);
}


fn part_b(_data: &Vec<Hand>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
