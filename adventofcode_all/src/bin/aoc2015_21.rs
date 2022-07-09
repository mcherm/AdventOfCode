use advent_lib::eznom;
#[macro_use]
extern crate lazy_static;


use std::fmt::{Debug, Formatter};
use std::fs;
use std::io;
use std::num::ParseIntError;
use itertools::Itertools;

use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::newline as nom_newline;
use nom::character::complete::u32 as nom_value;
use nom::sequence::tuple as nom_tuple;
use eznom::type_builder;


const PLAYER_HIT_POINTS: u32 = 100;


#[derive(Debug)]
enum Error {
    Io(io::Error),
    Parse(ParseIntError),
}
impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Error::Io(e) } }
impl From<ParseIntError> for Error { fn from(e: ParseIntError) -> Self { Error::Parse(e) } }



fn input() -> Result<Boss, Error> {
    let s = fs::read_to_string("input/2015/21/input.txt")?;
    match Boss::parse(&s) {
        Ok(("", boss)) => Ok(boss),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Slot {
    Weapon,
    Armor,
    Ring,
}

#[derive(Clone, Debug)]
struct Item {
    slot: Slot,
    name: String,
    cost: u32,
    damage: u32,
    armor: u32,
}

struct Shop {
    items: Vec<Item>,
}


struct Outfit<'a> {
    items: Vec<&'a Item>
}

trait CDR {
    fn cost(&self) -> u32;
    fn damage(&self) -> u32;
    fn armor(&self) -> u32;
}


impl Item {
    fn new(slot: Slot, name: &str, cost: u32, damage: u32, armor: u32) -> Self {
        let name_string = name.to_string();
        Item{slot, name: name_string, cost, damage, armor}
    }
}

impl Shop {
    /*
     * Given a slot, this returns the list of items that can go in that slot.
     */
    fn get_items(&self, slot: Slot) -> Vec<&Item> {
        return self.items.iter().filter(|item| item.slot == slot).collect();
    }

    fn get_outfits<'a>(&self) -> impl Iterator<Item=Outfit> {
        return PossibleOutfitIterator::new(self);
    }
}


impl<'a> Debug for Outfit<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let item_names = self.items.iter().map(|it| &it.name).join(", ");
        write!(f, "Cost {}, damage {}, armor {} from using {}.", self.cost(), self.damage(), self.armor(), item_names)
    }
}


impl<'a> CDR for Outfit<'a> {
    fn cost(&self) -> u32 { self.items.iter().map(|it| it.cost).sum() }
    fn damage(&self) -> u32 { self.items.iter().map(|it| it.damage).sum() }
    fn armor(&self) -> u32 { self.items.iter().map(|it| it.armor).sum() }
}

impl<'a> Combatant for Outfit<'a> {
    fn get_hit_points(&self) -> u32 { PLAYER_HIT_POINTS }
    fn get_damage(&self) -> u32 { self.damage() }
    fn get_armor(&self) -> u32 { self.armor() }
}

lazy_static! {
    static ref THE_SHOP: Shop = Shop{
        items: vec![
            Item::new(Slot::Weapon, "Dagger",       8, 4, 0),
            Item::new(Slot::Weapon, "Shortsword",  10, 5, 0),
            Item::new(Slot::Weapon, "Warhammer",   25, 6, 0),
            Item::new(Slot::Weapon, "Longsword",   40, 7, 0),
            Item::new(Slot::Weapon, "Greataxe",    74, 8, 0),
            Item::new(Slot::Armor,  "Leather",     13, 0, 1),
            Item::new(Slot::Armor,  "Chainmail",   31, 0, 2),
            Item::new(Slot::Armor,  "Splintmail",  53, 0, 3),
            Item::new(Slot::Armor,  "Bandedmail",  75, 0, 4),
            Item::new(Slot::Armor,  "Platemail",  102, 0, 5),
            Item::new(Slot::Ring,   "Damage +1",   25, 1, 0),
            Item::new(Slot::Ring,   "Damage +2",   50, 2, 0),
            Item::new(Slot::Ring,   "Damage +3",  100, 3, 0),
            Item::new(Slot::Ring,   "Defense +1",  20, 0, 1),
            Item::new(Slot::Ring,   "Defense +2",  40, 0, 2),
            Item::new(Slot::Ring,   "Defense +3",  80, 0, 3),
        ]
    };
}


#[derive(Copy, Clone, Debug)]
struct Boss {
    hit_points: u32,
    damage: u32,
    armor: u32,
}

trait Combatant {
    fn get_hit_points(&self) -> u32;
    fn get_damage(&self) -> u32;
    fn get_armor(&self) -> u32;
}

