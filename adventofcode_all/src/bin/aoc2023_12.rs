use anyhow;


// ======= Constants =======


// ======= Parsing =======


/// Type that represents a row in the input.
#[derive(Debug)]
pub struct ConditionRecord {
    pattern: String, // a specific pattern made up of '#', '.', and '?'
    groups: Vec<usize>,
}


type Input = Vec<ConditionRecord>;




mod parse {
    use super::{Input, ConditionRecord};
    use std::fs;
    use nom;
    use nom::{
        IResult,
        bytes::complete::{tag,take_while},
    };
    use nom::character::complete::i32 as nom_num;


    pub fn input<'a>() -> Result<Input, anyhow::Error> {
        let s = fs::read_to_string("input/2023/input_12.txt")?;
        match ConditionRecord::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }


    impl ConditionRecord {
        /// Parses the input. Assumes the grid is at least 1 row and at least one
        /// column or this will fail in various ways.
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            nom::combinator::map(
                nom::sequence::tuple((
                    take_while(|c| c == '.' || c == '#' || c == '?'),
                    tag(" "),
                    nom::multi::separated_list1(
                        tag(","),
                        nom_num
                    )
                )),
                |(pattern, _, groups): (&str, &str, Vec<i32>)| {
                    let pattern: String = pattern.to_string();
                    let groups: Vec<usize> = groups.iter().map(|x| *x as usize).collect();
                    ConditionRecord{pattern, groups}
                }
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


impl ConditionRecord {
    /// Returns the number of extra spaces we can position that weren't accounted for
    /// by the groups.
    fn extra_spaces(&self) -> usize {
        (self.pattern.len() - (self.groups.len() - 1)) - self.groups.iter().sum::<usize>()
    }

    /// Returns an iterator of all the possible patterns of '.' and '#' that match the
    /// given groups.
    ///
    /// NOTE: This is wasteful because I only need to COUNT them, not materialize them.
    ///   But I'm doing it this way for now, and I'll make it more efficient later.
    fn possible_patterns(&self) -> impl Iterator<Item=String> {
        let mut answer: Vec<String> = Vec::new();
        for split in iter_splits(self.extra_spaces(), self.groups.len() + 1) {
            let mut padding = split.iter();
            let mut s: String = String::with_capacity(self.pattern.len());
            fn push(s: &mut String, n: usize, c: char) {
                for _ in 0..n {
                    s.push(c);
                }
            }
            for group in self.groups.iter() {
                if s.len() != 0 { // if before a group but NOT the first group...
                    push(&mut s, 1, '.'); // push the minimum one dot to separate groups
                }
                push(&mut s, *padding.next().unwrap(), '.'); // push {split} extra dots before this group
                push(&mut s, *group, '#'); // push {group} # chars for this group
            }
            push(&mut s, *padding.next().unwrap(), '.'); // push {split} extra dots after last group
            answer.push(s)
        }
        answer.into_iter()
    }

    /// Returns an iterafor of all the possible patterns of '.' and '#' that match the given
    /// groups AND satisfy the pattern.
    fn valid_patterns(&self) -> impl Iterator<Item=String> + '_ {
        self.possible_patterns().filter(|pat| {
            assert!(pat.len() == self.pattern.len());
            std::iter::zip(pat.chars(), self.pattern.chars())
                .all(|(a,b)| {
                    match b {
                        '.' => a == '.',
                        '#' => a == '#',
                        '?' => true,
                        _ => panic!("invalid character in pattern"),
                    }
                })
        })
    }

    /// Return the count of valid patterns for this ConditionRecord.
    fn valid_pattern_count(&self) -> usize {
        self.valid_patterns().count()
    }
}

/// Given a number, n this returns an iterator of the different ways to up n items
/// among k buckets. Each one is expressed as an Vec of length k whose values sum
/// up to n.
///
/// NOTE: There are probably more efficient ways to implement this. I'm using the recursive
/// definition which is easy to write until I get performance problems, then I'll make it
/// smarter.
fn iter_splits(n: usize, k: usize) -> impl Iterator<Item=Vec<usize>> {
    assert!(k >= 1);

    /// Given a vector, makes a new vector with a value appended.
    fn vec_append(vec: Vec<usize>, last: usize) -> Vec<usize> {
        let mut answer = vec.clone();
        answer.push(last);
        answer
    }

    if n == 0 {
        let v: Vec<usize> = vec![0; k];
        let singleton_list = vec![v];
        singleton_list.into_iter()
    } else if k == 1 {
        vec![vec![n]].into_iter()
    } else {
        assert!(k > 1);
        let mut answer: Vec<Vec<usize>> = Vec::new();
        for last in 0..=n {
            for other_buckets in iter_splits(n - last, k - 1) {
                let vv = vec_append(other_buckets, last);
                answer.push(vv);
            }
        }
        answer.into_iter()
    }
}


// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let total_count: usize = input.iter().map(|x| x.valid_pattern_count()).sum();
    println!("The sum of the possible arrangements on each line is {}", total_count);
}


fn part_b(_input: &Input) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
