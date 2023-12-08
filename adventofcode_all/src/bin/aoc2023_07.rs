use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use anyhow;
use itertools::Itertools;
use std::collections::HashSet;


// ======= Parsing =======

const CARD_CHARS: &str = "AKQJT98765432";
const JOKER_CARD_CHARS: &str = "AKQT98765432J";

type Num = u32;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Card {
    c: char,
}

#[derive(Debug)]
pub struct NormalHand {
    cards: [Card; 5],
    bid: Num,
}

trait Hand: Ord + Display {
    fn bid(&self) -> Num;
    fn count(&self, c: char) -> usize;
    fn hand_type(&self) -> HandType;
    fn sorting_tuple(&self) -> (HandType, u8, u8, u8, u8, u8);
}


impl Card {
    fn new(c: char) -> Self {
        assert!(CARD_CHARS.contains(c));
        Card{c}
    }
}



mod parse {
    use std::fs;
    use super::{NormalHand, Card};
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
    };
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Vec<NormalHand>, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_07.txt")?;
        match NormalHand::parse_list(&s) {
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

    impl NormalHand {
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
                |(a,b,c,d,e,_,bid)| NormalHand{cards: [a,b,c,d,e], bid}
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

    /// Returns a number which can be used to sort the cards assuming J is a joker.
    fn joker_value(&self) -> u8 {
        14 - (JOKER_CARD_CHARS.find(self.c).unwrap() as u8)
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

impl Ord for HandType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}


impl Hand for NormalHand {
    fn bid(&self) -> Num {
        self.bid
    }

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

impl NormalHand {

    /// Returns this same hand, but with all instances of old_card replaced with new_card.
    fn substitute(&self, old_card: Card, new_card: Card) -> Self {
        let cards: [Card; 5] = self.cards.iter()
            .map(|card| if *card == old_card {new_card} else {*card}) // replace that one
            .collect_vec()
            .try_into()
            .unwrap();
        let bid = self.bid;
        NormalHand{cards, bid}
    }
}

impl PartialEq for NormalHand {
    fn eq(&self, other: &Self) -> bool {
        self.cards == other.cards
    }
}

impl Eq for NormalHand {}

impl PartialOrd for NormalHand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sorting_tuple().partial_cmp(&other.sorting_tuple())
    }
}

impl Ord for NormalHand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.c)
    }
}

impl Display for NormalHand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}{}{}", self.cards[0], self.cards[1], self.cards[2], self.cards[3], self.cards[4])
    }
}

impl<'a> Display for JokerHand<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


/// A wrapper for Hand that sorts differently, because it treats "J" as a joker.
struct JokerHand<'a>(&'a NormalHand);


const JOKER: Card = Card{c: 'J'};

impl<'a> Hand for JokerHand<'a> {
    fn bid(&self) -> Num { self.0.bid() }
    fn count(&self, c: char) -> usize { self.0.count(c) }

    fn hand_type(&self) -> HandType {
        let cards_in_use: HashSet<Card> = self.0.cards.iter().map(|card| card.clone()).collect();
        if cards_in_use.contains(&JOKER) {
            // -- It has one-or-more joker, so we'll need to try the possible combinations ---
            // NOTE: Because hands with more matches always beat hands with fewer matches
            //   we only need to try replacing ALL jokers with each other card type that appears
            //   somewhere in the hand. That's only a few combinations to try for each hand.
            cards_in_use.iter()
                .filter(|card| **card != JOKER)
                .map(|card| self.0.substitute(JOKER, *card).hand_type())
                .max()
                .unwrap_or_else(|| {
                    assert!(self.0.cards == [JOKER;5]); // I think this can only happen with a hand of all Jokers
                    HandType::FiveOfAKind
                })
        } else {
            self.0.hand_type()
        }
    }

    fn sorting_tuple(&self) -> (HandType, u8, u8, u8, u8, u8) {
        (
            self.hand_type(),
            self.0.cards[0].joker_value(),
            self.0.cards[1].joker_value(),
            self.0.cards[2].joker_value(),
            self.0.cards[3].joker_value(),
            self.0.cards[4].joker_value(),
        )
    }
}

impl<'a> PartialEq for JokerHand<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a> Eq for JokerHand<'a> {}

impl<'a> PartialOrd for JokerHand<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sorting_tuple().partial_cmp(&other.sorting_tuple())
    }
}

impl<'a> Ord for JokerHand<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

// ======= main() =======


fn part_a(data: &Vec<NormalHand>) {
    println!("\nPart a:");
    let hands: Vec<&NormalHand> = data.iter().sorted().collect();
    let total_win: Num = hands.into_iter()
        .enumerate()
        .map(|(i,hand): (usize,&NormalHand)| ((i as Num) + 1) * hand.bid())
        .sum();
    println!("Total winnings: {}", total_win);
}


fn part_b(data: &Vec<NormalHand>) {
    println!("\nPart b:");
    let hands: Vec<JokerHand> = data.iter().map(|h| JokerHand(h)).sorted().collect();
    let total_win: Num = hands.into_iter()
        .enumerate()
        .map(|(i,hand): (usize,JokerHand)| ((i as Num) + 1) * hand.bid())
        .sum();
    println!("Total winnings with jokers: {}", total_win);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
