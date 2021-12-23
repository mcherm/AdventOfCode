use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader};
use rand::RngCore;
use regex::Regex;
use rand::seq::SliceRandom;
use std::collections::HashMap;


const USE_SHUFFLE: bool = false;
const MIN_OVERLAPS_FOR_MATCH: usize = 12;
const UNIQUENESS_LEVEL: usize = 1;


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
        r"^--- scanner (.+) ---$"
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
        let name: String = format!("s{}", name_capture.get(1).unwrap().as_str().to_string());
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
        scanners.push(Scanner::new(name, beacons, None));
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

#[derive(Debug, Clone)]
struct Scanner {
    pub name: String,
    pub beacons: Vec<Beacon>,
    scanner_centers: Vec<Beacon>,
    length_set: LengthSet,
    segments_by_length: HashMap<LenSq,Vec<[Beacon;2]>>,
}

#[derive(Debug, Clone)]
struct LengthSet {
    lengths: Vec<LenSq>
}

/// Information helping to uniquely identify a point within a scanner.
#[derive(Debug, Clone)]
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

#[derive(Debug, Copy, Clone)]
struct Overlap {
    pos_1: usize,
    pos_2: usize,
    overlap_count: i32,
}

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

    fn manhattan_distance(&self, other: &Self) -> Coord {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }
}

impl Scanner {
    /// Constructor which verifies the Beacons are unique. If one is a
    /// duplicate it is simply skipped over.
    fn new(name: String, nonunique_beacons: Vec<Beacon>, scanners: Option<Vec<Beacon>>) -> Self {
        // --- collect copies of all beacons (only 1 copy if there are dupes) ---
        let mut beacons = Vec::new();
        for b in &nonunique_beacons {
            if !beacons.contains(b) {
                beacons.push(*b);
            }
        }

        // --- Make the list of scanner_centers ---
        let scanner_centers: Vec<Beacon>;
        match scanners {
            None => {
                scanner_centers = vec![Beacon{x:0, y:0, z:0}];
            },
            Some(extras) => {
                scanner_centers = extras;
            },
        }

        // --- pre-calculate the LengthSet for this Scanner ---
        let mut lengths: Vec<LenSq> = Vec::new();
        for (i,b1) in beacons.iter().enumerate() {
            for b2 in beacons[(i+1)..].iter() {
                lengths.push(get_length(&b1, &b2));
            }
        }
        let length_set = LengthSet::new(lengths);

        // --- pre-calculate the segments_by_length ---
        let mut segments_by_length: HashMap<LenSq,Vec<[Beacon;2]>> = HashMap::new();
        for (pos, beacon) in beacons.iter().enumerate() {
            for other_beacon in beacons[..pos].iter() {
                let length = get_length(beacon, other_beacon);
                if !segments_by_length.contains_key(&length) {
                    segments_by_length.insert(length, Vec::new());
                }
                let current_segments_for_this_length: &mut Vec<[Beacon;2]> = segments_by_length.get_mut(&length).unwrap();
                current_segments_for_this_length.push([beacon.clone(), other_beacon.clone()])
            }
        }

        // --- create the new struct ---
        Scanner{name, beacons, scanner_centers, length_set, segments_by_length}
    }

    /// Returns a count of the Beacons
    fn len(&self) -> usize {
        self.beacons.len()
    }

    /// Returns a LengthSet of the distances between pairs of points.
    fn get_lengths(&self) -> &LengthSet {
        &self.length_set
    }

    /// Returns the list of pairs of Beacons in this Scanner which form segments of the
    /// given length. If there IS none of that length, it panics. (I'd rather have it
    /// return an empty Vec in that case, but I don't know how.)
    fn segments_for_length(&self, length: LenSq) -> &Vec<[Beacon;2]> {
        self.segments_by_length.get(&length).unwrap()
    }


    /// This is given another scanner and the way to orient that one relative to
    /// this. It returns a new scanner that contains (a single copy of) each Beacon
    /// in both Scanners. The name will be "$1+$2".
    fn merge_with(&self, other: &Self, orient: Orient) -> Self {
        let name: String = format!("{}+{}", self.name, other.name);
        let mut beacons = Vec::new();
        beacons.extend(&self.beacons);
        for b in &other.beacons {
            beacons.push(orient.apply(b));
        }
        let mut scanners = Vec::new();
        scanners.extend(self.scanner_centers.iter().cloned());
        scanners.extend(other.scanner_centers.iter().map(|x| orient.apply(x)));
        Self::new(name, beacons, Some(scanners))
    }


