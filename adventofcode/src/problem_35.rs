use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;


/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    UnexpectedEnd,
    UnexpectedChar(char,char), // got first one; expected second
    ExpectedItem(char), // got this, expected '[' or digit
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
            InputError::UnexpectedEnd => write!(f, "End of line but expected more."),
            InputError::UnexpectedChar(c,exp) => write!(f, "Expected {} but got {}.", exp, c),
            InputError::ExpectedItem(c) => write!(f, "Expected '[' or digit but got {}.", c),
        }
    }
}



/// Read in the input file.
fn read_snailfish_file() -> Result<Vec<String>, InputError> {
    let filename = "data/2021/day/18/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();

    let mut output: Vec<String> = Vec::new();
    for line in lines {
        let text: String = line?;
        output.push(text);
    }
    Ok(output)
}


struct CharStream<'a> {
    iter: Box<dyn Iterator<Item=char> + 'a>,
    peeked: Option<char>,
    allow_multiple_digits: bool,
}

impl<'a> CharStream<'a> {
    fn new(s: &'a str, allow_multiple_digits: bool) -> CharStream<'a> {
        CharStream{iter: Box::new(s.chars()), peeked: None, allow_multiple_digits}
    }

    fn get_allow_multiple_digits(&self) -> bool {
        self.allow_multiple_digits
    }

    /// Returns the next char without consuming it, or InputError::UnexpectedEnd if there
    /// isn't a character to read.
    fn peek(&mut self) -> Result<char, InputError> {
        Ok(match self.peeked {
            Some(c) => c,
            None => {
                let c = self.get_next()?;
                self.peeked = Some(c);
                c
            }
        })
    }

    /// Consumes one char. Returns it, or InputError::UnexpectedEnd if there isn't a character to read.
    fn get_next(&mut self) -> Result<char, InputError> {
        match self.peeked {
            Some(c) => {
                self.peeked = None;
                Ok(c)
            },
            None => {
                self.iter.next().ok_or(InputError::UnexpectedEnd)
            },
        }
    }

    /// Consumes one char. Returns () if it matches expected, or InputError::UnexpectedChar
    /// if it doesn't or InputError::UnexpectedEnd if there isn't a character to read.
    fn expect(&mut self, expected: char) -> Result<(), InputError> {
        let c = self.get_next()?;
        if c == expected {
            Ok(())
        } else {
            Err(InputError::UnexpectedChar(c,expected))
        }
    }
}



#[derive(Debug, Eq, PartialEq)]
enum SnailfishItem {
    RegularNumber(u32),
    Pair(SnailfishPair),
}

#[derive(Debug, Eq, PartialEq)]
struct SnailfishPair {
    left: Rc<SnailfishItem>,
    right: Rc<SnailfishItem>,
}

#[derive(Debug, Eq, PartialEq)]
struct SnailfishNumber {
    top_pair: Rc<SnailfishPair>,
}


#[derive(Debug)]
enum ItemExplodeOutcome {
    None,
    Some(SnailfishItem),
    Exploding(u32, u32), // value_going_left, value_going_right
    ExplodingLeft(u32, SnailfishItem), // value_going_left, item
    ExplodingRight(u32, SnailfishItem), // value_going_right, item
}

#[derive(Debug)]
enum PairExplodeOutcome {
    None,
    Some(SnailfishPair),
    ExplodingLeft(u32, SnailfishPair),
    ExplodingRight(u32, SnailfishPair),
}


impl SnailfishItem {
    fn new_num(value: u32) -> Self {
        SnailfishItem::RegularNumber(value)
    }

    fn new_pair(pair: SnailfishPair) -> Self {
        SnailfishItem::Pair(pair)
    }

