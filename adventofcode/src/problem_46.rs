use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs;
use nom::bytes::complete::tag as nom_tag;
use nom::sequence::tuple as nom_tuple;
use nom::branch::alt as nom_alt;
use single_linked_list::List;
use Location::{
    Hall0, Hall1, Hall2, Hall3, Hall4, Hall5, Hall6,
    FrontOfA, FrontOfB, FrontOfC, FrontOfD,
    BackOfA, BackOfB, BackOfC, BackOfD,
    FarBackOfA, FarBackOfB, FarBackOfC, FarBackOfD,
    WayBackOfA, WayBackOfB, WayBackOfC, WayBackOfD,
};
use AmphipodType::{Amber, Bronze, Copper, Desert};


// ======== Single Linked List ========

mod single_linked_list {
    // Source: https://rust-unofficial.github.io/too-many-lists/third-final.html
    use std::rc::Rc;

    pub struct List<T> {
        head: Link<T>,
    }

    type Link<T> = Option<Rc<Node<T>>>;

    struct Node<T> {
        elem: T,
        next: Link<T>,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            List { head: None }
        }

        // pub fn is_empty(&self) -> bool {
        //     match self.head {
        //         None => true,
        //         Some(_) => false,
        //     }
        // }

        pub fn prepend(&self, elem: T) -> List<T> {
            List { head: Some(Rc::new(Node {
                elem: elem,
                next: self.head.clone(),
            }))}
        }

        // pub fn tail(&self) -> List<T> {
        //     List { head: self.head.as_ref().and_then(|node| node.next.clone()) }
        // }

        // pub fn head(&self) -> Option<&T> {
        //     self.head.as_ref().map(|node| &node.elem)
        // }

        pub fn iter(&self) -> Iter<'_, T> {
            Iter { next: self.head.as_deref() }
        }
    }

    impl<T> Drop for List<T> {
        fn drop(&mut self) {
            let mut head = self.head.take();
            while let Some(node) = head {
                if let Ok(mut node) = Rc::try_unwrap(node) {
                    head = node.next.take();
                } else {
                    break;
                }
            }
        }
    }

    pub struct Iter<'a, T> {
        next: Option<&'a Node<T>>,
    }

    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            self.next.map(|node| {
                self.next = node.next.as_deref();
                &node.elem
            })
        }
    }
}


// ======== Reading Input ========

/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    InvalidMazeFile,
}

impl From<std::io::Error> for InputError {
    fn from(error: std::io::Error) -> Self {
        InputError::IoError(error)
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::IoError(err) => write!(f, "{}", err),
            InputError::InvalidMazeFile => write!(f, "Invalid maze file."),
        }
    }
}

/// Read in the input file.
fn read_maze_file() -> Result<Position, InputError> {
    // --- read file ---
    let filename = "data/2021/day/23/input.txt";
    let contents = fs::read_to_string(filename)?;
    // NOTE: I should raise an error, not unwrap but I don't know how.
    let (rest, positions) = Position::parse_nom(&contents).unwrap();
    if rest != "" {
        return Err(InputError::InvalidMazeFile);
    }

    // --- return result ---
    Ok(positions)
}



// ======== Types ========

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
enum AmphipodType {
    Amber,
    Bronze,
    Copper,
    Desert,
}


#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
enum Location {
    Hall0 = 0,
    Hall1 = 1,
    Hall2 = 2,
    Hall3 = 3,
    Hall4 = 4,
    Hall5 = 5,
    Hall6 = 6,
    FrontOfA = 7,
    FrontOfB = 8,
    FrontOfC = 9,
    FrontOfD = 10,
    BackOfA = 11,
    BackOfB = 12,
    BackOfC = 13,
    BackOfD = 14,
    FarBackOfA = 15,
    FarBackOfB = 16,
    FarBackOfC = 17,
    FarBackOfD = 18,
    WayBackOfA = 19,
    WayBackOfB = 20,
    WayBackOfC = 21,
    WayBackOfD = 22,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Position {
    slots: [Option<AmphipodType>; Location::NUM_VALUES],
}

type Cost = u32;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Move {
    amph: AmphipodType,
    from: Location,
    to: Location,
}

// ======== Implementations ========

impl AmphipodType {
    const NUM_VALUES: usize = 4;
    const ALL: [AmphipodType; AmphipodType::NUM_VALUES] = [Amber, Bronze, Copper, Desert];