    /// Returns the largest manhattan distance between any two scanner centers. If there
    /// is only one scanner_center it returns 0.
    fn largest_scanner_manhattan_distance(&self) -> Coord {
        let mut largest: Coord = 0;
        for (pos, s1) in self.scanner_centers.iter().enumerate() {
            for s2 in self.scanner_centers[..pos].iter() {
                let manhat = s1.manhattan_distance(s2);
                println!("distance from {} to {} is {}", s1, s2, manhat); // FIXME: Remove
                if manhat > largest {
                    largest = manhat;
                }
            }
        }
        largest
    }
}
impl PartialEq for Scanner {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Scanner {}


impl Axis {
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

    fn index(&self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}


impl AxisOrient {
    /// Construct an Orient by parsing a string that looks like "Y-#-84". This
    /// is intended for use in tests, so it panics if the input is invalid.
    fn parse(s: &str) -> Self {
        let axis_orient_regex = Regex::new(r"^([XYZ])([-+])#(-?[0-9]+)$").unwrap();
        let cap = axis_orient_regex.captures(s).unwrap();
        let maps_to: Axis = match cap.get(1).unwrap().as_str() {
            "X" => Axis::X,
            "Y" => Axis::Y,
            "Z" => Axis::Z,
            _ => panic!("Regex error"),
        };
        let flip: bool = match cap.get(2).unwrap().as_str() {
            "-" => true,
            "+" => false,
            _ => panic!("Regex error"),
        };
        let offset: Coord = cap.get(3).unwrap().as_str().parse().unwrap();
        AxisOrient{maps_to, flip, offset}
    }

    /// Returns a new Coord obtained by applying this to the given beacon
    fn apply(&self, beacon: &Beacon) -> Coord {
        beacon.get(self.maps_to) * if self.flip {-1} else {1} + self.offset
    }
}


const LEGAL_ORIENT_KEYS: [&str;24] = [
    "X+Y+Z+",
    "Y+X-Z+",
    "X-Y-Z+",
    "Y-X+Z+",
    "Z-Y+X+",
    "Y+Z+X+",
    "Z+Y-X+",
    "Y-Z-X+",
    "X+Z-Y+",
    "Z-X-Y+",
    "X-Z+Y+",
    "Z+X+Y+",
    "X+Y-Z-",
    "Y-X-Z-",
    "X-Y+Z-",
    "Y+X+Z-",
    "Z+Y+X-",
    "Y+Z-X-",
    "Z-Y-X-",
    "Y-Z+X-",
    "X+Z+Y-",
    "Z+X-Y-",
    "X-Z-Y-",
    "Z-X+Y-",
];

impl Orient {
    /// Construct an Orient by parsing a string that looks like "[x:Y-#1165 y:Z-#-2385 z:X+#-3710]".
    #[allow(dead_code)]
    fn parse(s: &str) -> Self {
        let orient_regex = Regex::new(r"^\[x:([XYZ][-+]#-?[0-9]+) y:([XYZ][-+]#-?[0-9]+) z:([XYZ][-+]#-?[0-9]+)\]+$").unwrap();
        let cap = orient_regex.captures(s).unwrap();
        let x = AxisOrient::parse(cap.get(1).unwrap().as_str());
        let y = AxisOrient::parse(cap.get(2).unwrap().as_str());
        let z = AxisOrient::parse(cap.get(3).unwrap().as_str());
        Orient{orients: [x,y,z]}
    }

    /// Returns a new Beacon obtained by applying the orientation.
    fn apply(&self, beacon: &Beacon) -> Beacon {
        Beacon{
            x: self.orients[Axis::X.index()].apply(beacon),
            y: self.orients[Axis::Y.index()].apply(beacon),
            z: self.orients[Axis::Z.index()].apply(beacon),
        }
    }

