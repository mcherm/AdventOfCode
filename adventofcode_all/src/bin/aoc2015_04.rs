use std::fs;
use std::io;
use md5;


fn input() -> Result<String, io::Error> {
    fs::read_to_string("input/2015/04/input.txt")
}


fn find_answer_for_prefix(secret_key: &str, prefix_sought: &str) -> u32 {
    let mut num: u32 = 0;
    loop {
        let hash = md5::compute(format!("{}{}", secret_key, num));
        let hash_hex = format!("{:x}", hash);
        let prefix = &hash_hex[..prefix_sought.len()];
        if prefix == prefix_sought {
            break;
        }
        num += 1;
    }
    num
}


fn part_a(secret_key: &String) -> Result<(), io::Error> {
    let num = find_answer_for_prefix(secret_key, "00000");
    println!("For 5 zeros, the answer is {}.", num);
    Ok(())
}

fn part_b(secret_key: &String) -> Result<(), io::Error> {
    let num = find_answer_for_prefix(secret_key, "000000");
    println!("For 6 zeros, the answer is {}.", num);
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let route = input()?;
    part_a(&route)?;
    part_b(&route)?;
    Ok(())
}
