/// I build this so often I made a library. It's a grid x/y of things.

pub use coord::Coord;
pub use direction::Direction;
pub use grid::*;


// ============================================ Direction =============================================

mod direction {
    use std::fmt::{Debug, Display, Formatter};

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
    pub enum Direction {
        East, South, West, North
    }

    use Direction::*;

    impl Direction {
        pub const ALL: [Direction; 4] = [East, South, West, North];


        /// Returns the next Direction clockwise from this one.
        pub fn clockwise(&self) -> Direction {
            match self {
                East => South,
                South => West,
                West => North,
                North => East,
            }
        }

        /// Returns the next Direction clockwise from this one.
        pub fn counter_clockwise(&self) -> Direction {
            match self {
                East => North,
                South => East,
                West => South,
                North => West,
            }
        }

        /// Returns the opposite Direction.
        pub fn reverse(&self) -> Direction {
            match self {
                East => West,
                South => North,
                West => East,
                North => South,
            }
        }
    }

    impl Display for Direction {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", match self {
                East => 'E',
                South => 'S',
                West => 'W',
                North => 'N',
            })
        }
    }
}

// ============================================ Coord =============================================

mod coord {
    use std::fmt::{Debug, Display, Formatter};
    use std::ops::{Add, AddAssign};
    use std::cmp::{Ordering, PartialOrd};
    use std::cmp;
    use super::Direction;


    #[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
    pub struct Coord(pub usize, pub usize);


    impl Coord {
        /// Convenient alias for coord.0.
        pub fn x(&self) -> usize {
            self.0
        }

        /// Convenient alias for coord.1.
        pub fn y(&self) -> usize {
            self.1
        }

        /// This returns an iterator that will go through all the Coords that are less than this
        /// Coord (all smaller x and y values), changing x fastest, and then y.
        pub fn range_by_rows(&self) -> impl Iterator<Item=Coord> {
            ByRowsCoordIterator::new(*self)
        }

        /// This returns an iterator that will go through all the Coords that are less than this
        /// Coord (all smaller x and y values), changing y fastest, and then x.
        pub fn range_by_cols(&self) -> impl Iterator<Item=Coord> {
            ByColsCoordIterator::new(*self)
        }

        /// This gives the coord that is one step away in the given direction, or None if
        /// stepping that way would take us below 0. See also bounded_step() that ensures we
        /// stay within some rectangle and safe_step() that is simpler to use if we know
        /// that it won't go out of bounds.
        pub fn step(&self, dir: Direction) -> Option<Coord> {
            match dir {
                Direction::East => Some(Coord(self.x() + 1, self.y())),
                Direction::South => Some(Coord(self.x(), self.y() + 1)),
                Direction::West => if self.x() > 0 {Some(Coord(self.x() - 1, self.y()))} else {None},
                Direction::North => if self.y() > 0 {Some(Coord(self.x(), self.y() - 1))} else {None},
            }
        }

        /// This gives the coord that is one step away in the given direction. It panics if
        /// that goes below 0 -- use it only when it is known to be safe.
        pub fn safe_step(&self, dir: Direction) -> Coord {
            match dir {
                Direction::East => Coord(self.x() + 1, self.y()),
                Direction::South => Coord(self.x(), self.y() + 1),
                Direction::West => Coord(self.x() - 1, self.y()),
                Direction::North => Coord(self.x(), self.y() - 1),
            }
        }

        /// This gives the coord that is one step away in the given direction. But, it returns
        /// None instead if the step would take us below 0 or beyond the given bound.
        pub fn bounded_step(&self, dir: Direction, bound: Coord) -> Option<Coord> {
            match dir {
                Direction::East => if self.x() + 1 < bound.x() {Some(Coord(self.x() + 1, self.y()))} else {None},
                Direction::South => if self.y() + 1 < bound.y() {Some(Coord(self.x(), self.y() + 1))} else {None},
                Direction::West => if self.x() > 0 {Some(Coord(self.x() - 1, self.y()))} else {None},
                Direction::North => if self.y() > 0 {Some(Coord(self.x(), self.y() - 1))} else {None},
            }
        }

