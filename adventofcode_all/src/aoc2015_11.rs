use std::fmt::{Display, Formatter};
use std::fs;
use std::io;

struct Password {
    nums: Vec<u8>,
}

fn input() -> Result<String, io::Error> {
    let s = fs::read_to_string("input/2015/11/input.txt")?;
    assert!(s.chars().all(|c| c.is_ascii_lowercase()));
    Ok(s)
}


fn to_num(c: char) -> u8 {
    (c as u8) - ('a' as u8)
}

fn to_char(n: u8) -> char {
    (n + ('a' as u8)) as char
}

impl Password {
    fn new(s: &str) -> Self {
        assert!(s.len() > 0);
        Password{nums: s.chars().map(to_num).collect()}
    }

    fn len(&self) -> usize {
        self.nums.len()
    }

    /// Increment to the next password (which may or may not be valid)
    fn incr(&mut self) {
        let mut pos = self.len() - 1;
        self.nums[pos] += 1;
        while self.nums[pos] == 26 {
            self.nums[pos] = 0;
            if pos == 0 {
                pos = self.len() - 1
            } else {
                pos -= 1;
                self.nums[pos] += 1;
            };
        }
    }


    /// Returns true if the password meets the requirements
    fn is_valid(&self) -> bool {
        let mut has_run_of_3: bool = false;
        let mut has_prohibited_char: bool = false;
        let mut num_pairs: u32 = 0;
        let mut prev_prev_val: Option<u8> = None;
        let mut prev_val: Option<u8> = None;
        for val in self.nums.iter() {
            if *val == to_num('i') || *val == to_num('o') || *val == to_num('l') {
                has_prohibited_char = true
            }
            match (prev_prev_val, prev_val, Some(*val)) {
                (Some(x), Some(y), Some(z)) => {
                    if y == x + 1 && z == y + 1 {
                        has_run_of_3 = true;
                    }
                    if y == z && x != z {
                        num_pairs += 1;
                    }
                },
                (None, Some(y), Some(z)) => {
                    if y == z {
                        num_pairs += 1;
                    }
                },
                _ => {},
            }
            prev_prev_val = prev_val;
            prev_val = Some(*val);
        }
        has_run_of_3 && !has_prohibited_char && num_pairs >= 2
    }


    /// Increments until we reach the next valid password
    fn next_valid(&mut self) {
        self.incr();
        while !self.is_valid() {
            self.incr()
        }
    }
}

impl From<&Password> for String {
    fn from(password: &Password) -> Self {
        password.nums.iter().map(|n| to_char(*n)).collect()
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}



fn part_a(s: &str) -> Result<(), io::Error> {
    let mut pwd = Password::new(s);
    pwd.next_valid();
    println!("Next valid password is {}", pwd);
    Ok(())
}

fn part_b(s: &str) -> Result<(), io::Error> {
    let mut pwd = Password::new(s);
    pwd.next_valid();
    pwd.next_valid();
    println!("After that it's {}", pwd);
    Ok(())
}

fn main() -> Result<(), io::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
