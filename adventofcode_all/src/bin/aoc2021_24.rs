
// ======= part_a =======

mod part_a {
    use std::fmt;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::fmt::{Display, Formatter};
    use std::collections::{HashMap, HashSet};
    use itertools;
    use itertools::Itertools;
    use std::fmt::Write;
    use std::ops::Add;
    use nom::bytes::complete::tag as nom_tag;
    use nom::sequence::tuple as nom_tuple;
    use nom::branch::alt as nom_alt;
    use nom::character::complete::i64 as nom_value;


// ======== Reading Input ========

    /// An error that we can encounter when reading the input.
    #[derive(Debug)]
    enum InputError {
        IoError(std::io::Error),
        InvalidInstruction,
        NoStartingInputInstruction,
    }

    impl From<std::io::Error> for InputError {
        fn from(error: std::io::Error) -> Self {
            InputError::IoError(error)
        }
    }

    impl fmt::Display for InputError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InputError::IoError(err) => write!(f, "{}", err),
                InputError::InvalidInstruction => write!(f, "Invalid Instruction"),
                InputError::NoStartingInputInstruction => write!(f, "No starting input instruction"),
            }
        }
    }

    /// Read in the input file.
    fn read_alu_file() -> Result<Vec<Segment>, InputError> {
        // --- open file ---
        let filename = "input/2021/input_24.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        // --- read instructions ---
        let mut segments: Vec<Segment> = Vec::new();
        let mut input_register: Option<Register> = None;
        let mut computes: Vec<Compute> = Vec::new();
        for line in lines {
            let text: String = line?;
            match Instruction::parse(&text) {
                Ok(("", instruction)) => {  // the parse was OK
                    match instruction {
                        Instruction::Input(reg) => {
                            // -- Start a new segment --
                            if let Some(input) = input_register {
                                segments.push(Segment{input, computes: computes.clone()});
                            }
                            input_register = Some(reg);
                            computes.clear();
                        }
                        Instruction::Compute(compute) => {
                            if input_register.is_none() {
                                return Err(InputError::NoStartingInputInstruction);
                            }
                            computes.push(compute)
                        }
                    }
                },
                Ok((_, _)) => return Err(InputError::InvalidInstruction), // if extra stuff on the line
                Err(_) => return Err(InputError::InvalidInstruction), // if parse failed
            };
        }
        match input_register {
            None => return Err(InputError::NoStartingInputInstruction),
            Some(input) => segments.push(Segment{input, computes: computes.clone()}),
        }

        // --- return result ---
        Ok(segments)
    }