        /// This returns all the adjacent neighbors of this coord. It will *not* include any
        /// neighbors that would have an x or y coordinate < 0, or which would not be < bound.
        pub fn neighbors(&self, bound: Coord) -> Vec<Coord> {
            let mut answer = Vec::with_capacity(4);
            if self.0 > 0 { answer.push(Coord(self.0 - 1, self.1)) }
            if self.1 > 0 { answer.push(Coord(self.0, self.1 - 1)) }
            if self.0 + 1 < bound.0 { answer.push(Coord(self.0 + 1, self.1)) }
            if self.1 + 1 < bound.1 { answer.push(Coord(self.0, self.1 + 1)) }
            answer
        }

        /// This returns all the adjacent neighbors of this coord. It will panic if the coord
        /// given has x=0 or y=0 (so a neighbor would be out of bounds), so it should only
        /// be called if we are sure that is safe.
        pub fn safe_neighbors(&self) -> [Coord; 4] {
            [
                Coord(self.0 - 1, self.1),
                Coord(self.0, self.1 - 1),
                Coord(self.0 + 1, self.1),
                Coord(self.0, self.1 + 1),
            ]
        }

        /// This returns the directions of all adjacent neighbors of this coord. It will *not*
        /// include the direction to any neighbors hat would have an x or y coordinate < 0, or
        /// which would not be < bound.
        pub fn neighbor_directions(&self, bound: Coord) -> Vec<Direction> {
            let mut answer = Vec::with_capacity(4);
            use Direction::*;
            if self.0 > 0 { answer.push(West) }
            if self.1 > 0 { answer.push(North) }
            if self.0 + 1 < bound.0 { answer.push(East) }
            if self.1 + 1 < bound.1 { answer.push(South) }
            answer
        }
    }

    impl Display for Coord {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "({},{})", self.0, self.1)
        }
    }

    impl Add for Coord {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Coord(self.0 + rhs.0, self.1 + rhs.1)
        }
    }

    impl AddAssign for Coord {
        fn add_assign(&mut self, rhs: Self) {
            self.0 += rhs.0;
            self.1 += rhs.1;
        }
    }

    /// Defines a <= b where a and b are Coord values if and only if the x coordinate is
    /// <= and also the y coordinate are <=. This is a partial order -- it's quite likely
    /// that a <= b is false AND b <= a is false.
    impl PartialOrd for Coord {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            use cmp::Ordering::*;
            match (self.0.cmp(&other.0), self.1.cmp(&other.1)) {
                (Less, Less) => Some(Less),
                (Equal, Equal) => Some(Equal),
                (Greater, Greater) => Some(Greater),
                _ => None,
            }
        }
    }

    impl Default for Coord {
        fn default() -> Self {
            Coord(0,0)
        }
    }

    struct ByRowsCoordIterator {
        bound: Coord,
        next: Option<Coord>,
    }

    impl ByRowsCoordIterator {
        /// Create a new ByRowsCoordIterator with the given bound.
        fn new(bound: Coord) -> Self {
            let next = if bound.x() == 0 || bound.y() == 0 {
                None
            } else {
                Some(Coord(0,0))
            };
            Self{bound, next}
        }
    }

    impl Iterator for ByRowsCoordIterator {
        type Item = Coord;

        fn next(&mut self) -> Option<Self::Item> {
            let answer = self.next;
            if let Some(prev) = self.next {
                self.next = if prev.0 + 1 < self.bound.0 {
                    Some(Coord(prev.0 + 1, prev.1))
                } else if prev.1 + 1 < self.bound.1 {
                    Some(Coord(0, prev.1 + 1))
                } else {
                    None
                }
            }
            answer
        }
    }

    struct ByColsCoordIterator {
        bound: Coord,
        next: Option<Coord>,
    }

    impl ByColsCoordIterator {
        /// Create a new ByColsCoordIterator with the given bound.
        fn new(bound: Coord) -> Self {
            let next = if bound.x() == 0 || bound.y() == 0 {
                None
            } else {
                Some(Coord(0,0))
            };
            Self{bound, next}
        }
    }

    impl Iterator for ByColsCoordIterator {
        type Item = Coord;

        fn next(&mut self) -> Option<Self::Item> {
            let answer = self.next;
            if let Some(prev) = self.next {
                self.next = if prev.1 + 1 < self.bound.1 {
                    Some(Coord(prev.0, prev.1 + 1))
                } else if prev.0 + 1 < self.bound.0 {
                    Some(Coord(0 + 1, prev.1))
                } else {
                    None
                }
            }
            answer
        }
    }
}