    /// Returns true if this is physically possible; false if not.
    fn is_physical(&self) -> bool {
        // NOTE: I'm certain there's a formula for this. But I'm too tired to figure
        // it out so I'm using brute force.
        let key = format!(
            "{}{}{}{}{}{}",
            self.orients[0].maps_to, if self.orients[0].flip {'-'} else {'+'},
            self.orients[1].maps_to, if self.orients[1].flip {'-'} else {'+'},
            self.orients[2].maps_to, if self.orients[2].flip {'-'} else {'+'},
        );
        return LEGAL_ORIENT_KEYS.contains(&key.as_str())
    }
}



impl LengthSet {
    fn new(mut lengths: Vec<LenSq>) -> Self {
        lengths.sort();
        LengthSet{lengths}
    }

    #[allow(dead_code)] // Note: Even if not used, this code is worth keeping
    fn len(&self) -> usize {
        self.lengths.len()
    }

    #[allow(dead_code)] // Note: Even if not used, this code is worth keeping
    fn has_dupes(&self) -> bool {
        // use the fact that it's sorted: any dupes must be adjacent.
        for i in 1..self.lengths.len() {
            if self.lengths[i-1] == self.lengths[i] {
                return true;
            }
        }
        false
    }

    /// Finds number of matches between 2 lengthsets. A repeated item only counts
    /// once.
    fn overlaps(&self, other: &Self) -> i32 {
        let mut count = 0;
        let mut i = 0;
        let mut j = 0;
        loop {
            let self_value = self.lengths[i];
            let other_value = other.lengths[j];
            let should_incr_i;
            let should_incr_j;
            if self_value == other_value {
                count += 1;
                should_incr_i = true;
                should_incr_j = true;
            } else if self_value < other_value {
                should_incr_i = true;
                should_incr_j = false;
            } else {
                should_incr_i = true;
                should_incr_j = true;
            }
            if should_incr_i {
                loop { // keep going until the value changes
                    i += 1;
                    if i == self.lengths.len() {
                        return count; // no more overlaps to be found
                    }
                    if self.lengths[i] != self_value {
                        break; // we've incremented i enogh
                    }
                }
            }
            if should_incr_j {
                loop { // keep going until the value changes
                    j += 1;
                    if j == other.lengths.len() {
                        return count; // no more overlaps to be found
                    }
                    if self.lengths[j] != other_value {
                        break; // we've incremented j enough
                    }
                }
            }
        }
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

    #[allow(dead_code)] // This one is useful even if we're not using it at the moment
    fn contains(&self, val: LenSq) -> bool {
        return self.lengths.contains(&val)
    }

    /// Returns a new LengthSet which has all values present in either self or other.
    #[allow(dead_code)] // This one is useful even if we're not using it at the moment
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


    /// Returns a LengthSet that contains all the lengths that occur level_of_uniqueness
    /// times or fewer. If level_of_uniqueness == 1 it will find the unique lengths, if
    /// level_of_uniqueness == 2 it will fine all unique and duplicate lengths, and so
    /// forth. Each length will occur only ONCE in the resulting LengthSet.
    fn to_level_of_uniqueness(&self, level_of_uniqueness: usize) -> Self {
        // use the fact that it's sorted: any dupes must be adjacent.
        let mut counts: HashMap<LenSq, usize> = HashMap::new();

        for length in &self.lengths {
            counts.insert(*length, counts.get(length).unwrap_or(&0) + 1);
        }

        let mut answer_lengths: Vec<LenSq> = Vec::new();
        for (length, count) in counts {
            if count <= level_of_uniqueness {
                answer_lengths.push(length);
            }
        }
        LengthSet::new(answer_lengths)
    }


    /// Given another LengthSet, finds a LengthSet of the lengths that occur no more than
    /// level_of_uniqueness times in either set. This is not sorted in any particular
    /// order but it DOES guarantee that each length appears in it only once.
    fn shared_to_level_of_uniqueness(&self, other: &Self, level_of_uniqueness: usize) -> Self {
        let mine = self.to_level_of_uniqueness(level_of_uniqueness);
        let theirs = other.to_level_of_uniqueness(level_of_uniqueness);
        mine.intersect(&theirs)
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
        write!(f, "{}{}#{}",
               self.maps_to,
               if self.flip {"-"} else {"+"},
               self.offset,
        )
    }
}
impl fmt::Display for Orient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[x:{} y:{} z:{}]", self.orients[0], self.orients[1], self.orients[2])
    }
}
impl fmt::Display for Overlap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<({},{}): {}>", self.pos_1, self.pos_2, self.overlap_count)
    }
}