// ======== Types ========

    type Value = i64;

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    enum Register {
        W, X, Y, Z
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    enum Parameter {
        Constant(Value),
        Register(Register),
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    enum Compute {
        Add(Register, Parameter),
        Mul(Register, Parameter),
        Div(Register, Parameter),
        Mod(Register, Parameter),
        Eql(Register, Parameter),
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    enum Instruction {
        Input(Register),
        Compute(Compute),
    }

    /// One segment of instructions consists of one Input instruction followed
    /// by a series of Compute instructions.
    #[derive(Debug, Clone)]
    struct Segment {
        input: Register,
        computes: Vec<Compute>,
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
    struct Alu {
        values: [Value; Register::NUM_ITEMS],
    }


    /// Cache for ONE particular segment.
    struct SegmentCache {
        segment: Segment,
        cache: HashMap<(Alu, Value), Result<Alu,()>>, // map from (start_alu, input_value) to output Alu
    }

    /// Represents an input number.
    #[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
    struct Path {
        str: String,
    }

    /// A class I am creating to track a bunch of valid paths and print out some interesting
    /// information about them.
    struct PathSet {
        paths: HashSet<Path>,
        path_len: Option<usize>,
    }

// ======== Implementations ========

    impl Register {
        const NUM_ITEMS: usize = 4;

        fn id(&self) -> usize {
            match self {
                Register::W => 0,
                Register::X => 1,
                Register::Y => 2,
                Register::Z => 3,
            }
        }

        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom_alt((
                nom_tag("w"),
                nom_tag("x"),
                nom_tag("y"),
                nom_tag("z"),
            ))(input).map(|(rest, res)| (rest, match res {
                "w" => Register::W,
                "x" => Register::X,
                "y" => Register::Y,
                "z" => Register::Z,
                _ => panic!("should never happen")
            }))
        }
    }

    impl Display for Register {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}", match self {
                Register::W => "w",
                Register::X => "x",
                Register::Y => "y",
                Register::Z => "z",
            })
        }
    }



    impl Parameter {
        fn parse_constant(input: &str) -> nom::IResult<&str, Self> {
            nom_value(input).map(|(rest, x)| (rest, Parameter::Constant(x)))
        }

        fn parse_register(input: &str) -> nom::IResult<&str, Self> {
            Register::parse(input).map(|(rest, x)| (rest, Parameter::Register(x)))
        }

        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom_alt((
                Parameter::parse_constant,
                Parameter::parse_register,
            ))(input)
        }
    }
    impl Display for Parameter {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Parameter::Constant(val) => write!(f, "{}", val),
                Parameter::Register(reg) => write!(f, "{}", reg),
            }
        }
    }

    impl Compute {
        fn parse_add(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("add "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Add(reg, param)))
        }
        fn parse_mul(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("mul "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Mul(reg, param)))
        }
        fn parse_div(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("div "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Div(reg, param)))
        }
        fn parse_mod(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("mod "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Mod(reg, param)))
        }
        fn parse_eql(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("eql "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Eql(reg, param)))
        }

        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom_alt((
                Compute::parse_add,
                Compute::parse_mul,
                Compute::parse_div,
                Compute::parse_mod,
                Compute::parse_eql,
            ))(input)
        }
    }

    impl Display for Compute {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Compute::Add(reg, param) => write!(f, "add {} {}", reg, param),
                Compute::Mul(reg, param) => write!(f, "mul {} {}", reg, param),
                Compute::Div(reg, param) => write!(f, "div {} {}", reg, param),
                Compute::Mod(reg, param) => write!(f, "mod {} {}", reg, param),
                Compute::Eql(reg, param) => write!(f, "eql {} {}", reg, param),
            }
        }
    }


    impl Instruction {
        fn parse_inp(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("inp "),
                Register::parse,
            ))(input).map(|(rest, (_, reg))| (rest, Instruction::Input(reg)))
        }
        fn parse_compute(input: &str) -> nom::IResult<&str, Self> {
            Compute::parse(input).map(|(rest, ci)| (rest, Instruction::Compute(ci)))
        }

        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom_alt((
                Instruction::parse_inp,
                Instruction::parse_compute,
            ))(input)
        }

    }

    impl Display for Instruction {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Instruction::Input(reg) => write!(f, "inp {}", reg),
                Instruction::Compute(ci) => write!(f, "{}", ci),
            }
        }
    }


    impl Segment {
        /// Applies the segment. Either returns Ok(alu) for the Alu that results OR
        /// returns Err(()) if the computation would result in an illegal operation.
        fn apply(&self, start_alu: Alu, input: Value) -> Result<Alu,()> {
            let mut alu = start_alu;
            alu = alu.eval_input(self.input, input);
            for compute in self.computes.iter() {
                alu = alu.eval_compute(*compute)?;
            }
            Ok(alu)
        }
    }


    impl Alu {
        /// Given a register, tells the value stored in that register.
        fn value_in(&self, reg: Register) -> Value {
            self.values[reg.id()]
        }

        /// Given a param, tells the value of that parameter.
        fn value_of(&self, param: Parameter) -> Value {
            match param {
                Parameter::Constant(val) => val,
                Parameter::Register(reg) => self.value_in(reg),
            }
        }


        /// Returns true if the ALU is a valid final accept state.
        fn valid(&self) -> bool {
            self.values[Register::Z.id()] == 0
        }

        /// Executes any instruction OTHER than input. Either returns the Alu
        /// that results OR Err(()) if the computation hit an invalid snag.
        fn eval_compute(&self, compute: Compute) -> Result<Alu, ()> {
            let mut values: [Value; Register::NUM_ITEMS] = self.values.clone();
            match compute {
                Compute::Add(reg, param) => {
                    values[reg.id()] = self.value_in(reg) + self.value_of(param);
                },
                Compute::Mul(reg, param) => {
                    values[reg.id()] = self.value_in(reg) * self.value_of(param);
                },
                Compute::Div(reg, param) => {
                    let p = self.value_of(param);
                    if p == 0 {
                        return Err(());
                    }
                    values[reg.id()] = self.value_in(reg) / p;
                },
                Compute::Mod(reg, param) => {
                    let r = self.value_in(reg);
                    let p = self.value_of(param);
                    if r < 0 || p <= 0 {
                        return Err(());
                    }
                    values[reg.id()] = r % p;
                },
                Compute::Eql(reg, param) => {
                    values[reg.id()] = if self.value_in(reg) == self.value_of(param) {1} else {0};
                },
            }
            Ok(Alu{values})
        }

        fn eval_input(&self, input_reg: Register, input: Value) -> Alu {
            let mut values: [Value; Register::NUM_ITEMS] = self.values.clone();
            values[input_reg.id()] = input;
            Alu{values}
        }
    }

    impl Display for Alu {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "[{} {} {} {}]", self.values[0], self.values[1], self.values[2], self.values[3])
        }
    }


    impl SegmentCache {
        fn new(segment: Segment) -> Self {
            SegmentCache{segment, cache: HashMap::new()}
        }

        fn apply_segment(&mut self, start_alu: Alu, input: Value) -> Result<Alu,()> {
            match self.cache.get(&(start_alu, input)) {
                Some(alu) => *alu,
                None => {
                    let answer: Result<Alu,()> = self.segment.apply(start_alu, input);
                    self.cache.insert((start_alu, input), answer);
                    answer
                },
            }
        }
    }


    impl Path {
        /// Return an empty path.
        fn empty() -> Self {
            Path{str: String::new()}
        }

        fn iter(&self) -> PathIterator {
            PathIterator{chars: self.str.chars()}
        }

        fn len(&self) -> usize {
            self.str.len()
        }

        fn prepend(&self, v: Value) -> Path {
            assert!(v >= 1 && v <= 9);
            Path{str: self.str.to_owned().add(&v.to_string())}
        }

        fn append(&self, v: Value) -> Path {
            assert!(v >= 1 && v <= 9);
            Path{str: v.to_string().add(&self.str)}
        }

        #[allow(dead_code)]
        fn concat(&self, other: &Path) -> Path {
            Path{str: self.str.to_owned().add(&other.str)}
        }
    }
    impl Display for Path {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.str)
        }
    }
    struct PathIterator<'a> {
        chars: core::str::Chars<'a>,
    }
    impl<'a> Iterator for PathIterator<'a> {
        type Item = &'a Value;

        fn next(&mut self) -> Option<&'a Value> {
            match self.chars.next() {
                None => None,
                Some(c) => {
                    Some(match c {
                        '1' => &1,
                        '2' => &2,
                        '3' => &3,
                        '4' => &4,
                        '5' => &5,
                        '6' => &6,
                        '7' => &7,
                        '8' => &8,
                        '9' => &9,
                        _ => panic!()
                    })
                },
            }
        }
    }


    impl PathSet {
        fn new() -> Self {
            PathSet{paths: HashSet::new(), path_len: None}
        }

        /// Add a path into PathSet.
        #[allow(dead_code)]
        fn add(&mut self, path: Path) {
            match self.path_len {
                None => self.path_len = Some(path.len()),
                Some(len) => if path.len() != len {panic!("PathSet with paths of different lengths")},
            }
            self.paths.insert(path);
        }

        fn print_all(&self) {
            println!("The valid paths are:");
            for path in self.paths.iter().sorted() {
                println!("{}", path);
            }
        }

        fn print_analysis(&self) {
            if self.paths.len() == 0 {
                println!("There were no paths!");
                return;
            }
            println!("Analyzing {} paths:", self.paths.len());
            let path_len = self.path_len.unwrap();
            println!("The top few are:");
            for path in self.paths.iter().sorted().rev().take(15) {
                println!("    {}", path);
            }

            let mut counts: Vec<HashSet<Value>> = (0..path_len).map(|_| HashSet::new()).collect();
            let mut diffs: Vec<HashSet<Value>> = (0..(path_len - 1)).map(|_| HashSet::new()).collect();
            for path in self.paths.iter() {
                let mut prev_v: Option<Value> = None; // None will never be used
                for (i, v) in path.iter().enumerate() {
                    counts[i].insert(*v);
                    if i > 0 {
                        let diff = prev_v.unwrap() - v;
                        diffs[i-1].insert(diff);
                    }
                    prev_v = Some(*v);
                }
            }

            println!("Frequencies:");
            for i in 0..path_len {
                println!("    Position {}: {}", i, print_value_set(&counts[i]));
            }
            println!("Diffs:");
            for i in 0..(path_len - 1) {
                println!("    Position {} to {}: {}", i, i+1, print_value_set(&counts[i]));
            }
        }
    }


