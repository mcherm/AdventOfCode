
extern crate anyhow;



// ======= Constants =======

const NUM_ROCKS: usize = 2022;

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
    use crate::parse::Jet;
    use crate::NUM_ROCKS;


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
        tower_height: usize,
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
            Self{board_height: 0, tower_height: 0, grid: Vec::new()}
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
            self.tower_height
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
    pub fn play(known_shapes: &Vec<Shape>, jets: &Vec<Jet>, print: bool) -> Board {
        let mut board = Board::new();
        let mut shape_iter = known_shapes.iter().cycle();
        let mut shape = shape_iter.next().unwrap();
        let mut shape_counter = 0;
        let mut piece_loc = new_piece_loc(&board);
        let mut jet_iter = jets.iter().cycle();
        if print {println!("Starting:\n{}\n", TetrisGame{board: &mut board, shape, piece_loc});}
        loop {
            {
                let jet = jet_iter.next().unwrap();
                let test_loc = piece_loc.apply_jet(jet);
                if shape.fits_at(&board, test_loc) {
                    piece_loc = test_loc;
                }
                if print {println!("Move {:?}:\n{}\n", jet, TetrisGame{board: &mut board, shape, piece_loc});}
            }
            {
                let test_loc = piece_loc.apply_down();
                if shape.fits_at(&board, test_loc) {
                    piece_loc = test_loc;
                    if print {println!("Move Down:\n{}\n", TetrisGame{board: &mut board, shape, piece_loc});}
                } else {
                    shape.freeze(&mut piece_loc, &mut board);
                    if print {println!("Freeze:\n{}\n", board);}
                    shape_counter += 1;
                    if shape_counter == NUM_ROCKS {
                        if print {println!("And stop now:\n{}\n", board);}
                        return board; // Here is where we exit the loop
                    }
                    shape = shape_iter.next().unwrap();
                    piece_loc = new_piece_loc(&board);
                    if print {println!("New Piece:\n{}\n", TetrisGame{board: &mut board, shape, piece_loc});}
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
    let board = play(&known_shapes, input, false);
    println!("Ended with:\n{}\n", board);
    println!("Which has a total height of {}", board.tower_height())
}


fn part_b(_input: &Vec<Jet>) {
    println!("\nPart b:");
}


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    let data = input()?;
    part_a(&data);
    part_b(&data);
    Ok(())
}


// ======= Tests =======

