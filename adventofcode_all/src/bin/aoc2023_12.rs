use std::fmt::{Display, Formatter};
use anyhow;
use itertools::Itertools;


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

    /// Given a ConditionRecord, this unfolds it into a bigger one.
    fn unfold(&self) -> Self {
        let mut pattern: String = String::with_capacity(self.pattern.len() * 5 + 4);
        pattern.push_str(&self.pattern);
        pattern.push('?');
        pattern.push_str(&self.pattern);
        pattern.push('?');
        pattern.push_str(&self.pattern);
        pattern.push('?');
        pattern.push_str(&self.pattern);
        pattern.push('?');
        pattern.push_str(&self.pattern);

        let mut groups: Vec<usize> = Vec::with_capacity(self.groups.len() * 5);
        groups.extend_from_slice(&self.groups);
        groups.extend_from_slice(&self.groups);
        groups.extend_from_slice(&self.groups);
        groups.extend_from_slice(&self.groups);
        groups.extend_from_slice(&self.groups);

        ConditionRecord{pattern, groups}
    }
}


#[derive(Debug)]
struct Split {
    n: usize,
    k: usize,
    first: usize,
    rest: Option<Box<Split>>, // None if k == 1; otherwise a Split with k that is 1 less
}

impl Split {
    /// Gives the beginning of an iteration.
    fn start(n: usize, k: usize) -> Self {
        assert!(k > 0);
        if k == 1 {
            let first = n;
            let rest = None;
            Split{n, k, first, rest}
        } else {
            let first = 0;
            let rest = Some(Box::new(Split::start(n, k - 1)));
            Split{n, k, first, rest}
        }
    }

    /// This attempts ot proceed to the next split, whatever that is. If it succeeds, then
    /// it returns true; if there are no more then it returns false (and will keep returning
    /// false if you keep trying to use it).
    fn next(&mut self) -> bool {
        self.next_at(self.k - 1)
    }

    /// This attempts to proceed to the next split which advances position p. If it succeeds, then
    /// it returns true; if there are no more then it returns false (and will keep returning
    /// false if you keep trying to use it). Normally p < k, but p == k is also allowed, which
    /// will be treated the same as p == k - 1.
    fn next_at(&mut self, p: usize) -> bool {
        assert!(p < self.k); // you are not allowed to advance a slot we don't have!
        if p == 0 {
            if self.first == self.n {
                // want to advance 1st spot, and it can't advance -- we're done.
                false
            } else {
                // need to advance 1st spot (making a new 'rest')
                assert!(self.k > 1 && self.rest.is_some()); // if k==1 then self.first == self.n so we never got here
                assert!(self.first < self.n); // and this got checked in the if clause
                self.first += 1;
                self.rest = Some(Box::new(Split::start(self.n - self.first, self.k - 1)));
                true
            }
        } else {
            // want to advance something in 'rest'
            assert!(self.k > 1 && self.rest.is_some());
            if let Some(ref mut rest) = &mut self.rest {
                if rest.next_at(p - 1) { // recurse!
                    // we advanced the 'rest' successfully
                    true
                } else {
                    // tried to advance 'rest' but it couldn't go any further. So advance the 1st position
                    self.next_at(0) // recurse! (but differently)
                }
            } else {
                panic!("we already used an assert to check this");
            }
        }
    }


    /// This is given a slice, and populates it with the values from this Split.
    /// All values will be overwritten.
    fn fill_values(&self, output: &mut [usize]) {
        assert!(self.k == output.len());
        output[0] = self.first;
        if let Some(rest) = &self.rest {
            rest.fill_values( &mut output[1..] );
        }
    }

    /// This returns the m'th position within the Split.
    fn get_at(&self, m: usize) -> usize {
        assert!(m <= self.k);
        if m == 0 {
            self.first
        } else {
            if let Some(rest) = &self.rest {
                rest.get_at(m - 1)
            } else {
                panic!("should be impossible to reach this");
            }
        }
    }
}


impl Display for Split {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = vec![0; self.k];
        self.fill_values(&mut output);
        write!(f, "{:?}", output)
    }
}

