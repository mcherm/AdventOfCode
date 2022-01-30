use std::fmt::{Display, Formatter};
// use std::collections::HashMap;
use itertools::Itertools;
use rand::random;


type Cell = bool;

pub trait HLifeBlock {
    /// Gets the size of the block
    fn size(&self) -> usize;

    /// Find the value at location (x,y). Assumes x < self.size() and
    /// x < self.size().
    fn val(&self, x: usize, y: usize) -> Cell;

    /// Returns a square array of the bits of the block
    fn get_bits(&self) -> Vec<Vec<Cell>>;

    /// Returns one of the 4 quadrants one size smaller
    fn get_quadrant(&self, x: bool, y: bool) -> &dyn HLifeBlock;
}


impl Display for dyn HLifeBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = self.get_bits().iter().map(
            |row| row.iter().map(|b| if *b {"#"} else {"."}).join("")
        ).join("\n");
        writeln!(f, "{}", text)
    }
}


/// Represents a square, power-of-two life board, evaluated using something resembling
/// HashLife.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct HLifeBlock0 {
    content: Cell,
}

impl HLifeBlock0 {
    fn size() -> usize {1}

    fn new(data: Vec<Vec<Cell>>) -> Self {
        assert!(data.len() == Self::size());
        assert!(data.iter().all(|x| x.len() == Self::size()));
        let content = data[0][0];
        Self{content}
    }

    // FIXME: Remove later
    // fn new_i(data: impl IntoIterator<Item = impl IntoIterator<Item = Cell>>) -> Self {
    //     let content = data.into_iter().next().unwrap().into_iter().next().unwrap();
    //     Self{content}
    // }
}

impl HLifeBlock for HLifeBlock0 {
    fn size(&self) -> usize {Self::size()}

    fn val(&self, x: usize, y: usize) -> Cell {
        assert!(x < self.size());
        assert!(y < self.size());
        self.content
    }

    fn get_bits(&self) -> Vec<Vec<Cell>> {
        vec![vec![self.content]]
    }

    fn get_quadrant(&self, _x: bool, _y: bool) -> &dyn HLifeBlock {
        panic!();
    }
}

struct HLifeBlockQuad1 {
    quads: [HLifeBlock0; 4],
}

impl HLifeBlockQuad1 {
    fn size() -> usize {2}

    fn new(data: Vec<Vec<Cell>>) -> Self {
        assert!(data.len() == Self::size());
        assert!(data.iter().all(|x| x.len() == Self::size()));
        let quads = [
            HLifeBlock0::new(vec![vec![data[0][0]]]),
            HLifeBlock0::new(vec![vec![data[0][1]]]),
            HLifeBlock0::new(vec![vec![data[1][0]]]),
            HLifeBlock0::new(vec![vec![data[1][1]]]),
        ];
        Self{quads}
    }
}


fn quad_index(x: bool, y: bool) -> usize {
    match (x,y) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    }
}

impl HLifeBlock for HLifeBlockQuad1 {
    fn size(&self) -> usize {Self::size()}

    fn val(&self, x: usize, y: usize) -> Cell {
        assert!(x < self.size());
        assert!(y < self.size());
        let index = quad_index(x >= 1, y >= 1);
        self.quads[index].val(0,0)
    }

    fn get_bits(&self) -> Vec<Vec<Cell>> {
        vec![
            vec![self.quads[0].get_bits()[0][0], self.quads[1].get_bits()[0][0]],
            vec![self.quads[2].get_bits()[0][0], self.quads[3].get_bits()[0][0]],
        ]
    }

    fn get_quadrant(&self, x: bool, y: bool) -> &dyn HLifeBlock {
        &self.quads[quad_index(x,y)]
    }
}

struct HLifeBlockQuad2 {
    quads: [HLifeBlockQuad1; 4],
}

impl HLifeBlockQuad2 {
    fn size() -> usize {4}