    /// Reads a field which could be an AmphipodType or a "." for None.
    fn parse_nom(input: &str) -> nom::IResult<&str, Option<Self>> {
        nom_alt((
            nom_tag("A"),
            nom_tag("B"),
            nom_tag("C"),
            nom_tag("D"),
            nom_tag("."),
        ))(input).map(|(rest, res)| (rest, match res {
            "A" => Some(Amber),
            "B" => Some(Bronze),
            "C" => Some(Copper),
            "D" => Some(Desert),
            "." => None,
            _ => panic!("should never happen")
        }))
    }

    fn to_str(&self) -> &'static str {
        match self {
            Amber => &"A",
            Bronze => &"B",
            Copper => &"C",
            Desert => &"D",
        }
    }

    /// Returns the nook index for a given AmphipodType
    fn nook(&self) -> usize {
        match self {
            Amber => 0,
            Bronze => 1,
            Copper => 2,
            Desert => 3,
        }
    }

    /// Returns the cost per step for this AmphipodType
    fn step_cost(&self) -> Cost {
        match self {
            Amber => 1,
            Bronze => 10,
            Copper => 100,
            Desert => 1000,
        }
    }
}

impl Display for AmphipodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}


impl Location {
    const NUM_VALUES: usize = 23;
    #[allow(dead_code)]
    const ALL: [Location; Location::NUM_VALUES] = [
        Hall0, Hall1, Hall2, Hall3, Hall4, Hall5, Hall6,
        FrontOfA, FrontOfB, FrontOfC, FrontOfD,
        BackOfA, BackOfB, BackOfC, BackOfD,
        FarBackOfA, FarBackOfB, FarBackOfC, FarBackOfD,
        WayBackOfA, WayBackOfB, WayBackOfC, WayBackOfD,
    ];
    #[allow(dead_code)]
    const HALL_SLOTS: [Location; 7] = [Hall0, Hall1, Hall2, Hall3, Hall4, Hall5, Hall6];
    const FRONT_SLOTS: [Location; 4] = [FrontOfA, FrontOfB, FrontOfC, FrontOfD];
    const BACK_SLOTS: [Location; 4] = [BackOfA, BackOfB, BackOfC, BackOfD];
    #[allow(dead_code)]
    const FAR_BACK_SLOTS: [Location; 4] = [FarBackOfA, FarBackOfB, FarBackOfC, FarBackOfD];
    #[allow(dead_code)]
    const WAY_BACK_SLOTS: [Location; 4] = [WayBackOfA, WayBackOfB, WayBackOfC, WayBackOfD];