    fn read_from(stream: &mut CharStream) -> Result<Self, InputError> {
        let next_c = stream.peek()?;
        match next_c {
            '[' => {
                let pair: SnailfishPair = SnailfishPair::read_from(stream)?;
                Ok(SnailfishItem::new_pair(pair))
            },
            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                let num: u32;
                if stream.get_allow_multiple_digits() {
                    let mut build_num: u32 = 0;
                    loop {
                        match stream.peek()? {
                            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {},
                            _ => break, // exit the loop when the next thing isn't a digit
                        }
                        let digit: u32 = stream.get_next()?.to_string().parse()?;
                        build_num *= 10;
                        build_num += digit;
                    }
                    num = build_num;
                } else {
                    num = stream.get_next()?.to_string().parse()?;
                }
                Ok(SnailfishItem::new_num(num))
            },
            _ => Err(InputError::ExpectedItem(next_c)),
        }
    }

    /// Returns a new SnailfishItem whose left-most element has going_right added to it.
    fn add_going_right(&self, going_right: u32) -> Self {
        match self {
            SnailfishItem::RegularNumber(old_val) => SnailfishItem::new_num(old_val + going_right),
            SnailfishItem::Pair(pair) => SnailfishItem::new_pair(pair.add_going_right(going_right)),
        }
    }

    /// Returns a new SnailfishItem whose rught-most element has going_left added to it.
    fn add_going_left(&self, going_left: u32) -> Self {
        match self {
            SnailfishItem::RegularNumber(old_val) => SnailfishItem::new_num(old_val + going_left),
            SnailfishItem::Pair(pair) => SnailfishItem::new_pair(pair.add_going_left(going_left)),
        }
    }

    /// If this item can be reduced by exploding, returns the single-step
    /// reduction of it. If it can't, returns None.
    fn explode_once(&self, level: u32) -> ItemExplodeOutcome {
        match self {
            SnailfishItem::RegularNumber(_) => return ItemExplodeOutcome::None,
            SnailfishItem::Pair(rc_pair) => {
                if level == 3 {
                    let going_left: u32;
                    if let SnailfishItem::RegularNumber(x) = *(*rc_pair).left {
                        going_left = x;
                    } else {
                        panic!("We were promised the 4th level would only contain numbers.")
                    }
                    let going_right: u32;
                    if let SnailfishItem::RegularNumber(x) = *(*rc_pair).right {
                        going_right = x;
                    } else {
                        panic!("We were promised the 4th level would only contain numbers.")
                    }
                    return ItemExplodeOutcome::Exploding(going_left, going_right)
                } else {
                    let explode_outcome = (*rc_pair).explode_once(level + 1);
                    match explode_outcome {
                        PairExplodeOutcome::None => {
                            return ItemExplodeOutcome::None
                        },
                        PairExplodeOutcome::Some(pair) => {
                            return ItemExplodeOutcome::Some(SnailfishItem::new_pair(pair))
                        },
                        PairExplodeOutcome::ExplodingLeft(going_left, pair) => {
                            return ItemExplodeOutcome::ExplodingLeft(going_left, SnailfishItem::new_pair(pair))
                        },
                        PairExplodeOutcome::ExplodingRight(going_right, pair) => {
                            return ItemExplodeOutcome::ExplodingRight(going_right, SnailfishItem::new_pair(pair))
                        },
                    }
                }
            },
        }
    }

    /// If this item can be reduced by splitting, returns the single-step
    /// reduction of it. If it can't, returns None.
    fn split_once(&self) -> Option<Self> {
        match self {
            SnailfishItem::RegularNumber(val) => {
                if val < &10u32 {
                    None
                } else {
                    let round_down: u32 = val / 2;
                    let round_up: u32 = (val / 2) + (val % 2);
                    let pair = SnailfishPair::new(
                        &Rc::new(SnailfishItem::RegularNumber(round_down)),
                        &Rc::new(SnailfishItem::RegularNumber(round_up))
                    );
                    Some(SnailfishItem::new_pair(pair))
                }
            },
            SnailfishItem::Pair(rc_pair) => {
                if let Some(pair) = (*rc_pair).split_once() {
                    Some(SnailfishItem::new_pair(pair))
                } else {
                    None
                }
            },
        }
    }

