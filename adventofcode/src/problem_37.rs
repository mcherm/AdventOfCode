use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;


/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    InvalidNameLine,
    InvalidBeaconLine,
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl From<std::num::ParseIntError> for InputError {
    fn from(error: std::num::ParseIntError) -> Self {
        InputError::BadInt(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err) => write!(f, "{}", err),
            InputError::BadInt(err) => write!(f, "{}", err),
            InputError::InvalidNameLine => write!(f, "Invalid name line"),
            InputError::InvalidBeaconLine => write!(f, "Invalid beacon line"),
        }
    }
}



/// Read in the input file.
fn read_beacon_file() -> Result<Vec<Scanner>, InputError> {
    let filename = "data/2021/day/19/input.txt";
    let name_line_regex = Regex::new(
        r"^--- (.+) ---$"
    ).unwrap();
    let beacon_line_regex = Regex::new(
        r"^(-?\d+),(-?\d+),(-?\d+)$"
    ).unwrap();

    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();
    let mut scanners: Vec<Scanner> = Vec::new();
    while let Some(name_line) = lines.next() {
        let name_text = name_line?;
        let name_capture = name_line_regex.captures(&name_text).ok_or(InputError::InvalidNameLine)?;
        let name: String = name_capture.get(1).unwrap().as_str().to_string();
        let mut beacons: Vec<Beacon> = Vec::new();
        loop {
            if let Some(line) = lines.next() {
                let text: String = line?;
                if text.len() == 0 {
                    break; // blank lines end the beacons
                }
                let beacon_capture = beacon_line_regex.captures(&text).ok_or(InputError::InvalidBeaconLine)?;
                let x: Coord = beacon_capture.get(1).unwrap().as_str().parse()?;
                let y: Coord = beacon_capture.get(2).unwrap().as_str().parse()?;
                let z: Coord = beacon_capture.get(3).unwrap().as_str().parse()?;
                let beacon: Beacon = Beacon{x,y,z};
                beacons.push(beacon);
            } else {
                break; // Out of lines ends the file
            }
        }
        scanners.push(Scanner::new(name, beacons));
    }
    Ok(scanners)
}


type Coord = i32;
type LenSq = u32;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Beacon {
    x: Coord,
    y: Coord,
    z: Coord
}

struct Scanner {
    name: String,
    beacons: Vec<Beacon>
}

#[derive(Debug)]
struct LengthSet {
    lengths: Vec<LenSq>
}

/// Information helping to uniquely identify a point within a scanner.
#[derive(Debug)]
struct PointDescription {
    scanner_name: String,
    pos: usize,
    beacon: Beacon,
    lengths: LengthSet,
    x_lengths: LengthSet,
    y_lengths: LengthSet,
    z_lengths: LengthSet,
}


/// A direction.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Axis {X, Y, Z}

/// Substructure of Orient
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct AxisOrient {
    maps_to: Axis, // tells, for whatever axis of scanner_0 we are determining, which axis of scanner_1 to use
    flip: bool,  // if true, then scanner_1 values should be multiplied by -1
    offset: Coord, // add this to a scanner_1 value to get the scanner_0 value.
}

/// Specification of how to orient one scanner relative to another.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Orient {
    orients: [AxisOrient; 3]
}


/// Represents a possible mapping of source axis to dest axis
#[derive(Debug, Copy, Clone)]
struct AxisMapping {
    maps_to: [Axis;3],
    maps_back: [Axis;3],
    flip: [bool;3],
}

// FIXME: Maybe later? may be overengineered
// struct PointRef<'a> {
//     scanner: &'a Scanner,
//     pos: usize,
// }

// FIXME: Maybe later? may be overengineered
// struct PointRefPair<'a> {
//     a: PointRef<'a>,
//     b: PointRef<'a>,
//     dist: LenSq,
// }

// FIXME: Maybe later? may be overengineered
// /// A set of point pairs from one scanner
// #[derive(Debug)]
// struct PairSet<'a> {
// }


// FIXME: Maybe later? may be overengineered
// /// Represents a pair of points from two different Scanners
// /// that might match.
// struct Pair<'a> {
//     scanner_1: &'a Scanner,
//     scanner_2: &'a Scanner,
//     s1_positions: [usize;2],
//     s2_positions: [usize;2],
// }