    fn to_str(&self) -> &'static str {
        match self {
            Hall0 => "Hall0",
            Hall1 => "Hall1",
            Hall2 => "Hall2",
            Hall3 => "Hall3",
            Hall4 => "Hall4",
            Hall5 => "Hall5",
            Hall6 => "Hall6",
            FrontOfA => "FrontOfA",
            FrontOfB => "FrontOfB",
            FrontOfC => "FrontOfC",
            FrontOfD => "FrontOfD",
            BackOfA => "BackOfA",
            BackOfB => "BackOfB",
            BackOfC => "BackOfC",
            BackOfD => "BackOfD",
            FarBackOfA => "FarBackOfA",
            FarBackOfB => "FarBackOfB",
            FarBackOfC => "FarBackOfC",
            FarBackOfD => "FarBackOfD",
            WayBackOfA => "WayBackOfA",
            WayBackOfB => "WayBackOfB",
            WayBackOfC => "WayBackOfC",
            WayBackOfD => "WayBackOfD",
        }
    }

    /// Pass this a nook number and it will return three vectors of Locations:
    /// The first is those Hall locations reachable by turning left; the
    /// second is those Hall lcoations reachable by turning right; the third
    /// is those locations IN the nook. Each list is in the order in which the
    /// locations need to be entered.
    fn hall_from(nook: AmphipodType) -> (Vec<Location>, Vec<Location>, Vec<Location>) {
        match nook {
            Amber => (
                vec![Hall1, Hall0],
                vec![Hall2, Hall3, Hall4, Hall5, Hall6],
                vec![FrontOfA, BackOfA, FarBackOfA, WayBackOfA],
            ),
            Bronze => (
                vec![Hall2, Hall1, Hall0],
                vec![Hall3, Hall4, Hall5, Hall6],
                vec![FrontOfB, BackOfB, FarBackOfB, WayBackOfB],
            ),
            Copper => (
                vec![Hall3, Hall2, Hall1, Hall0],
                vec![Hall4, Hall5, Hall6],
                vec![FrontOfC, BackOfC, FarBackOfC, WayBackOfC],
            ),
            Desert => (
                vec![Hall4, Hall3, Hall2, Hall1, Hall0],
                vec![Hall5, Hall6],
                vec![FrontOfD, BackOfD, FarBackOfD, WayBackOfD],
            ),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}



impl Position {
    /// Parses a position, assuming it is valid and panicking if it isn't.
    #[allow(dead_code)]
    fn parse_good(input: &str) -> Self {
        let (rest, answer) = Self::parse_nom(input).unwrap();
        assert!(rest.len() == 0);
        answer
    }

    fn parse_nom(input: &str) -> nom::IResult<&str, Self> {
        // This tuple was so long we had to break it into 2 separate tuples
        nom_tuple((
            nom_tuple((
                nom_tag("#############\n#"),
                AmphipodType::parse_nom,
                AmphipodType::parse_nom,
                nom_tag("."),
                AmphipodType::parse_nom,
                nom_tag("."),
                AmphipodType::parse_nom,
                nom_tag("."),
                AmphipodType::parse_nom,
                nom_tag("."),
                AmphipodType::parse_nom,
                AmphipodType::parse_nom,
                nom_tag("#\n"),
            )),
            nom_tuple((
                nom_tag("###"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("###\n"),
            )),
            nom_tuple((
                nom_tag("  #"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#\n"),
            )),
            nom_tuple((
                nom_tag("  #"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#\n"),
            )),
            nom_tuple((
                nom_tag("  #"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#"),
                AmphipodType::parse_nom,
                nom_tag("#\n"),
            )),
            nom_tuple((
                nom_tag("  #########\n"),
            ))
        ))(input).map(
            |(rest, (
                (_, h0, h1, _, h2, _, h3, _, h4, _, h5, h6, _),
                (_, fa, _, fb, _, fc, _, fd, _),
                (_, ba, _, bb, _, bc, _, bd, _),
                (_, ra, _, rb, _, rc, _, rd, _),
                (_, wa, _, wb, _, wc, _, wd, _),
                (_,),
            ))| {(
                rest,
                Position {
                    slots: [h0, h1, h2, h3, h4, h5, h6, fa, fb, fc, fd, ba, bb, bc, bd, ra, rb, rc, rd, wa, wb, wc, wd],
                }
            )}
        )
    }

    fn at(&self, loc: Location) -> Option<AmphipodType> {
        self.slots[loc as usize]
    }

    fn show_opt_amphipod_at(&self, loc: Location) -> &'static str {
        match self.slots[loc as usize] {
            None => ".",
            Some(a) => a.to_str(),
        }
    }

    /// Returns true if the problem is solved.
    fn is_complete(&self) -> bool {
        *self == FINAL_POSITION
    }


    /// Returns a vector of all legal moves from this position. They will be sorted
    /// by value.
    #[allow(dead_code)]
    fn legal_moves_old(&self) -> Vec<Move> {
        let mut answer = Vec::new();

        // -- Moves out of a nook (if it's not YOUR nook or if you are blocking someone) --
        for a in AmphipodType::ALL {
            let front = Location::FRONT_SLOTS[a.nook()];
            let back = Location::BACK_SLOTS[a.nook()];
            let from_opt: Option<Location> = match self.at(front) { // FIXME: Need to apply "deeper" logic for far and way slots
                Some(amph) => {
                    if amph == a { // it's my row; I can only move if I'm blocking someone
                        match self.at(back) {
                            Some(x) if x != a => Some(front),
                            _ => None
                        }
                    } else {
                        Some(front)
                    }
                },
                None => match self.at(back) {
                    Some(amph) => {
                        if amph == a { // it's my row
                            None
                        } else { // not my row; I'm allowed to leave
                            Some(back)
                        }
                    },
                    None => None
                },
            };
            if let Some(from) = from_opt {
                let amph: AmphipodType = self.at(from).unwrap();
                let (left_hall, right_hall, _) = Location::hall_from(a);
                for to in left_hall {
                    match self.at(to) {
                        Some(_) => break, // no more space on the left
                        None => answer.push(Move{amph, from, to}),
                    }
                }
                for to in right_hall {
                    match self.at(to) {
                        Some(_) => break, // no more space on the right
                        None => answer.push(Move{amph, from, to}),
                    }
                }
            }
        }

        // -- Moves into a nook --
        for a in AmphipodType::ALL {
            let front = Location::FRONT_SLOTS[a.nook()];
            let back = Location::BACK_SLOTS[a.nook()];
            let to_opt: Option<Location> = match self.at(front) {
                Some(_) => None,
                None => match self.at(back) {
                    Some(amph) => {
                        if amph == a { // back is filled in properly
                            Some(front)
                        } else { // back has someone else; we can't go in yet
                            None
                        }
                    },
                    None => Some(back),
                },
            };
            if let Some(to) = to_opt {
                let (left_hall, right_hall, _) = Location::hall_from(a);
                'left_hall:
                for from in left_hall {
                    match self.at(from) {
                        Some(amph) => {
                            if amph == a {
                                answer.push(Move{amph, from, to});
                            }
                            break 'left_hall; // no more to the left
                        },
                        None => {}, // keep looking to the left
                    }
                }
                for from in right_hall {
                    match self.at(from) {
                        Some(amph) => {
                            if amph == a {
                                answer.push(Move{amph, from, to});
                            }
                            break; // no more to the right
                        },
                        None => {}, // keep looking to the right
                    }
                }
            }
        }

        // -- Sort and return answer --
        answer.sort();
        answer
    }


    /// Returns a vector of all legal moves from this position. They will be sorted
    /// by value.
    fn legal_moves(&self) -> Vec<Move> {
        let mut answer = Vec::new();
        for nook_a in AmphipodType::ALL {
            answer.extend(self.legal_moves_out_of_nook(nook_a));
            answer.extend(self.legal_moves_into_nook(nook_a));
        }
        answer.sort();
        answer
    }

    fn legal_moves_out_of_nook(&self, nook_a: AmphipodType) -> Vec<Move> {
        let (left_hall, right_hall, nook) = Location::hall_from(nook_a);
        let amph: AmphipodType;
        let from: Location;
        let mut nook_iter = nook.iter();
        loop {
            match nook_iter.next() {
                None => { // out of nook locations
                    return Vec::new();
                },
                Some(nook_loc) => {
                    match self.at(*nook_loc) {
                        None => {}, // this nook_loc is empty; continue the loop
                        Some(amph_found) => {
                            // if it's in its own nook then it can move ONLY if it's blocking someone
                            if amph_found == nook_a && nook_iter.all(|x| self.at(*x).expect("empty slot behind full one") == nook_a) {
                                return Vec::new();
                            }
                            from = *nook_loc;
                            amph = amph_found;
                            break; // exit loop
                        }
                    }
                },
            }
        }

        let mut answer: Vec<Move> = Vec::new();
        for hall in [left_hall, right_hall] {
            'hall:
            for to in hall {
                match self.at(to) {
                    Some(_) => break 'hall, // no more space on this side
                    None => answer.push(Move{amph, from, to: to}),
                }
            }
        }
        answer
    }

    fn legal_moves_into_nook(&self, nook_a: AmphipodType) -> Vec<Move> {
        let (left_hall, right_hall, nook) = Location::hall_from(nook_a);
        let amph: AmphipodType = nook_a;
        let to: Location;
        let mut nook_iter = nook.iter().rev();
        loop {
            match nook_iter.next() {
                None => { // out of nook locations
                    return Vec::new();
                },
                Some(nook_loc) => {
                    match self.at(*nook_loc) {
                        Some(a) if a == nook_a => {}, // properly filled, continue upward
                        Some(_) => { // wrong type found; can't fill this nook yet
                            return Vec::new();
                        }
                        None => { // found the first open slot
                            to = *nook_loc;
                            break;
                        },
                    }
                },
            }
        }

        let mut answer: Vec<Move> = Vec::new();
        // -- Try moving from each hall --
        for hall in [left_hall, right_hall] {
            'hall:
            for from in hall {
                match self.at(from) {
                    None => {}, // it's empty, so we can keep searching
                    Some(a) if a == amph => { // found a valid one
                        answer.push(Move{amph, from, to});
                    },
                    Some(_) => { // found a wrong amph; this hall won't work
                        break 'hall;
                    }
                }
            }
        }
        answer
    }


    fn perform(&self, mv: Move) -> Position {
        let mut slots = self.slots.clone();
        slots[mv.from as usize] = None;
        slots[mv.to as usize] = Some(mv.amph);
        Position{slots}
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\
#############
#{}{}.{}.{}.{}.{}{}#
###{}#{}#{}#{}###
  #{}#{}#{}#{}#
  #{}#{}#{}#{}#
  #{}#{}#{}#{}#
  #########
",
            self.show_opt_amphipod_at(Hall0),
            self.show_opt_amphipod_at(Hall1),
            self.show_opt_amphipod_at(Hall2),
            self.show_opt_amphipod_at(Hall3),
            self.show_opt_amphipod_at(Hall4),
            self.show_opt_amphipod_at(Hall5),
            self.show_opt_amphipod_at(Hall6),
            self.show_opt_amphipod_at(FrontOfA),
            self.show_opt_amphipod_at(FrontOfB),
            self.show_opt_amphipod_at(FrontOfC),
            self.show_opt_amphipod_at(FrontOfD),
            self.show_opt_amphipod_at(BackOfA),
            self.show_opt_amphipod_at(BackOfB),
            self.show_opt_amphipod_at(BackOfC),
            self.show_opt_amphipod_at(BackOfD),
            self.show_opt_amphipod_at(FarBackOfA),
            self.show_opt_amphipod_at(FarBackOfB),
            self.show_opt_amphipod_at(FarBackOfC),
            self.show_opt_amphipod_at(FarBackOfD),
            self.show_opt_amphipod_at(WayBackOfA),
            self.show_opt_amphipod_at(WayBackOfB),
            self.show_opt_amphipod_at(WayBackOfC),
            self.show_opt_amphipod_at(WayBackOfD),
        )
    }
}

const FINAL_POSITION: Position = Position{slots: [
    None, None, None, None, None, None, None,
    Some(Amber), Some(Bronze), Some(Copper), Some(Desert),
    Some(Amber), Some(Bronze), Some(Copper), Some(Desert),
    Some(Amber), Some(Bronze), Some(Copper), Some(Desert),
    Some(Amber), Some(Bronze), Some(Copper), Some(Desert),
]};



impl Move {
    /// Calculate the cost of this move
    fn cost(&self) -> Cost {
        distance(self.from, self.to) * self.amph.step_cost()
    }

    /// Calculate the "value" of this move -- which a heuristic for the order I want
    /// to try things
    fn value(&self) -> Cost {
        // FIXME: I want things like putting away valuable thing before getting out other stuff.
        //   But for now it's just pretty much "use cheapest".
        self.cost()
    }

    fn sort_tuple(&self) -> (Cost, Location, Location, AmphipodType) {
        (self.value(), self.to, self.from, self.amph)
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_tuple().cmp(&other.sort_tuple())
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


// ======== Functions ========

const DISTANCE_MAP: [[Cost; Location::NUM_VALUES]; Location::NUM_VALUES] = [
    //h0 h1 h2 h3 h4 h5 h6 fa fb fc fd ba bb bc bd ra rb rc rd wa wb wc wd
    [  0, 1, 3, 5, 7, 9,10, 3, 5, 7, 9, 4, 6, 8,10, 5, 7, 9,11, 6, 8,10,12], // h0
    [  1, 0, 2, 4, 6, 8, 9, 2, 4, 6, 8, 3, 5, 7, 9, 4, 6, 8,10, 5, 7, 9,11], // h1
    [  3, 2, 0, 2, 4, 6, 7, 2, 2, 4, 6, 3, 3, 5, 7, 4, 4, 6, 8, 5, 5, 7, 9], // h2
    [  5, 4, 2, 0, 2, 4, 5, 4, 2, 2, 4, 5, 3, 3, 5, 6, 4, 4, 6, 7, 5, 5, 7], // h3
    [  7, 6, 4, 2, 0, 2, 3, 6, 4, 2, 2, 7, 5, 3, 3, 8, 6, 4, 4, 9, 7, 5, 5], // h4
    [  9, 8, 6, 4, 2, 0, 1, 8, 6, 4, 2, 9, 7, 5, 3,10, 8, 6, 4,11, 9, 7, 5], // h5
    [ 10, 9, 7, 5, 3, 1, 0, 9, 7, 5, 3,10, 8, 6, 4,11, 9, 7, 5,12,10, 8, 6], // h6
    [  3, 2, 2, 4, 6, 8, 9, 0, 4, 6, 8, 1, 5, 7, 9, 2, 6, 8,10, 3, 7, 9,11], // fa
    [  5, 4, 2, 2, 4, 6, 7, 4, 0, 4, 6, 5, 1, 5, 7, 6, 2, 6, 8, 7, 3, 7, 9], // fb
    [  7, 6, 4, 2, 2, 4, 5, 6, 4, 0, 4, 7, 5, 1, 5, 8, 6, 2, 6, 9, 7, 3, 7], // fc
    [  9, 8, 6, 4, 2, 2, 3, 8, 6, 4, 0, 9, 7, 5, 1,10, 8, 6, 2,11, 9, 7, 3], // fd
    [  4, 3, 3, 5, 7, 9,10, 1, 5, 7, 9, 0, 6, 8,10, 1, 7, 9,11, 2, 8,10,12], // ba
    [  6, 5, 3, 3, 5, 7, 8, 5, 1, 5, 7, 6, 0, 6, 8, 7, 1, 7, 9, 8, 2, 8,10], // bb
    [  8, 7, 5, 3, 3, 5, 6, 7, 5, 1, 5, 8, 6, 0, 6, 9, 7, 1, 7,10, 8, 2, 8], // bc
    [ 10, 9, 7, 5, 3, 3, 4, 9, 7, 5, 1,10, 8, 6, 0,11, 9, 7, 1,12,10, 8, 2], // bd
    [  5, 4, 4, 6, 8,10,11, 2, 6, 8,10, 1, 7, 9,11, 0, 8,10,12, 1, 9,11,13], // ra
    [  7, 6, 4, 4, 6, 8, 9, 6, 2, 6, 8, 7, 1, 7, 9, 8, 0, 8,10, 9, 1, 9,11], // rb
    [  9, 8, 6, 4, 4, 6, 7, 8, 6, 2, 6, 9, 7, 1, 7,10, 8, 0, 8,11, 9, 1, 9], // rc
    [ 11,10, 8, 6, 4, 4, 5,10, 8, 6, 2,11, 9, 7, 1,12,10, 8, 0,13,11, 9, 1], // rd
    [  6, 5, 5, 7, 9,11,12, 3, 7, 9,11, 2, 8,10,12, 1, 9,11,13, 0,10,12,14], // wa
    [  8, 7, 5, 5, 7, 9,10, 7, 3, 7, 9, 8, 2, 8,10, 9, 1, 9,11,10, 0,10,12], // wb
    [ 10, 9, 7, 5, 5, 7, 8, 9, 7, 3, 7,10, 8, 2, 8,11, 9, 1, 9,12,10, 0,10], // wc
    [ 12,11, 9, 7, 5, 5, 6,11, 9, 7, 3,12,10, 8, 2,13,11, 9, 1,14,12,10, 0], // wd
];

/// Returns the number of "steps" between 2 locations.
fn distance(loc1: Location, loc2: Location) -> Cost {
    DISTANCE_MAP[loc1 as usize][loc2 as usize]
}



/// Returns some (Vec<Move>, Cost) that will "solve" this position or None if it
/// is unsolvable.
fn best_solution(position: &Position) -> Option<(Vec<Move>, Cost)> {
    match best_solution_internal(position, 0, None) {
        None => None,
        Some((path_list, cost)) => {
            let mut path_vec: Vec<Move> = Vec::new();
            path_vec.extend(path_list.iter());
            Some((path_vec, cost))
        },
    }
}

/// Internal recursive routine for best_solution().
/// Inputs: position, cost of moves taken so far, best known cost (prune if we exceed this).
/// Outputs: moves from here to the end, cost of the entire path
fn best_solution_internal(position: &Position, cost_to_here: Cost, mut best_known_cost: Option<Cost>) -> Option<(List<Move>, Cost)> {
    if position.is_complete() {
        Some((List::new(), cost_to_here))
    } else {
        let mut answer: Option<(List<Move>, Cost)> = None;
        for mv in position.legal_moves() {
            let cost_to_next = cost_to_here + mv.cost();
            if best_known_cost.is_none() || cost_to_next < best_known_cost.unwrap() {
                if let Some((path, cost)) = best_solution_internal(&position.perform(mv), cost_to_next, best_known_cost) {
                    match answer {
                        None => {
                            best_known_cost = Some(cost); // we found our first cost
                            answer = Some((path.prepend(mv), cost));
                        },
                        Some((_, answer_cost)) => {
                            if answer_cost > cost {
                                answer = Some((path.prepend(mv), cost));
                            }
                        },
                    }
                }
            }
        }
        answer
    }
}



// ======== run() and main() ========


fn run() -> Result<(),InputError> {
    let position: Position = read_maze_file()?;

    println!("---------------------");

    let start = std::time::Instant::now();
    let best_opt = best_solution(&position);
    println!("The solution took took {:?}", start.elapsed());

    match best_opt {
        None => println!("There were no solutions."),
        Some((path, cost)) => {
            println!("At at cost of {} we can do this:", cost);
            for mv in path {
                println!("    {:?}", mv);
            }
        }
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

// ======== Tests ========

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file() {
        let _ = read_maze_file().unwrap();
    }

    #[test]
    fn test_legal_moves_1() {
        let position = Position::parse_good("\
#############
#B..A.C.....#
###.#.#B#D###
  #.#D#C#A#
  #A#B#C#D#
  #A#B#C#D#
  #########
");
        assert_eq!(
            vec![
                Move{amph: Amber, from: Hall2, to: BackOfA},
                Move{amph: Bronze, from: FrontOfC, to: Hall4},
                Move{amph: Bronze, from: FrontOfC, to: Hall5},
                Move{amph: Bronze, from: FrontOfC, to: Hall6},
                Move{amph: Desert, from: FrontOfD, to: Hall4},
                Move{amph: Desert, from: FrontOfD, to: Hall5},
                Move{amph: Desert, from: FrontOfD, to: Hall6},
            ],
            position.legal_moves()
        );
    }

    #[test]
    fn test_legal_moves_2() {
        let position = Position::parse_good("#############
#A..........#
###.#B#C#D###
  #A#B#C#D#
  #A#B#C#D#
  #A#B#C#D#
  #########\n");
        assert_eq!(
            vec![
                Move{amph: Amber, from: Hall0, to: FrontOfA},
            ],
            position.legal_moves()
        );
    }

    #[test]
    fn test_legal_moves_3() {
        let position = Position::parse_good("#############
#A..........#
###.#A#C#D###
  #B#B#C#D#
  #A#B#C#D#
  #A#B#C#D#
  #########\n");
        assert_eq!(
            vec![
                Move{amph: Amber, from: FrontOfB, to: Hall2},
                Move{amph: Amber, from: FrontOfB, to: Hall3},
                Move{amph: Amber, from: FrontOfB, to: Hall1},
                Move{amph: Amber, from: FrontOfB, to: Hall4},
                Move{amph: Amber, from: FrontOfB, to: Hall5},
                Move{amph: Amber, from: FrontOfB, to: Hall6},
                Move{amph: Bronze, from: BackOfA, to: Hall1},
                Move{amph: Bronze, from: BackOfA, to: Hall2},
                Move{amph: Bronze, from: BackOfA, to: Hall3},
                Move{amph: Bronze, from: BackOfA, to: Hall4},
                Move{amph: Bronze, from: BackOfA, to: Hall5},
                Move{amph: Bronze, from: BackOfA, to: Hall6},
            ],
            position.legal_moves()
        );
    }

    #[test]
    fn test_distance_map_is_symmetric() {
        for loc_1 in Location::ALL {
            for loc_2 in Location::ALL {
                assert_eq!(DISTANCE_MAP[loc_1 as usize][loc_2 as usize], DISTANCE_MAP[loc_2 as usize][loc_1 as usize]);
            }
        }
    }
}