    fn magnitude(&self) -> u32 {
        match self {
            SnailfishItem::RegularNumber(val) => *val,
            SnailfishItem::Pair(pair) => pair.magnitude(),
        }
    }
}

impl SnailfishPair {
    fn new(left: &Rc<SnailfishItem>, right: &Rc<SnailfishItem>) -> Self {
        SnailfishPair{left: left.clone(), right: right.clone()}
    }

    fn read_from(stream: &mut CharStream) -> Result<Self, InputError> {
        stream.expect('[')?;
        let left: SnailfishItem = SnailfishItem::read_from(stream)?;
        stream.expect(',')?;
        let right: SnailfishItem = SnailfishItem::read_from(stream)?;
        stream.expect(']')?;
        Ok(SnailfishPair::new(&Rc::new(left), &Rc::new(right)))
    }

    /// Returns a new SnailfishPair whose left-most element has going_right added to it.
    fn add_going_right(&self, going_right: u32) -> Self {
        SnailfishPair::new(&Rc::new(self.left.add_going_right(going_right)), &self.right.clone())
    }

    /// Returns a new SnailfishPair whose right-most element has going_left added to it.
    fn add_going_left(&self, going_left: u32) -> Self {
        SnailfishPair::new(&self.left.clone(), &Rc::new(self.right.add_going_left(going_left)))
    }

    /// If this pair can be reduced by exploding, returns the single-step
    /// reduction of it. If it can't, returns None.
    fn explode_once(&self, level: u32) -> PairExplodeOutcome {

        // --- see if the left will explode ---
        let explode_outcome = self.left.explode_once(level);
        match explode_outcome {
            ItemExplodeOutcome::Some(item) => {
                return PairExplodeOutcome::Some(SnailfishPair::new(&Rc::new(item), &self.right))
            },
            ItemExplodeOutcome::Exploding(going_left, going_right) => {
                // explosion which we can apply to our right, and must explode to our left
                let new_left = SnailfishItem::new_num(0);
                let new_right = self.right.add_going_right(going_right);
                let new_pair = SnailfishPair::new(&Rc::new(new_left), &Rc::new(new_right));
                return PairExplodeOutcome::ExplodingLeft(going_left, new_pair)
            },
            ItemExplodeOutcome::ExplodingLeft(going_left, new_left_item) => {
                // our left side is spitting out stuff going left
                let new_pair = SnailfishPair::new(&Rc::new(new_left_item), &self.right.clone());
                return PairExplodeOutcome::ExplodingLeft(going_left, new_pair)
            },
            ItemExplodeOutcome::ExplodingRight(going_right, new_left) => {
                // our left side is spitting out stuff going right
                let new_right = self.right.add_going_right(going_right);
                let new_pair = SnailfishPair::new(&Rc::new(new_left), &Rc::new(new_right));
                return PairExplodeOutcome::Some(new_pair);
            },
            ItemExplodeOutcome::None => {}, // Not resolved; we will move on to the right side
        }

        // --- see if the right will explode ---
        let explode_outcome = self.right.explode_once(level);
        match explode_outcome {
            ItemExplodeOutcome::Some(item) => {
                return PairExplodeOutcome::Some(SnailfishPair::new(&self.left, &Rc::new(item)))
            },
            ItemExplodeOutcome::Exploding(going_left, going_right) => {
                // explosion which we can apply to our left, and must explode to our right
                let new_left = self.left.add_going_left(going_left);
                let new_right = SnailfishItem::new_num(0);
                let new_pair = SnailfishPair::new(&Rc::new(new_left), &Rc::new(new_right));
                return PairExplodeOutcome::ExplodingRight(going_right, new_pair)
            }
            ItemExplodeOutcome::ExplodingLeft(going_left, new_right) => {
                // our right side is spitting out stuff going left
                let new_left = self.left.add_going_left(going_left);
                let new_pair = SnailfishPair::new(&Rc::new(new_left), &Rc::new(new_right));
                return PairExplodeOutcome::Some(new_pair);
            },
            ItemExplodeOutcome::ExplodingRight(going_right, new_right_item) => {
                // our right side is spitting out stuff going right
                let new_pair = SnailfishPair::new(&self.left.clone(), &Rc::new(new_right_item));
                return PairExplodeOutcome::ExplodingRight(going_right, new_pair)
            },
            ItemExplodeOutcome::None => {}, // Not resolved; we will must not be able to explode
        }

        // --- apparently neither one will explode ---
        return PairExplodeOutcome::None;
    }

