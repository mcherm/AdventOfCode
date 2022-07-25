
extern crate anyhow;

use std::cmp::{min, max};
use std::fmt::{Display, Formatter};
use std::fs;
use anyhow::Error;
use itertools::Itertools;


use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::newline,
    combinator::map,
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u32 as nom_u32;


fn input() -> Result<Vec<Span>, Error> {
    let s = fs::read_to_string("input/2016/input_20.txt")?;
    match Span::parse_list(&s) {
        Ok(("", spans)) => Ok(spans),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


#[derive(Debug, Eq, PartialEq, Copy, Clone, PartialOrd, Ord)]
struct Span {
    low: u32,
    high: u32,
}

/// This collection of spans excluded promises to be (1) ordered,
/// (2) non-overlapping and non-touching. The spans inserted into it
/// are considered owned and WILL be modified.
struct SpanList {
    items: Vec<Span>
}


impl Span {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                nom_u32,
                tag("-"),
                nom_u32,
            )),
            |(low, _, high)| Self{low, high}
        )(input)
    }

    fn parse_list(input: &str) -> IResult<&str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }

    /// Grows this span by merging in another span. The span must overlap or touch this one
    /// or else it will panic.
    fn merge_in(&mut self, other: &Span) {
        assert!(other.high.saturating_add(1) >= self.low); // other isn't too low to overlap/touch
        assert!(self.high.saturating_add(1) >= other.low); // other isn't too high to overlap/touch
        self.low = min(self.low, other.low);
        self.high = max(self.high, other.high);
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.low, self.high)
    }
}




impl SpanList {
    fn new() -> Self {
        SpanList{items: Vec::new()}
    }


    /// Inserts a span into this SpanList while maintaining the invariants.
    fn insert(&mut self, new_span: &Span) {
        let orig_items_len = self.items.len();
        let mut pos: usize = 0;

        // -- skip past spans that are too low --
        while pos < orig_items_len {
            if self.items.get(pos).unwrap().high.saturating_add(1) < new_span.low {
                pos += 1;
            } else {
                break;
            }
        }

        if pos >= orig_items_len {
            // -- Got to the end and everything was smaller than new_span --
            self.items.push(new_span.clone());
            return; // all finished!
        }

        // -- found one that isn't smaller; maybe we'll merge with this span, maybe we'll go before it --
        let not_smaller_span: &mut Span = self.items.get_mut(pos).unwrap();
        if new_span.high.saturating_add(1) < not_smaller_span.low {
            // -- new_span goes before not_smaller_span --
            self.items.insert(pos, new_span.clone());
            return; // all finished!
        } else {
            // we merge with not_smaller_span (and maybe others after it)
            not_smaller_span.merge_in(new_span); // Modifies not_smaller_span in place
            let inserted_pos = pos;
            pos += 1;
            while pos < orig_items_len {
                let next_span = self.items.get(pos).unwrap().clone();
                let growing_span: &mut Span = self.items.get_mut(inserted_pos).unwrap();
                if growing_span.high + 1 < next_span.low {
                    break; // no more to find that should be merged with growing_span
                } else {
                    growing_span.merge_in(&next_span); // modifies growing_span in place
                    pos += 1;
                }
            }
            // -- drop the ones we merged (all at once so it isn't O(n^2).) --
            if pos > inserted_pos + 1 {
                self.items.drain((inserted_pos + 1)..=pos - 1);
            }
        }
    }
}

impl Display for SpanList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for span in &self.items {
            write!(f, "{}, ", span)?;
        }
        write!(f, "]")
    }
}


/// Given a SpanList of blocked IPs, this returns the count of unblocked IPs. It
/// returns None if EVERY IP is unblocked; otherwise it returns Some() with the
/// count of unblocked IPs.
fn count_unblocked(merged_spans: &SpanList) -> Option<u32> {
    if merged_spans.items.len() == 0 {
        None
    } else {
        let mut allowed_count: u32 = 0;
        allowed_count += merged_spans.items.first().unwrap().low;
        allowed_count += u32::MAX - merged_spans.items.last().unwrap().high;
        for (low_span, high_span) in merged_spans.items.iter().tuple_windows() {
            allowed_count += high_span.low - low_span.high - 1;
        }
        Some(allowed_count)
    }
}