impl Boss {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let recognize = |s| nom_tuple((
            nom_tag("Hit Points: "),
            nom_value,
            nom_newline,
            nom_tag("Damage: "),
            nom_value,
            nom_newline,
            nom_tag("Armor: "),
            nom_value,
            nom_newline,
        ))(s);
        let build = |(_, hit_points, _, _, damage, _, _, armor, _): (&str, u32, char, &str, u32, char, &str, u32, char)|
            Self{hit_points, damage, armor};
        type_builder(recognize, build)(input)
    }
}

impl Combatant for Boss {
    fn get_hit_points(&self) -> u32 { self.hit_points }
    fn get_damage(&self) -> u32 { self.damage }
    fn get_armor(&self) -> u32 { self.armor }
}

/*
 * Performs a/b but fractions round up.
 */
fn divide_round_up(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}

/*
 * Performs a - b, unless it would be negative, then returns 0
 */
fn subtract_or_zero(a: u32, b: u32) -> u32 {
    if b > a {
        0
    } else {
        a - b
    }
}


/*
 * This accepts 2 combatants, the first is the player the second is the boss. It
 * returns true if the player wins and false if the boss wins.
 */
fn wins_fight(player: &dyn Combatant, boss: &dyn Combatant) -> bool {
    let p_blow = subtract_or_zero(player.get_damage(), boss.get_armor());
    if p_blow == 0 {
        return false;
    }
    let b_blow = subtract_or_zero(boss.get_damage(), player.get_armor());
    if b_blow == 0 {
        return true;
    }
    return divide_round_up(boss.get_hit_points(), p_blow) <= divide_round_up(player.get_hit_points(), b_blow)
}



struct PossibleOutfitIterator<'a> {
    weapons: Vec<&'a Item>,
    armors: Vec<&'a Item>,
    rings: Vec<&'a Item>,
    next_pos: Option<(usize, usize, usize, usize)>,
}

impl<'a> PossibleOutfitIterator<'a> {
    fn new(shop: &'a Shop) -> Self {
        let weapons = shop.get_items(Slot::Weapon);
        let armors = shop.get_items(Slot::Armor);
        let rings = shop.get_items(Slot::Ring);
        let next_pos = Some((0, 0, 0, 0));
        PossibleOutfitIterator{weapons, armors, rings, next_pos}
    }

    fn incr(&mut self) {
        // invariants:
        //     0 < w < weapons.len()
        //     0 < a <= armors.len()
        //     0 < r1 <= rings.len()
        //     0 < r2 <= rings.len() && (if r2 > 0 then r2 < r1)
        self.next_pos = match self.next_pos {
            None => None,
            Some((w, a, r1, r2)) => {
                if r2 < self.rings.len() && r2 + 1 < r1 {
                    Some((w, a, r1, r2 + 1)) // increment r2
                } else if r1 < self.rings.len() {
                    Some((w, a, r1 + 1, 0)) // increment r1
                } else if a < self.armors.len() {
                    Some((w, a + 1, 0, 0)) // increment a
                } else if w < self.weapons.len() - 1 {
                    Some((w + 1, 0, 0, 0)) // increment w
                } else {
                    None
                }
            }
        }
    }
}

impl<'a> Iterator for PossibleOutfitIterator<'a> {
    type Item = Outfit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_pos {
            None => None,
            Some((w, a, r1, r2)) => {
                let mut items: Vec<&'a Item> = Vec::new();
                items.push(self.weapons[w]);
                if a > 0 {
                    items.push(self.armors[a - 1]);
                }
                if r1 > 0 {
                    items.push(self.rings[r1 - 1]);
                }
                if r2 > 0 {
                    items.push(self.rings[r2 - 1]);
                }
                self.incr();
                Some(Outfit{items})
            },
        }
    }
}


fn part_a(boss: &Boss) {
    let mut cheapest_winning_outfit: Option<Outfit> = None;
    for outfit in THE_SHOP.get_outfits() {
        if wins_fight(&outfit, boss) {
            cheapest_winning_outfit = Some(match cheapest_winning_outfit {
                None => outfit,
                Some(old_outfit) => if outfit.cost() < old_outfit.cost() {
                    outfit
                } else {
                    old_outfit
                }
            });
        }
    }
    match cheapest_winning_outfit {
        None => println!("There is no winning option."),
        Some(outfit) => println!("The cheapest winning option is {:?}", outfit),
    }
}


fn part_b(boss: &Boss) {
    let mut dearest_winning_outfit: Option<Outfit> = None;
    for outfit in THE_SHOP.get_outfits() {
        if !wins_fight(&outfit, boss) {
            dearest_winning_outfit = Some(match dearest_winning_outfit {
                None => outfit,
                Some(old_outfit) => if outfit.cost() > old_outfit.cost() {
                    outfit
                } else {
                    old_outfit
                }
            });
        }
    }
    match dearest_winning_outfit {
        None => println!("There is no way to lose."),
        Some(outfit) => println!("The most expensive losing option is {:?}", outfit),
    }
}

fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
