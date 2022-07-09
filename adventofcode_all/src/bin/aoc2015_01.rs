use std::fs;
use std::io;

fn input() -> Result<String, io::Error> {
    fs::read_to_string("input/2015/01/input.txt")
}

fn part_a(s: &String) -> Result<(), io::Error> {
    let floor: i32 = s.chars().map(|c|
        match c {
            '(' => 1,
            ')' => -1,
            _ => panic!("Invalid char")
        }
    ).sum();
    println!("Ends on floor {}", floor);
    Ok(())
}

fn part_b(s: &String) -> Result<(), io::Error> {
    let mut floor: i32 = 0;
    for (pos, mv) in s.chars().map(|c|
        match c {
            '(' => 1,
            ')' => -1,
            _ => panic!("invalid char")
        }).enumerate()
    {
        floor += mv;
        if floor < 0 {
            println!("Reaches the basement at position {}", pos + 1);
            break;
        }
    };
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let s = input()?;
    part_a(&s)?;
    part_b(&s)?;
    Ok(())
}