fn squared(c: Coord) -> LenSq {
    LenSq::try_from(c * c).unwrap()
}

// Returns the length squared between the two points
fn get_length(b1: &Beacon, b2: &Beacon) -> LenSq {
    squared(b1.x - b2.x) + squared(b1.y - b2.y) + squared(b1.z - b2.z)
}



impl Beacon {
    fn get(&self, axis: Axis) -> Coord {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }
}

impl Scanner {
    /// Constructor which verifies the Beacons are unique. If one is a
    /// duplicate it is simply skipped over.
    fn new(name: String, nonunique_beacons: Vec<Beacon>) -> Self {
        let mut beacons = Vec::new();
        for b in &nonunique_beacons {
            if !beacons.contains(b) {
                beacons.push(*b);
            }
        }
        Scanner{name, beacons}
    }

    // Returns a sorted list of the squares of the distances between pairs of points.
    fn get_lengths(&self) -> LengthSet {
        let mut lengths: Vec<LenSq> = Vec::new();
        for (i,b1) in self.beacons.iter().enumerate() {
            for b2 in self.beacons[(i+1)..].iter() {
                lengths.push(get_length(&b1, &b2));
            }
        }
        LengthSet::new(lengths)
    }

    /// Given the index of a beacon, returns the PointDescription for that beacon.
    fn get_point_description(&self, pos: usize) -> PointDescription {
        let beacon: Beacon = self.beacons[pos];
        let mut lengths: Vec<LenSq> = Vec::new();
        let mut x_lengths: Vec<LenSq> = Vec::new();
        let mut y_lengths: Vec<LenSq> = Vec::new();
        let mut z_lengths: Vec<LenSq> = Vec::new();
        for (p, other_beacon) in self.beacons.iter().enumerate() {
            if p != pos {
                lengths.push(get_length(&beacon, other_beacon));
                x_lengths.push(squared(beacon.x));
                y_lengths.push(squared(beacon.y));
                z_lengths.push(squared(beacon.z));
            }
        }
        PointDescription{
            scanner_name: self.name.clone(),
            pos,
            beacon,
            lengths: LengthSet::new(lengths),
            x_lengths: LengthSet::new(x_lengths),
            y_lengths: LengthSet::new(y_lengths),
            z_lengths: LengthSet::new(z_lengths),
        }
    }

    /// Returns the list of PointDescriptions for points that include this length
    /// as one of their lengths. The length MUST be unique, which is why this can assume
    /// it will  always return exactly 2 points.
    fn descriptions_for_unique_length(&self, length: LenSq) -> [PointDescription;2] {
        for (pos, beacon) in self.beacons.iter().enumerate() {
            for (other_pos, other_beacon) in self.beacons.iter().enumerate() {
                if get_length(beacon, other_beacon) == length {
                    return [self.get_point_description(pos), self.get_point_description(other_pos)];
                }
            }
        }
        panic!("To reach here the length wasn't in the scanner.");
    }


    /// This is given another scanner and the way to orient that one relative to
    /// this. It returns a new scanner that contains (a single copy of) each Beacon
    /// in both Scanners. The name will be "$1 & $2".
    fn merge_with(&self, other: &Self, orient: Orient) -> Self {
        let name: String = format!("{} & {}", self.name, other.name);
        let mut beacons = Vec::new();
        beacons.extend(&self.beacons);
        for b in &other.beacons {
            beacons.push(orient.apply(*b));
        }
        Self::new(name, beacons)
    }
}
impl PartialEq for Scanner {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Scanner {}


impl Axis {
    #[allow(dead_code)] // FIXME: Remove
    fn all() -> [Axis;3] {
        [Axis::X, Axis::Y, Axis::Z]
    }

    /// Returns the other axes, besides this one.
    fn others(&self) -> [Axis;2] {
        match self {
            Axis::X => [Axis::Y, Axis::Z],
            Axis::Y => [Axis::X, Axis::Z],
            Axis::Z => [Axis::X, Axis::Y],
        }
    }

    // This is passed 2 axes which MUST be different. It returns the third one.
    fn remaining(a: Self, b: Self) -> Self {
        assert_ne!(a, b);
        for v in Axis::all() {
            if v != a && v != b {
                return v;
            }
        }
        panic!("With only 3 options, we must have found one.")
    }

