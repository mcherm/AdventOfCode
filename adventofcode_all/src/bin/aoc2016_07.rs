
extern crate anyhow;

use std::fmt::{Display, Formatter};
use std::fs;
use anyhow::{anyhow, Error};
use std::collections::HashSet;


fn input() -> Result<Vec<IpString>, Error> {
    let s = fs::read_to_string("input/2016/input_07.txt")?;

    let mut ip_strings = Vec::new();
    let mut chunks = Vec::new();
    let mut these_chars = Vec::new();
    let mut in_brackets = false;
    let mut start_of_line = true;
    for c in s.chars() {
        start_of_line = false;
        match c {
            '[' => {
                if !in_brackets {
                    chunks.push(Chunk{data: these_chars, seq: SequenceType::Supernet });
                    these_chars = Vec::new();
                    in_brackets = true;
                } else {
                    return Err(anyhow!("Unexpected '['"));
                }
            }
            ']' => {
                if in_brackets {
                    chunks.push(Chunk{data: these_chars, seq: SequenceType::Hypernet});
                    these_chars = Vec::new();
                    in_brackets = false;
                } else {
                    return Err(anyhow!("Unexpected ']'"));
                }
            },
            '\n' => {
                if in_brackets {
                    return Err(anyhow!("Square brackets not closed"));
                } else if these_chars.len() > 0 {
                    chunks.push(Chunk{data: these_chars, seq: SequenceType::Supernet });
                    these_chars = Vec::new();
                    ip_strings.push(IpString{chunks});
                    chunks = Vec::new();
                    start_of_line = true;
                }
            },
            'a'..='z' => {
                these_chars.push(c);
            }
            _ => todo!()
        }
    }
    if !start_of_line {
        return Err(anyhow!("No newline on final line")); // the file needs to end in a \n.
    }
    Ok(ip_strings)
}


#[derive(Debug, Copy, Clone)]
enum SequenceType {
    Supernet,
    Hypernet,
}


#[derive(Debug)]
struct Chunk {
    data: Vec<char>,
    seq: SequenceType,
}

impl Chunk {
    fn to_string(&self) -> String {
        self.data.iter().collect()
    }

    fn has_abba(&self) -> bool {
        for w in self.data[..].windows(4) {
            if w[0] != w[1] && w[0] == w[3] && w[1] == w[2] {
                return true;
            }
        }
        false
    }

    fn collect_abas(&self, set: &mut HashSet<[char;3]>) {
        for w in self.data[..].windows(3) {
            if w[0] != w[1] && w[0] == w[2] {
                set.insert([w[0], w[1], w[2]]);
            }
        }
    }

    fn has_bab(&self, aba: &[char;3]) -> bool {
        for w in self.data[..].windows(3) {
            if w[0] == aba[1] && w[1] == aba[0] && w[2] == aba[1] {
                return true;
            }
        }
        false
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.seq {
            SequenceType::Supernet => write!(f, "{}", self.to_string()),
            SequenceType::Hypernet => write!(f, "[{}]", self.to_string()),
        }
    }
}


#[derive(Debug)]
struct IpString {
    chunks: Vec<Chunk>
}

impl IpString {
    fn supports_tls(&self) -> bool {
        if self.chunks.iter().any(|x| matches!(x.seq, SequenceType::Hypernet) && x.has_abba()) {
            return false;
        }
        if self.chunks.iter().any(|x| matches!(x.seq, SequenceType::Supernet) && x.has_abba()) {
            return true;
        }
        false
    }

    fn supports_ssl(&self) -> bool {
        let mut set = HashSet::new();
        for chunk in self.chunks.iter().filter(|x| matches!(x.seq, SequenceType::Supernet)) {
            chunk.collect_abas(&mut set);
        }
        for aba in set.iter() {
            for chunk in self.chunks.iter().filter(|x| matches!(x.seq, SequenceType::Hypernet)) {
                if chunk.has_bab(aba) {
                    return true;
                }
            }
        }
        false
    }
}

impl Display for IpString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for chunk in self.chunks.iter() {
            write!(f, "{}", chunk)?;
        }
        Ok(())
    }
}

fn part_a(ip_strings: &Vec<IpString>) {
    println!("\nPart a:");
    let count = ip_strings.iter().filter(|x| x.supports_tls()).count();
    println!("In that file, {} addresses support TLS", count);
}


fn part_b(ip_strings: &Vec<IpString>) {
    println!("\nPart b:");
    let count = ip_strings.iter().filter(|x| x.supports_ssl()).count();
    println!("In that file, {} addresses support SSL", count);
}


fn main() -> Result<(), Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}
