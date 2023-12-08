use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use anyhow;
use itertools::Itertools;


// ======= Parsing =======

const CARD_CHARS: &str = "AKQJT98765432";

type Num = u32;

#[derive(Debug, Eq, PartialEq)]
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
        assert!(CARD_CHARS.contains(c));
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum HandType {
    FiveOfAKind,
    FourOfAKind,
    FullHouse,
    ThreeOfAKind,
    TwoPair,
    OnePair,
    HighCard,
}

impl Card {
    /// Returns a number which can be used to sort the cards.
    fn value(&self) -> u8 {
        14 - (CARD_CHARS.find(self.c).unwrap() as u8)
    }
}

impl HandType {
    /// Returns a number which can be used to sort the hands.
    fn rank_num(&self) -> u8 {
        match self {
            HandType::FiveOfAKind => 7,
            HandType::FourOfAKind => 6,
            HandType::FullHouse => 5,
            HandType::ThreeOfAKind => 4,
            HandType::TwoPair => 3,
            HandType::OnePair => 2,
            HandType::HighCard => 1,
        }
    }
}


impl PartialOrd for HandType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.rank_num().partial_cmp(&other.rank_num())
    }
}


impl Hand {
    /// Counts how many Cards with c appear in the hand.
    fn count(&self, c: char) -> usize {
        self.cards.iter()
            .filter(|card| card.c == c)
            .count()
    }

    fn hand_type(&self) -> HandType {
        let counts = CARD_CHARS.chars()
            .map(|c| self.count(c))
            .sorted()
            .rev()
            .take(5)
            .collect_tuple()
            .unwrap();
        match counts {
            (5,0,0,0,0) => HandType::FiveOfAKind,
            (4,1,0,0,0) => HandType::FourOfAKind,
            (3,2,0,0,0) => HandType::FullHouse,
            (3,1,1,0,0) => HandType::ThreeOfAKind,
            (2,2,1,0,0) => HandType::TwoPair,
            (2,1,1,1,0) => HandType::OnePair,
            (1,1,1,1,1) => HandType::HighCard,
            _ => panic!("Invalid hand type"),
        }
    }

    /// Returns a tuple that's useful for sorting. The tuple will sort the same way as
    /// the hand would.
    fn sorting_tuple(&self) -> (HandType, u8, u8, u8, u8, u8) {
        (
            self.hand_type(),
            self.cards[0].value(),
            self.cards[1].value(),
            self.cards[2].value(),
            self.cards[3].value(),
            self.cards[4].value(),
        )
    }
}


impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.cards == other.cards
    }
}

impl Eq for Hand {}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sorting_tuple().partial_cmp(&other.sorting_tuple())
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.c)
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}{}{}", self.cards[0], self.cards[1], self.cards[2], self.cards[3], self.cards[4])
    }
}


// ======= main() =======


fn part_a(data: &Vec<Hand>) {
    println!("\nPart a:");
    let hands: Vec<&Hand> = data.iter().sorted().collect();
    let total_win: Num = hands.iter()
        .enumerate()
        .map(|(i,hand): (usize,&&Hand)| ((i as Num) + 1) * hand.bid)
        .sum();
    println!("Total winnings: {}", total_win);
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
