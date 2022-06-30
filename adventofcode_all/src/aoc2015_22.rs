mod eznom;


use std::cmp::max;
use std::fmt::{Debug};
use std::fs;
use std::io;
use std::num::ParseIntError;

use nom::bytes::complete::tag as nom_tag;
use nom::character::complete::newline as nom_newline;
use nom::character::complete::u32 as nom_value;
use nom::sequence::tuple as nom_tuple;
use eznom::type_builder;


const WIZARD_STARTING_HIT_POINTS: u32 = 10;
const WIZARD_STARTING_MANA: u32 = 250;


#[derive(Debug)]
enum Error {
    Io(io::Error),
    Parse(ParseIntError),
}
impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Error::Io(e) } }
impl From<ParseIntError> for Error { fn from(e: ParseIntError) -> Self { Error::Parse(e) } }



fn input() -> Result<Boss, Error> {
    let s = fs::read_to_string("input/2015/22/input.txt")?;
    match Boss::parse(&s) {
        Ok(("", boss)) => Ok(boss),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


/// Modifies the number a points at, subtracting b from it but saturating at zero.
fn subtract_capped(a: &mut u32, b: u32) {
    if b > *a {
        *a = 0
    } else {
        *a -= b
    }
}


trait Combatant {
    fn get_hit_points(&self) -> u32;
    fn get_damage(&self) -> u32;
    fn get_armor(&self) -> u32;
}


#[derive(Copy, Clone, Debug)]
struct Boss {
    hit_points: u32,
    damage: u32,
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
        ))(s);
        let build = |(_, hit_points, _, _, damage, _): (&str, u32, char, &str, u32, char)|
            Self{hit_points, damage};
        type_builder(recognize, build)(input)
    }
}

impl Combatant for Boss {
    fn get_hit_points(&self) -> u32 { self.hit_points }
    fn get_damage(&self) -> u32 { self.damage }
    fn get_armor(&self) -> u32 { 0 }
}


#[derive(Copy, Clone, Debug)]
enum Spell {
    MagicMissile,
    Drain,
    Shield,
    Poison,
    Recharge,
}

impl Spell {
    fn cost(&self) -> u32 {
        match self {
            Spell::MagicMissile => 53,
            Spell::Drain => 73,
            Spell::Shield => 113,
            Spell::Poison => 173,
            Spell::Recharge => 229,
        }
    }
}


#[derive(Copy, Clone, Debug)]
struct Wizard {
    hit_points: u32,
    mana: u32,
}

impl Wizard {
    fn new() -> Self {
        Wizard{
            hit_points: WIZARD_STARTING_HIT_POINTS,
            mana: WIZARD_STARTING_MANA,
        }
    }
}


#[derive(Clone, Debug)]
struct GameState {
    wizard: Wizard,
    boss: Boss,
    shield_effect_turns: u32,
    poison_effect_turns: u32,
    recharge_effect_turns: u32,
    spells_cast: Vec<Spell>,
    spell_cost: u32,
}

impl GameState {
    /// Returns true if the wizard has won in this game state; false if they haven't.
    fn winning(&self) -> bool {
        self.boss.hit_points == 0
    }

    /// Create a game state for beginning the fight with a given Wizard and Boss.
    fn new(wizard: Wizard, boss: Boss) -> Self {
        GameState{
            wizard,
            boss,
            shield_effect_turns: 0,
            poison_effect_turns: 0,
            recharge_effect_turns: 0,
            spells_cast: Vec::with_capacity(0),
            spell_cost: 0}
    }

    /// Returns true if the spell can be cast now; false if not.
    fn spell_allowed(&self, spell: Spell) -> bool {
        match spell {
            Spell::Shield => self.shield_effect_turns == 0,
            Spell::Poison => self.poison_effect_turns == 0,
            Spell::Recharge => self.recharge_effect_turns == 0,
            _ => true,
        }
    }

