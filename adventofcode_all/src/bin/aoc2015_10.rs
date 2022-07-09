
use std::fs;
use std::io;



fn input() -> Result<String, io::Error> {
    fs::read_to_string("input/2015/10/input.txt")
}


fn generate(s: &str) -> String {
    let mut answer: String = String::new();
    let mut iter = s.chars().peekable();
    loop {
        match iter.next() {
            None => break,
            Some(c) => {
                assert!(['1', '2', '3'].contains(&c));
                let count;
                match iter.peek() {
                    Some(next_c) if *next_c == c => {
                        let _ = iter.next();
                        match iter.peek() {
                            Some(next_c) if *next_c == c => {
                                let _ = iter.next();
                                match iter.peek() {
                                    Some(next_c) if *next_c == c => panic!("more than 3 in a row"),
                                    _ => count = '3'
                                }
                            },
                            _ => count = '2'
                        }
                    },
                    _ => count = '1',
                }
                answer.push(count);
                answer.push(c);
            },
        }
    }
    answer
}


fn part_a(s: &str) -> Result<(), io::Error> {
    let mut string = s.to_string();
    for _ in 0..40 {
        string = generate(&string);
    }
    println!("The string is of length {}", string.len());
    Ok(())
}

fn part_b(s: &str) -> Result<(), io::Error> {
    let mut string = s.to_string();
    for _ in 0..50 {
        string = generate(&string);
    }
    println!("The longer string is of length {}", string.len());
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