    #[allow(dead_code)] // FIXME: Remove
    fn index(&self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

// FIXME: Remove if not needed
// impl AxisMapping {
//     fn make(maps_to: [Axis;3], flip: [bool;3]) -> Self {
//         let maps_back: [Axis;3] = [
//             *Axis::all().iter().filter(|v| maps_to[v.index()] == Axis::X).next().unwrap(),
//             *Axis::all().iter().filter(|v| maps_to[v.index()] == Axis::Y).next().unwrap(),
//             *Axis::all().iter().filter(|v| maps_to[v.index()] == Axis::Z).next().unwrap(),
//         ];
//         AxisMapping{maps_to, maps_back, flip}
//     }
//
//     fn all() -> [AxisMapping;6] {
//         [
//             AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [false, false, false]),
//             AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [false, false, false]),
//             AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [false, false, false]),
//             AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [false, false, false]),
//             AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [false, false, false]),
//             AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [false, false, false]),
//
//             // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [false, false,  true]),
//             // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [false, false,  true]),
//             // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [false, false,  true]),
//             // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [false, false,  true]),
//             // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [false, false,  true]),
//             // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [false, false,  true]),
//             //
//             // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [false,  true, false]),
//             // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [false,  true, false]),
//             // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [false,  true, false]),
//             // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [false,  true, false]),
//             // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [false,  true, false]),
//             // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [false,  true, false]),
//             //
//             // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [false,  true,  true]),
//             // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [false,  true,  true]),
//             // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [false,  true,  true]),
//             // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [false,  true,  true]),
//             // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [false,  true,  true]),
//             // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [false,  true,  true]),
//             //
//             // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [ true, false, false]),
//             // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [ true, false, false]),
//             // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [ true, false, false]),
//             // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [ true, false, false]),
//             // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [ true, false, false]),
//             // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [ true, false, false]),
//             //
//             // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [ true, false,  true]),
//             // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [ true, false,  true]),
//             // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [ true, false,  true]),
//             // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [ true, false,  true]),
//             // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [ true, false,  true]),
//             // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [ true, false,  true]),
//             //
//             // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [ true,  true, false]),
//             // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [ true,  true, false]),
//             // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [ true,  true, false]),
//             // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [ true,  true, false]),
//             // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [ true,  true, false]),
//             // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [ true,  true, false]),
//             //
//             // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [ true,  true,  true]),
//             // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [ true,  true,  true]),
//             // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [ true,  true,  true]),
//             // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [ true,  true,  true]),
//             // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [ true,  true,  true]),
//             // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [ true,  true,  true]),
//         ]
//     }
// }

impl AxisOrient {
    /// Returns a new Coord obtained by applying this to the given beacon
    fn apply(&self, beacon: Beacon) -> Coord {
        beacon.get(self.maps_to) * if self.flip {-1} else {1} + self.offset
    }
}


impl Orient {
    // FIXME: May be overengineered. Add if needed.
    // /// Construct an Orient from a vector with exactly 3 values
    // fn new(orient_vec: Vec<AxisOrient>) -> Self {
    //     assert_eq!(orient_vec.len(), 3);
    //     Orient{orients: [orient_vec[0], orient_vec[1], orient_vec[2]]}
    // }

    /// Returns a new Beacon obtained by applying the orientation.
    fn apply(&self, beacon: Beacon) -> Beacon {
        Beacon{
            x: self.orients[Axis::X.index()].apply(beacon),
            y: self.orients[Axis::Y.index()].apply(beacon),
            z: self.orients[Axis::Z.index()].apply(beacon),
        }
    }
}


// FIXME: Maybe later? may be overengineered
// impl<'a> Pair<'a> {
//     fn new(
//         scanner_1: &'a Scanner,
//         s1_positions: [usize;2],
//         scanner_2: &'a Scanner,
//         s2_positions: [usize;2],
//     ) -> Self {
//         assert_eq!(
//             get_length(&scanner_1.beacons[s1_positions[0]], &scanner_1.beacons[s1_positions[1]]),
//             get_length(&scanner_2.beacons[s2_positions[0]], &scanner_2.beacons[s2_positions[1]])
//         );
//         Pair{scanner_1, scanner_2, s1_positions, s2_positions}
//     }
// }


// FIXME: Maybe later? may be overengineered
// impl<'a> PointRef<'a> {
//     fn get_beacon(&self) -> &Beacon {
//         &self.scanner.beacons[self.pos]
//     }
// }

// FIXME: Maybe later? may be overengineered
// impl<'a> PointRefPair {
//     fn new(a: PointRef<'a>, b: PointRef<'a>) -> Self {
//         let dist = get_length(a.get_beacon(), b.get_beacon());
//         PointRefPair(a, b, dist)
//     }
// }


impl LengthSet {
    fn new(mut lengths: Vec<LenSq>) -> Self {
        lengths.sort();
        LengthSet{lengths}
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.lengths.len()
    }

    #[allow(dead_code)] // FIXME: Remove
    fn has_dupes(&self) -> bool {
        for p in 1..self.lengths.len() {
            if self.lengths[p] == self.lengths[p-1] {
                return true;
            }
        }
        false
    }

    // finds number of matches between 2 lengthsets
    fn overlaps(&self, other: &Self) -> u32 {
        // Note: Could exploit sorting to be more efficient if needed.
        let mut count: u32 = 0;
        for len_1 in &self.lengths {
            for len_2 in &other.lengths {
                if len_1 == len_2 {
                    count += 1;
                }
            }
        }
        count
    }

    /// Returns a new LengthSet containing only the elements of this one that occur
    /// precisely once.
    fn uniques(&self) -> Self {
        // NOTE: Can be made more efficient if desired using the sorted property
        let counts: Vec<usize> = self.lengths.iter().map(|x| {
            self.lengths.iter().filter(|v| *v == x).count()
        }).collect();
        let mut uniques: Vec<LenSq> = Vec::new();
        for (i, val) in self.lengths.iter().enumerate() {
            if counts[i] == 1 {
                uniques.push(*val)
            }
        }
        return LengthSet::new(uniques)
    }

    /// Returns a new LengthSet that only has values that appear in both self and other.
    fn intersect(&self, other: &Self) -> Self {
        let mut values: Vec<LenSq> = Vec::new();
        for val in &self.lengths {
            if other.lengths.contains(&val) {
                values.push(*val);
            }
        }
        return LengthSet::new(values)
    }

    #[allow(dead_code)]
    fn contains(&self, val: LenSq) -> bool {
        return self.lengths.contains(&val)
    }

    /// Returns a new LengthSet which has all values present in either self or other.
    fn union(&self, other: &Self) -> Self {
        let mut values: Vec<LenSq> = Vec::new();
        for val in &self.lengths {
            values.push(*val);
        }
        for val in &other.lengths {
            values.push(*val);
        }
        return LengthSet::new(values)
    }

    /// Given another LengthSet, finds a LengthSet of the lengths that are unique within
    /// each LengthSet but also in common between them. In principle, there might not
    /// be any, and it will return an empty LengthSet.
    fn shared_uniques(&self, other: &LengthSet) -> LengthSet {
        self.uniques().intersect(&other.uniques())
    }

}


impl PointDescription {
    /// Returns a LengthSet of ALL "axis lengths" (x_lengths, y_lengths, z_lengths).
    #[allow(dead_code)] // FIXME: Remove
    fn all_axis_lengths(&self) -> LengthSet {
        self.x_lengths.union(&self.y_lengths).union(&self.z_lengths)
    }
}



impl fmt::Display for Beacon {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}
impl fmt::Display for Scanner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "--- {} ---", self.name)?;
        for beacon in &self.beacons {
            writeln!(f, "{}", beacon)?;
        }
        writeln!(f)
    }
}
impl fmt::Display for Axis {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        })
    }
}
impl fmt::Display for AxisOrient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(maps to {} {}, then add {})",
            self.maps_to,
            if self.flip {"flipped"} else {"forward"},
            self.offset,
        )
    }
}
impl fmt::Display for Orient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[x: {} y: {} z: {}]", self.orients[0], self.orients[1], self.orients[2])
    }
}



