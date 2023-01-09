
extern crate anyhow;



// ======= Constants =======


// ======= Parsing =======

mod parse {
    use std::fs;
    use nom;
    use nom::{
        IResult,
        bytes::complete::tag,
        combinator::value,
        branch::alt,
        character::complete::line_ending,
        sequence::terminated,
        multi::many0,
    };


    pub fn input() -> Result<Vec<Jet>, anyhow::Error> {
        let s = fs::read_to_string("input/2022/input_17.txt")?;
        match Jet::parse_list(&s) {
            Ok(("", x)) => Ok(x),
            Ok((s, _)) => panic!("Extra input starting at {}", s),
            Err(_) => panic!("Invalid input"),
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Jet {Left, Right}

    impl Jet {
        fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
            alt((
                value(Jet::Left, tag("<")),
                value(Jet::Right, tag(">")),
            ))(input)
        }

        /// Parses a newline-terminated list of LineSpecs
        fn parse_list<'a>(input: &'a str) -> IResult<&'a str, Vec<Self>> {
            terminated(many0(Self::parse), line_ending)(input)
        }
    }

}



// ======= Part 1 Compute =======

mod tetris {
    use std::cmp::max;
    use std::fmt::{Display, Formatter};
    use std::ops::{Add, Sub};
    use im::HashMap;
    use itertools::Itertools;
    use crate::parse::Jet;


    const WIDTH: usize = 7;
    const BIGGEST_VALID_X: usize = WIDTH + 1;


    /*
           4|.......|
           3|...#...|
           2|...#...|
           1|..##...|
           0+-------+
            012345678
     */



    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Coord(pub usize, pub usize);


    // FIXME: don't need both board_height and also tower_height because they are (I think) always the same
    // FIXME: I bet the Board doesn't need to support queries that hit the walls and floors. It would be simpler without them.
    #[derive(Clone)] // FIXME: Do I NEED to be able to clone it?
    pub struct Board {
        board_height: usize,
        tower_height: usize,
        pruned_height: usize,
        grid: Vec<bool>, // true == free
    }

    pub struct Shape {
        offsets: Vec<Coord>,
        top: usize,
    }