    /// If this pair can be reduced by splitting, returns the single-step
    /// reduction of it. If it can't, returns None.
    fn split_once(&self) -> Option<SnailfishPair> {
        if let Some(item) = self.left.split_once() {
            return Some(SnailfishPair::new(&Rc::new(item), &self.right))
        } else if let Some(item) = self.right.split_once() {
            return Some(SnailfishPair::new(&self.left, &Rc::new(item)))
        } else {
            None
        }
    }

    fn magnitude(&self) -> u32 {
        (*self.left).magnitude() * 3 + (*self.right).magnitude() * 2
    }
}

impl SnailfishNumber {
    fn new(top_pair: &Rc<SnailfishPair>) -> Self {
        SnailfishNumber{top_pair: top_pair.clone()}
    }

    /// Parse a string to return a SnailfishNumber or an InputError
    fn parse(s: &str) -> Result<Self, InputError> {
        let mut stream: CharStream = CharStream::new(s, false);
        let top_pair: SnailfishPair = SnailfishPair::read_from(&mut stream)?;
        Ok(SnailfishNumber::new(&Rc::new(top_pair)))
    }

    /// Parse a string to return a SnailfishNumber which might not be reduced.
    #[allow(dead_code)]
    fn parse_unreduced(s: &str) -> Result<Self, InputError> {
        let mut stream: CharStream = CharStream::new(s, true);
        let top_pair: SnailfishPair = SnailfishPair::read_from(&mut stream)?;
        Ok(SnailfishNumber::new(&Rc::new(top_pair)))
    }

    /// Perform a single step of reduction. Returns the new SnailfishNumber if it
    /// reduced, or None if it was already fully reduced.
    fn reduce_step(&self) -> Option<SnailfishNumber> {
        // --- Check for exploding ---
        let explode_outcome = (*self.top_pair).explode_once(0);
        match explode_outcome {
            PairExplodeOutcome::Some(pair) |
            PairExplodeOutcome::ExplodingLeft(_, pair) |
            PairExplodeOutcome::ExplodingRight(_, pair) => {
                return Some(SnailfishNumber::new(&Rc::new(pair)));
            }
            PairExplodeOutcome::None => {} // Didn't explode, so let's go on
        }

        // --- Check for splitting ---
        if let Some(pair) = (*self.top_pair).split_once() {
            return Some(SnailfishNumber::new(&Rc::new(pair)))
        }

        // --- Give up on reducing ---
        return None
    }

    // Fully reduce this SnailfishNumber.
    fn reduce(&self) -> SnailfishNumber {
        let mut s_num_best = self.clone();
        loop {
            if let Some(s_num) = s_num_best.reduce_step() {
                s_num_best = s_num;
            } else {
                return s_num_best;
            }
        }
    }