// FIXME: I tried to create the below, but I just can't figure out how to pass an iterator
// /// This is given an iterable over LengthSets and values, along with a single LenSq. It expects
// /// to find that LenSq in EXACTLY one of the LengthSets (and panics if that isn't true). It
// /// returns the corresponding value for the LengthSet that contained the LenSq.
// fn find_only_containing<'a, I>(
//     len_sq: LenSq,
//     set_value_pairs: impl Iterator<Item=(&'a LengthSet, &'a PointDescription)>
// ) -> &'a PointDescription
//     where
//         I: IntoIterator,
//         I::Item: &'a (&'a LengthSet, &'a PointDescription),
// {
//     let mut answer: Option<&PointDescription> = None;
//     for (length_set, val) in set_value_pairs.into_iter() {
//         if length_set.contains(len_sq) {
//             match answer {
//                 None => answer = Some(val),
//                 Some(_) => panic!("Multiple LengthSets contained the value.")
//             }
//         }
//     }
//     match answer {
//         Some(val) => return val,
//         None => panic!("No LengthSets contained the value."),
//     }
// }


/// Given an axis in the source, the two source beacons, the two dest beacons, and a few maps we're allowed to use,
/// this returns a Vec of the possible AxisOrients for that axis of the source which will successfully
/// map these two endpoints.
fn get_possible_orients(this_axis: Axis, source_beacons: [Beacon;2], dest_beacons: [Beacon;2], allowed_maps: Vec<Axis>) -> Vec<AxisOrient> {
    // println!("get_possible_orients({})", this_axis); // FIXME: May need this again
    let [source_point, other_source_point] = source_beacons;
    let mut axis_orients: Vec<AxisOrient> = Vec::new();
    for (dest_point, other_dest_point) in [(dest_beacons[0], dest_beacons[1]), (dest_beacons[1], dest_beacons[0])] {
        for maps_to in allowed_maps.iter().cloned() {
            for flip in [false, true] {
                let offset: Coord = source_point.get(this_axis) - dest_point.get(maps_to) * (if flip {-1} else {1});
                let axis_orient = AxisOrient{maps_to, flip, offset};
                // println!("  does {} = {}? if so, {} to {} works.", axis_orient.apply(other_dest_point), other_source_point.get(this_axis), dest_point, axis_orient); // FIXME: May need this again
                if axis_orient.apply(other_dest_point) == other_source_point.get(this_axis) {
                    // println!("  YES"); // FIXME: May need this again
                    assert_eq!(axis_orient.apply(dest_point), source_point.get(this_axis)); // the other point's Y works
                    // The mapping works on x for both ends of this line
                    axis_orients.push(axis_orient)
                }
            }
        }
    }
    axis_orients
}

