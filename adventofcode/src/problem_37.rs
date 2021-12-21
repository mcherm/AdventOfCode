use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader};
use rand::RngCore;
use regex::Regex;
use rand::seq::SliceRandom;


const USE_SHUFFLE: bool = false;


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

#[derive(Clone)]
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

    /// Returns a count of the Beacons
    fn len(&self) -> usize {
        self.beacons.len()
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
    /// Returns a new Coord obtained by applying this to the given beacon
    fn apply(&self, beacon: Beacon) -> Coord {
        beacon.get(self.maps_to) * if self.flip {-1} else {1} + self.offset
    }
}


const LEGAL_ORIENT_KEYS: [&str;24] = [
    "X+Y+Z+",
    "Y+X-Z+",
    "Y+X-Z+",
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
    "Y+X+Z-",
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
    /// Returns a new Beacon obtained by applying the orientation.
    fn apply(&self, beacon: Beacon) -> Beacon {
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

    /// Returns a new LengthSet containing only the elements of this one that occur
    /// precisely once.
    fn uniques(&self) -> Self {
        // use the fact that it's sorted: any dupes must be adjacent.
        let mut uniques: Vec<LenSq> = Vec::new();
        let mut previous: LenSq = self.lengths[0];
        let mut previous_is_dup = false;
        for i in 1..self.lengths.len() {
            let subsequent = self.lengths[i];
            if subsequent == previous {
                // it's a dupe
                previous_is_dup = true;
            } else {
                // new one is different
                if !previous_is_dup {
                    uniques.push(previous);
                }
                previous = subsequent;
                previous_is_dup = false;
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

    /// Given another LengthSet, finds a LengthSet of the lengths that are unique within
    /// each LengthSet but also in common between them. In principle, there might not
    /// be any, and it will return an empty LengthSet.
    fn shared_uniques(&self, other: &LengthSet) -> LengthSet {
        self.uniques().intersect(&other.uniques())
    }

}


impl PointDescription {
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
                if axis_orient.apply(other_dest_point) == other_source_point.get(this_axis) {
                    assert_eq!(axis_orient.apply(dest_point), source_point.get(this_axis)); // the other point's Y works
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
            if po.apply(dest_points[0]) == source_points[0] && po.apply(dest_points[1]) == source_points[1] ||
                po.apply(dest_points[1]) == source_points[0] && po.apply(dest_points[0]) == source_points[1]
            {
                if !orients.contains(&po) {
                    orients.push(po);
                }
            }
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


/// Given two Scanners which may have overlapping Beacons, this finds unique lengths among
/// the overlap to figure out how they are oriented, then returns a new Scanner that consists
/// of the two combined (with the orientation of the first one). If it cannot find a fit
/// then it returns None instead.
fn merge_overlapping_scanners(source: &Scanner, dest: &Scanner) -> Option<Scanner> {
    // println!("Merging {} --with-- {}", source.name, dest.name); // Keep this for monitoring progress
    let shared_uniques = source.get_lengths().shared_uniques(&dest.get_lengths());

    let mut unique_lengths = shared_uniques.lengths.iter();
    let first_unique_length_opt = unique_lengths.next();
    if first_unique_length_opt.is_none() {
        println!("  Problems! There were no unique lengths.");
        return None;
    }
    let first_unique_length = first_unique_length_opt.unwrap();

    let mut orients = orients_for_unique_seg_length(source, dest, *first_unique_length);
    if orients.len() > 1 {
        for next_unique_length in unique_lengths {
            let new_orients = orients_for_unique_seg_length(source, dest, *next_unique_length);
            orients.retain(|orient| new_orients.contains(orient));
            if orients.len() <= 1 {
                break; // once we have just one, we can quit.
            }
        }
    }

    if orients.len() == 0 {
        println!("  Problems! there were no orients");
        return None;
    } else if orients.len() > 1 {
        // There are 2+ orients. We COULD try counting overlaps, but in practice
        // this happens so rarely that the code to support it isn't written yet.
        panic!("Code could handle multiple possible orients, but it isn't implemented yet.");
    }

    // --- Build the response and check how many beacons overlapped ----
    assert!(orients.len() == 1);
    let merged: Scanner = source.merge_with(dest, orients[0]);
    let overlapping = (source.len() + dest.len()) - merged.len();
    if overlapping < 12 {
        // The best fit we could find had fewer than 12 overlapping beacons. The
        // problem promised us at least 12 for each match. So we're going to declare
        // that that these two Scanners simply didn't fit together.
        println!("Rejecting the match because there are only {} overlapping beacons.", overlapping);
        return None;
    } else {
        // We've got a good fit!
        Some(merged)
    }
}


/// Finds one highly-connected pair of scanners (starting from the front of the list)
/// and merges them (hopefully... we panic if things go wrong). Then returns a new
/// list of scanners with the merged one at the front.
fn merge_once(scanners: Vec<Scanner>) -> Vec<Scanner> {
    assert!(scanners.len() > 1);

    // --- Review the overlaps and pick the order in which we want to try to merge them ---
    let mut overlaps: Vec<Overlap> = Vec::new();
    for (i, scanner1) in scanners.iter().enumerate() {
        for (beyond_i, scanner2) in scanners[(i+1)..].iter().enumerate() {
            let j = i + 1 + beyond_i;
            let overlap_count = scanner1.get_lengths().overlaps(&scanner2.get_lengths());
            overlaps.push(Overlap{pos_1: i, pos_2: j, overlap_count});
        }
    }
    overlaps.sort_by_key(|x| -1 * x.overlap_count);
    assert!(overlaps.len() >= 1);

    // --- Merge it ---
    let mut overlap_iter = overlaps.iter();
    let mut overlap: &Overlap = overlap_iter.next().unwrap();
    let mut merged_scanner_opt: Option<Scanner>;
    loop {
        merged_scanner_opt = merge_overlapping_scanners(&scanners[overlap.pos_1], &scanners[overlap.pos_2]);
        if merged_scanner_opt.is_some() {
            break
        } else {
            match overlap_iter.next() {
                Some(next_overlap) => overlap = next_overlap,
                None => {
                    println!("Before giving up, the list of scanners was this:");
                    for scanner in &scanners {
                        println!("  {}", scanner.name);
                    }
                    panic!("We can't do it... ran out of overlaps to try!");
                }
            }
        }
    }
    assert!(merged_scanner_opt.is_some());

    // --- Build the new list of scanners ---
    let mut new_scanners = Vec::new();
    new_scanners.push(merged_scanner_opt.unwrap());
    for (i, scanner) in scanners.iter().enumerate() {
        if i != overlap.pos_1 && i != overlap.pos_2 {
            new_scanners.push(scanner.clone());
        }
    }
    new_scanners
}



fn run() -> Result<(),InputError> {
    let mut scanners = read_beacon_file()?;

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
        assert_eq!(b, Beacon{x: 201, y: 102, z: 303});
    }
}
