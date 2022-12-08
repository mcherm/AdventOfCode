
extern crate anyhow;

use std::fs;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::newline,
    combinator::{value, map},
    multi::many0,
    sequence::{terminated, tuple},
};
use nom::character::complete::u64 as nom_file_size;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use itertools::Itertools;


// ======= Parsing =======

fn input() -> Result<Vec<Command>, anyhow::Error> {
    let s = fs::read_to_string("input/2022/input_07.txt")?;
    match Command::parse_list(&s) {
        Ok(("", x)) => Ok(x),
        Ok((s, _)) => panic!("Extra input starting at {}", s),
        Err(_) => panic!("Invalid input"),
    }
}


type FileSize = u64;

#[derive(Debug, Clone)]
enum CdDestination {
    Root,
    Up,
    Down(String),
}

#[derive(Debug, Clone)]
enum DirEntry {
    Dir(String),
    File(FileSize, String),
}

#[derive(Debug, Clone)]
enum Command {
    Cd(CdDestination),
    Ls(Vec<DirEntry>),
}



fn parse_filename<'a>(input: &'a str) -> IResult<&'a str, String> {
    map(
        take_while1(|c| (c as char).is_ascii_alphabetic() || (c as char) == '.'),
        |s: &str| s.to_string()
    )(input)
}

impl CdDestination {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(CdDestination::Root, tag("/")),
            value(CdDestination::Up, tag("..")),
            map(parse_filename, |s| CdDestination::Down(s)),
        ))(input)
    }
}

impl DirEntry {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                tuple((
                    tag("dir "),
                    parse_filename
                )),
                |(_, s)| DirEntry::Dir(s)
            ),
            map(
                tuple((
                    nom_file_size,
                    tag(" "),
                    parse_filename,
                )),
                |(size, _, name)| DirEntry::File(size, name)
            ),
        ))(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0( terminated(Self::parse, newline) )(input)
    }

    fn name(&self) -> &str {
        match self {
            DirEntry::File(_,name) => name,
            DirEntry::Dir(name) => name,
        }
    }
}

impl Command {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                tuple((tag("$ cd "), CdDestination::parse, newline)),
                |(_, cd_dest, _)| Command::Cd(cd_dest)
            ),
            map(
                tuple((tag("$ ls"), newline, DirEntry::parse_list)),
                |(_, _, dir_entries)| Command::Ls(dir_entries)
            ),
        ))(input)
    }

    fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        many0(Self::parse)(input)
    }
}


// ======= Constructing =======

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Path {
    full_path: String, // the path with leading and trailing "/". Like "/foo/bar/".
}

#[derive(Debug, Clone)]
enum DirContent {
    File(FileSize),
    DirUnknown(),
    DirKnown(HashSet<Path>),
}

/// Contains an entire directory structure, starting from a root node.
#[derive(Debug)]
struct FileSystem {
    files: HashMap<Path,DirContent>
}


impl Path {
    /// Create the root path.
    fn root() -> Self {
        Path{full_path: "/".to_string()}
    }

    /// Given a Path, returns a new Path that extends it by "name".
    fn extend(&self, name: &str) -> Self {
        Path{full_path: format!("{}{}/", self.full_path, name)}
    }

    /// Tests if this is the root path.
    fn is_root(&self) -> bool {
        self.full_path.len() == 1
    }

    /// Given a Path, returns a new Path that is the parent of this path, or an error if we
    /// attempt to find the parent of the root.
    fn parent(&self) -> Result<Self, anyhow::Error> {
        if self.is_root() {
            return Err(anyhow::Error::msg("Attempt to find parent of the root directory."));
        }
        let mut path_str: String = self.full_path.clone();
        path_str.pop(); // remove trailing "/"
        let last_slash = path_str.rfind("/").unwrap(); // MUST have at least one "/" so unwrap() is safe
        path_str.truncate(last_slash + 1); // trims path_str so it ends in the second-to-last "/"
        Ok(Path{full_path: path_str})
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_path)
    }
}