fn orients_for_segment(source_points: [Beacon;2], dest_points: [Beacon;2]) -> Vec<Orient> {
    // --- determine possible x_axis_orients ---
    let x_axis_orients = get_possible_orients(
        Axis::X, // first work out the X axis options
        source_points, // the source points in order
        dest_points, // will try using them in either order
        Axis::all().to_vec(), // all axes can work for x
    );

    // --- determine possible (x_axis_orient y_axis_orient) pairs ---
    let mut xy_axis_orients: Vec<[AxisOrient;2]> = Vec::new();
    for x_axis_orient in &x_axis_orients {
        let y_axis_orients = get_possible_orients(
            Axis::Y, // this time, work out the Y axis options
            source_points, // the source points in order
            dest_points, // will try using them in either order
            x_axis_orient.maps_to.others().to_vec(), // just the axes we haven't used yet
        );
        for y_axis_orient in y_axis_orients {
            xy_axis_orients.push([*x_axis_orient, y_axis_orient]);
        }
    }

    // --- determine possible (x_axis_orient, y_axis_orient, z_axis_orient) triples ---
    let mut xyz_axis_orients: Vec<[AxisOrient;3]> = Vec::new();
    for [x_axis_orient, y_axis_orient] in &xy_axis_orients {
        let z_axis_orients = get_possible_orients(
            Axis::Z, // this time, work out the Z axis options
            source_points, // the source points in order
            dest_points, // will try using them in either order
            [Axis::remaining(x_axis_orient.maps_to, y_axis_orient.maps_to)].to_vec(), // just the one remaining axis we haven't used yet
        );
        for z_axis_orient in z_axis_orients {
            xyz_axis_orients.push([*x_axis_orient, *y_axis_orient, z_axis_orient])
        }
    }
    let possible_orients: Vec<Orient> = xyz_axis_orients.iter().map(|ors| Orient{orients: *ors}).collect();

    // --- Eliminate any that don't map these two points successfully ---
    let mut orients: Vec<Orient> = Vec::new();
    for po in possible_orients {
        if po.apply(dest_points[0]) == source_points[0] && po.apply(dest_points[1]) == source_points[1] ||
            po.apply(dest_points[1]) == source_points[0] && po.apply(dest_points[0]) == source_points[1]
        {
            orients.push(po);
        }
    }
    orients
}


