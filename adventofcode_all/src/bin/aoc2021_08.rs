
// ======= part_a =======

mod part_a {
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


    #[derive(Debug)]
    struct SevenSegData {
        combos: [String; 10],
        digits: [String; 4],
    }


    /// This is given a string and a delimiter to split by and it returns an array of
    /// exactly n parts (or panics if that doesn't work out). The parts are copied
    /// into String objects, which are now owned by the caller..
    fn split_into_n_strings<const N: usize>(s: &str, delim: &str) -> [String; N] {
        let vec: Vec<String> = s.split(delim).map(|x| x.to_string()).collect();
        let array: [String; N] = vec.try_into().unwrap();
        array
    }


    fn read_seven_seg_display_file() -> Result<Vec<SevenSegData>, InputError> {
        let filename = "input/2021/input_08.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        let mut results: Vec<SevenSegData> = Vec::new();
        for line in lines {
            let text = line?;
            let (combos_txt, digits_txt): (&str, &str) = text.split(" | ").collect_tuple().unwrap();
            let combos: [String; 10] = split_into_n_strings(combos_txt, " ");
            let digits: [String; 4] = split_into_n_strings(digits_txt, " ");
            results.push(SevenSegData{combos, digits});
        }

        return Ok(results);
    }



    pub fn main() {
        match read_seven_seg_display_file() {
            Ok(seven_seg_data_list) => {
                println!("seven_seg_data_list {:#?}", seven_seg_data_list);
                let mut count = 0;
                for seven_seg_data in seven_seg_data_list {
                    for digit in seven_seg_data.digits {
                        match digit.len() {
                            2 | 3 | 4 | 7 => count += 1,
                            5 | 6 => {},
                            _ => panic!("Invalid number of characters in digit: {}", digit)
                        }
                    }
                }
                println!("The count is {}", count);
            },
            Err(err) => println!("Error: {}", err),
        }
    }
}

// ======= part_b =======

mod part_b {
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


    // A struct used to represent a set of "wires" (letters a through g).
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


    /// Read in the input file.
    fn read_seven_seg_display_file() -> Result<Vec<SevenSegData>, InputError> {
        let filename = "input/2021/input_08.txt";
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


    /// Searches through this collection of WireSets and finds the single
    /// one that makes the condition true. Panics if none or more than one
    /// meets the condition.
    fn find_sole_match<'a, I, F>(wire_sets: I, cond: F) -> WireSet
        where
            I: Iterator<Item = &'a WireSet>,
            F: Fn(&WireSet) -> bool,
    {
        let mut result_opt: Option<WireSet> = None;
        let mut num_found = 0;
        for wire_set in wire_sets {
            if cond(wire_set) {
                result_opt = Some(*wire_set);
                num_found += 1;
            }
        }
        assert!(num_found == 1);
        result_opt.unwrap()
    }



    fn break_code(combos: [WireSet; 10]) -> [WireSet; 10] {
        let mut size5_combos: Vec<WireSet> = Vec::new();
        let mut size6_combos: Vec<WireSet> = Vec::new();
        for combo in combos {
            match combo.size() {
                2 | 3 | 4 | 7 => {},
                5 => size5_combos.push(combo),
                6 => size6_combos.push(combo),
                _ => panic!("Combo of an invalid size: {}", combo)
            }
        }
        assert!(size5_combos.len() == 3 && size6_combos.len() == 3);

        let mapping_1 = find_sole_match(combos.iter(), |x| x.size() == 2);
        let mapping_7 = find_sole_match(combos.iter(), |x| x.size() == 3);
        let mapping_4 = find_sole_match(combos.iter(), |x| x.size() == 4);
        let mapping_8 = find_sole_match(combos.iter(), |x| x.size() == 7);
        let mapping_3 = find_sole_match(size5_combos.iter(), |x| x.minus(&mapping_1).size() == 3);
        let mapping_6 = find_sole_match(size6_combos.iter(), |x| x.minus(&mapping_1).size() == 5);
        let just_true_c = mapping_7.minus(&mapping_6);
        let just_true_f = mapping_1.minus(&just_true_c);
        let mapping_2 = find_sole_match(size5_combos.iter(), |x| x.minus(&just_true_f).size() == 5);
        let mapping_5 = find_sole_match(size5_combos.iter(), |x| *x != mapping_2 && *x != mapping_3);
        let just_true_e = mapping_2.minus(&mapping_4).minus(&mapping_5);
        let mapping_9 = find_sole_match(size6_combos.iter(), |x| x.minus(&just_true_e).size() == 6);
        let mapping_0 = find_sole_match(size6_combos.iter(), |x| *x != mapping_6 && *x != mapping_9);
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
                    println!("digits: {}{}{}{}", dd_0, dd_1, dd_2, dd_3);
                    sum += dd_0 * 1000 + dd_1 * 100 + dd_2 * 10 + dd_3;
                }
                println!("Total sum: {}", sum);
            },
            Err(err) => println!("Error: {}", err),
        }
    }
}


// ======= main() =======


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    println!("\nPart a:");
    part_a::main();
    println!("\nPart b:");
    part_b::main();
    Ok(())
}
