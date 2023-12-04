use anyhow;


// ======= Parsing =======

type Num = u32;

#[derive(Debug)]
pub struct Card {
    winning: Vec<Num>,
    have: Vec<Num>,
}


mod parse {
    use std::fs;
    use super::{Card};
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
        character::complete::line_ending,
    };
    use nom::character::complete::u32 as nom_num;


    pub fn input<'a>() -> Result<Vec<Card>, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_04.txt")?;
        match Card::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    impl Card {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    tag("Card"),
                    nom::multi::many1(tag(" ")),
                    nom_num,
                    tag(": "),
                    nom::multi::separated_list1(
                        tag(" "),
                        nom::sequence::preceded(
                            nom::combinator::opt( tag(" ") ), // optional leading space we ignore
                            nom_num, // a number
                        )
                    ),
                    tag(" | "),
                    nom::multi::separated_list1(
                        tag(" "),
                        nom::sequence::preceded(
                            nom::combinator::opt( tag(" ") ), // optional leading space we ignore
                            nom_num, // a number
                        )
                    ),
                )),
                |(_, _, _, _, winning, _, have)| {
                    Card{winning, have}
                }
            )(input)
        }

        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            nom::multi::many1( nom::sequence::terminated(Self::parse, line_ending) )(input)
        }

    }
}


// ======= Compute =======

use std::collections::HashSet;

impl Card {
    /// Returns the number of items in the winning list that match.
    fn win_count(&self) -> usize {
        let have_set: HashSet<Num> = self.have.iter().copied().collect();
        assert!(have_set.len() == self.have.len());
        self.winning.iter().filter(|x| have_set.contains(x)).count()
    }

    fn points(&self) -> Num {
        match self.win_count() {
            0 => 0,
            count => 1 << (count - 1)
        }
    }
}


/// Examines the matching numbers for a specific card and increases the multiplier
/// of subsequent cards
fn score_card(cards: &Vec<Card>, multipliers: &mut Vec<usize>, which: usize) {
    assert!(cards.len() == multipliers.len());
    assert!(which < cards.len());
    let matches = cards.get(which).unwrap().win_count();
    let multiplier: usize = *multipliers.get(which).unwrap();
    for card_number_won in (which + 1) .. (which + 1 + matches) {
        assert!(card_number_won < cards.len()); // Rules weren't clear, if this is violated I want to know
        multipliers[card_number_won] += multiplier;
    }
}

/// Performs the scoring of part b.
fn score_cards(cards: &Vec<Card>) -> usize {
    let mut multipliers = vec![1; cards.len()];
    for i in 0 .. cards.len() {
        score_card(cards, &mut multipliers, i);
    }
    multipliers.iter().sum()
}


// ======= main() =======


fn part_a(data: &Vec<Card>) {
    println!("\nPart a:");
    let sum: Num = data.iter().map(|x| x.points()).sum();
    println!("Sum of cards: {:?}", sum);
}


fn part_b(data: &Vec<Card>) {
    println!("\nPart b:");
    let scratchcards = score_cards(data);
    println!("Scratchcards = {}", scratchcards);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = parse::input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