// ======== Functions ========


    /// Applies a Path and set of Segments to an Alu and returns the result.
    #[allow(dead_code)]
    fn apply_path(caches: &mut Vec<SegmentCache>, path: &Path, start_alu: Alu) -> Result<Alu,()> {
        let mut alu = start_alu;
        for (i, v) in path.iter().enumerate() {
            alu = caches[i].apply_segment(alu, *v)?;
        }
        Ok(alu)
    }


    /// caches: the vector of SegmentCaches
    /// pos: the position of that vector we are evaluating
    /// start_alu: the starting Alu
    ///
    /// This evaluates possible inputs for a series of segments. It returns a list of
    /// input value sequences that will give valid results.
    #[allow(dead_code)]
    fn evaluate_to_end(caches: &mut Vec<SegmentCache>, pos: usize, start_alu: Alu) -> Vec<Path> {
        let mut answer: Vec<Path> = Vec::new();
        for input in (1..=9).rev() {
            let apply_result = caches[pos].apply_segment(start_alu, input);
            match apply_result {
                Err(()) => {}, // that failed... move on
                Ok(alu) => { // found an output
                    if pos + 1 == caches.len() {
                        // -- last one; check for validity --
                        if alu.valid() {
                            let path = Path::empty().append(input);
                            answer.push(path);
                        }
                    } else {
                        // -- not last one; recurse --
                        for tail in evaluate_to_end(caches, pos + 1, alu) {
                            answer.push(tail.prepend(input));
                        }
                    }
                },
            }
        }
        answer
    }



    /// caches: the vector of SegmentCaches
    /// pos: the position of that vector where we are analyzing.
    /// stop_pos: the position of that vector where we will stop.
    /// start_alu: the starting Alu
    /// num_results: the number of results we should attempt to obtain
    ///
    /// This evaluates possible inputs for a series of segments. It will consider possible
    /// inputs starting from 999... and working downward, skipping any that cause errors
    /// and continuing until it has collected num_results values OR all possible values.
    /// For each, it returns a (path, alu) pair.
    #[allow(dead_code)]
    fn evaluate_from_start(
        caches: &mut Vec<SegmentCache>,
        stop_pos: usize,
        start_alu: Alu,
        num_results: usize,
    ) -> Vec<(Path,Alu)> {
        assert!(caches.len() >= 1);
        assert!(stop_pos <= caches.len());
        let mut answer: Vec<(Path,Alu)> = Vec::new();
        evaluate_from_start_internal(caches, 0, stop_pos, start_alu, Path::empty(), num_results, &mut answer);
        answer
    }

    /// Internal recursive part of evaluate_from_start()
    fn evaluate_from_start_internal(
        caches: &mut Vec<SegmentCache>,
        pos: usize,
        stop_pos: usize,
        start_alu: Alu,
        path_so_far: Path,
        num_results: usize,
        answer: &mut Vec<(Path,Alu)>,
    ) {
        for input in (1..=9).rev() {
            let apply_result = caches[pos].apply_segment(start_alu, input);
            match apply_result {
                Err(()) => {}, // that failed... move on
                Ok(alu) => { // found an output
                    let path: Path = path_so_far.prepend(input);
                    if qualified(&path, alu) {
                        if pos + 1 == stop_pos {
                            // -- last one; return results --
                            answer.push((path, alu));
                            if answer.len() == num_results {
                                return;
                            }
                        } else {
                            // -- not last one; recurse --
                            evaluate_from_start_internal(caches, pos + 1, stop_pos, alu, path, num_results, answer);
                            if answer.len() == num_results {
                                return;
                            }
                        }
                    }
                },
            }
        }
    }


    fn print_value_set(set: &HashSet<Value>) -> String {
        let mut s: String = "".into();
        write!(s, "{{").unwrap();
        let mut first_item = true;
        for v in set.iter().sorted() {
            if !first_item {
                write!(s, ", ").unwrap();
            }
            first_item = false;
            write!(s, "{}", v).unwrap();
        }
        write!(s, "}}").unwrap();
        s
    }


    /// So, I have some guesses about what might be required. I'm going to have
    /// this function return true for the ones I want to explore further (and prune
    /// the rest).
    fn qualified(path: &Path, alu: Alu) -> bool {
        match path.len() {
            0 => true,
            1 => true,
            2 => true,
            3 => true,
            4 => true,
            5 => alu.values[1] != 1,
            6 => true,
            7 => true,
            8 => alu.values[1] != 1,
            9 => true,
            10 => alu.values[1] != 1,
            11 => alu.values[1] != 1,
            12 => alu.values[1] != 1,
            13 => true,
            14 => alu.values[3] == 0,
            _ => panic!("Invalid path length")
        }
    }