/// For complex internal reasons, this is an array of bools (where true means we
/// got a '#' there, and false means we got a '.'). It also has an equal-length array
/// of numbers telling which spot between groups to advance if we get a mismatch in that
/// location.
#[derive(Debug)]
struct Pat {
    is_hash: Vec<bool>,
    to_advance: Vec<usize>,
}

impl Pat {

    fn len(&self) -> usize {
        assert!(self.is_hash.len() == self.to_advance.len());
        self.is_hash.len()
    }

    fn is_hash(&self, i: usize) -> bool {
        assert!(i < self.len());
        self.is_hash[i]
    }

    /// Given a ConditionRecord and a particular split, return the "Pat" for that length, which
    /// knows which ones are dots and which are hashes but also knows which spot between groups
    /// to advance if there is a mismatch at this spot.
    fn new(record: &ConditionRecord, split: &Split) -> Self {
        // helper function -- this pushes an equal number of copies of a boolean and a number
        // onto the corresponding vectors.
        fn push(n: usize, is_hash: &mut Vec<bool>, b: bool, to_advance: &mut Vec<usize>, x: usize) {
            for _ in 0..n {
                is_hash.push(b);
                to_advance.push(x);
            }
        }
        let mut is_hash: Vec<bool> = Vec::with_capacity(record.pattern.len());
        let mut to_advance: Vec<usize> = Vec::with_capacity(record.pattern.len());
        for (i, group) in record.groups.iter().enumerate() {
            if is_hash.len() != 0 { // if before a group but NOT the first group...
                push(1, &mut is_hash, false, &mut to_advance, i); // push the minimum one dot to separate groups; advance before the group
            }
            let items_from_split = split.get_at(i);
            push(items_from_split, &mut is_hash, false, &mut to_advance, i); // push {split} extra dots before this group; advance before the group
            push(*group, &mut is_hash, true, &mut to_advance, i); // push {group} # chars for this group; advance before the group
        }
        let items_from_split = split.get_at(record.groups.len());
        // push {split} extra dots after last group; if these fail to match, advance the second-to-last
        push(items_from_split, &mut is_hash, false, &mut to_advance, record.groups.len() - 1);
        Pat{is_hash, to_advance}
    }
}

impl ConditionRecord {

    // A faster version for doing part b.
    fn fast_valid_pattern_count(&self) -> usize {
        let n = self.extra_spaces();
        let k = self.groups.len() + 1;
        let mut split = Split::start(n, k);
        let mut answer: usize = 0;
        loop {
            let pat = Pat::new(self, &split);

            // Inner function that looks through pat and pattern and discovers whether they
            // match. It then advances split -- by the minimum possible if they matched or
            // next_at() the first position that didn't match if they don't. It returns two
            // booleans: the first telling if they matched and the second telling if split
            // advanced (it will fail to advance only if we're out of splits to try).
            fn check_for_match_and_advance(split: &mut Split, pat: &Pat, pattern: &String) -> (bool,bool) {
                assert!(pat.len() == pattern.len());
                for (i,c) in pattern.chars().enumerate() {
                    let allowed = if pat.is_hash(i) {
                        c == '#' || c == '?'
                    } else {
                        c == '.' || c == '?'
                    };
                    if !allowed {
                        return (false, split.next_at(pat.to_advance[i]));
                    }
                }
                return (true, split.next());
            }
            let (matches, had_next) = check_for_match_and_advance(&mut split, &pat, &self.pattern);
            if matches {
                answer += 1;
            }
            if !had_next {
                break;
            }
        }
        answer
    }
}

// ======= main() =======


fn part_a(input: &Input) {
    println!("\nPart a:");
    let total_count: usize = input.iter().map(|x| x.fast_valid_pattern_count()).sum();
    println!("The sum of the possible arrangements on each line is {}", total_count);
}


fn part_b(input: &Input) {
    println!("\nPart b:");
    let unfolded = input.iter().map(|x| x.unfold()).collect_vec();
    let total_count: usize = unfolded.iter()
        .map(|x| {
            println!("--- doing {:?}", x);
            x.fast_valid_pattern_count()
        })
        .sum();
    println!("The sum of the possible arrangements on each line is {}", total_count);
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let input = parse::input()?;
    part_a(&input);
    part_b(&input);
    Ok(())
}