impl FileSystem {
    /// Create a FileSystem from a list of commands.
    fn build_from_commands(cmds: &Vec<Command>) -> Result<Self, anyhow::Error> {
        // --- get iterator ---
        let mut cmd_iter = cmds.iter();

        // --- create helper function ---
        fn confirm_path_is_known_to_be_dir(files: &HashMap<Path,DirContent>, path: &Path) -> Result<(), anyhow::Error> {
            match files.get(path) {
                Some(DirContent::DirUnknown()) => {}, // just fine
                Some(DirContent::DirKnown(_)) => {}, // just fine
                Some(DirContent::File(_)) => {
                    return Err(anyhow!("Attempt to cd to {} which is a file, not a directory.", path));
                },
                None => {
                    return Err(anyhow!("Attempt to cd to {} which is not a known directory.", path));
                },
            }
            Ok(())
        }

        // --- verify starting state, then create state variables ---
        if !matches!(cmd_iter.next(), Some(Command::Cd(CdDestination::Root))) {
            return Err(anyhow::Error::msg("First command isn't \"cd \\\"."))
        }
        let mut files: HashMap<Path,DirContent> = HashMap::new();
        let mut current_path: Path = Path::root();
        files.insert(current_path.clone(), DirContent::DirUnknown());

        // --- main loop... go until there are no more commands ---
        for cmd in cmd_iter {
            match cmd {

                Command::Cd(CdDestination::Down(child_dir)) => {
                    current_path = current_path.extend(child_dir);
                    confirm_path_is_known_to_be_dir(&files, &current_path)?;
                },

                Command::Cd(CdDestination::Up) => {
                    current_path = current_path.parent()?;
                    confirm_path_is_known_to_be_dir(&files, &current_path)?;
                },

                Command::Cd(CdDestination::Root) => {
                    current_path = Path::root();
                    confirm_path_is_known_to_be_dir(&files, &current_path)?;
                },

                Command::Ls(dir_entries) => {
                    // --- make sure we have a parent dir ---

                    // --- get the parent dir and just overwrite it ---
                    // FIXME: For better robustness, we COULD verify that if it's already known we have the same list
                    let parent_dir = files.get_mut(&current_path).unwrap(); // confirm_path_is_known_to_be_dir() makes this safe to unwrap
                    assert!(matches!(parent_dir, DirContent::DirUnknown() | DirContent::DirKnown(_)));
                    *parent_dir = DirContent::DirKnown(HashSet::from_iter(dir_entries.iter().map(
                        |dir_entry| current_path.extend(dir_entry.name())
                    )));

                    // --- put all the child entries into files, checking for inconsistencies along the way ---
                    for dir_entry in dir_entries {
                        match dir_entry {
                            DirEntry::File(size, name) => {
                                let new_path = current_path.extend(name);
                                let prev_value = files.insert(new_path, DirContent::File(*size));
                                match prev_value {
                                    None => {}, // discovered a new file -- that's fine
                                    Some(DirContent::File(prev_size)) if prev_size == *size => {} // revisited same file, also fine
                                    Some(DirContent::File(prev_size)) => {
                                        return Err(anyhow!("At {}, file size was {} then later {}.", current_path.extend(name), prev_size, size));
                                    }
                                    Some(_) => {
                                        return Err(anyhow!("Path {} was initially a dir, and later a file.", current_path.extend(name)));
                                    },
                                }
                            },
                            DirEntry::Dir(name) => {
                                let new_path = current_path.extend(name);
                                let prev_value = files.insert(new_path, DirContent::DirUnknown());
                                match prev_value {
                                    None => {}, // discovered a new dir -- that's fine
                                    Some(DirContent::DirUnknown()) => {}, // rediscovered same (never explored) dir, also fine
                                    Some(DirContent::DirKnown(_)) => {}, // rediscovered same (previously explored) dir, also fine
                                    Some(DirContent::File(_)) => {
                                        return Err(anyhow!("Path {} was initially a file, and later a dir.", current_path.extend(name)));
                                    },
                                }
                            },
                        }
                    }
                },

            }
        }

        Ok(FileSystem{files})
    }
}

impl Display for FileSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n")?;
        for item in self.files.iter().sorted_by_key(|pair| pair.0) {
            write!(f, "{}: {:?}\n", item.0, item.1)?;
        }
        Ok(())
    }
}



// ======= Analyzing =======

// ======= main() =======

fn part_a(input: &Vec<Command>) -> Result<(), anyhow::Error> {
    println!("\nPart a:");
    let file_sys = FileSystem::build_from_commands(input)?;
    println!("FILE SYS: {}", file_sys);
    Ok(())
}


fn part_b(_input: &Vec<Command>) -> Result<(), anyhow::Error> {
    println!("\nPart b:");
    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
