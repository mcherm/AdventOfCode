mod eznom;

use std::fs;
use std::io;
use std::cmp::Reverse;

use itertools::Itertools;


use nom::character::complete::u32 as nom_num;
use nom::character::complete::newline as nom_newline;
use nom::multi::many0 as nom_many0;
use nom::sequence::tuple as nom_tuple;
use eznom::type_builder;


const DISPLAY_WORK: bool = false;

type PkgSize = u32;
type QeSize = u64;


#[derive(Debug)]
enum Error {
    Io(io::Error),
}
impl From<io::Error> for Error { fn from(e: io::Error) -> Self { Error::Io(e) } }



fn input() -> Result<Vec<PkgSize>, Error> {
    let s = fs::read_to_string("input/2015/24/input.txt")?;
    match parse_input(&s) {
        Ok(("", mut sizes)) => {
            // Sort them from largest to smallest
            sizes.sort_by_key(|x| Reverse(*x));
            Ok(sizes)
        },
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


fn parse_line(input: &str) -> nom::IResult<&str, PkgSize> {
    let recognize = |s| nom_tuple((
        nom_num,
        nom_newline,
    ))(s);
    let build = |(n, _): (PkgSize, char)| n;
    type_builder(recognize, build)(input)
}

fn parse_input(input: &str) -> nom::IResult<&str, Vec<PkgSize>> {
    nom_many0(parse_line)(input)
}



/// Given a vector of references to PkgSize, this sums them.
fn sum_refs(refs: &Vec<&PkgSize>) -> PkgSize {
    refs.iter().map(|x| *x).sum()
}

/// Calculates the "quantum entanglement" for group1.
fn quantum_entanglement(group1: &Vec<&PkgSize>) -> QeSize {
    assert!(group1.len() >= 1);
    group1.iter().map(|x| (**x) as QeSize).product()
}


/// Returns the minimum QE
fn solve(all_sizes: &Vec<PkgSize>) -> QeSize {
    assert!(all_sizes.len() >= 3);
    let sum: PkgSize = all_sizes.iter().sum::<PkgSize>();
    assert!(sum % 3 == 0);
    let group_sum = sum / 3;

    let mut min_qe: Option<QeSize> = None;

    'group1size: for group1_size in 1..(all_sizes.len() - 2) {
        for group1 in all_sizes.iter().combinations(group1_size) {
            if sum_refs(&group1) == group_sum {

                let qe = quantum_entanglement(&group1);
                if min_qe.is_none() || qe < min_qe.unwrap() {

                    let group23: Vec<&PkgSize> = all_sizes.iter().filter(|x| !group1.contains(x)).collect();
                    assert!(group23.len() >= 2);

                    'group23: for group2_size in 1..(group23.len() - 1) {
                        for group2 in group23.iter().map(|x| *x).combinations(group2_size) {
                            if sum_refs(&group2) == group_sum {
                                let group3: Vec<&PkgSize> = group23.iter().map(|x| *x).filter(|x| !group2.contains(x)).collect();
                                assert!(group3.len() >= 1);
                                assert!(sum_refs(&group3) == group_sum);
                                if DISPLAY_WORK {
                                    println!("group1: {:?}  (QE={})  group2: {:?}  group3: {:?}", group1, qe, group2, group3);
                                }
                                min_qe = Some(match min_qe {
                                    None => qe,
                                    Some(old_qe) => std::cmp::min(old_qe, qe),
                                });
                                break 'group23; // We only need to find ONE way to split groups 2 & 3.
                            }
                        }
                    }

                }
            }
        }
        if min_qe.is_some() {
            break 'group1size; // We don't have to explore larger sizes.
        }
    }
    assert!(min_qe.is_some()); // assert we had a solution
    min_qe.unwrap()
}


fn part_a(all_sizes: &Vec<PkgSize>) {
    println!("---- Part A ----");
    let min_qe = solve(all_sizes);
    println!("The lowest QE is {}", min_qe);
}


fn part_b(_sizes: &Vec<PkgSize>) {
    println!("---- Part B ----");
}

fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