fn orients_for_unique_seg_length(source: &Scanner, dest: &Scanner, unique_length: LenSq) -> Vec<Orient> {
    let source_descs = source.descriptions_for_unique_length(unique_length);
    let dest_descs = dest.descriptions_for_unique_length(unique_length);
    let orients = orients_for_segment(
        [source_descs[0].beacon, source_descs[1].beacon],
        [dest_descs[0].beacon, dest_descs[1].beacon],
    );
    orients
}


fn merge_overlapping_scanners(source: &Scanner, dest: &Scanner) -> Scanner {
    let shared_uniques = source.get_lengths().shared_uniques(&dest.get_lengths());
    println!("shared_uniques: {:?}", shared_uniques);

    let mut unique_lengths = shared_uniques.lengths.iter();
    let first_unique_length = unique_lengths.next().expect("There were no unique lengths");
    let mut orients = orients_for_unique_seg_length(source, dest, *first_unique_length);
    println!("The {} orients are {:?}", orients.len(), orients);
    for orient in &orients {
        println!("  Orient: {}", orient);
    }

    for next_unique_length in unique_lengths {
        let new_orients = orients_for_unique_seg_length(source, dest, *next_unique_length);
        println!("The NEXT {} orients are {:?}", orients.len(), orients);
        orients.retain(|orient| new_orients.contains(orient));
        println!("After filtering, we have:");
        for orient in &orients {
            println!("  NEXT Orient: {}", orient);
        }
        if orients.len() <= 1 {
            break; // once we have just one, we can quit.
        }
    }
    source.merge_with(dest, orients[0])
}




fn run() -> Result<(),InputError> {
    let scanners = read_beacon_file()?;
    for (i, scanner1) in scanners.iter().enumerate() {
        for scanner2 in &scanners[(i+1)..] {
            let overlaps = scanner1.get_lengths().overlaps(&scanner2.get_lengths());
            println!("{} to {}: {}", scanner1.name, scanner2.name, overlaps);
        }
    }
    println!("----------");
    let s0: &Scanner = &scanners[0];
    let s1: &Scanner = &scanners[1];

    let merged_scanner = merge_overlapping_scanners(s0, s1);
    println!("merged_scanner: {}", merged_scanner);

    Ok(())
}


pub fn main() {
    match run() {
        Ok(()) => {
            println!("Done");
        },
        Err(err) => println!("Error: {}", err),
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file() {
        let _ = read_beacon_file();
    }


    #[test]
    fn test_axis_orient_apply() {
        assert_eq!(AxisOrient{maps_to: Axis::X, flip: false, offset: 0}.apply(Beacon{x: 27, y: 0, z: 0}), 27);
        assert_eq!(AxisOrient{maps_to: Axis::X, flip: false, offset: 2}.apply(Beacon{x: -1, y: 1, z: 0}),  1);
        assert_eq!(AxisOrient{maps_to: Axis::Y, flip: false, offset: 2}.apply(Beacon{x: -1, y: 1, z: 0}),  3);
        assert_eq!(AxisOrient{maps_to: Axis::Y, flip: true,  offset: 2}.apply(Beacon{x: -1, y: 1, z: 0}),  1);
        assert_eq!(AxisOrient{maps_to: Axis::Z, flip: true,  offset: 5}.apply(Beacon{x:  1, y: 3, z: 5}),  0);
    }

    #[test]
    fn test_orient_apply() {
        let or: Orient = Orient{orients: [
            AxisOrient{maps_to: Axis::Y, flip: false, offset: 1},
            AxisOrient{maps_to: Axis::X, flip: false, offset: 2},
            AxisOrient{maps_to: Axis::Z, flip: false, offset: 3},
        ]};
        let b = or.apply(Beacon{x: 100, y: 200, z: 300});
        println!("b: {}", b);
        assert_eq!(b, Beacon{x: 201, y: 102, z: 303});
    }
}
