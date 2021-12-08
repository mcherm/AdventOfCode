use std::fmt;
use std::num::ParseIntError;
use std::fs::File;
use std::io::{BufRead, BufReader};
use itertools::Itertools;
use std::convert::TryInto;



/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(ParseIntError),
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl From<ParseIntError> for InputError {
    fn from(error: ParseIntError) -> Self {
        InputError::BadInt(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err)   => write!(f, "{}", err),
            InputError::BadInt(err)    => write!(f, "{}", err),
        }
    }
}


#[derive(Copy, Clone)]
struct WireSet {
    contains: [bool; 7]
}

impl WireSet {
    /// Create a WireSet from a string of letters in a..g
    fn new(s: &str) -> WireSet {
        let mut contains = [false; 7];
        for c in s.chars() {
            contains[WireSet::index(c)] = true;
        }
        WireSet{contains}
    }

    fn index(c: char) -> usize {
        let index = c as usize - 'a' as usize;
        assert!(index < 7);
        index
    }

    fn has_wire(&self, wire: char) -> bool {
        self.contains[WireSet::index(wire)]
    }

    fn size(&self) -> u8 {
        let mut count = 0;
        for i in 0..7 {
            if self.contains[i] {
                count += 1;
            }
        }
        count
    }

    fn _to_id(&self) -> u8 {
        let mut id = 0;
        for i in 0..7 {
            if i != 0 {
                id *= 2;
            }
            if self.contains[i] {
                id += 1;
            }
        }
        id
    }

    fn minus(&self, other: &WireSet) -> WireSet {
        let mut new_contains: [bool; 7] = self.contains.clone();
        for i in 0..7 {
            if other.contains[i] {
                new_contains[i] = false;
            }
        }
        WireSet{contains: new_contains}
    }
}

impl fmt::Display for WireSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for c in 'a'..='g' {
            let ch = if self.has_wire(c) {
                c
            } else {
                '-'
            };
            write!(f, "{}", ch)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl fmt::Debug for WireSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::cmp::PartialEq for WireSet {
    fn eq(&self, other: &Self) -> bool {
        let mut result = true;
        for i in 0..7 {
            result = result && (self.contains[i] == other.contains[i]);
        }
        result
    }
}


#[derive(Debug)]
struct SevenSegData {
    combos: [WireSet; 10],
    digits: [WireSet; 4],
}



/// This is given a string and a delimiter to split by and it returns an array of
/// exactly n parts (or panics if that doesn't work out). The parts are copied
/// into String objects, which are now owned by the caller..
fn split_into_n_strings<'a, const N: usize>(s: &'a str, delim: &'a str) -> [&'a str; N] {
    let vec: Vec<&'a str> = s.split(delim).collect();
    let array: [&'a str; N] = vec.try_into().unwrap();
    array
}


fn read_seven_seg_display_file() -> Result<Vec<SevenSegData>, InputError> {
    let filename = "data/2021/day/8/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    let mut results: Vec<SevenSegData> = Vec::new();
    for line in lines {
        let text = line?;
        let (combos_txt, digits_txt): (&str, &str) = text.split(" | ").collect_tuple().unwrap();
        let combos: [WireSet; 10] = split_into_n_strings(combos_txt, " ").map(WireSet::new);
        let digits: [WireSet; 4] = split_into_n_strings(digits_txt, " ").map(WireSet::new);
        results.push(SevenSegData{combos, digits});
    }

    return Ok(results);
}


fn break_code(combos: [WireSet; 10]) -> [WireSet; 10] {
    let mut mappings: [Option<WireSet>; 10] = [None, None, None, None, None, None, None, None, None, None];
    let mut size5_combos: Vec<WireSet> = Vec::new();
    let mut size6_combos: Vec<WireSet> = Vec::new();
    for combo in combos {
        match combo.size() {
            2 => mappings[1] = Some(combo),
            3 => mappings[7] = Some(combo),
            4 => mappings[4] = Some(combo),
            5 => size5_combos.push(combo),
            6 => size6_combos.push(combo),
            7 => mappings[8] = Some(combo),
            _ => panic!("Combo of an invalid size: {}", combo)
        }
    }
    let mapping_1 = mappings[1].unwrap();
    let mapping_7 = mappings[7].unwrap();
    let mapping_4 = mappings[4].unwrap();
    let mapping_8 = mappings[8].unwrap();
    assert!(size5_combos.len() == 3 && size6_combos.len() == 3);
    for combo in &size5_combos {
        if combo.minus(&mapping_1).size() == 3 {
            mappings[3] = Some(*combo);
        }
    }
    let mapping_3 = mappings[3].unwrap();
    for combo in &size6_combos {
        if combo.minus(&mapping_1).size() == 5 {
            mappings[6] = Some(*combo);
        }
    }
    let mapping_6 = mappings[6].unwrap();
    let just_true_c = mapping_7.minus(&mapping_6);
    let just_true_f = mapping_1.minus(&just_true_c);
    for combo in &size5_combos {
        if combo.minus(&just_true_f).size() == 5 {
            mappings[2] = Some(*combo);
        }
    }
    let mapping_2 = mappings[2].unwrap();
    for combo in &size5_combos {
        if *combo != mapping_2 && *combo != mapping_3 {
            mappings[5] = Some(*combo);
        }
    }
    let mapping_5 = mappings[5].unwrap();
    let partial = mapping_2.minus(&mapping_4);
    let just_true_e = partial.minus(&mapping_5);
    for combo in &size6_combos {
        if combo.minus(&just_true_e).size() == 6 {
            mappings[9] = Some(*combo);
        }
    }
    let mapping_9 = mappings[9].unwrap();
    for combo in &size6_combos {
        if *combo != mapping_6 && *combo != mapping_9 {
            mappings[0] = Some(*combo);
        }
    }
    let mapping_0 = mappings[0].unwrap();
    [mapping_0, mapping_1, mapping_2, mapping_3, mapping_4, mapping_5, mapping_6, mapping_7, mapping_8, mapping_9]
}


fn to_display_digit(data_mapping: &[WireSet; 10], digit: &WireSet) -> Option<i32> {
    for i in 0..10 {
        if data_mapping[i] == *digit {
            return Some(i as i32);
        }
    }
    None
}


pub fn main() {
    match read_seven_seg_display_file() {
        Ok(seven_seg_data_list) => {
            let mut sum: i32 = 0;
            for seven_seg_data in seven_seg_data_list {
                let data_mapping = break_code(seven_seg_data.combos);
                let dd_0 = to_display_digit(&data_mapping, &seven_seg_data.digits[0]).unwrap();
                let dd_1 = to_display_digit(&data_mapping, &seven_seg_data.digits[1]).unwrap();
                let dd_2 = to_display_digit(&data_mapping, &seven_seg_data.digits[2]).unwrap();
                let dd_3 = to_display_digit(&data_mapping, &seven_seg_data.digits[3]).unwrap();
//                assert!(dd_0 != 0);
                println!("digits: {}{}{}{}", dd_0, dd_1, dd_2, dd_3);
                sum += dd_0 * 1000 + dd_1 * 100 + dd_2 * 10 + dd_3;
            }
            println!("Total sum: {}", sum);
        },
        Err(err) => println!("Error: {}", err),
    }
}
