use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};


/// An error that we can encounter when reading the input.
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


#[derive(Debug)]
enum SnailfishItem {
    RegularNumber(u32),
    Pair(Box<SnailfishPair>),
}

#[derive(Debug)]
struct SnailfishPair {
    left: SnailfishItem,
    right: SnailfishItem,
}

#[derive(Debug)]
struct SnailfishNumber {
    top_pair: SnailfishPair,
}



struct CharStream<'a> {
    iter: Box<dyn Iterator<Item=char> + 'a>,
    peeked: Option<char>,
}

impl<'a> CharStream<'a> {
    fn new(s: &'a str) -> CharStream<'a> {
        CharStream{iter: Box::new(s.chars()), peeked: None}
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


impl SnailfishItem {
    fn new_num(value: u32) -> Self {
        SnailfishItem::RegularNumber(value)
    }

    fn new_pair(pair: SnailfishPair) -> Self {
        SnailfishItem::Pair(Box::new(pair))
    }

    fn read_from(stream: &mut CharStream) -> Result<Self, InputError> {
        let next_c = stream.peek()?;
        match next_c {
            '[' => {
                let pair: SnailfishPair = SnailfishPair::read_from(stream)?;
                Ok(SnailfishItem::new_pair(pair))
            },
            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                let num: u32 = stream.get_next()?.to_string().parse()?;
                Ok(SnailfishItem::new_num(num))
            },
            _ => Err(InputError::ExpectedItem(next_c)),
        }
    }
}

impl SnailfishPair {
    fn new(left: SnailfishItem, right: SnailfishItem) -> Self {
        SnailfishPair{left, right}
    }

    fn read_from(stream: &mut CharStream) -> Result<Self, InputError> {
        stream.expect('[')?;
        let left: SnailfishItem = SnailfishItem::read_from(stream)?;
        stream.expect(',')?;
        let right: SnailfishItem = SnailfishItem::read_from(stream)?;
        stream.expect(']')?;
        Ok(SnailfishPair::new(left, right))
    }
}

impl SnailfishNumber {
    /// Parse a string to return a SnailfishNumber or an InputError
    fn parse(s: &str) -> Result<Self, InputError> {
        let mut stream: CharStream = CharStream::new(s);
        let top_pair: SnailfishPair = SnailfishPair::read_from(&mut stream)?;
        Ok(SnailfishNumber{top_pair})
    }
}

fn run() -> Result<(),InputError> {
    let lines = read_snailfish_file()?;
    println!("Lines: {:#?}", lines);
    for line in lines {
        let s_num = SnailfishNumber::parse(&line)?;
        println!("SnailfishNumber: {:#?}", s_num);
    }
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
}
