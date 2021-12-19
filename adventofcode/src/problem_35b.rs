use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{AddAssign, RangeBounds};


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


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Token {
    Number(u32),
    BeginBracket,
    EndBracket,
    Comma,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SnailfishNum {
    tokens: Vec<Token>,
}

enum Direction {
    Left,
    Right,
}

impl SnailfishNum {
    fn parse(s: &str) -> Result<Self, InputError> {
        let mut stream: CharStream = CharStream::new(s, true);
        let mut tokens: Vec<Token> = Vec::new();

        fn parse_num(stream: &mut CharStream, tokens: &mut Vec<Token>) -> Result<(), InputError> {
            let val: u32;
            if stream.get_allow_multiple_digits() {
                let mut build_number: u32 = 0;
                loop {
                    match stream.peek()? {
                        '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {},
                        _ => break, // exit the loop when the next thing isn't a digit
                    }
                    let digit: u32 = stream.get_next()?.to_string().parse()?;
                    build_number *= 10;
                    build_number += digit;
                }
                val = build_number;
            } else {
                let c = stream.get_next()?;
                val = c.to_string().parse()?;
            }
            tokens.push(Token::Number(val));
            Ok(())
        }

        fn parse_pair(stream: &mut CharStream, tokens: &mut Vec<Token>) -> Result<(), InputError> {
            stream.expect('[')?;
            tokens.push(Token::BeginBracket);
            parse_value(stream, tokens)?;
            stream.expect(',')?;
            tokens.push(Token::Comma);
            parse_value(stream, tokens)?;
            stream.expect(']')?;
            tokens.push(Token::EndBracket);
            Ok(())
        }

        fn parse_value(stream: &mut CharStream, tokens: &mut Vec<Token>) -> Result<(), InputError> {
            let c = stream.peek()?;
            match c {
                '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => parse_num(stream, tokens)?,
                '[' => parse_pair(stream, tokens)?,
                _ => Err(InputError::ExpectedItem(c))?,
            }
            Ok(())
        }

        parse_pair(&mut stream, &mut tokens)?;
        Ok(SnailfishNum{tokens})
    }

    /// Replaces all tokens from "from" to just before "up_to" with the
    /// subrange
    fn replace<R,I>(&mut self, range: R, new_tokens: I)
        where
            R: RangeBounds<usize>,
            I: IntoIterator<Item=Token>
    {
        self.tokens.splice(range, new_tokens);
        ()
    }

    /// distributes an amount the the nearest number
    fn distribute_explosion(&mut self, start_from: usize, amount: u32, direction: Direction) {
        let mut pos = start_from;
        loop {
            // --- increment pos ---
            match direction {
                Direction::Left => {
                    if pos == 0 {
                        return
                    } else {
                        pos -= 1;
                    }
                },
                Direction::Right => {
                    pos += 1;
                    if pos == self.tokens.len() {
                        return;
                    }
                },
            }
            // --- check for number ---
            match self.tokens[pos] {
                Token::Number(old_val) => {
                    self.tokens[pos] = Token::Number(old_val + amount);
                    return;
                },
                _ => {},
            }
        }
    }

    /// Attempts to perform one explode. Returns true if it did; false if there wasn't one
    /// to do.
    fn explode_one(&mut self) -> bool {
        let mut start_of_explode: Option<usize> = None;
        let mut end_of_explode: Option<usize> = None;
        let mut first_num: Option<u32> = None;
        let mut second_num: Option<u32> = None;
        let mut nesting_count = 0;
        for (pos, tok) in self.tokens.iter().enumerate() {
            match tok {
                Token::BeginBracket => {
                    nesting_count += 1;
                    if start_of_explode.is_none() && nesting_count == 5 {
                        start_of_explode = Some(pos);
                    }
                },
                Token::EndBracket => {
                    nesting_count -= 1;
                    if start_of_explode.is_some() && nesting_count < 5 {
                        end_of_explode = Some(pos);
                        break; // we can quit the for loop now
                    }
                },
                Token::Number(val) => {
                    if start_of_explode.is_some() && first_num.is_none() {
                        first_num = Some(*val);
                    }
                    if start_of_explode.is_some() && first_num.is_some() {
                        second_num = Some(*val);
                    }
                },
                _ => {}
            }
        }
        if end_of_explode.is_some() {
            let start_pos = start_of_explode.unwrap();
            let end_pos = end_of_explode.unwrap();
            let new_tokens = [Token::Number(0)];
            self.replace(start_pos..=end_pos, new_tokens);
            self.distribute_explosion(start_pos, first_num.unwrap(), Direction::Left);
            self.distribute_explosion(start_pos + 1, second_num.unwrap(), Direction::Right);
            return true;
        } else {
            return false;
        }
    }

    /// Perform once split. Returns true if it split; false if it didn't.
    fn split_one(&mut self) -> bool {
        for (pos, tok) in self.tokens.iter().enumerate() {
            match tok {
                Token::Number(val) => {
                    if *val >= 10 {
                        let new_tokens = [
                            Token::BeginBracket,
                            Token::Number(val / 2),
                            Token::Comma,
                            Token::Number((val / 2) + (val % 2)),
                            Token::EndBracket,
                        ];
                        self.replace(pos..=pos, new_tokens);
                        return true;
                    }
                },
                _ => {}
            }
        }
        return false;
    }

    /// Perform one reduction step. Returns true if it reduced, false if it stayed the same.
    fn reduce_step(&mut self) -> bool {
        // --- look for explodes ---
        let exploded = self.explode_one();
        if exploded {
            return true;
        }

        // --- look for splits ---
        let split = self.split_one();
        if split {
            return true;
        }

        // --- guess there was nothing to do ---
        return false;
    }

    /// Performs reductions until there aren't any more to perform.
    fn reduce(&mut self) {
        while self.reduce_step() {
        }
    }

    /// Finds the magnitude, given a position which is the beginning of a pair or number.
    fn magnitude(&self) -> u32 {
        // Recursive helper that returns (value, new_position)
        fn magnitude_of_value(num: &SnailfishNum, pos: usize) -> (u32, usize) {
            match num.tokens[pos] {
                Token::Number(val) => (val, pos + 1),
                Token::BeginBracket => {
                    let (left, comma_pos) = magnitude_of_value(num, pos + 1);
                    assert!(matches!(num.tokens[comma_pos], Token::Comma));
                    let (right, close_pos) = magnitude_of_value(num, comma_pos + 1);
                    assert!(matches!(num.tokens[close_pos], Token::EndBracket));
                    let mag = left * 3 + right * 2;
                    (mag, close_pos + 1)
                },
                _ => panic!(),
            }
        }
        let (mag, pos) = magnitude_of_value(self, 0);
        assert_eq!(pos, self.tokens.len());
        return mag;
    }
}

impl AddAssign for SnailfishNum {
    fn add_assign(&mut self, rhs: Self) {
        self.tokens.insert(0, Token::BeginBracket);
        self.tokens.push(Token::Comma);
        self.tokens.extend(rhs.tokens);
        self.tokens.push(Token::EndBracket);
        self.reduce();
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Comma => write!(f, ","),
            Token::BeginBracket => write!(f, "["),
            Token::EndBracket => write!(f, "]"),
            Token::Number(v) => write!(f, "{}", v),
        }
    }
}
impl Display for SnailfishNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for token in &self.tokens {
            write!(f, "{}", token)?;
        }
        Ok(())
    }
}