/// Given an axis in the source, the two source beacons, the two dest beacons, and a few maps we're allowed to use,
/// this returns a Vec of the possible AxisOrients for that axis of the source which will successfully
/// map these two endpoints.
fn get_possible_orients(this_axis: Axis, source_beacons: [Beacon;2], dest_beacons: [Beacon;2], allowed_maps: Vec<Axis>) -> Vec<AxisOrient> {
    let [source_point, other_source_point] = source_beacons;
    let mut axis_orients: Vec<AxisOrient> = Vec::new();
    for (dest_point, other_dest_point) in [(dest_beacons[0], dest_beacons[1]), (dest_beacons[1], dest_beacons[0])] {
        for maps_to in allowed_maps.iter().cloned() {
            for flip in [false, true] {
                let offset: Coord = source_point.get(this_axis) - dest_point.get(maps_to) * (if flip {-1} else {1});
                let axis_orient = AxisOrient{maps_to, flip, offset};
                if axis_orient.apply(&other_dest_point) == other_source_point.get(this_axis) {
                    assert_eq!(axis_orient.apply(&dest_point), source_point.get(this_axis)); // the other point's Y works
                    // The mapping works on x for both ends of this line
                    axis_orients.push(axis_orient)
                }
            }
        }
    }
    axis_orients
}


/// Given two beacons in one Scanner and two beacons in another scanner where we suspect
/// that the pairs correspond to each other (in some order) this returns a vector of the
/// different orientations that could be applied to the "dest" Scanner to make them line
/// up.
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
        if po.is_physical() {
            if po.apply(&dest_points[0]) == source_points[0] && po.apply(&dest_points[1]) == source_points[1] ||
                po.apply(&dest_points[1]) == source_points[0] && po.apply(&dest_points[0]) == source_points[1]
            {
                if !orients.contains(&po) {
                    orients.push(po);
                }
            }
        }
    }
    orients
}


// Given two scanners and a length, this returns the list of all orientations that might
// work for any segments (one in source, the other in dest) that have that length
fn orients_for_seg_length(source: &Scanner, dest: &Scanner, length: LenSq) -> Vec<Orient> {
    let source_pairs = source.segments_for_length(length);
    let dest_pairs = dest.segments_for_length(length);
    let mut orients: Vec<Orient> = Vec::new();
    for source_pair in source_pairs {
        for dest_pair in dest_pairs {
            // Got a point-pair from each one.
            let orients_for_seg = orients_for_segment(
                [source_pair[0], source_pair[1]],
                [dest_pair[0], dest_pair[1]],
            );
            // --- Add any that are new ---
            for orient in &orients_for_seg {
                if !orients.contains(orient) {
                    orients.push(*orient);
                }
            }
        }
    }
    orients
}


/// This is passed two Scanners and it returns a list of lengths (squared) to try
/// for fitting these two scanners together. We want to try to lengths that are rare
/// in each scanner, so level_of_uniqueness is the maximum number of times a length can occur
/// in either scanner in order to be included. So if level_of_uniqueness == 1 then it will
/// only return lengths that are unique within each scanner and exist in both
/// scanners, while if level_of_uniqueness == 2 it will return lengths that occur no more
/// than twice in each.
fn find_lengths_to_try(s1: &Scanner, s2: &Scanner, level_of_uniqueness: usize) -> Vec<LenSq> {
    let shared_lengths = s1.get_lengths().shared_to_level_of_uniqueness(&s2.get_lengths(), level_of_uniqueness);
    shared_lengths.lengths
}