    /// Given this GameState, this method simulates one Wizard turn followed by one
    /// Boss turn, assuming the Wizard chooses to cast the given Spell. It returns
    /// None if the result is the death of the Wizard, or the resulting GameState
    /// if the Wizard survives. (If the Boss dies on the Wizard's turn, then only
    /// that turn will have been simulated.)
    fn perform(&self, spell: Spell) -> Option<Self> {
        // --- Set up variables ---
        if spell.cost() > self.wizard.mana {
            return None
        }
        let mut wizard = self.wizard.clone();
        wizard.mana -= spell.cost();
        let mut spells_cast: Vec<Spell> = self.spells_cast.clone();
        spells_cast.push(spell);
        let spell_cost = self.spell_cost + spell.cost();
        let mut boss = self.boss.clone();
        let mut shield_effect_turns = self.shield_effect_turns;
        let mut poison_effect_turns = self.poison_effect_turns;
        let mut recharge_effect_turns = self.recharge_effect_turns;


        // --- Create reusable helper fn ---
        fn apply_effects(
            shield_effect_turns: &mut u32,
            poison_effect_turns: &mut u32,
            recharge_effect_turns: &mut u32,
            wizard_mana: &mut u32,
            boss_hit_points: &mut u32,
        ) {
            if *poison_effect_turns > 0 {
                subtract_capped(boss_hit_points, 3);
            }
            subtract_capped(poison_effect_turns, 1);
            subtract_capped(shield_effect_turns, 1);
            if *recharge_effect_turns > 0 {
                *wizard_mana += 101;
            }
            subtract_capped(recharge_effect_turns, 1);
        }

        // --- Process wizard attack ---
        apply_effects(&mut shield_effect_turns, &mut poison_effect_turns, &mut recharge_effect_turns, &mut wizard.mana, &mut boss.hit_points);
        match spell {
            Spell::MagicMissile => {
                subtract_capped(&mut boss.hit_points, 4);
            },
            Spell::Drain => {
                subtract_capped(&mut boss.hit_points, 2);
                wizard.hit_points += 2;
            },
            Spell::Shield => {
                shield_effect_turns += 6;
            },
            Spell::Poison => {
                poison_effect_turns += 6;
            },
            Spell::Recharge => {
                recharge_effect_turns += 5;
            },
        };

        // --- Process Boss attack ---
        if wizard.hit_points > 0 && boss.hit_points > 0 {
            apply_effects(&mut shield_effect_turns, &mut poison_effect_turns, &mut recharge_effect_turns, &mut wizard.mana, &mut boss.hit_points);
            subtract_capped(&mut wizard.hit_points, damage_done_to_wizard(shield_effect_turns, boss.damage));
        }

        // --- Return the new game state ---
        if wizard.hit_points == 0 {
            None
        } else {
            Some(GameState{
                wizard,
                boss,
                shield_effect_turns,
                poison_effect_turns,
                recharge_effect_turns,
                spells_cast,
                spell_cost
            })
        }
    }
}


fn damage_done_to_wizard(shield_effect_turns: u32, boss_damage: u32) -> u32 {
    let shield = if shield_effect_turns == 0 {0} else {7};
    let mut damage = boss_damage;
    subtract_capped(&mut damage, shield);
    max(damage, 1)
}


fn part_a(boss: &Boss) {
    let mut reachable_states: Vec<GameState> = Vec::new();
    let initial_state = GameState::new(Wizard::new(), *boss);
    reachable_states.push(initial_state);

    while !reachable_states.is_empty() {
        println!("");
        println!("ALL states: [");
        for state in reachable_states.iter() {
            println!("    {:?}", state);
        }
        println!("]");

        let first_state = reachable_states.swap_remove(0);
        for spell in [Spell::MagicMissile, Spell::Drain, Spell::Shield, Spell::Poison, Spell::Recharge] {
            if first_state.spell_allowed(spell) {
                match first_state.perform(spell) {
                    None => {},
                    Some(next_state) => {
                        println!("next_state: {:?}", next_state);
                        if next_state.winning() { // tried the spells in order by cost, so the first winner is the best overall
                            println!("Wizard wins!");
                            return ();
                        }
                        reachable_states.push(next_state);
                    },
                }
            }
        }
        reachable_states.sort_by(|a,b| a.spell_cost.cmp(&b.spell_cost));
    }
    println!("Wizard cannot win.");
}


fn part_b(_boss: &Boss) {
}

fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
