
extern crate anyhow;

use std::fs;
use nom;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::line_ending,
};
use nom::character::complete::i32 as nom_i32;
use std::fmt::{Display, Formatter};
use std::cmp::{min, max};
use std::collections::BTreeSet;


// ======= Parsing =======

fn input() -> Result<Vec<SensorAndBeacon>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_15.txt")?;
    match SensorAndBeacon::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Num = i32;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Point(Num, Num);

#[derive(Debug)]
struct SensorAndBeacon {
    sensor: Point,
    beacon: Point,
}


impl Point {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        nom::combinator::map(
            nom::sequence::tuple((
                tag("x="),
                nom_i32,
                tag(", y="),
                nom_i32,
            )),
            |(_, x, _, y)| Point(x,y)
        )(input)
    }

    /// Returns the (always positive) distance between self and other, using manhattan
    /// distance (steps along the grid).
    fn manhattan_dist(&self, other: &Self) -> Num {
        (self.0.abs_diff(other.0) + self.1.abs_diff(other.1)).try_into().unwrap()
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}


impl SensorAndBeacon {
    /// Parses a single SensorAndBeacon
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        nom::combinator::map(
            nom::sequence::tuple((
                tag("Sensor at "),
                Point::parse,
                tag(": closest beacon is at "),
                Point::parse,
            )),
            |(_, sensor, _, beacon)| SensorAndBeacon{sensor, beacon}
        )(input)
    }

    /// Parses a newline-terminated list of LineSpecs
    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        nom::multi::many0( nom::sequence::terminated(Self::parse, line_ending) )(input)
    }

    /// Returns the (manhattan) distance between the sensor and it's beacon.
    fn radius(&self) -> Num {
        self.sensor.manhattan_dist(&self.beacon)
    }

    /// Returns the span of spaces in the given row where this sensor would have detected a
    /// beacon (which may or may not include the beacon we DID detect), or None if this sensor
    /// wouldn't have detected anything on that row.
    fn span_at_row(&self, row: Num) -> Option<Span> {
        let r = self.radius();
        let dist_to_row: Num = self.sensor.1.abs_diff(row).try_into().unwrap();
        if dist_to_row > r {
            None
        } else {
            let center = self.sensor.0;
            let spare_length = r - dist_to_row;
            let span = Span::new(center - spare_length, center + spare_length);
            Some(span)
        }
    }
}


// ======= Compute =======

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Span {
    start: Num, // inclusive
    end: Num, // inclusive
}

#[derive(Debug)]
struct Row {
    row_num: Num,
    spans: Vec<Span>,
    beacons: BTreeSet<Num>,
}


impl Span {
    /// Construct a new Span. start and end are both inclusive and
    /// start <= end.
    fn new(start: Num, end: Num) -> Self {
        assert!(start <= end);
        Span{start, end}
    }

    /// If this span overlaps or abuts the other span, this returns the joined
    /// span. If it doesn't, then a single span can't account for both of them
    /// so this returns None.
    fn merged(&self, other: &Span) -> Option<Span> {
        if self.start > other.end + 1 {
            None
        } else if self.end + 1 < other.start {
            None
        } else {
            Some(Span::new(min(self.start, other.start), max(self.end, other.end)))
        }
    }

    /// Returns the number of spaces this span covers. (Always >= 1)
    fn spaces_covered(&self) -> Num {
        1 + self.end - self.start
    }
}

impl Row {
    fn new(row_num: Num) -> Self {
        Row{spans: Vec::new(), beacons: BTreeSet::new(), row_num}
    }

    /// Adds this span to the list of spans. But ALSO consolidates any spans that
    /// either overlap or abut each other.
    ///
    /// NOTE: a better algorithm would keep them sorted and only need to test a few.
    ///   Should probably switch to that, but right now the list of spans is quite
    ///   short so I haven't bothered yet.
    fn add_span(&mut self, span: &Span) {
        let mut span_to_add = span.clone();
        let mut changed = true;
        while changed {
            changed = false;
            let mut i = 0;
            while i < self.spans.len() {
                match span_to_add.merged(self.spans.get(i).unwrap()) {
                    None => {
                        i += 1;
                    }
                    Some(merged) => {
                        changed = true;
                        span_to_add = merged;
                        self.spans.remove(i);
                        break; // out of i loop
                    }
                }
            }
        }
        self.spans.push(span_to_add);
    }

    /// Adds this beacon to the row.
    fn add_beacon(&mut self, beacon: Num) {
        self.beacons.insert(beacon);
    }

    /// This is given a SensorAndBeacon and records its impact on this row.
    fn record_sensor_and_beacon(&mut self, sab: &SensorAndBeacon) {
        match sab.span_at_row(self.row_num) {
            None => {}
            Some(span) => {
                self.add_span(&span);
                if sab.beacon.1 == self.row_num {
                    self.add_beacon(sab.beacon.0);
                }
            }
        }
    }

    /// Returns the count of locations in this row known to not have a beacon.
    fn count_non_beacons(&self) -> usize {
        let mut count: usize = 0;
        for span in self.spans.iter() {
            count += usize::try_from(span.spaces_covered()).unwrap();
        }
        count -= self.beacons.len(); // don't count the beacons
        count
    }
}




// ======= main() =======

fn part_a(input: &Vec<SensorAndBeacon>) {
    println!("\nPart a:");
    let mut row = Row::new(2000000);
    for sab in input {
        row.record_sensor_and_beacon(sab);
    }
    println!("{:?}", row);
    println!("The count of known non-beacons in row {} is {}", row.row_num, row.count_non_beacons());
}


fn part_b(_input: &Vec<SensorAndBeacon>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}


// ======= Tests =======


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row() {
        let mut row = Row::new(0);
        row.add_span(&Span::new(5,8));
        assert_eq!(row.spans, vec![Span::new(5,8)]);
        row.add_span(&Span::new(11,11));
        assert_eq!(row.spans, vec![Span::new(5,8), Span::new(11,11)]);
        row.add_span(&Span::new(11,13));
        assert_eq!(row.spans, vec![Span::new(5,8), Span::new(11,13)]);
        row.add_span(&Span::new(9,10));
        assert_eq!(row.spans, vec![Span::new(5,13)]);
    }

}