fn run() -> Result<(),InputError> {
    let lines = read_snailfish_file()?;
    let mut line_iter = lines.iter();
    let mut running_sum: SnailfishNum = SnailfishNum::parse(line_iter.next().unwrap())?;
    for line in line_iter {
        let num = SnailfishNum::parse(&line)?;
        running_sum += num;
    }
    println!("Sum: {}", running_sum);
    println!("Magnitude = {}", running_sum.magnitude());
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
    fn test_add() {
        let mut a = SnailfishNum::parse("[1,2]").unwrap();
        let b = SnailfishNum::parse("[3,4]").unwrap();
        let expect = SnailfishNum::parse("[[1,2],[3,4]]").unwrap();
        a += b;
        assert_eq!(a, expect)
    }

    #[test]
    fn test_replace() {
        let mut a = SnailfishNum::parse("[1,[2,3]]").unwrap();
        let replace = SnailfishNum::parse("[9,9]").unwrap();
        a.replace(1..2, replace.tokens.iter().cloned());
        let expect = SnailfishNum::parse("[[9,9],[2,3]]").unwrap();
        assert_eq!(a, expect);
        a.replace(7..=11, replace.tokens.iter().cloned());
        let expect = SnailfishNum::parse("[[9,9],[9,9]]").unwrap();
        assert_eq!(a, expect);
    }

    #[test]
    fn test_magnitude() {
        let test_cases = [
            ("[1,2]", 7),
            ("[[1,2],[[3,4],5]]", 143),
            ("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]", 1384),
            ("[[[[1,1],[2,2]],[3,3]],[4,4]]", 445),
            ("[[[[3,0],[5,3]],[4,4]],[5,5]]", 791),
        ];
        for (input, expected) in test_cases {
            let num = SnailfishNum::parse(input).unwrap();
            assert_eq!(num.magnitude(), expected)
        }
    }

}