// ======== run() and main() ========


    fn run() -> Result<(),InputError> {
        let segments: Vec<Segment> = read_alu_file()?;

        let mut caches: Vec<SegmentCache> = segments.iter().map(|x| SegmentCache::new(x.clone())).collect();
        let min_val = 0;
        let max_val = 0;
        #[allow(unused_mut)]
            let mut valid_paths = PathSet::new();
        for a in min_val..=max_val {
            for b in min_val..=max_val {
                for c in min_val..=max_val {
                    for d in min_val..=max_val {
                        let start_alu = Alu{values: [a, b, c, d]};

                        // -- Work From Start --
                        let stop_pos = 14;
                        let num_results = 1;
                        let mut data = evaluate_from_start(&mut caches, stop_pos, start_alu, num_results);
                        println!("Got results from start:");
                        data.sort();
                        for (path, alu) in data.iter().rev() {
                            println!("    {} -> {}", path, alu);
                        }

                        // -- Work Toward End --
                        // for (start_path, alu) in data.iter().rev() {
                        //     let start_pos = caches.len() - 5;
                        //     let paths = evaluate_to_end(&mut caches, start_pos, *alu);
                        //     for end_path in paths.iter() {
                        //         valid_paths.add(start_path.concat(end_path));
                        //     }
                        // }

                    }
                }
            }
        }
        println!();
        valid_paths.print_all();
        println!();
        println!();
        valid_paths.print_analysis();


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

// ======== Tests ========

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_read_file() {
            let _ = read_alu_file().unwrap();
        }

        #[test]
        fn test_sort_path() {
            let mut paths: Vec<Path> = vec![
                "96158517619692",
                "96159617619692",
                "96154128619692",
                "96155228619692",
                "96156328619692",
                "96157428619692",
                "96158528619692",
                // "8517",
                // "9617",
                // "4128",
                // "5228",
                // "6328",
                // "7428",
                // "8528",
            ].iter().map(|x| Path{str: x.to_string()}).collect();
            paths.sort();
            for p in paths.iter() {
                println!("{}", p);
            }
        }

    }

    /*
    NOTES:
      For the last 2 digits, I tried all combinations from -10 to +20
      The ONLY input values that passed the checks were
        79
        35
        24
        13
        46
        57
        68
      Interestingly, all of those work with a starting value of [0,0,0,0].
    
        Analyzing 38232 paths:
        The top few are:
            9699979
            9699968
            9699957
            9699946
            9699935
            9699924
            9699913
            9699879
            9699868
            9699857
            9699846
            9699835
            9699824
            9699813
            9699779
        Frequencies:
            Position 0: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 1: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 2: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 3: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 4: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 5: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 6: {3, 4, 5, 6, 7, 8, 9}
        Diffs:
            Position 0 to 1: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 1 to 2: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 2 to 3: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 3 to 4: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 4 to 5: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 5 to 6: {1, 2, 3, 4, 5, 6, 7, 8, 9}
    
        I don't learn much more ending at position 13. But if I end anywhere BEFORE 13 then I get
        absolutely no valid paths. Apparently it's only in the last (and second-to-last?) place
        where we ever set z=0.
    
        FROM START:
           Assuming you begin with [0,0,0,0], the first Segment does the following:
               w -> set to input digit
               x -> set to 1
               y -> set to 9 + input digit
               z -> set to 9 + input digit
           The first, then second Segment does the following
               w -> set to 2nd input digit
               x -> set to 1
               y -> set to 2 + 2nd input digit
               z -> set to some bigger number
    
       I guessed 96489639919992 as a solution, but it was too high.
     */
}

// ======= part_b =======

