
extern crate md5;

use std::{fs, io};
use std::io::{Error, Write};


fn input() -> Result<String, Error> {
    fs::read_to_string("input/2016/input_05.txt")
}




fn part_a(input: &String) {
    println!("\nPart a:");
    println!("Using DoorID of {}", input);
    let mut password: String = String::with_capacity(8);
    let mut x: u64 = 0;
    print!("Generating: ");
    io::stdout().flush().unwrap();
    loop {
        let s = format!("{}{}", input, x);
        let hex = format!("{:x}", md5::compute(s));
        if hex.starts_with("00000") {
            let key_char: char = hex.chars().into_iter().nth(5).unwrap();
            password.push(key_char);
            print!("{}", key_char);
            io::stdout().flush().unwrap();
            if password.len() == 8 {
                break;
            }
        }
        x += 1;
    }
    println!();
    println!("The password is {}", password);
}


fn part_b(input: &String) {
    println!("\nPart b:");
    println!("Using DoorID of {}", input);
    let mut password: [char;8] = ['_';8];
    let mut chars_found = 0;
    let mut x: u64 = 0;
    println!("Solving... {}", password.iter().collect::<String>());
    io::stdout().flush().unwrap();
    loop {
        let s = format!("{}{}", input, x);
        let hex = format!("{:x}", md5::compute(s));
        if hex.starts_with("00000") {
            let key_pos_char: char = hex.chars().into_iter().nth(5).unwrap();
            match key_pos_char {
                '0'..='7' => {
                    let key_pos: usize = key_pos_char.to_string().parse().unwrap();
                    let key_char: char = hex.chars().into_iter().nth(6).unwrap();
                    if password[key_pos] == '_' {
                        password[key_pos] = key_char;
                        chars_found += 1;
                        println!("Solving... {}", password.iter().collect::<String>());
                        if chars_found == 8 {
                            break;
                        }
                    }
                },
                _ => {
                },
            }
        }
        x += 1;
    }
    println!();
    println!("The password is {}", password.iter().collect::<String>());
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
