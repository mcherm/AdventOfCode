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
        scanners.push(Scanner{name, beacons})
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
#[derive(Debug, Copy, Clone)]
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

    #[allow(dead_code)] // FIXME: Remove
    fn index(&self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

impl AxisMapping {
    fn make(maps_to: [Axis;3], flip: [bool;3]) -> Self {
        let maps_back: [Axis;3] = [
            *Axis::all().iter().filter(|v| maps_to[v.index()] == Axis::X).next().unwrap(),
            *Axis::all().iter().filter(|v| maps_to[v.index()] == Axis::Y).next().unwrap(),
            *Axis::all().iter().filter(|v| maps_to[v.index()] == Axis::Z).next().unwrap(),
        ];
        AxisMapping{maps_to, maps_back, flip}
    }

    fn all() -> [AxisMapping;6] {
        [
            AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [false, false, false]),
            AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [false, false, false]),
            AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [false, false, false]),
            AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [false, false, false]),
            AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [false, false, false]),
            AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [false, false, false]),

            // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [false, false,  true]),
            // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [false, false,  true]),
            // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [false, false,  true]),
            // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [false, false,  true]),
            // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [false, false,  true]),
            // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [false, false,  true]),
            //
            // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [false,  true, false]),
            // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [false,  true, false]),
            // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [false,  true, false]),
            // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [false,  true, false]),
            // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [false,  true, false]),
            // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [false,  true, false]),
            //
            // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [false,  true,  true]),
            // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [false,  true,  true]),
            // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [false,  true,  true]),
            // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [false,  true,  true]),
            // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [false,  true,  true]),
            // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [false,  true,  true]),
            //
            // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [ true, false, false]),
            // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [ true, false, false]),
            // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [ true, false, false]),
            // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [ true, false, false]),
            // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [ true, false, false]),
            // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [ true, false, false]),
            //
            // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [ true, false,  true]),
            // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [ true, false,  true]),
            // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [ true, false,  true]),
            // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [ true, false,  true]),
            // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [ true, false,  true]),
            // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [ true, false,  true]),
            //
            // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [ true,  true, false]),
            // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [ true,  true, false]),
            // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [ true,  true, false]),
            // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [ true,  true, false]),
            // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [ true,  true, false]),
            // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [ true,  true, false]),
            //
            // AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [ true,  true,  true]),
            // AxisMapping::make([Axis::X, Axis::Z, Axis::Y], [ true,  true,  true]),
            // AxisMapping::make([Axis::Y, Axis::X, Axis::Z], [ true,  true,  true]),
            // AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [ true,  true,  true]),
            // AxisMapping::make([Axis::Z, Axis::X, Axis::Y], [ true,  true,  true]),
            // AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [ true,  true,  true]),
        ]
    }
}

impl AxisOrient {
}


impl Orient {
    // FIXME: May be overengineered. Add if needed.
    // /// Construct an Orient from a vector with exactly 3 values
    // fn new(orient_vec: Vec<AxisOrient>) -> Self {
    //     assert_eq!(orient_vec.len(), 3);
    //     Orient{orients: [orient_vec[0], orient_vec[1], orient_vec[2]]}
    // }