mod part_b {
    use std::fmt;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::fmt::{Display, Formatter};
    use std::collections::{HashMap, HashSet};
    use itertools;
    use itertools::Itertools;
    use std::fmt::Write;
    use std::ops::Add;
    use nom::bytes::complete::tag as nom_tag;
    use nom::sequence::tuple as nom_tuple;
    use nom::branch::alt as nom_alt;
    use nom::character::complete::i64 as nom_value;


// ======== Reading Input ========

    /// An error that we can encounter when reading the input.
    #[derive(Debug)]
    enum InputError {
        IoError(std::io::Error),
        InvalidInstruction,
        NoStartingInputInstruction,
    }

    impl From<std::io::Error> for InputError {
        fn from(error: std::io::Error) -> Self {
            InputError::IoError(error)
        }
    }

    impl fmt::Display for InputError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InputError::IoError(err) => write!(f, "{}", err),
                InputError::InvalidInstruction => write!(f, "Invalid Instruction"),
                InputError::NoStartingInputInstruction => write!(f, "No starting input instruction"),
            }
        }
    }

    /// Read in the input file.
    fn read_alu_file() -> Result<Vec<Segment>, InputError> {
        // --- open file ---
        let filename = "input/2021/input_24.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        // --- read instructions ---
        let mut segments: Vec<Segment> = Vec::new();
        let mut input_register: Option<Register> = None;
        let mut computes: Vec<Compute> = Vec::new();
        for line in lines {
            let text: String = line?;
            match Instruction::parse(&text) {
                Ok(("", instruction)) => {  // the parse was OK
                    match instruction {
                        Instruction::Input(reg) => {
                            // -- Start a new segment --
                            if let Some(input) = input_register {
                                segments.push(Segment{input, computes: computes.clone()});
                            }
                            input_register = Some(reg);
                            computes.clear();
                        }
                        Instruction::Compute(compute) => {
                            if input_register.is_none() {
                                return Err(InputError::NoStartingInputInstruction);
                            }
                            computes.push(compute)
                        }
                    }
                },
                Ok((_, _)) => return Err(InputError::InvalidInstruction), // if extra stuff on the line
                Err(_) => return Err(InputError::InvalidInstruction), // if parse failed
            };
        }
        match input_register {
            None => return Err(InputError::NoStartingInputInstruction),
            Some(input) => segments.push(Segment{input, computes: computes.clone()}),
        }

        // --- return result ---
        Ok(segments)
    }



