extern crate cached;
extern crate primal;

use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io;
use std::num::ParseIntError;
use cached::proc_macro::cached;
use primal::Primes;
use std::collections::btree_map::BTreeMap;



#[derive(Debug)]
enum Error {
    Io(io::Error),
    Parse(ParseIntError),
}
impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Error::Io(e) } }
impl From<ParseIntError> for Error { fn from(e: ParseIntError) -> Self { Error::Parse(e) } }

fn input() -> Result<u64, Error> {
    let s = fs::read_to_string("input/2015/20/input.txt")?;
    let answer = s.parse()?;
    Ok(answer)
}


fn sign_sequence() -> impl Iterator<Item=i64> {
    [1i64, 1, -1, -1].into_iter().cycle()
}


/// Returns an unbounded iterator of the generalized pentagonal numbers
/// (1, 2, 5, 7, 12, 15...) https://oeis.org/A001318
fn pentagonal_numbers() -> impl Iterator<Item=u64> {
    let mut x: i64 = 0;
    std::iter::from_fn(move || {
        x = if x <= 0 {
            -x + 1
        } else {
            -x
        };
        let val = (3 * x * x - x) / 2;
        Some(val as u64)
    })
}

/// Calculates sigma(x), the sum of the divisors of x. (See https://oeis.org/A000203 ).
/// See formula from one of the answers at
/// https://math.stackexchange.com/questions/22721/is-there-a-formula-to-calculate-the-sum-of-all-proper-divisors-of-a-number
#[cached]
fn sigma(x: u64) -> u64 {
    let mut pents = pentagonal_numbers();
    let mut signs = sign_sequence();
    let mut sum: i64 = 0;
    loop {
        let pent = pents.next().unwrap();
        let arg: i64 = (x as i64) - (pent as i64);
        let term: u64 = match arg {
            _ if arg < 0 => break,
            _ if arg == 0 => x,
            _ => sigma(arg as u64),
        };
        match signs.next().unwrap() {
            -1 => sum -= term as i64,
            1 => sum += term as i64,
            _ => panic!(),
        }
    }
    sum as u64
}


struct PrimeList {
    powers: Vec<(u64, u64)>,
}
impl Display for PrimeList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (p,ex) in self.powers.iter() {
            if *p != 2 {
                write!(f, " * ")?;
            }
            write!(f, "{}^{}", p, ex)?;
        }
        Ok(())
    }
}

/// Returns a Vec of (prime, power) tuples.
fn factor(x: u64) -> PrimeList {
    let mut powers: Vec<(u64, u64)> = Vec::new();
    let mut remaining = x;
    let mut primes = Primes::all();
    while remaining > 1 {
        let p = primes.next().unwrap() as u64;
        let mut exponent = 0;
        while remaining % p == 0 {
            remaining = remaining / p;
            exponent += 1;
        }
        powers.push((p, exponent));
    }
    PrimeList{powers}
}



fn part_a(presents: u64) {
    let sigma_target = (presents + 9) / 10; // divide by 10, rounding up
    println!("sigma_target = {}", sigma_target);
    let mut house = 1;
    let mut biggest_seen = 0;
    loop {
        if sigma(house) > biggest_seen {
            biggest_seen = sigma(house);
            println!("New max: σ({}) -> {}  Factors are {}", house, biggest_seen, factor(house));
        }
        if sigma(house) >= sigma_target {
            println!("Deliveries to house {} will reach or exceed {}.", house, presents);
            break;
        }
        house += 1;
    }
}

// 831600


const ELF_STOP_AT: usize = 50;


fn simulate_elves(desired_presents: u64) -> u64 {
    let mut pending_presents: BTreeMap<u64,u64> = BTreeMap::new();
    let mut house: u64 = 1;
    let mut max_presents_seen: u64 = 0;
    loop {
        let elf_num = house;
        let previous_presents = pending_presents.remove(&house).unwrap_or(0);
        let house_presents = previous_presents + elf_num * 11;
        if house_presents >= desired_presents {
            println!("FOUND IT AT HOUSE {}", house);
            return house;
        }
        if house_presents > max_presents_seen {
            println!("House {} has {} presents", house, house_presents);
            max_presents_seen = house_presents;
        }
        for houses_visited in 2..=ELF_STOP_AT {
            *(pending_presents.entry(elf_num * (houses_visited as u64)).or_insert(0)) += elf_num * 11;
        }
        house += 1;
    }
}


fn part_b(presents: u64) {
    simulate_elves(presents);
}

fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(data);
    part_b(data);
    Ok(())
}