    // This method is terrible which is a symptom of poor object design.
    fn add(&self, other: &SnailfishNumber) -> SnailfishNumber {
        let i1: &Rc<SnailfishItem> = &((*self.top_pair).left.clone());
        let i2: &Rc<SnailfishItem> = &((*self.top_pair).right.clone());
        let i3: &Rc<SnailfishItem> = &((*other.top_pair).left.clone());
        let i4: &Rc<SnailfishItem> = &((*other.top_pair).right.clone());
        let a_pair: SnailfishPair = SnailfishPair::new(i1, i2);
        let b_pair: SnailfishPair = SnailfishPair::new(i3, i4);
        let a_item: SnailfishItem = SnailfishItem::new_pair(a_pair);
        let b_item: SnailfishItem = SnailfishItem::new_pair(b_pair);
        let top_pair: SnailfishPair = SnailfishPair::new(
            &Rc::new(a_item),
            &Rc::new(b_item)
        );
        let sum = SnailfishNumber::new(&Rc::new(top_pair));
        sum.reduce()
    }

    fn magnitude(&self) -> u32 {
        (*self.top_pair).magnitude()
    }
}


impl fmt::Display for SnailfishItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SnailfishItem::RegularNumber(val) => write!(f, "{}", val),
            SnailfishItem::Pair(pair) => write!(f, "{}", pair),
        }
    }
}
impl fmt::Display for SnailfishPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{}]", self.left, self.right)
    }
}
impl fmt::Display for SnailfishNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.top_pair)
    }
}
impl fmt::Display for ItemExplodeOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ItemExplodeOutcome::None => write!(f, "None"),
            ItemExplodeOutcome::Some(item) => write!(f, "Some({})", item),
            ItemExplodeOutcome::Exploding(l,r) => write!(f, "Exploding({},{})", l, r),
            ItemExplodeOutcome::ExplodingLeft(l,it) => write!(f, "ExplodingLeft({},{})", l, it),
            ItemExplodeOutcome::ExplodingRight(r,it) => write!(f, "ExplodingRight({},{})", r, it),
        }
    }
}
impl fmt::Display for PairExplodeOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PairExplodeOutcome::None => write!(f, "None"),
            PairExplodeOutcome::Some(pair) => write!(f, "Some({})", pair),
            PairExplodeOutcome::ExplodingLeft(l,it) => write!(f, "ExplodingLeft({},{})", l, it),
            PairExplodeOutcome::ExplodingRight(r,it) => write!(f, "ExplodingRight({},{})", r, it),
        }
    }
}


impl Clone for SnailfishNumber {
    fn clone(&self) -> Self {
        SnailfishNumber::new(&self.top_pair.clone())
    }
}