// ======== Types ========

    type Value = i64;

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    enum Register {
        W, X, Y, Z
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    enum Parameter {
        Constant(Value),
        Register(Register),
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    enum Compute {
        Add(Register, Parameter),
        Mul(Register, Parameter),
        Div(Register, Parameter),
        Mod(Register, Parameter),
        Eql(Register, Parameter),
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    enum Instruction {
        Input(Register),
        Compute(Compute),
    }

    /// One segment of instructions consists of one Input instruction followed
    /// by a series of Compute instructions.
    #[derive(Debug, Clone)]
    struct Segment {
        input: Register,
        computes: Vec<Compute>,
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
    struct Alu {
        values: [Value; Register::NUM_ITEMS],
    }


    /// Cache for ONE particular segment.
    struct SegmentCache {
        segment: Segment,
        cache: HashMap<(Alu, Value), Result<Alu,()>>, // map from (start_alu, input_value) to output Alu
    }

    /// Represents an input number.
    #[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd)]
    struct Path {
        str: String,
    }

    /// A class I am creating to track a bunch of valid paths and print out some interesting
    /// information about them.
    struct PathSet {
        paths: HashSet<Path>,
        path_len: Option<usize>,
    }

// ======== Implementations ========

    impl Register {
        const NUM_ITEMS: usize = 4;

        fn id(&self) -> usize {
            match self {
                Register::W => 0,
                Register::X => 1,
                Register::Y => 2,
                Register::Z => 3,
            }
        }

        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom_alt((
                nom_tag("w"),
                nom_tag("x"),
                nom_tag("y"),
                nom_tag("z"),
            ))(input).map(|(rest, res)| (rest, match res {
                "w" => Register::W,
                "x" => Register::X,
                "y" => Register::Y,
                "z" => Register::Z,
                _ => panic!("should never happen")
            }))
        }
    }

    impl Display for Register {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}", match self {
                Register::W => "w",
                Register::X => "x",
                Register::Y => "y",
                Register::Z => "z",
            })
        }
    }



    impl Parameter {
        fn parse_constant(input: &str) -> nom::IResult<&str, Self> {
            nom_value(input).map(|(rest, x)| (rest, Parameter::Constant(x)))
        }

        fn parse_register(input: &str) -> nom::IResult<&str, Self> {
            Register::parse(input).map(|(rest, x)| (rest, Parameter::Register(x)))
        }

        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom_alt((
                Parameter::parse_constant,
                Parameter::parse_register,
            ))(input)
        }
    }
    impl Display for Parameter {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Parameter::Constant(val) => write!(f, "{}", val),
                Parameter::Register(reg) => write!(f, "{}", reg),
            }
        }
    }

    impl Compute {
        fn parse_add(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("add "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Add(reg, param)))
        }
        fn parse_mul(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("mul "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Mul(reg, param)))
        }
        fn parse_div(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("div "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Div(reg, param)))
        }
        fn parse_mod(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("mod "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Mod(reg, param)))
        }
        fn parse_eql(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("eql "),
                Register::parse,
                nom_tag(" "),
                Parameter::parse,
            ))(input).map(|(rest, (_, reg, _, param))| (rest, Compute::Eql(reg, param)))
        }

        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom_alt((
                Compute::parse_add,
                Compute::parse_mul,
                Compute::parse_div,
                Compute::parse_mod,
                Compute::parse_eql,
            ))(input)
        }
    }

    impl Display for Compute {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Compute::Add(reg, param) => write!(f, "add {} {}", reg, param),
                Compute::Mul(reg, param) => write!(f, "mul {} {}", reg, param),
                Compute::Div(reg, param) => write!(f, "div {} {}", reg, param),
                Compute::Mod(reg, param) => write!(f, "mod {} {}", reg, param),
                Compute::Eql(reg, param) => write!(f, "eql {} {}", reg, param),
            }
        }
    }


    impl Instruction {
        fn parse_inp(input: &str) -> nom::IResult<&str, Self> {
            nom_tuple((
                nom_tag("inp "),
                Register::parse,
            ))(input).map(|(rest, (_, reg))| (rest, Instruction::Input(reg)))
        }
        fn parse_compute(input: &str) -> nom::IResult<&str, Self> {
            Compute::parse(input).map(|(rest, ci)| (rest, Instruction::Compute(ci)))
        }

        fn parse(input: &str) -> nom::IResult<&str, Self> {
            nom_alt((
                Instruction::parse_inp,
                Instruction::parse_compute,
            ))(input)
        }

    }

    impl Display for Instruction {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Instruction::Input(reg) => write!(f, "inp {}", reg),
                Instruction::Compute(ci) => write!(f, "{}", ci),
            }
        }
    }


    impl Segment {
        /// Applies the segment. Either returns Ok(alu) for the Alu that results OR
        /// returns Err(()) if the computation would result in an illegal operation.
        fn apply(&self, start_alu: Alu, input: Value) -> Result<Alu,()> {
            let mut alu = start_alu;
            alu = alu.eval_input(self.input, input);
            for compute in self.computes.iter() {
                alu = alu.eval_compute(*compute)?;
            }
            Ok(alu)
        }
    }


    impl Alu {
        /// Given a register, tells the value stored in that register.
        fn value_in(&self, reg: Register) -> Value {
            self.values[reg.id()]
        }

        /// Given a param, tells the value of that parameter.
        fn value_of(&self, param: Parameter) -> Value {
            match param {
                Parameter::Constant(val) => val,
                Parameter::Register(reg) => self.value_in(reg),
            }
        }


        /// Returns true if the ALU is a valid final accept state.
        fn valid(&self) -> bool {
            self.values[Register::Z.id()] == 0
        }

        /// Executes any instruction OTHER than input. Either returns the Alu
        /// that results OR Err(()) if the computation hit an invalid snag.
        fn eval_compute(&self, compute: Compute) -> Result<Alu, ()> {
            let mut values: [Value; Register::NUM_ITEMS] = self.values.clone();
            match compute {
                Compute::Add(reg, param) => {
                    values[reg.id()] = self.value_in(reg) + self.value_of(param);
                },
                Compute::Mul(reg, param) => {
                    values[reg.id()] = self.value_in(reg) * self.value_of(param);
                },
                Compute::Div(reg, param) => {
                    let p = self.value_of(param);
                    if p == 0 {
                        return Err(());
                    }
                    values[reg.id()] = self.value_in(reg) / p;
                },
                Compute::Mod(reg, param) => {
                    let r = self.value_in(reg);
                    let p = self.value_of(param);
                    if r < 0 || p <= 0 {
                        return Err(());
                    }
                    values[reg.id()] = r % p;
                },
                Compute::Eql(reg, param) => {
                    values[reg.id()] = if self.value_in(reg) == self.value_of(param) {1} else {0};
                },
            }
            Ok(Alu{values})
        }

        fn eval_input(&self, input_reg: Register, input: Value) -> Alu {
            let mut values: [Value; Register::NUM_ITEMS] = self.values.clone();
            values[input_reg.id()] = input;
            Alu{values}
        }
    }

    impl Display for Alu {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "[{} {} {} {}]", self.values[0], self.values[1], self.values[2], self.values[3])
        }
    }


    impl SegmentCache {
        fn new(segment: Segment) -> Self {
            SegmentCache{segment, cache: HashMap::new()}
        }

        fn apply_segment(&mut self, start_alu: Alu, input: Value) -> Result<Alu,()> {
            match self.cache.get(&(start_alu, input)) {
                Some(alu) => *alu,
                None => {
                    let answer: Result<Alu,()> = self.segment.apply(start_alu, input);
                    self.cache.insert((start_alu, input), answer);
                    answer
                },
            }
        }
    }


    impl Path {
        /// Return an empty path.
        fn empty() -> Self {
            Path{str: String::new()}
        }

        fn iter(&self) -> PathIterator {
            PathIterator{chars: self.str.chars()}
        }

        fn len(&self) -> usize {
            self.str.len()
        }

        fn prepend(&self, v: Value) -> Path {
            assert!(v >= 1 && v <= 9);
            Path{str: self.str.to_owned().add(&v.to_string())}
        }

        fn append(&self, v: Value) -> Path {
            assert!(v >= 1 && v <= 9);
            Path{str: v.to_string().add(&self.str)}
        }

        #[allow(dead_code)]
        fn concat(&self, other: &Path) -> Path {
            Path{str: self.str.to_owned().add(&other.str)}
        }
    }
    impl Display for Path {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.str)
        }
    }
    struct PathIterator<'a> {
        chars: core::str::Chars<'a>,
    }
    impl<'a> Iterator for PathIterator<'a> {
        type Item = &'a Value;

        fn next(&mut self) -> Option<&'a Value> {
            match self.chars.next() {
                None => None,
                Some(c) => {
                    Some(match c {
                        '1' => &1,
                        '2' => &2,
                        '3' => &3,
                        '4' => &4,
                        '5' => &5,
                        '6' => &6,
                        '7' => &7,
                        '8' => &8,
                        '9' => &9,
                        _ => panic!()
                    })
                },
            }
        }
    }


    impl PathSet {
        fn new() -> Self {
            PathSet{paths: HashSet::new(), path_len: None}
        }

        /// Add a path into PathSet.
        #[allow(dead_code)]
        fn add(&mut self, path: Path) {
            match self.path_len {
                None => self.path_len = Some(path.len()),
                Some(len) => if path.len() != len {panic!("PathSet with paths of different lengths")},
            }
            self.paths.insert(path);
        }

        fn print_all(&self) {
            println!("The valid paths are:");
            for path in self.paths.iter().sorted() {
                println!("{}", path);
            }
        }

        fn print_analysis(&self) {
            if self.paths.len() == 0 {
                println!("There were no paths!");
                return;
            }
            println!("Analyzing {} paths:", self.paths.len());
            let path_len = self.path_len.unwrap();
            println!("The top few are:");
            for path in self.paths.iter().sorted().rev().take(15) {
                println!("    {}", path);
            }

            let mut counts: Vec<HashSet<Value>> = (0..path_len).map(|_| HashSet::new()).collect();
            let mut diffs: Vec<HashSet<Value>> = (0..(path_len - 1)).map(|_| HashSet::new()).collect();
            for path in self.paths.iter() {
                let mut prev_v: Option<Value> = None; // None will never be used
                for (i, v) in path.iter().enumerate() {
                    counts[i].insert(*v);
                    if i > 0 {
                        let diff = prev_v.unwrap() - v;
                        diffs[i-1].insert(diff);
                    }
                    prev_v = Some(*v);
                }
            }

            println!("Frequencies:");
            for i in 0..path_len {
                println!("    Position {}: {}", i, print_value_set(&counts[i]));
            }
            println!("Diffs:");
            for i in 0..(path_len - 1) {
                println!("    Position {} to {}: {}", i, i+1, print_value_set(&counts[i]));
            }
        }
    }