fn part_a(spans: &Vec<Span>) {
    println!("\nPart a:");

    let mut merged_spans: SpanList = SpanList::new();
    for new_span in spans {
        merged_spans.insert(&new_span);
    }

    let lowest_ip = match merged_spans.items.first() {
        None => Some(0),
        Some(span) => if span.low != 0 {
            Some(0)
        } else if span.high == u32::MAX {
            None
        } else {
            Some(span.high + 1)
        },
    };
    match lowest_ip {
        None => println!("Every single IP address is blocked."),
        Some(lowest_ip) => println!("The lowest non-blocked IP is {}.", lowest_ip),
    }
}



fn part_b(spans: &Vec<Span>) {
    println!("\nPart b:");
    let mut merged_spans: SpanList = SpanList::new();
    for new_span in spans {
        merged_spans.insert(&new_span);
    }

    match count_unblocked(&merged_spans) {
        None => println!("Since no IPs are blocked, there are {} IPs allowed.", 1u64 << 32),
        Some(count) => println!("There are {} unblocked IP addresses.", count),
    }
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}



// ==========================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_span_merge_in_lower() {
        let mut s1 = Span{low: 10, high: 15};
        let s2 = Span{low: 0, high: 8};
        s1.merge_in(&s2);
    }

    #[test]
    fn test_span_merge_in_low() {
        let mut s1 = Span{low: 10, high: 15};
        let s2 = Span{low: 5, high: 9};
        s1.merge_in(&s2);
        assert_eq!(s1, Span{low: 5, high: 15});
    }

    #[test]
    fn test_span_merge_in_within() {
        let mut s1 = Span{low: 10, high: 15};
        let s2 = Span{low: 12, high: 13};
        s1.merge_in(&s2);
        assert_eq!(s1, Span{low: 10, high: 15});
    }

    #[test]
    fn test_span_merge_in_around() {
        let mut s1 = Span{low: 10, high: 15};
        let s2 = Span{low: 6, high: 20};
        s1.merge_in(&s2);
        assert_eq!(s1, Span{low: 6, high: 20});
    }

    #[test]
    fn test_span_merge_in_high() {
        let mut s1 = Span{low: 10, high: 15};
        let s2 = Span{low: 13, high: 20};
        s1.merge_in(&s2);
        assert_eq!(s1, Span{low: 10, high: 20});
    }

    #[test]
    #[should_panic]
    fn test_span_merge_in_higher() {
        let mut s1 = Span{low: 10, high: 15};
        let s2 = Span{low: 20, high: 25};
        s1.merge_in(&s2);
    }

    #[test]
    fn test_span_list() {
        let mut slist: SpanList = SpanList::new();
        assert_eq!(0, slist.items.len());
        slist.insert(&Span{low: 10, high: 15});
        assert_eq!(vec![Span{low: 10, high: 15}], slist.items);
        slist.insert(&Span{low:40, high: 50});
        assert_eq!(vec![Span{low: 10, high: 15}, Span{low: 40, high: 50}], slist.items);
        slist.insert(&Span{low:20, high: 30});
        assert_eq!(vec![Span{low: 10, high: 15}, Span{low: 20, high: 30}, Span{low: 40, high: 50}], slist.items);
        slist.insert(&Span{low:5, high: 12});
        assert_eq!(vec![Span{low: 5, high: 15}, Span{low: 20, high: 30}, Span{low: 40, high: 50}], slist.items);
        slist.insert(&Span{low:4, high: 4});
        assert_eq!(vec![Span{low: 4, high: 15}, Span{low: 20, high: 30}, Span{low: 40, high: 50}], slist.items);
        slist.insert(&Span{low:31, high: 32});
        assert_eq!(vec![Span{low: 4, high: 15}, Span{low: 20, high: 32}, Span{low: 40, high: 50}], slist.items);
        slist.insert(&Span{low:8, high: 25});
        assert_eq!(vec![Span{low: 4, high: 32}, Span{low: 40, high: 50}], slist.items);
        slist.insert(&Span{low:55, high: 60});
        slist.insert(&Span{low:65, high: 70});
        slist.insert(&Span{low:75, high: 80});
        assert_eq!(vec![Span{low: 4, high: 32}, Span{low: 40, high: 50}, Span{low: 55, high: 60}, Span{low: 65, high: 70}, Span{low: 75, high: 80}], slist.items);
        slist.insert(&Span{low:45, high: 68});
        assert_eq!(vec![Span{low: 4, high: 32}, Span{low: 40, high: 70}, Span{low: 75, high: 80}], slist.items);
    }
}
