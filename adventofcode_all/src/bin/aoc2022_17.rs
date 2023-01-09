
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


    pub struct Board {
        board_height: usize,
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

        /// Applies a jet. Assumes things are in bounds and if they aren't, we might break things.
        fn apply_jet(&self, jet: &Jet) -> Coord {
            match jet {
                Jet::Left => Coord(self.x() - 1, self.y()),
                Jet::Right => Coord(self.x() + 1, self.y()),
            }
        }

        /// Moves downward. Assumes things are in bounds and if they aren't, we might break things.
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
            Self{board_height: 0, pruned_height: 0, grid: Vec::new()}
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

        /// Sets a block at the given coordinate. The coordinate MUST not be a wall or fllor
        /// (this will panic if it is).
        fn occupy(&mut self, c: &Coord) {
            self.grow(c.y());
            let idx = self.idx(c);
            *self.grid.get_mut(idx).unwrap() = false;
        }

        /// Returns the current height of the tower.
        pub fn tower_height(&self) -> usize {
            self.pruned_height + self.board_height
        }

        /// For internal use, this provides a reference to the row of booleans at coord y.
        pub fn row(&self, y: usize) -> &[bool; WIDTH] {
            let y_idx = (y - 1) * WIDTH;
            let y_top = y_idx + WIDTH;
            (self.grid[y_idx..y_top]).try_into().unwrap()
        }


        /// Returns the amount that's safe to prune away. It's possible that this needs more
        /// smarts (if they carefully avoid creating full-across barriers), but I suspect it
        /// will be good enough. It returns a (one-based) height of the last row to KEEP.
        fn prune_location(&self) -> usize {
            let mut prev_row_accessible: [bool; WIDTH] = [true; WIDTH]; // items on previous row that can be reached
            for yz in (0..self.board_height).rev() {
                let this_row_open: &[bool; WIDTH] = self.row(yz+1);
                let mut new_row_accessible: [bool; WIDTH] = [false; WIDTH];
                // new row items are accessible if open and below an accessible item
                for xz in 0..WIDTH {
                    new_row_accessible[xz] = this_row_open[xz] && prev_row_accessible[xz];
                }
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

        /// This works its way down from the top and finds the lowest row that any piece (even a
        /// 1x1) could possibly reach. Then it returns a section of bools (part of self.grid)
        /// that represents all rows above that point, to the highest populated location. This
        /// can be used to compare whether two different Board positions are "the same at least
        /// as far down as it could possibly matter".
        fn get_accessible_grid<'a>(&self) -> Vec<bool> {
            let prune_height = self.prune_location() - 1;
            self.grid[(prune_height * WIDTH)..].to_vec()
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
                board.occupy(&c);
            }
        }
    }


    /// Returns the right location to place a new piece.
    fn new_piece_loc(board: &Board) -> Coord {
        Coord(3, board.board_height + 4)
    }


    /// Plays the entire game of Tetris, returning the Board.
    pub fn play(known_shapes: &Vec<Shape>, jets: &Vec<Jet>, num_rocks: usize, print: bool) -> Board {
        let mut board = Board::new();
        let mut have_zoomed_to_end = false;
        let mut shape_iter = known_shapes.iter();
        let mut shape = shape_iter.next().unwrap();
        let mut shape_counter = 0;
        let mut piece_loc = new_piece_loc(&board);
        let mut jet_iter = jets.iter();
        let mut jet_num = 0;
        let mut prev_states: HashMap<usize,(usize,usize,Vec<bool>)>
            = HashMap::new(); // jet_num -> (old_shape_height, old_tower_height, old_accessible_grid)
        if print {println!("Starting:\n{}\n", TetrisGame{board: &mut board, shape, piece_loc});}
        loop {
            {
                let jet = match jet_iter.next() {
                    Some(jet) => {
                        jet_num += 1;
                        jet
                    }
                    None => {
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
                        if print {println!("Have completed {} shapes; tower height is {}", shape_counter, board.tower_height());}
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
                    let accessible_grid = board.get_accessible_grid();
                    if let Some((old_shape_counter, old_tower_height, old_accessible_grid)) = prev_states.get(&jet_num) {
                        if *old_accessible_grid == accessible_grid {
                            // We found a "repeat" spot!!!
                            println!("Found a \"repeat\" spot at jet {jet_num}, shape {}, shape_counter={shape_counter}! It had old_shape_counter={old_shape_counter} and old_tower_height={old_tower_height}", 0);
                            let shapes_per_repeat = shape_counter - old_shape_counter;
                            let height_per_repeat = board.tower_height() - old_tower_height;
                            let shapes_left = num_rocks - shape_counter;
                            let repeats_to_skip = shapes_left / shapes_per_repeat;
                            if print {
                                println!("WE CAN SKIP NOW. There are {} shapes per repeat and {} height per repeat.", shapes_per_repeat, height_per_repeat);
                                println!("    It also has jet_num={jet_num}; shape_counter={shape_counter}; height={}", board.tower_height());
                                println!("    It has {shapes_left} shapes left which means we can skip {repeats_to_skip} repeats.");
                            }
                            board.pruned_height += repeats_to_skip * height_per_repeat;
                            shape_counter += repeats_to_skip * shapes_per_repeat;
                            if shape_counter == num_rocks { // we've just finished! Return the answer.
                                return board;
                            } else {
                                have_zoomed_to_end = true;
                            }
                            if print {
                                println!("HAVE ZOOMED TO END! skipped past {} shapes with {} per repeat to\n{} ", repeats_to_skip * shapes_per_repeat, height_per_repeat, board);
                            }
                        }
                    }
                    prev_states.insert(jet_num, (shape_counter, board.tower_height(), accessible_grid)); // save this state
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

const PRINT_WORK: bool = false;


fn part_a(input: &Vec<Jet>) {
    println!("\nPart a:");
    let known_shapes = Shape::known_shapes();
    let board = play(&known_shapes, input, 2022, PRINT_WORK);
    if PRINT_WORK { println!("Ended with:\n{}\n", board); }
    println!("It has a total height of {}", board.tower_height())
}


fn part_b(input: &Vec<Jet>) {
    println!("\nPart b:");
    let known_shapes = Shape::known_shapes();
    const LARGE_VALUE: usize = 1000000000000;
    let board = play(&known_shapes, input, LARGE_VALUE, PRINT_WORK);
    println!("It has a total height of {}", board.tower_height())
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}


// ======= Tests =======