// ======== Functions ========


    /// Applies a Path and set of Segments to an Alu and returns the result.
    #[allow(dead_code)]
    fn apply_path(caches: &mut Vec<SegmentCache>, path: &Path, start_alu: Alu) -> Result<Alu,()> {
        let mut alu = start_alu;
        for (i, v) in path.iter().enumerate() {
            alu = caches[i].apply_segment(alu, *v)?;
        }
        Ok(alu)
    }


    /// caches: the vector of SegmentCaches
    /// pos: the position of that vector we are evaluating
    /// start_alu: the starting Alu
    ///
    /// This evaluates possible inputs for a series of segments. It returns a list of
    /// input value sequences that will give valid results.
    #[allow(dead_code)]
    fn evaluate_to_end(caches: &mut Vec<SegmentCache>, pos: usize, start_alu: Alu) -> Vec<Path> {
        let mut answer: Vec<Path> = Vec::new();
        for input in (1..=9).rev() {
            let apply_result = caches[pos].apply_segment(start_alu, input);
            match apply_result {
                Err(()) => {}, // that failed... move on
                Ok(alu) => { // found an output
                    if pos + 1 == caches.len() {
                        // -- last one; check for validity --
                        if alu.valid() {
                            let path = Path::empty().append(input);
                            answer.push(path);
                        }
                    } else {
                        // -- not last one; recurse --
                        for tail in evaluate_to_end(caches, pos + 1, alu) {
                            answer.push(tail.prepend(input));
                        }
                    }
                },
            }
        }
        answer
    }



    /// caches: the vector of SegmentCaches
    /// pos: the position of that vector where we are analyzing.
    /// stop_pos: the position of that vector where we will stop.
    /// start_alu: the starting Alu
    /// num_results: the number of results we should attempt to obtain
    ///
    /// This evaluates possible inputs for a series of segments. It will consider possible
    /// inputs starting from 999... and working downward, skipping any that cause errors
    /// and continuing until it has collected num_results values OR all possible values.
    /// For each, it returns a (path, alu) pair.
    #[allow(dead_code)]
    fn evaluate_from_start(
        caches: &mut Vec<SegmentCache>,
        stop_pos: usize,
        start_alu: Alu,
        num_results: usize,
    ) -> Vec<(Path,Alu)> {
        assert!(caches.len() >= 1);
        assert!(stop_pos <= caches.len());
        let mut answer: Vec<(Path,Alu)> = Vec::new();
        evaluate_from_start_internal(caches, 0, stop_pos, start_alu, Path::empty(), num_results, &mut answer);
        answer
    }

    /// Internal recursive part of evaluate_from_start()
    fn evaluate_from_start_internal(
        caches: &mut Vec<SegmentCache>,
        pos: usize,
        stop_pos: usize,
        start_alu: Alu,
        path_so_far: Path,
        num_results: usize,
        answer: &mut Vec<(Path,Alu)>,
    ) {
        for input in 1..=9 {
            let apply_result = caches[pos].apply_segment(start_alu, input);
            match apply_result {
                Err(()) => {}, // that failed... move on
                Ok(alu) => { // found an output
                    let path: Path = path_so_far.prepend(input);
                    if qualified(&path, alu) {
                        if pos + 1 == stop_pos {
                            // -- last one; return results --
                            answer.push((path, alu));
                            if answer.len() == num_results {
                                return;
                            }
                        } else {
                            // -- not last one; recurse --
                            evaluate_from_start_internal(caches, pos + 1, stop_pos, alu, path, num_results, answer);
                            if answer.len() == num_results {
                                return;
                            }
                        }
                    }
                },
            }
        }
    }


    fn print_value_set(set: &HashSet<Value>) -> String {
        let mut s: String = "".into();
        write!(s, "{{").unwrap();
        let mut first_item = true;
        for v in set.iter().sorted() {
            if !first_item {
                write!(s, ", ").unwrap();
            }
            first_item = false;
            write!(s, "{}", v).unwrap();
        }
        write!(s, "}}").unwrap();
        s
    }


    /// So, I have some guesses about what might be required. I'm going to have
    /// this function return true for the ones I want to explore further (and prune
    /// the rest).
    fn qualified(path: &Path, alu: Alu) -> bool {
        match path.len() {
            0 => true,
            1 => true,
            2 => true,
            3 => true,
            4 => true,
            5 => alu.values[1] != 1,
            6 => true,
            7 => true,
            8 => alu.values[1] != 1,
            9 => true,
            10 => alu.values[1] != 1,
            11 => alu.values[1] != 1,
            12 => alu.values[1] != 1,
            13 => true,
            14 => alu.values[3] == 0,
            _ => panic!("Invalid path length")
        }
    }