    pub struct TetrisGame<'a> {
        board: &'a mut Board,
        shape: &'a Shape,
        piece_loc: Coord,
    }


    impl Coord {
        fn x(&self) -> usize {
            self.0
        }

        fn y(&self) -> usize {
            self.1
        }

        /// Applies a jet. Assumes things are in bounds and if they aren't, we might break things. // FIXME: Handle that better
        fn apply_jet(&self, jet: &Jet) -> Coord {
            match jet {
                Jet::Left => Coord(self.x() - 1, self.y()),
                Jet::Right => Coord(self.x() + 1, self.y()),
            }
        }

        /// Moves downward. Assumes things are in bounds and if they aren't, we might break things. // FIXME: Handle that better
        fn apply_down(&self) -> Coord {
            assert!(self.y() > 0);
            Coord(self.x(), self.y() - 1)
        }
    }

    /// Adds two coordinates. Panics if they go out of bounds.
    impl Add for Coord {
        type Output = Coord;

        fn add(self, rhs: Self) -> Self::Output {
            Coord(self.x() + rhs.x(), self.y() + rhs.y())
        }
    }

    /// When you subtract two coordinates, you get EITHER another Coord OR you get None, if
    /// the subtraction goes "out of bounds".
    impl Sub for Coord {
        type Output = Option<Coord>;

        fn sub(self, rhs: Self) -> Self::Output {
            if self.x() < rhs.x() {
                return None;
            }
            if self.y() < rhs.y() {
                return None;
            }
            let x = self.x() - rhs.x();
            let y = self.y() - rhs.y();
            Some(Coord(x, y))
        }
    }


    impl Board {
        pub fn new() -> Self {
            Self{board_height: 0, tower_height: 0, pruned_height: 0, grid: Vec::new()}
        }

        /// Returns the index into grid for a given (x,y) location. This location might
        /// not exist yet. The coordinate MUST not be a wall or floor (this will panic if it is).
        fn idx(&self, c: &Coord) -> usize {
            assert!(c.x() >= 1 && c.x() <= WIDTH);
            assert!(c.y() >= 1);
            (c.y() - 1) * WIDTH + (c.x() - 1)
        }

        /// Returns true if the x,y location is free. Returns false for all walls, floor,
        /// and spaces with a no-longer-falling piece.
        fn get(&self, c: &Coord) -> bool {
            match c.x() {
                0 => false,
                1..=WIDTH => {
                    if c.y() > self.board_height {
                        true
                    } else if c.y() == 0 {
                        false
                    } else {
                        *self.grid.get(self.idx(c)).unwrap()
                    }
                },
                BIGGEST_VALID_X => false,
                x => panic!("Invalid x value: {}", x),
            }
        }

        /// Grows the grid (if needed) to be able to accommodate Coords with the given
        /// y-value. If no growth is needed, this does nothing.
        fn grow(&mut self, y: usize) {
            if y > self.board_height {
                self.board_height = y;
                self.grid.resize(WIDTH * self.board_height, true);
            }
        }

        /// Modifies the value at a given coordinate. That coordinate MUST not be a wall or
        /// floor (this will panic if it is).
        fn set(&mut self, c: &Coord, b: bool) {
            self.grow(c.y());
            let idx = self.idx(c);
            *self.grid.get_mut(idx).unwrap() = b;
            self.tower_height = max(self.tower_height, c.y());
        }

        /// Returns the current height of the tower.
        pub fn tower_height(&self) -> usize {
            self.pruned_height + self.tower_height
        }


        /// Returns the amount that's safe to prune away. It's possible that this needs more
        /// smarts (if they carefully avoid creating full-across barriers), but I suspect it
        /// will be good enough. It returns a (one-based) height of the last row to KEEP.
        pub fn prune_location(&self) -> usize { // FIXME: Not pub
            let mut prev_row_accessible = vec![true; WIDTH]; // items on previous row that can be reached
            for yz in (0..self.board_height).rev() {
                let this_row_open: Vec<bool> = (0..WIDTH)
                    .map(|xz| self.get(&Coord(xz+1, yz+1)))
                    .collect(); // FIXME: I could provide a way to get this via slice which would be more efficient.
                // new row items are accessible if open and below an accessible item
                let mut new_row_accessible: Vec<bool> = prev_row_accessible.iter()
                    .enumerate()
                    .map(|(xz,prev)| this_row_open[xz] && *prev)
                    .collect();
                // new row items ALSO accessible if they can go left and reach an accessible item
                for xz in (0..(WIDTH - 1)).rev() {
                    if this_row_open[xz] && new_row_accessible[xz + 1] {
                        new_row_accessible[xz] = true;
                    }
                }
                // new row items ALSO accessible if they can go right and reach an accessible item
                for xz in 1..WIDTH {
                    if this_row_open[xz] && new_row_accessible[xz - 1] {
                        new_row_accessible[xz] = true;
                    }
                }
                // If NOTHING is accessible, then we've found the prune location
                if !new_row_accessible.iter().any(|x| *x) {
                    return yz + 2; // +1 to switch to one-based, +1 because it's the prev row we keep
                }
                // Otherwise, move down a row
                prev_row_accessible = new_row_accessible;
            }
            1 // got to the bottom with nothing we could prune. Return 1 to keep the lowest row
        }

        /// Hides all complexity below a certain point.
        pub fn prune(&mut self) {
            // println!("PRUNE -----------------"); // FIXME: Remove
            // println!("board_height={}, tower_height={}, pruned_height={}", self.board_height, self.tower_height, self.pruned_height); // FIXME: Remove
            // println!("{}", self); // FIXME: Remove
            let prune_height = self.prune_location() - 1;
            if prune_height > 0 {
                // println!("    Will prune {}!", prune_height); // FIXME : Remove
                self.pruned_height += prune_height;
                self.board_height -= prune_height;
                self.tower_height -= prune_height;
                self.grid.drain(0..(prune_height * WIDTH));
            }
            // println!("board_height={}, tower_height={}, pruned_height={}", self.board_height, self.tower_height, self.pruned_height); // FIXME: Remove
            // println!("{}", self); // FIXME: Remove
            // println!("END PRUNE -------------"); // FIXME: Remove
        }
    }

    impl Display for Board {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            for yz in (0..self.board_height).rev() {
                write!(f, "|")?;
                for xz in 0..WIDTH {
                    write!(f, "{}", if self.get(&Coord(xz+1, yz+1)) {'.'} else {'#'})?;
                }
                write!(f, "|\n")?;
            }
            write!(f, "+")?;
            for _ in 0..WIDTH {
                write!(f, "-")?;
            }
            write!(f, "+\n")?;
            Ok(())
        }
    }


    impl Shape {
        fn new(offsets: Vec<Coord>) -> Self {
            assert!(offsets.len() > 0);
            let top = offsets.iter().map(|x| x.y()).max().unwrap();
            Shape{offsets, top}
        }

        pub fn known_shapes() -> Vec<Shape> {
            vec![
                Shape::new(vec![Coord(0,0), Coord(1,0), Coord(2,0), Coord(3,0)]),
                Shape::new(vec![Coord(1,0), Coord(0,1), Coord(1,1), Coord(2,1), Coord(1,2)]),
                Shape::new(vec![Coord(0,0), Coord(1,0), Coord(2,0), Coord(2,1), Coord(2,2)]),
                Shape::new(vec![Coord(0,0), Coord(0,1), Coord(0,2), Coord(0,3)]),
                Shape::new(vec![Coord(0,0), Coord(1,0), Coord(0,1), Coord(1,1)]),
            ]
        }

        /// The height above the bottom-left coord of the tallest cell in this shape. Eg: it will
        /// be zero for a horizontal line.
        fn top(&self) -> usize {
            self.top
        }


        /// Given an offset, this checks whether the offset is in this shape. If shapes were
        /// large, this might not be the most efficient implementation, but given they are
        /// small it probably is.
        fn contains(&self, offset: &Coord) -> bool {
            self.offsets.contains(offset)
        }

        /// Tests whether a shape fits at a given location. Returns true if it does; false if not.
        fn fits_at(&self, board: &Board, loc: Coord) -> bool {
            self.offsets.iter()
                .map(|off| loc + *off)
                .all(|c| board.get(&c))
        }

        /// Freezes this piece in place by setting all the locations it covers to be occupied.
        fn freeze(&self, piece_loc: &Coord, board: &mut Board) {
            for offset in self.offsets.iter() {
                let c = *piece_loc + *offset;
                board.set(&c, false);
            }
        }
    }


    /// Returns the right location to place a new piece.
    fn new_piece_loc(board: &Board) -> Coord {
        Coord(3, board.tower_height + 4)
    }


    /// Plays the entire game of Tetris, returning the Board.
    pub fn play(known_shapes: &Vec<Shape>, jets: &Vec<Jet>, num_rocks: usize, prune: bool, print: bool) -> Board {
        let mut board = Board::new();
        let mut have_zoomed_to_end = false;
        let mut shape_iter = known_shapes.iter();
        let mut shape = shape_iter.next().unwrap();
        let mut shape_counter = 0;
        let mut piece_loc = new_piece_loc(&board);
        let mut jet_iter = jets.iter();
        let mut jet_num = 0;
        let mut prev_states: HashMap<usize,(usize,usize,Vec<bool>)>
            = HashMap::new(); // jet_num -> (old_shape_height, old_tower_height, old_grid)
        if print {println!("Starting:\n{}\n", TetrisGame{board: &mut board, shape, piece_loc});}
        loop {
            // FIXME: Remove
            // if have_zoomed_to_end {
            //     println!("NEAR-THE-END: jet_num={jet_num}; shape_counter={shape_counter}; height={}\n{}\n", board.tower_height(),  board); // FIXME: REMOVE
            // }
            let jet_cycle_ended: bool;
            {
                let jet = match jet_iter.next() {
                    Some(jet) => {
                        jet_cycle_ended = false;
                        jet_num += 1;
                        jet
                    }
                    None => {
                        jet_cycle_ended = true;
                        jet_num = 0;
                        jet_iter = jets.iter();
                        jet_iter.next().unwrap()
                    }
                };
                let test_loc = piece_loc.apply_jet(jet);
                if shape.fits_at(&board, test_loc) {
                    piece_loc = test_loc;
                }
                if print {println!("Move {:?}:\n{}\n", jet, TetrisGame{board: &mut board, shape, piece_loc});}
            }
            let shape_cycle_ended: bool;
            {
                let test_loc = piece_loc.apply_down();
                if shape.fits_at(&board, test_loc) {
                    piece_loc = test_loc;
                    if print {println!("Move Down:\n{}\n", TetrisGame{board: &mut board, shape, piece_loc});}
                    shape_cycle_ended = false;
                } else {
                    shape.freeze(&mut piece_loc, &mut board);
                    if print {println!("Freeze:\n{}\n", board);}
                    shape_counter += 1;
                    if shape_counter % 1000000 == 0 {
                        println!("Have completed {} shapes; tower height is {}", shape_counter, board.tower_height);
                    }
                    if shape_counter == num_rocks {
                        if print {println!("And stop now:\n{}\n", board);}
                        return board; // Here is where we exit the loop
                    }
                    shape = match shape_iter.next() {
                        Some(shape) => {
                            shape_cycle_ended = false;
                            shape
                        },
                        None => {
                            shape_cycle_ended = true;
                            shape_iter = known_shapes.iter();
                            shape_iter.next().unwrap()
                        }
                    };
                    piece_loc = new_piece_loc(&board);
                    if print {println!("New Piece:\n{}\n", TetrisGame{board: &mut board, shape, piece_loc});}
                }
                if shape_cycle_ended & !have_zoomed_to_end {
                    // FIXME: Next lines are faking the prune... sort of
                    let mut pruned_board = board.clone();
                    pruned_board.prune();
                    // FIXME: Restore next? Maybe?
                    // board.prune(); // prune the stack to someplace where it filled all the way across
                    if let Some((old_shape_counter, old_tower_height, old_grid)) = prev_states.get(&jet_num) {
                        // FIXME: Remove
                        // println!("Old_Board\n{}\nNew_Board\n{}\n",
                        //          old_grid.iter().rev().enumerate().map(|(i,x)| format!(
                        //              "{}{}",
                        //              if *x {"#"} else {"."},
                        //              if i % WIDTH == WIDTH - 1 {"\n"} else {""}
                        //          )).join(""),
                        //          board.grid.iter().rev().enumerate().map(|(i,x)| format!(
                        //              "{}{}",
                        //              if *x {"#"} else {"."},
                        //              if i % WIDTH == WIDTH - 1 {"\n"} else {""}
                        //          )).join(""),
                        // ); // FIXME: Remove this
                        if *old_grid == pruned_board.grid {
                            // We found a "repeat" spot!!!
                            println!("Found a \"repeat\" spot at jet {jet_num}, shape {}, shape_counter={shape_counter}! It had old_shape_counter={old_shape_counter} and old_tower_height={old_tower_height}", 0);
                            let shapes_per_repeat = shape_counter - old_shape_counter;
                            let height_per_repeat = board.tower_height() - old_tower_height;
                            let shapes_left = num_rocks - shape_counter;
                            let repeats_to_skip = shapes_left / shapes_per_repeat;
                            const SKIP_REPEATS: bool = true; // FIXME: This is only for debugging.
                            if SKIP_REPEATS {
                                println!("WE CAN SKIP NOW. There are {} shapes per repeat and {} height per repeat.", shapes_per_repeat, height_per_repeat); // FIXME: Remove
                                println!("    It also has jet_num={jet_num}; shape_counter={shape_counter}; height={}", board.tower_height()); // FIXME: Remove
                                println!("    It has {shapes_left} shapes left which means we can skip {repeats_to_skip} repeats."); // FIXME: Remove
                                board.pruned_height += repeats_to_skip * height_per_repeat;
                                shape_counter += repeats_to_skip * shapes_per_repeat;
                                if shape_counter == num_rocks { // we've just finished! Return the answer.
                                    return board;
                                } else {
                                    have_zoomed_to_end = true;
                                }
                                println!("HAVE ZOOMED TO END! skipped past {} shapes with {} per repeat to\n{} ", repeats_to_skip * shapes_per_repeat, height_per_repeat, board);
                                println!("There are just {} shapes to go.", num_rocks - shape_counter); // FIXME: remove
                            } else {
                                println!("Choosing NOT to ZOOM TO END! Shapes: {shape_counter}, height: {}\n{}", board.tower_height(),  board);
                                //prev_states.clear(); // FIXME: This will hack it into repeating on the same thing again
                            }
                        }
                    }
                    prev_states.insert(jet_num, (shape_counter, board.tower_height(), pruned_board.grid)); // save this state
                    println!("    Shape cycle ended; jet_num={jet_num}; shape_counter={shape_counter}; height={}", board.tower_height()); // FIXME: Remove
                    // if board.tower_height() >= 36 { panic!("Exit")} // FIXME :Remove
                }
            }
        }
    }


    impl<'a> Display for TetrisGame<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let height = max(self.board.board_height, self.piece_loc.y() + self.shape.top());
            for yz in (0..height).rev() {
                write!(f, "|")?;
                for xz in 0..WIDTH {
                    let c = Coord(xz+1, yz+1);
                    let offset = c - self.piece_loc;
                    let ch = if offset.is_some() && self.shape.contains(&offset.unwrap()) {
                        '@'
                    } else if self.board.get(&c) {
                        '.'
                    } else {
                        '#'
                    };
                    write!(f, "{ch}")?;
                }
                write!(f, "|\n")?;
            }
            write!(f, "+")?;
            for _ in 0..WIDTH {
                write!(f, "-")?;
            }
            write!(f, "+\n")?;
            Ok(())
        }
    }

}




// ======= main() =======

use crate::parse::{Jet, input};
use crate::tetris::{Shape, play};


fn part_a(input: &Vec<Jet>) {
    println!("\nPart a:");
    let known_shapes = Shape::known_shapes();
    let board = play(&known_shapes, input, 2022, false, false);
    println!("Ended with:\n{}\n", board);
    println!("Which has a total height of {}", board.tower_height())
}


fn part_b(input: &Vec<Jet>) {
    println!("\nPart b:");
    let known_shapes = Shape::known_shapes();
    const LARGE_VALUE: usize = 1000000000000;
    let mut board = play(&known_shapes, input, LARGE_VALUE, true, false);
    // FIXME: RESTORE? MAYBE?
    // board.prune();
    // println!("After prune:\n{}\n", board);
    println!("Which has a total height of {}", board.tower_height())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    //part_a(&data);
    part_b(&data);
    Ok(())
}


// ======= Tests =======