    /// Returns a new Beacon obtained by applying the orientation.
    #[allow(dead_code)] // FIXME: Remove this eventually
    fn apply(&self, beacon: Beacon) -> Beacon {
        println!("beacon.x = {}", beacon.x);
        println!("self.orients[0].offset = {}", self.orients[0].offset);
        Beacon{
            x: beacon.get(self.orients[0].maps_to) * if self.orients[0].flip {-1} else {1} + self.orients[0].offset,
            y: beacon.get(self.orients[1].maps_to) * if self.orients[0].flip {-1} else {1} + self.orients[1].offset,
            z: beacon.get(self.orients[2].maps_to) * if self.orients[0].flip {-1} else {1} + self.orients[2].offset,
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


/// Given 2 points in s0 and two points in s1 that we believe may be the same points in the same
/// order, this returns a vector (possibly of length 0) of Orients that could make that true.
#[allow(dead_code)] // FIXME: Remove this eventually
fn orient(source_scanner: &Scanner, source_positions: [usize;2], dest_scanner: &Scanner, dest_positions: [usize;2]) -> Vec<Orient> {
    let s0 = source_scanner.beacons[source_positions[0]];
    let s1 = source_scanner.beacons[source_positions[1]];
    let d0 = dest_scanner.beacons[dest_positions[0]];
    let d1 = dest_scanner.beacons[dest_positions[1]];
    assert_eq!(get_length(&s0, &s1), get_length(&d0, &d1));


    // Assuming b0 -> g0 and b1 -> g1
    // Assuming x -> x, y -> y, z -> z
    // Assuming xflip -> false, yflip -> false, z-flip -> false
    // Assuming yoffset -> 0, zoffset -> 0


    // Find offset that works for both with these constraints OR None if there isn't one
    fn find_offset(source_axis: Axis, axis_mapping: AxisMapping, [s0, s1, d0, d1]: [Beacon;4]) -> Option<Coord> {
        println!("find_offset({}, {:?}, ...)", source_axis, axis_mapping); // FIXME: Remove
        let dest_axis = axis_mapping.maps_to[source_axis.index()];
        let _flip = axis_mapping.flip[source_axis.index()]; // FIXME: Incorporate this later
        let offset = s0.get(dest_axis) - d0.get(source_axis);
        println!("  try {} as offset because {} - {}", offset, d0.get(dest_axis), s0.get(source_axis)); // FIXME: Remove
        println!("  compare to {} because {} - {}", (d1.get(dest_axis) - s1.get(source_axis)), d1.get(dest_axis), s1.get(source_axis)); // FIXME: Remove
        if (s1.get(dest_axis) - d1.get(source_axis)) == offset {
            println!("  Found offset {}", offset); // FIXME: Remove
            Some(offset)
        } else {
            println!("  No offset",); // FIXME: Remove
            None
        }
    }


    let mut ret_val: Vec<Orient> = Vec::new();
    for ax_map in AxisMapping::all() {
        println!("Trying axis mapping {:?}", ax_map);
        let x_offset_opt = find_offset(Axis::X, ax_map, [s0, s1, d0, d1]);
        let y_offset_opt = find_offset(Axis::Y, ax_map, [s0, s1, d0, d1]);
        let z_offset_opt = find_offset(Axis::Z, ax_map, [s0, s1, d0, d1]);
        if x_offset_opt.is_some() && y_offset_opt.is_some() && z_offset_opt.is_some() {
            // Found a possible Orient!
            let offsets = [x_offset_opt.unwrap(), y_offset_opt.unwrap(), z_offset_opt.unwrap()];
            println!("Found a possible mapping with offsets {} / {} / {}", offsets[0], offsets[1], offsets[2]);
            let orients: [AxisOrient;3] = [
                AxisOrient{maps_to: ax_map.maps_back[0], flip: ax_map.flip[0], offset: offsets[ax_map.maps_back[0].index()]},
                AxisOrient{maps_to: ax_map.maps_back[1], flip: ax_map.flip[1], offset: offsets[ax_map.maps_back[1].index()]},
                AxisOrient{maps_to: ax_map.maps_back[2], flip: ax_map.flip[2], offset: offsets[ax_map.maps_back[2].index()]},
            ];
            ret_val.push(Orient{orients});
        }
    }

    ret_val
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
    println!("description for s0d0: {:?}", s0.get_point_description(0));
    let shared_uniques = s0.get_lengths().shared_uniques(&s1.get_lengths());
    println!("shared_uniques: {:?}", shared_uniques);
    let unique_length = shared_uniques.lengths[0];
    println!("some unique length: {}", unique_length);
    let [s0d0, s0d1] = s0.descriptions_for_unique_length(unique_length);
    let [s1d0, s1d1] = s1.descriptions_for_unique_length(unique_length);
    println!("s0d0: {:?}", s0d0);
    println!("s0d1: {:?}", s0d1);
    println!("s1d0: {:?}", s1d0);
    println!("s1d1: {:?}", s1d1);
    println!("----------");

    let source_point: Beacon = s0d0.beacon;
    let this_axis: Axis = Axis::X;
    let mut x_axis_orients: Vec<AxisOrient> = Vec::new();
    for dest_point in [s1d0.beacon, s1d1.beacon] {
        for maps_to in Axis::all() {
            for flip in [false, true] {
                let offset: Coord = source_point.get(this_axis) - dest_point.get(maps_to) * (if flip {-1} else {1});
                x_axis_orients.push(AxisOrient{maps_to, flip, offset})
            }
        }
    }
    println!("x_axis_orients = {:?}", x_axis_orients);


    // FIXME: Apparently this whole approach won't work!!!
    // let s0xs = s0d0.x_lengths.union(&s0d1.x_lengths);
    // let s1_axis_lengths = s1d0.all_axis_lengths().union(&s1d1.all_axis_lengths());
    // println!("s0xs: {:?}", s0xs);
    // println!("s1_axis_lengths: {:?}", s1_axis_lengths);
    // let shared_uniques_for_xs = s0xs.shared_uniques(&s1_axis_lengths);
    // // shared_uniques_for_xs is a list of axis values that occur in s0 x_lengths, and ANYWHERE
    // // in the axis lengths for s1, and which is unique within each. Any one of these can be
    // // used to determine where the x axis of s0 maps to in s1.
    // println!("shared_uniques_for_xs: {:?}", shared_uniques_for_xs);
    // assert!(shared_uniques_for_xs.len() != 0); // FIXME: instead of panicking we should back up and try another unique_length
    // let unique_for_x = shared_uniques_for_xs.lengths[0];
    // println!("unique_for_x: {}", unique_for_x);
    // let s0_point_for_x: PointDescription = if s0d0.x_lengths.contains(unique_for_x) {
    //     s0d0
    // } else if s0d1.x_lengths.contains(unique_for_x) {
    //     s0d1
    // } else {
    //     panic!("One of them must have it!")
    // };
    // println!("s0_point_for_x: {:?}", s0_point_for_x);
    // let (s1_point_for_x, map_x_axis_to): (PointDescription, Axis) = if false {
    //     panic!("branch should never happen")
    // } else if s1d0.x_lengths.contains(unique_for_x) { (s1d0, Axis::X)
    // } else if s1d0.y_lengths.contains(unique_for_x) { (s1d0, Axis::Y)
    // } else if s1d0.z_lengths.contains(unique_for_x) { (s1d0, Axis::Z)
    // } else if s1d1.x_lengths.contains(unique_for_x) { (s1d1, Axis::X)
    // } else if s1d1.y_lengths.contains(unique_for_x) { (s1d1, Axis::Y)
    // } else if s1d1.z_lengths.contains(unique_for_x) { (s1d1, Axis::Z)
    // } else {
    //     panic!("One of them must have it!")
    // };
    // println!("s1_point_for_x: {:?}\nmap_x_axis_to: {}", s1_point_for_x, map_x_axis_to);
    // let flip: bool = false; // FIXME: Not right
    // let offset: Coord = 0; // FIXME: Not right
    // let x_axis_orient: AxisOrient = AxisOrient{maps_to: map_x_axis_to, flip, offset};
    // println!("X AxisOrient = {:?}", x_axis_orient);
    println!("----------");
    // let orients: Vec<Orient> = orient(s0, [0,1], s1, [0,1]);
    // assert!(orients.len() == 1);
    // println!("Orient: {:?}", orients[0]);
    // println!("If you remap using orient then ({})-to-({}) becomes ({})-to-({})",
    //          s1.beacons[0], s1.beacons[1], orients[0].apply(s1.beacons[0]), orients[0].apply(s1.beacons[1]));
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
    fn test_orient_2() {
        fn newb(x: Coord, y: Coord, z: Coord) -> Beacon {
            Beacon{x, y, z}
        }
        let s0 = Scanner{name: "Zero".to_string(), beacons: vec![newb(2,3,0), newb(3,0,0)]};
        let s1 = Scanner{name: "One".to_string(),  beacons: vec![newb(0,1,0), newb(1,-2,0)]};
        let orients: Vec<Orient> = orient(&s0, [0,1], &s1, [0,1]);
        println!("{}", orients.len()); // FIXME: Remove
        assert!(orients.len() == 1);
        let or = orients[0];
        assert_eq!(or.orients[0].offset, 2); assert_eq!(or.orients[0].maps_to, Axis::X);
        assert_eq!(or.orients[1].offset, 2); assert_eq!(or.orients[1].maps_to, Axis::Y);
        assert_eq!(or.orients[2].offset, 0); assert_eq!(or.orients[2].maps_to, Axis::Z);
        println!("Orient: {:?}", orients[0]);
    }

    #[test]
    fn test_orient_3() {
        fn newb(x: Coord, y: Coord, z: Coord) -> Beacon {
            Beacon{x, y, z}
        }
        let s0 = Scanner{name: "Zero".to_string(), beacons: vec![newb(2,3,0), newb(3,0,0)]};
        let s1 = Scanner{name: "One".to_string(),  beacons: vec![newb(1,0,0), newb(-2,1,0)]};
        let orients: Vec<Orient> = orient(&s0, [0,1], &s1, [0,1]);
        assert_eq!(orients.len(), 1);
        let or = orients[0];
        assert_eq!(or.orients[0].offset, 2); assert_eq!(or.orients[0].maps_to, Axis::Y);
        assert_eq!(or.orients[1].offset, 2); assert_eq!(or.orients[1].maps_to, Axis::X);
        assert_eq!(or.orients[2].offset, 0); assert_eq!(or.orients[2].maps_to, Axis::Z);
        println!("Orient: {:?}", orients[0]);
    }

    #[test]
    fn test_orient_4() {
        fn newb(x: Coord, y: Coord, z: Coord) -> Beacon {
            Beacon{x, y, z}
        }
        let s0 = Scanner{name: "Zero".to_string(), beacons: vec![newb(0,2,0), newb(2,0,0)]};
        let s1 = Scanner{name: "One".to_string(),  beacons: vec![newb(-1,0,-3), newb(-3,0,-1)]};
        let orients: Vec<Orient> = orient(&s0, [0,1], &s1, [0,1]);
        assert_eq!(orients.len(), 1);
        let or = orients[0];
        println!("Orient: {:?}", orients[0]);
        assert_eq!(or.orients[0].offset, 3); assert_eq!(or.orients[0].maps_to, Axis::Z);
        assert_eq!(or.orients[1].offset, 3); assert_eq!(or.orients[1].maps_to, Axis::X);
        assert_eq!(or.orients[2].offset, 0); assert_eq!(or.orients[2].maps_to, Axis::Y);
    }

    #[test]
    fn test_orient_5() {
        fn newb(x: Coord, y: Coord, z: Coord) -> Beacon {
            Beacon{x, y, z}
        }
        let s0 = Scanner{name: "Zero".to_string(), beacons: vec![newb(0,2,0), newb(2,0,0)]};
        let s1 = Scanner{name: "One".to_string(),  beacons: vec![newb(1,0,-2), newb(-1,0,0)]};
        let orients: Vec<Orient> = orient(&s0, [0,1], &s1, [0,1]);
        assert_eq!(orients.len(), 1);
        let or = orients[0];
        println!("Orient: {:?}", orients[0]);
        assert_eq!(or.orients[0].offset, 2); assert_eq!(or.orients[0].maps_to, Axis::Z);
        assert_eq!(or.orients[1].offset, 1); assert_eq!(or.orients[1].maps_to, Axis::X);
        assert_eq!(or.orients[2].offset, 0); assert_eq!(or.orients[2].maps_to, Axis::Y);
    }

    #[test]
    fn test_orient_6() {
        fn newb(x: Coord, y: Coord, z: Coord) -> Beacon {
            Beacon{x, y, z}
        }
        let s0 = Scanner{name: "Zero".to_string(), beacons: vec![newb(0,2,0), newb(2,0,0)]};
        let s1 = Scanner{name: "One".to_string(),  beacons: vec![newb(1,0,-2), newb(-1,0,0)]};
        let orients: Vec<Orient> = orient(&s0, [0,1], &s1, [0,1]);
        assert_eq!(orients.len(), 1);
        let or = orients[0];
        println!("Orient: {:?}", orients[0]);
        assert_eq!(or.orients[0].offset, 2); assert_eq!(or.orients[0].maps_to, Axis::Z);
        assert_eq!(or.orients[1].offset, 1); assert_eq!(or.orients[1].maps_to, Axis::X);
        assert_eq!(or.orients[2].offset, 0); assert_eq!(or.orients[2].maps_to, Axis::Y);
    }

    #[test]
    fn test_axis_mapping_make() {
        let am = AxisMapping::make([Axis::X, Axis::Y, Axis::Z], [false, false, false]);
        assert_eq!(am.maps_back, [Axis::X, Axis::Y, Axis::Z]);
        let am = AxisMapping::make([Axis::Z, Axis::Y, Axis::X], [false, false, false]);
        assert_eq!(am.maps_back, [Axis::Z, Axis::Y, Axis::X]);
        let am = AxisMapping::make([Axis::Y, Axis::Z, Axis::X], [false, false, false]);
        assert_eq!(am.maps_back, [Axis::Z, Axis::X, Axis::Y]);
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
