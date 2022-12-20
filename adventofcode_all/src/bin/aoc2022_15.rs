
extern crate anyhow;

use std::fs;
use nom;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::line_ending,
};
use nom::character::complete::i64 as nom_i64;
use std::fmt::{Display, Formatter};
use std::cmp::{min, max};
use std::collections::{BTreeSet, BTreeMap, HashSet};


// ======= Parsing =======

fn input() -> Result<Vec<SensorAndBeacon>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_15.txt")?;
    match SensorAndBeacon::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type Num = i64;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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
                nom_i64,
                tag(", y="),
                nom_i64,
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

    /// Returns the list of 4 diagonals that are just OUTSIDE the radius -- the only
    /// ones that could contain the sought-after location. Does not include the corners.
    fn bounding_diags(&self) -> [DiagSpan; 4] {
        let Point(sx, sy) = self.sensor;
        let r = self.radius();
        let center_sum = sx + sy;
        let center_dif = sx - sy;
        [
            DiagSpan::new(Diagonal::PosSlope(center_sum + (r + 1)), Point(sx + 1, sy + r), Point(sx + r, sy + 1), FillDirection::Left),
            DiagSpan::new(Diagonal::PosSlope(center_sum - (r + 1)), Point(sx - r, sy - 1), Point(sx - 1, sy - r), FillDirection::Right),
            DiagSpan::new(Diagonal::NegSlope(center_dif - (r + 1)), Point(sx - r, sy + 1), Point(sx - 1, sy + r), FillDirection::Right),
            DiagSpan::new(Diagonal::NegSlope(center_dif + (r + 1)), Point(sx + 1, sy - r), Point(sx + r, sy - 1), FillDirection::Left),
        ]
    }
}


// ======= Part 1 Compute =======

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

// ======= Part 2 Compute =======

/// Represents a specific diagonal "row".
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Diagonal {
    PosSlope(Num), // goes SW<->NE; x + y is a fixed value
    NegSlope(Num), // goes NW<->SE; x - y is a fixed value
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum FillDirection {Left, Right}

/// Represents a span in a particular diagonal.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct DiagSpan {
    diag: Diagonal,
    x_span: Span,
    fill: FillDirection,
}

/// Represents a collection of DiagSpans.
#[derive(Debug)]
struct DiagSpanCollection {
    pos_spans: BTreeMap<Num,Vec<DiagSpan>>,
    neg_spans: BTreeMap<Num,Vec<DiagSpan>>,
}


impl Diagonal {
    /// Returns true if the point is on this diagonal; false if not.
    fn contains(&self, p: Point) -> bool {
        match self {
            Diagonal::PosSlope(n) => p.0 + p.1 == *n,
            Diagonal::NegSlope(n) => p.0 - p.1 == *n,
        }
    }

    /// Returns the point along this Diagonal that has the given x value.
    fn point_at_x(&self, x_value: Num) -> Point {
        match self {
            Diagonal::PosSlope(n) => Point(x_value, *n - x_value),
            Diagonal::NegSlope(n) => Point(x_value, x_value - *n),
        }
    }
}

impl DiagSpan {
    /// Constructs a DiagSpan. start and end can be in any order.
    fn new(diag: Diagonal, start: Point, end: Point, fill: FillDirection) -> Self {
        assert!(diag.contains(start) && diag.contains(end));
        let x_span = Span::new(min(start.0, end.0), max(start.0, end.0));
        DiagSpan{diag, x_span, fill}
    }


    /// If these two spans overlap at all, this returns a span containing the overlap;
    /// if not, it returns None. This may ONLY be called on spans that have the same
    /// Diagonal, or this will panic. The span returned will always have the FillDirection
    /// of self, regardless of other.
    fn intersect(&self, other: &Self) -> Option<Self> {
        assert!(self.diag == other.diag);
        if self.x_span.start <= other.x_span.end && self.x_span.end >= other.x_span.start {
            let start_x = max(self.x_span.start, other.x_span.start);
            let end_x = min(self.x_span.end, other.x_span.end);
            let start_p = self.diag.point_at_x(start_x);
            let end_p = self.diag.point_at_x(end_x);
            let diag_span = DiagSpan::new(self.diag, start_p, end_p, self.fill);
            Some(diag_span)
        } else {
            None
        }
    }

    /// Iterates
    fn iter_points(&self) -> impl Iterator<Item=Point> {
        DiagSpanPointIter{diag: self.diag, next_x: self.x_span.start, max_x: self.x_span.end}
    }
}

struct DiagSpanPointIter {
    diag: Diagonal,
    next_x: Num,
    max_x: Num,
}
impl<'a> Iterator for DiagSpanPointIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_x > self.max_x {
            None
        } else {
            let x = self.next_x;
            self.next_x += 1;
            Some(self.diag.point_at_x(x))
        }
    }
}


impl DiagSpanCollection {
    fn new() -> Self {
        DiagSpanCollection{pos_spans: BTreeMap::new(), neg_spans: BTreeMap::new()}
    }

    fn add_span(&mut self, dspan: DiagSpan) {
        match dspan {
            DiagSpan{diag: Diagonal::PosSlope(n), ..} => {
                self.pos_spans.entry(n).or_default().push(dspan);
            }
            DiagSpan{diag: Diagonal::NegSlope(n), ..} => {
                self.neg_spans.entry(n).or_default().push(dspan);
            }
        }
    }

    /// We can have a lone point ONLY if it lies along two PosSlope spans, one with
    /// fill=Left and one with fill=Right and ALSO lies along two NegSlope spans that
    /// also face opposite directions. This method exhaustively finds all such points
    /// and returns a list of them.
    ///
    /// NOTE: It does NOT find points along the edge of the region, which could (in
    /// theory) satisfy the requirements. It also does not enforce that the point
    /// must be in the region.
    fn lone_points(&self) -> Vec<Point> {
        fn matches_from_span_map(span_map: &BTreeMap<Num,Vec<DiagSpan>>) -> Vec<DiagSpan> {
            let mut answer = Vec::new();
            for (_num, spans) in span_map.iter() {
                let (left_spans, right_spans): (Vec<&DiagSpan>, Vec<&DiagSpan>)
                    = spans.iter().partition(|x| x.fill == FillDirection::Left);
                for a_span in &left_spans {
                    for b_span in &right_spans {
                        match a_span.intersect(b_span) {
                            None => {},
                            Some(span) => {
                                answer.push(span);
                            },
                        }
                    }
                }
            }
            answer
        }
        let pos_hits: HashSet<Point> = matches_from_span_map(&self.pos_spans).iter().flat_map(|x| x.iter_points()).collect();
        let neg_hits: HashSet<Point> = matches_from_span_map(&self.neg_spans).iter().flat_map(|x| x.iter_points()).collect();
        let hits = pos_hits.intersection(&neg_hits);
        hits.map(|x| *x).collect()
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


fn part_b(input: &Vec<SensorAndBeacon>) {
    println!("\nPart b:");
    let mut diag_spans = DiagSpanCollection::new();
    for sab in input {
        for d_span in sab.bounding_diags() {
            diag_spans.add_span(d_span);
        }
    }
    let lone_points = diag_spans.lone_points();
    for p in lone_points {
        if p.0 >= 0 && p.0 <= 4000000 && p.1 >= 0 && p.1 <= 4000000 {
            let tuning_frequency = p.0 * 4000000 + p.1;
            println!("A possible answer is point {} with frequency {}", p, tuning_frequency);
        }
    }
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