/// Given two Scanners which may have overlapping Beacons, this finds unique lengths among
/// the overlap to figure out how they are oriented, then returns a new Scanner that consists
/// of the two combined (with the orientation of the first one). If it cannot find a fit
/// then it returns None instead. It will consider lengths that have a level of uniqueness
/// of up to level_of_uniqueness repetitions.
fn merge_overlapping_scanners(source: &Scanner, dest: &Scanner, level_of_uniqueness: usize) -> Option<Scanner> {
    println!("Merging {} --with-- {}", source.name, dest.name); // Keep this for monitoring progress
    let lengths_to_try_vec = find_lengths_to_try(source, dest, level_of_uniqueness);

    let mut orients: Vec<Orient> = Vec::new();
    for length in lengths_to_try_vec {
        let orients_for_this_length = orients_for_seg_length(source, dest, length);
        for orient in orients_for_this_length {
            if !orients.contains(&orient) {
                orients.push(orient);
            }
        }
    }

    if orients.len() == 0 {
        println!("  Problems! there were no orients");
        return None;
    }

    // --- Build the response and check how many beacons overlapped ----
    for orient in orients {
        let merged: Scanner = source.merge_with(dest, orient);
        let overlapping = (source.len() + dest.len()) - merged.len();
        if overlapping >= MIN_OVERLAPS_FOR_MATCH {
            // We've got a good fit!
            println!("  Success! We merged it using orient {}", orient);
            return Some(merged)
        }
    }
    println!("  Problems! We tried every orient and none matched.");
    return None;
}


/// Finds one highly-connected pair of scanners (starting from the front of the list)
/// and merges them (hopefully... we panic if things go wrong). Then returns a new
/// list of scanners with the merged one at the front.
fn merge_once(scanners: Vec<Scanner>) -> Vec<Scanner> {
    assert!(scanners.len() > 1);

    // --- Review the overlaps between scanners and pick the order in which we want to try to merge them ---
    // FIXME: Regenerating this again after every call to merge_once is a lot of wasted work. But it's not the source of my bug.
    let mut overlaps = Vec::new();
    for (i, scanner1) in scanners.iter().enumerate() {
        for (beyond_i, scanner2) in scanners[(i+1)..].iter().enumerate() {
            let j = i + 1 + beyond_i;
            let overlap_count = scanner1.get_lengths().overlaps(&scanner2.get_lengths());
            if overlap_count > 0 {
                overlaps.push(Overlap{pos_1: i, pos_2: j, overlap_count});
            }
        }
    }
    overlaps.sort_by_key(|x| -1 * x.overlap_count);
    assert!(overlaps.len() >= 1);

    // --- try the overlaps until something works or we give up ---
    for overlap in overlaps {
        let merged_scanner_opt: Option<Scanner> = merge_overlapping_scanners(
            &scanners[overlap.pos_1], &scanners[overlap.pos_2], UNIQUENESS_LEVEL
        );
        match merged_scanner_opt {
            Some(merged_scanner) => {
                // --- Build the new list of scanners ---
                let mut new_scanners = Vec::new();
                new_scanners.push(merged_scanner);
                for (i, scanner) in scanners.iter().enumerate() {
                    if i != overlap.pos_1 && i != overlap.pos_2 {
                        new_scanners.push(scanner.clone());
                    }
                }
                return new_scanners;
            },
            None => {}, // didn't merge that pair so try the next overlap
        }
    }

    println!("Before giving up, the list of scanners was this:");
    for scanner in &scanners {
        println!("  {}", scanner.name);
    }
    panic!("We can't do it... ran out of overlaps to try!");
}