fn run() -> Result<(),InputError> {
    let lines = read_snailfish_file()?;
    let mut running_sum: Option<SnailfishNumber> = None;
    for line in lines {
        let s_num = SnailfishNumber::parse(&line)?;
        match running_sum {
            None => {
                running_sum = Some(s_num)
            },
            Some(_) => {
                running_sum = Some(running_sum.unwrap().add(&s_num));
            }
        }
        println!("Running Sum: {}", running_sum.as_ref().unwrap());
    }
    let mag: u32 = running_sum.as_ref().unwrap().magnitude();
    println!("Magnitude = {}", mag);
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
        let _ = read_snailfish_file();
    }


    #[test]
    fn test_create_sn() {
        let line = "[[1,2],3]";
        let s_num: SnailfishNumber = SnailfishNumber::parse(&line).unwrap();

        let outer_pair: &SnailfishPair = &*s_num.top_pair;
        let first_pair: &SnailfishItem = &*outer_pair.left;
        if let SnailfishItem::Pair(pair) = first_pair {
            let first_bit: &SnailfishItem = &pair.left;
            assert!(matches!(first_bit, SnailfishItem::RegularNumber(1)));
            let second_bit: &SnailfishItem = &pair.right;
            assert!(matches!(second_bit, SnailfishItem::RegularNumber(2)));
        } else {
            assert!(false);
        }
        let third_bit: &SnailfishItem = &*outer_pair.right;
        assert!(matches!(third_bit, SnailfishItem::RegularNumber(3)));
    }

    #[test]
    fn test_reduce_step() {
        let test_cases = [
            ("[1,2]", None),
            ("[11,2]", Some("[[5,6],2]")),
            ("[2,11]", Some("[2,[5,6]]")),
            ("[14,2]", Some("[[7,7],2]")),
            ("[1,[2,[3,11]]]", Some("[1,[2,[3,[5,6]]]]]")),
            ("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]", Some("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]")),
            ("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]", Some("[[3,[2,[8,0]]],[9,[5,[7,0]]]]")),
            ("[[[[[9,8],1],2],3],4]", Some("[[[[0,9],2],3],4]")),
            ("[7,[6,[5,[4,[3,2]]]]]", Some("[7,[6,[5,[7,0]]]]")),
            ("[[6,[5,[4,[3,2]]]],1]", Some("[[6,[5,[7,0]]],3]")),
            (
                "[[[[4,0],[5,0]],[[[4,5],[2,6]],0]],0]",
                Some("[[[[4,0],[5,4]],[[0,[7,6]],0]],0]")
            ),
            (
                "[[[[4,0],[5,0]],[[[4,5],[2,6]],[9,5]]],[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]]",
                Some("[[[[4,0],[5,4]],[[0,[7,6]],[9,5]]],[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]]")
            )
        ];
        for (input, expected) in test_cases {
            let s_num: SnailfishNumber = SnailfishNumber::parse_unreduced(input).unwrap();
            let step_1 = s_num.reduce_step();
            match expected {
                None => assert!(step_1.is_none()),
                Some(exp) => assert_eq!(step_1.unwrap(), SnailfishNumber::parse_unreduced(exp).unwrap()),
            }
        }
    }


    #[test]
    fn test_reduce() {
        let test_cases = [
            ("[1,2]", "[1,2]"),
            ("[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]", "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]"),
        ];
        for (input, expected) in test_cases {
            let s_num: SnailfishNumber = SnailfishNumber::parse_unreduced(input).unwrap();
            let reduced = s_num.reduce();
            assert_eq!(reduced, SnailfishNumber::parse_unreduced(expected).unwrap());
        }
    }

    #[test]
    fn test_magnitude() {
        let test_cases = [
            ("[[1,2],[[3,4],5]]", 143),
            ("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]", 1384),
            ("[[[[1,1],[2,2]],[3,3]],[4,4]]", 445),
            ("[[[[3,0],[5,3]],[4,4]],[5,5]]", 791),
        ];
        for (input, expected) in test_cases {
            let s_num = SnailfishNumber::parse(input).unwrap();
            assert_eq!(s_num.magnitude(), expected)
        }
    }

    #[test]
    fn test_sum() {
        let test_cases = [
            (
                "[[[[4,3],4],4],[7,[[8,4],9]]]",
                "[1,1]",
                "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]"
            ), (
                "[1,1]",
                "[2,2]",
                "[[1,1],[2,2]]"
            ), (
                "[[1,1],[2,2]]",
                "[3,3]",
                "[[[1,1],[2,2]],[3,3]]"
            ), (
                "[[[1,1],[2,2]],[3,3]]",
                "[4,4]",
                "[[[[1,1],[2,2]],[3,3]],[4,4]]"
            ), (
                "[[[[1,1],[2,2]],[3,3]],[4,4]]",
                "[5,5]",
                "[[[[3,0],[5,3]],[4,4]],[5,5]]"
            ), (
                "[[[[3,0],[5,3]],[4,4]],[5,5]]",
                "[6,6]",
                "[[[[5,0],[7,4]],[5,5]],[6,6]]"
            ), (
                "[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]",
                "[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]",
                "[[[[4,0],[5,4]],[[7,7],[6,0]]],[[8,[7,7]],[[7,9],[5,0]]]]"
            ),
        ];
        for (a_str, b_str, expect_str) in test_cases {
            let a = SnailfishNumber::parse(a_str).unwrap();
            let b = SnailfishNumber::parse(b_str).unwrap();
            let expect = SnailfishNumber::parse(expect_str).unwrap();
            let sum = a.add(&b);
            assert_eq!(sum, expect)
        }
    }
}