    fn new(data: Vec<Vec<Cell>>) -> Self {
        assert!(data.len() == Self::size());
        assert!(data.iter().all(|x| x.len() == Self::size()));
        let mid = Self::size() / 2;
        let quads = [
            HLifeBlockQuad1::new( data.iter().take(mid).map(|x| x.iter().take(mid).copied().collect_vec()).collect_vec() ),
            HLifeBlockQuad1::new( data.iter().take(mid).map(|x| x.iter().skip(mid).copied().collect_vec()).collect_vec() ),
            HLifeBlockQuad1::new( data.iter().skip(mid).map(|x| x.iter().take(mid).copied().collect_vec()).collect_vec() ),
            HLifeBlockQuad1::new( data.iter().skip(mid).map(|x| x.iter().skip(mid).copied().collect_vec()).collect_vec() ),
        ];
        Self{quads}
    }
}

impl HLifeBlock for HLifeBlockQuad2 {
    fn size(&self) -> usize {Self::size()}

    fn val(&self, x: usize, y: usize) -> Cell {
        assert!(x < self.size());
        assert!(y < self.size());
        let mid: usize = self.size() / 2;
        self.quads[quad_index(x >= mid, y >= mid)].val(x % mid, y % mid)
    }

    fn get_bits(&self) -> Vec<Vec<Cell>> {
        let top_left = &self.quads[0].get_bits();
        let top_right = &self.quads[1].get_bits();
        let bot_left = &self.quads[2].get_bits();
        let bot_right = &self.quads[3].get_bits();
        vec![
            top_left[0].iter().chain(top_right[0].iter()).copied().collect_vec(),
            top_left[1].iter().chain(top_right[1].iter()).copied().collect_vec(),
            bot_left[0].iter().chain(bot_right[0].iter()).copied().collect_vec(),
            bot_left[1].iter().chain(bot_right[1].iter()).copied().collect_vec(),
        ]
    }

    fn get_quadrant(&self, x: bool, y: bool) -> &dyn HLifeBlock {
        &self.quads[quad_index(x,y)]
    }
}

enum HLifeBlockKind {
    Pow0(HLifeBlock0),
    Pow1(HLifeBlockQuad1),
    Pow2(HLifeBlockQuad2),
    PowN(HLifeBlockQuadN),
}

impl HLifeBlockKind {
    fn get_trait(&self) -> &dyn HLifeBlock {
        match self {
            HLifeBlockKind::Pow0(b) => b,
            HLifeBlockKind::Pow1(b) => b,
            HLifeBlockKind::Pow2(b) => b,
            HLifeBlockKind::PowN(b) => b,
        }
    }
}


struct HLifeBlockQuadN {
    size: usize,
    quads: [Box<HLifeBlockKind>; 4],
}

impl HLifeBlockQuadN {
    fn new(data: Vec<Vec<Cell>>) -> Self {
        let size = data.len();
        assert!(size >= 4 && size.is_power_of_two());
        let mid = size / 2;
        let new_func = match mid {
            1 => |x| Box::new(HLifeBlockKind::Pow0(HLifeBlock0::new(x))),
            2 => |x| Box::new(HLifeBlockKind::Pow1(HLifeBlockQuad1::new(x))),
            4 => |x| Box::new(HLifeBlockKind::Pow2(HLifeBlockQuad2::new(x))),
            _ => |x| Box::new(HLifeBlockKind::PowN(HLifeBlockQuadN::new(x))),
        };
        let quads = [
            new_func( data.iter().take(mid).map(|x| x.iter().take(mid).copied().collect_vec()).collect_vec() ),
            new_func( data.iter().take(mid).map(|x| x.iter().skip(mid).copied().collect_vec()).collect_vec() ),
            new_func( data.iter().skip(mid).map(|x| x.iter().take(mid).copied().collect_vec()).collect_vec() ),
            new_func( data.iter().skip(mid).map(|x| x.iter().skip(mid).copied().collect_vec()).collect_vec() ),
        ];
        Self{size, quads}
    }
}