// ============================================ Grid =============================================

mod grid {
    use std::fmt::{Debug, Display, Formatter};
    use std::error::Error;
    use super::Coord;


    pub struct Grid<T> {
        bound: Coord,
        data: Vec<T>,
    }


    impl<T> Grid<T> {
        // FIXME: As a library, maybe get() and set() should returns errors instead of
        //   panicing when the coord is out of bounds. But this is more convenient.

        /// Gets the value at a location
        pub fn get(&self, coord: Coord) -> &T {
            assert!(coord < self.bound);
            &self.data[self.bound.0 * coord.1 + coord.0]
        }

        /// Gets a mutable reference to the value at a location.
        pub fn get_mut(&mut self, coord: Coord) -> &mut T {
            assert!(coord < self.bound);
            &mut self.data[self.bound.0 * coord.1 + coord.0]
        }

        /// Sets a value at a location.
        pub fn set(&mut self, coord: Coord, val: T) {
            *self.get_mut(coord) = val;
        }

        /// Returns tbe bound on the Grid.
        pub fn bound(&self) -> Coord {
            self.bound
        }

    }

    impl<T: Debug> Debug for Grid<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            for coord in self.bound.range_by_rows() {
                if coord.x() == 0 {
                    writeln!(f)?;
                }
                write!(f, "{:?}", self.get(coord))?;
            }
            writeln!(f)
        }
    }


    impl<T: Display> Display for Grid<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            for coord in self.bound.range_by_rows() {
                if coord.x() == 0 {
                    writeln!(f)?;
                }
                write!(f, "{}", self.get(coord))?;
            }
            writeln!(f)
        }
    }

    impl<T: Default> Grid<T> {
        /// Creates a Grid of the given size by filling in the default value everywhere.
        pub fn new_default(bound: Coord) -> Self {
            let data: Vec<T> = bound.range_by_rows().map(|_| T::default()).collect();
            Self{bound, data}
        }
    }

    /// An error type to return from the from_char_string() method.
    #[derive(Debug)]
    pub enum GridReadError<ItemE: Error> {
        ItemConversionError(ItemE),
        RowsOfUnevenLength,
    }

    impl<ItemE: Error> Display for GridReadError<ItemE> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                GridReadError::ItemConversionError(err) => {
                    write!(f, "error converting item {}", err)
                }
                GridReadError::RowsOfUnevenLength => {
                    write!(f, "rows are of uneven length")
                }
            }
        }
    }

    /// Any Item conversion error can become a GridReadError.
    impl<ItemE: Error> From<ItemE> for GridReadError<ItemE> {
        fn from(value: ItemE) -> Self {
            GridReadError::ItemConversionError(value)
        }
    }

    //
    impl<ItemE: Error> Error for GridReadError<ItemE> {
        // FIXME: I would LIKE to do the following, so my Error type follows the normal rules
        //   for Error and can refer to its source. But the below doesn't compile (because
        //   err doesn't live long enough) and I don't know how to do it right.
        //
        // fn source(&self) -> Option<&(dyn Error + 'static)> {
        //     match self {
        //         GridReadError::ItemConversionError(err) => None,
        //         GridReadError::RowsOfUnevenLength => None,
        //     }
        // }
    }


    impl<T: TryFrom<char,Error=ItemE>, ItemE: Error> Grid<T> {
        /// If you happen to have a grid of characters (in an &str) in which each character
        /// indicates a particular item, and there is a TryFrom to create a T from a character,
        /// then this function can be used to construct a Grid.
        pub fn from_char_string(s: &str, ) -> Result<Self, GridReadError<ItemE>> {
            let mut width = 0;
            let mut height = 0;
            let mut data: Vec<T> = Vec::new();
            for (y, line) in s.lines().enumerate() {
                let mut row_width = 0;
                for (x, c) in line.chars().enumerate() {
                    data.push( c.try_into()? );
                    assert_eq!(row_width, x);
                    row_width += 1;
                }
                if y == 0 {
                    width = row_width;
                } else {
                    if row_width != width {
                        return Err(GridReadError::RowsOfUnevenLength);
                    }
                }
                assert_eq!(height, y);
                height += 1;
            }
            let bound = Coord(width, height);
            Ok(Self{bound, data})
        }
    }

    impl<T> IntoIterator for Grid<T> {
        type Item = T;
        type IntoIter = std::vec::IntoIter<T>;

        fn into_iter(self) -> Self::IntoIter {
            self.data.into_iter()
        }
    }

    impl<'a, T> IntoIterator for &'a Grid<T> {
        type Item = &'a T;
        type IntoIter = core::slice::Iter<'a, T>;

        fn into_iter(self) -> Self::IntoIter {
            (&self.data).into_iter()
        }
    }

    impl<'a, T> IntoIterator for &'a mut Grid<T> {
        type Item = &'a mut T;
        type IntoIter = core::slice::IterMut<'a, T>;

        fn into_iter(self) -> Self::IntoIter {
            (&mut self.data).into_iter()
        }
    }

    impl<T> Grid<T> {
        /// Iterate over (references to) the items in the array. Will go rows-first.
        pub fn iter(&self) -> impl Iterator<Item=&T> {
            (&self.data).iter()
        }

        /// Iterate over (references to) the items in the array. Will go rows-first.
        pub fn iter_mut(&mut self) -> impl Iterator<Item=& mut T> {
            (&mut self.data).iter_mut()
        }
    }


    /// An error type for converting from a square grid.
    #[derive(Debug)]
    pub struct RowsOfUnevenLengthError;

    impl Display for RowsOfUnevenLengthError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "row lengths are not even")
        }
    }

    impl Error for RowsOfUnevenLengthError {}

    /// Support for converting from a "square vector" to a Grid. The rows must be of equal length.
    impl<T: Clone> TryFrom<&Vec<Vec<T>>> for Grid<T> {
        type Error = RowsOfUnevenLengthError;

        fn try_from(value: &Vec<Vec<T>>) -> Result<Self, Self::Error> {
            if value.len() == 0 || value.len() == 1 && value[0].len() ==  0 {
                Ok(Self{bound: Coord(0,0), data: Vec::new()})
            } else {
                // -- check that the rows are the same length --
                let width = value[0].len();
                if value.iter().any(|row| row.len() != width) {
                    return Err(RowsOfUnevenLengthError)
                }

                // -- copy the data over --
                let bound = Coord(width, value.len());
                let data = value.iter()
                    .flat_map(|row| row.iter()
                        .map(|item| item.clone())
                    )
                    .collect();
                Ok(Self{bound, data})
            }
        }
    }

    /// Support for converting from a "square vector" to a Grid. The rows must be of equal length.
    impl<T: Clone> TryFrom<Vec<Vec<T>>> for Grid<T> {
        type Error = RowsOfUnevenLengthError;

        fn try_from(value: Vec<Vec<T>>) -> Result<Self, Self::Error> {
            (&value).try_into()
        }
    }

}
