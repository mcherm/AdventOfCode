
// ======= part_a =======

mod part_a {
    use std::fmt;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    /// An error that we can encounter when reading the input.
    enum InputError {
        IoError(std::io::Error),
    }

    impl From<std::io::Error> for InputError {
        fn from(error: std::io::Error) -> Self {
            InputError::IoError(error)
        }
    }

    impl fmt::Display for InputError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InputError::IoError(err)   => write!(f, "{}", err),
            }
        }
    }


    /// Read in the input file.
    fn read_chunk_file() -> Result<Vec<String>, InputError> {
        let filename = "input/2021/input_10.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        let mut chunk_lines: Vec<String> = Vec::new();
        for line in lines {
            let text = line?;
            chunk_lines.push(text);
        }

        return Ok(chunk_lines);
    }


    /// An error that we can encounter when processing chunk lines.
    enum ChunkError {
        ExtraCloseBracket(char), // there was an extra close bracket of type c
        WrongCloseBracket(char, char), // a chunk starting with opener (first) ended with closer (second)
        UnclosedBracket(char), // there was a bracket of type c that wasn't closed
    }

    impl fmt::Display for ChunkError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ChunkError::ExtraCloseBracket(c) => write!(f, "Extra close bracket: {}", c),
                ChunkError::WrongCloseBracket(o, c) => write!(f, "Opened with {} but closed with {}.", o, c),
                ChunkError::UnclosedBracket(c) => write!(f, "Unclosed bracket: {}", c),
            }
        }
    }


    fn check_chunk_line(line: &str) -> Result<(),ChunkError> {
        fn expect_close(stack: &mut Vec<char>, closer: char, expected: char) -> Result<(),ChunkError> {
            match stack.pop() {
                Some(opener) => {
                    if opener != expected {
                        Err(ChunkError::WrongCloseBracket(opener, closer))
                    } else {
                        Ok(())
                    }
                },
                None => {
                    Err(ChunkError::ExtraCloseBracket(closer))
                }
            }
        }

        let mut stack: Vec<char> = Vec::new();
        for c in line.chars() {
            match c {
                '(' => stack.push(c),
                '[' => stack.push(c),
                '{' => stack.push(c),
                '<' => stack.push(c),
                ')' => expect_close(&mut stack, c, '(')?,
                ']' => expect_close(&mut stack, c, '[')?,
                '}' => expect_close(&mut stack, c, '{')?,
                '>' => expect_close(&mut stack, c, '<')?,
                _ => panic!("Invalid character {}", c),
            }
        }
        match stack.pop() {
            Some(c) => return Err(ChunkError::UnclosedBracket(c)),
            None => return Ok(()),
        }
    }


    pub fn main() {
        match read_chunk_file() {
            Ok(chunk_lines) => {
                let mut score = 0;
                for chunk_line in &chunk_lines {
                    match check_chunk_line(chunk_line) {
                        Err(ChunkError::WrongCloseBracket(op, cl)) => {
                            println!("Expected {} but found {} instead.", op, cl);
                            score += match cl {
                                ')' => 3,
                                ']' => 57,
                                '}' => 1197,
                                '>' => 25137,
                                _ => panic!("Invalid closer {}", cl),
                            }
                        }
                        _ => {},
                    }
                }
                println!("The total syntax score is {}", score);
            },
            Err(err) => println!("Error: {}", err),
        }
    }
}

// ======= part_b =======

mod part_b {
    use std::fmt;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    /// An error that we can encounter when reading the input.
    enum InputError {
        IoError(std::io::Error),
    }

    impl From<std::io::Error> for InputError {
        fn from(error: std::io::Error) -> Self {
            InputError::IoError(error)
        }
    }

    impl fmt::Display for InputError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InputError::IoError(err)   => write!(f, "{}", err),
            }
        }
    }


    /// Read in the input file.
    fn read_chunk_file() -> Result<Vec<String>, InputError> {
        let filename = "input/2021/input_10.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        let mut chunk_lines: Vec<String> = Vec::new();
        for line in lines {
            let text = line?;
            chunk_lines.push(text);
        }

        return Ok(chunk_lines);
    }


    /// An error that we can encounter when processing chunk lines.
    enum ChunkError {
        ExtraCloseBracket(char), // there was an extra close bracket of type c
        WrongCloseBracket(char, char), // a chunk starting with opener (first) ended with closer (second)
        UnclosedBrackets(Vec<char>), // the given stack of brackets weren't closed
    }

    impl fmt::Display for ChunkError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ChunkError::ExtraCloseBracket(c) => write!(f, "Extra close bracket: {}", c),
                ChunkError::WrongCloseBracket(o, c) => write!(f, "Opened with {} but closed with {}.", o, c),
                ChunkError::UnclosedBrackets(stack) => write!(f, "Unclosed brackets: {:?}", stack),
            }
        }
    }


    fn check_chunk_line(line: &str) -> Result<(),ChunkError> {
        fn expect_close(stack: &mut Vec<char>, closer: char, expected: char) -> Result<(),ChunkError> {
            match stack.pop() {
                Some(opener) => {
                    if opener != expected {
                        Err(ChunkError::WrongCloseBracket(opener, closer))
                    } else {
                        Ok(())
                    }
                },
                None => {
                    Err(ChunkError::ExtraCloseBracket(closer))
                }
            }
        }

        let mut stack: Vec<char> = Vec::new();
        for c in line.chars() {
            match c {
                '(' => stack.push(c),
                '[' => stack.push(c),
                '{' => stack.push(c),
                '<' => stack.push(c),
                ')' => expect_close(&mut stack, c, '(')?,
                ']' => expect_close(&mut stack, c, '[')?,
                '}' => expect_close(&mut stack, c, '{')?,
                '>' => expect_close(&mut stack, c, '<')?,
                _ => panic!("Invalid character {}", c),
            }
        }
        if stack.len() == 0 {
            Ok(())
        } else {
            Err(ChunkError::UnclosedBrackets(stack))
        }
    }


    pub fn main() {
        match read_chunk_file() {
            Ok(chunk_lines) => {
                let mut line_scores: Vec<u64> = Vec::new();
                for chunk_line in &chunk_lines {
                    match check_chunk_line(chunk_line) {
                        Err(ChunkError::WrongCloseBracket(_,_)) => {
                            // "corrupted line"; ignore it
                        },
                        Ok(()) => {
                            panic!("Valid line found.");
                        },
                        Err(ChunkError::ExtraCloseBracket(_)) => {
                            panic!("Extra close bracket found.");
                        },
                        Err(ChunkError::UnclosedBrackets(mut stack)) => {
                            let mut line_score: u64 = 0;
                            while let Some(c) = stack.pop() {
                                let char_score = match c {
                                    '(' => 1,
                                    '[' => 2,
                                    '{' => 3,
                                    '<' => 4,
                                    _ => panic!("Impossible value, {}", c)
                                };
                                line_score *= 5;
                                line_score += char_score;
                            }
                            line_scores.push(line_score);
                        },
                    }
                }

                line_scores.sort();
                assert!(line_scores.len() % 2 == 1);
                println!("The median score is: {}", line_scores[line_scores.len() / 2]);
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