impl HLifeBlock for HLifeBlockQuadN {
    fn size(&self) -> usize {self.size}

    fn val(&self, x: usize, y: usize) -> Cell {
        assert!(x < self.size);
        assert!(y < self.size);
        let mid: usize = self.size / 2;
        self.quads[quad_index(x >= mid, y >= mid)].get_trait().val(x % mid, y % mid)
    }

    fn get_bits(&self) -> Vec<Vec<Cell>> {
        let top_left  = &self.quads[0].get_trait().get_bits();
        let top_right = &self.quads[1].get_trait().get_bits();
        let bot_left  = &self.quads[2].get_trait().get_bits();
        let bot_right = &self.quads[3].get_trait().get_bits();
        // FIXME: This is a fixed size in the y direction. I need to make it generalized.
        vec![
            top_left[0].iter().chain(top_right[0].iter()).copied().collect_vec(),
            top_left[1].iter().chain(top_right[1].iter()).copied().collect_vec(),
            top_left[2].iter().chain(top_right[2].iter()).copied().collect_vec(),
            top_left[3].iter().chain(top_right[3].iter()).copied().collect_vec(),
            bot_left[0].iter().chain(bot_right[0].iter()).copied().collect_vec(),
            bot_left[1].iter().chain(bot_right[1].iter()).copied().collect_vec(),
            bot_left[2].iter().chain(bot_right[2].iter()).copied().collect_vec(),
            bot_left[3].iter().chain(bot_right[3].iter()).copied().collect_vec(),
        ]
    }

    fn get_quadrant(&self, x: bool, y: bool) -> &dyn HLifeBlock {
        self.quads[quad_index(x,y)].get_trait()
    }
}




#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_main() {
        let block0 = HLifeBlock0::new(vec![vec![true]]);
        let v = block0.val(0,0);
        println!("block0 = {}", v);
        println!("----------");

        let block1 = HLifeBlockQuad1::new(vec![vec![true, false], vec![true, true]]);
        println!("block1 = {:?}", block1.get_bits());
        println!("Values:");
        for y in 0..2 {
            for x in 0..2 {
                print!(" {}", block1.val(x,y));
            }
            println!();
        }
        println!("----------");

        let block2 = HLifeBlockQuad2::new(vec![
            vec![false, true, false, true],
            vec![true, true, true,  true],
            vec![true, false, true, false],
            vec![true,  true, true, false],
        ]);
        println!("block2 = {:?}", block2.get_bits());
        println!("block2 quad = {:?}", block2.get_quadrant(true, false).get_bits());
        println!("Values:");
        for y in 0..4 {
            for x in 0..4 {
                print!(" {}", block2.val(x,y));
            }
            println!();
        }
        println!("----------");

        let size = 8;
        let vec3 = (0..(size * size)).into_iter()
            .chunks(size).into_iter()
            .map(|x| x.into_iter().map(|_| random::<bool>()).collect_vec())
            .collect_vec();
        println!("{:?}", vec3);
        let block3 = HLifeBlockQuadN::new(vec3);
        println!("block3 = {:?}", block3.get_bits());
        println!("block3 quad = {:?}", block3.get_quadrant(true, false).get_bits());
        println!("Values:");
        for y in 0..size {
            for x in 0..size {
                print!(" {}", block3.val(x,y));
            }
            println!();
        }
        println!("----------");

        let size = 16;
        let vec4 = (0..(size * size)).into_iter()
            .chunks(size).into_iter()
            .map(|x| x.into_iter().map(|_| random::<bool>()).collect_vec())
            .collect_vec();
        println!("{:?}", vec4);
        let block4 = HLifeBlockQuadN::new(vec4);
        println!("block3 = {:?}", block4.get_bits());
        println!("block3 quad = {:?}", block4.get_quadrant(true, false).get_bits());
        println!("Values:");
        println!("{}", &block4 as &dyn HLifeBlock);
        println!("----------");
    }
}