fn run() -> Result<(),InputError> {
    let mut scanners: Vec<Scanner> = read_beacon_file()?;

    if USE_SHUFFLE {
        let mut rng = rand::thread_rng();
        println!("Trying a shuffle first, in case that helps. Random is {}", rng.next_u32());
        scanners.shuffle(&mut rng);
        println!("Shuffled!");
    }

    assert!(scanners.len() > 0);
    while scanners.len() > 1 {
        scanners = merge_once(scanners);
        println!("  I now have {} scanner groups left.", scanners.len());
    }
    println!("IN THE END, scanners: {}", scanners[0]);
    println!("There are {} beacons.", scanners[0].beacons.len());
    println!("The largest Manhattan distance is {}.", scanners[0].largest_scanner_manhattan_distance());

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
        assert_eq!(AxisOrient{maps_to: Axis::X, flip: false, offset: 0}.apply(&Beacon{x: 27, y: 0, z: 0}), 27);
        assert_eq!(AxisOrient{maps_to: Axis::X, flip: false, offset: 2}.apply(&Beacon{x: -1, y: 1, z: 0}),  1);
        assert_eq!(AxisOrient{maps_to: Axis::Y, flip: false, offset: 2}.apply(&Beacon{x: -1, y: 1, z: 0}),  3);
        assert_eq!(AxisOrient{maps_to: Axis::Y, flip: true,  offset: 2}.apply(&Beacon{x: -1, y: 1, z: 0}),  1);
        assert_eq!(AxisOrient{maps_to: Axis::Z, flip: true,  offset: 5}.apply(&Beacon{x:  1, y: 3, z: 5}),  0);
    }

    #[test]
    fn parse_axis_orient() {
        assert_eq!(
            AxisOrient::parse("Y+#-10"),
            AxisOrient{maps_to: Axis::Y, flip: false, offset: -10}
        );
    }

    #[test]
    fn parse_orient() {
        assert_eq!(
            Orient::parse("[x:Y+#127 y:Z-#84 z:X-#-1223]"),
            Orient{orients: [
                AxisOrient{maps_to: Axis::Y, flip: false, offset: 127},
                AxisOrient{maps_to: Axis::Z, flip: true, offset: 84},
                AxisOrient{maps_to: Axis::X, flip: true, offset: -1223},
            ]}
        );
    }

    #[test]
    fn test_orient_apply() {
        let or: Orient = Orient{orients: [
            AxisOrient{maps_to: Axis::Y, flip: false, offset: 1},
            AxisOrient{maps_to: Axis::X, flip: false, offset: 2},
            AxisOrient{maps_to: Axis::Z, flip: false, offset: 3},
        ]};
        let b = or.apply(&Beacon{x: 100, y: 200, z: 300});
        assert_eq!(b, Beacon{x: 201, y: 102, z: 303});
    }

    #[test]
    fn two_orients_1() {
        let orients = orients_for_segment(
            [Beacon{ x:  745, y: -1757, z: -4521 }, Beacon { x:  792, y: -1762, z: -4502 }],
            [Beacon{ x: -835, y:   572, z:   942 }, Beacon { x: -788, y:   577, z:   961 }]
        );
        assert_eq!(orients.len(), 1);
    }

    #[test]
    fn two_orients_2() {
        let orients = orients_for_segment(
            [Beacon{ x: -811, y: 420, z: -628 }, Beacon { x: -792, y: 373, z: -623 }],
            [Beacon{ x: -835, y: 572, z:  942 }, Beacon { x: -788, y: 577, z:  961 }]
        );
        assert_eq!(orients.len(), 1);
    }


    #[test]
    fn can_find_orients_after_transform_1() {
        let source = [Beacon{ x: -811, y: 420, z: -628 }, Beacon { x: -792, y: 373, z: -623 }];
        let dest = [Beacon{ x: -835, y:   572, z:   942 }, Beacon { x: -788, y:   577, z:   961 }];
        let orients = orients_for_segment(source, dest);
        assert_eq!(orients.len(), 1);
        println!("{}", orients[0]);
        let o1 = Orient::parse("[x:X-#1143 y:Z+#-1265 z:Y+#-2362]");
        let o2 = Orient::parse("[x:Y+#1144 y:X-#-30 z:Z+#-1163]");
        let source_t = [o2.apply(&o1.apply(&source[0])), o2.apply(&o1.apply(&source[1]))];
        let orients = orients_for_segment(source_t, dest);
        assert_eq!(orients.len(), 1);
        println!("{}", orients[0]);
    }

    #[test]
    fn can_find_orients_after_transform_3() {
        let source = [Beacon { x: 0, y: 0, z: 0 }, Beacon { x: 1, y: 2, z: 3 }];
        let dest = [Beacon { x: 10, y: 10, z: 10 }, Beacon { x: 11, y: 12, z: 13 }];
        let orients = orients_for_segment(source, dest);
        for o in &orients {
            println!("{}", o);
        }
        assert_eq!(orients.len(), 1);
        assert_eq!(orients[0], Orient::parse("[x:X+#-10 y:Y+#-10 z:Z+#-10]"));
    }

    #[test]
    fn test_manhattan_distance() {
        let b1 = Beacon{x: 1105, y: -1205, z: 1229};
        let b2 = Beacon{x: -92, y: -2380, z: -20};
        assert_eq!(b1.manhattan_distance(&b2), 3621);
    }

}