// ======== run() and main() ========


    fn run() -> Result<(),InputError> {
        let segments: Vec<Segment> = read_alu_file()?;

        let mut caches: Vec<SegmentCache> = segments.iter().map(|x| SegmentCache::new(x.clone())).collect();
        let min_val = 0;
        let max_val = 0;
        #[allow(unused_mut)]
            let mut valid_paths = PathSet::new();
        for a in min_val..=max_val {
            for b in min_val..=max_val {
                for c in min_val..=max_val {
                    for d in min_val..=max_val {
                        let start_alu = Alu{values: [a, b, c, d]};

                        // -- Work From Start --
                        let stop_pos = 14;
                        let num_results = 1;
                        let mut data = evaluate_from_start(&mut caches, stop_pos, start_alu, num_results);
                        println!("Got results from start:");
                        data.sort();
                        for (path, alu) in data.iter().rev() {
                            println!("    {} -> {}", path, alu);
                        }

                        // -- Work Toward End --
                        // for (start_path, alu) in data.iter().rev() {
                        //     let start_pos = caches.len() - 5;
                        //     let paths = evaluate_to_end(&mut caches, start_pos, *alu);
                        //     for end_path in paths.iter() {
                        //         valid_paths.add(start_path.concat(end_path));
                        //     }
                        // }

                    }
                }
            }
        }
        println!();
        valid_paths.print_all();
        println!();
        println!();
        valid_paths.print_analysis();


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

// ======== Tests ========

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_read_file() {
            let _ = read_alu_file().unwrap();
        }

        #[test]
        fn test_sort_path() {
            let mut paths: Vec<Path> = vec![
                "96158517619692",
                "96159617619692",
                "96154128619692",
                "96155228619692",
                "96156328619692",
                "96157428619692",
                "96158528619692",
            ].iter().map(|x| Path{str: x.to_string()}).collect();
            paths.sort();
            for p in paths.iter() {
                println!("{}", p);
            }
        }

    }

    /*
    NOTES:
      For the last 2 digits, I tried all combinations from -10 to +20
      The ONLY input values that passed the checks were
        79
        35
        24
        13
        46
        57
        68
      Interestingly, all of those work with a starting value of [0,0,0,0].
    
        Analyzing 38232 paths:
        The top few are:
            9699979
            9699968
            9699957
            9699946
            9699935
            9699924
            9699913
            9699879
            9699868
            9699857
            9699846
            9699835
            9699824
            9699813
            9699779
        Frequencies:
            Position 0: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 1: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 2: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 3: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 4: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 5: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 6: {3, 4, 5, 6, 7, 8, 9}
        Diffs:
            Position 0 to 1: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 1 to 2: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 2 to 3: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 3 to 4: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 4 to 5: {1, 2, 3, 4, 5, 6, 7, 8, 9}
            Position 5 to 6: {1, 2, 3, 4, 5, 6, 7, 8, 9}
    
        I don't learn much more ending at position 13. But if I end anywhere BEFORE 13 then I get
        absolutely no valid paths. Apparently it's only in the last (and second-to-last?) place
        where we ever set z=0.
    
        FROM START:
           Assuming you begin with [0,0,0,0], the first Segment does the following:
               w -> set to input digit
               x -> set to 1
               y -> set to 9 + input digit
               z -> set to 9 + input digit
           The first, then second Segment does the following
               w -> set to 2nd input digit
               x -> set to 1
               y -> set to 2 + 2nd input digit
               z -> set to some bigger number
    
       I guessed 96489639919992 as a solution, but it was too high.
     */
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
