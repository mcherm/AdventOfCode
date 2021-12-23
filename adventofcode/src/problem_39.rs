use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};


/// An error that we can encounter when reading the input.
#[derive(Debug)]
enum InputError {
    IoError(std::io::Error),
    NotRectangular,
    InvalidCharacter,
    InvalidAlgorithm,
    NoAlgorithm,
    NoImage,
    NoBlankLine,
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
            InputError::NotRectangular => write!(f, "Not Rectangular"),
            InputError::InvalidCharacter => write!(f, "Invalid character"),
            InputError::InvalidAlgorithm => write!(f, "Invalid algorithm"),
            InputError::NoAlgorithm => write!(f, "No algorithm"),
            InputError::NoImage => write!(f, "No image"),
            InputError::NoBlankLine => write!(f, "No blank line"),
        }
    }
}

/// Read in the input file.
fn read_image_enhance_file() -> Result<(EnhanceAlgo, Image), InputError> {
    // --- open file ---
    let filename = "data/2021/day/20/input.txt";
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    // --- read algorithm ---
    let algo_text = lines.next().ok_or(InputError::NoAlgorithm)??;
    let algo: EnhanceAlgo = EnhanceAlgo::new(&algo_text)?;

    // --- skip blank line ---
    let blank_line: String = lines.next().ok_or(InputError::NoImage)??;
    if !blank_line.is_empty() {
        return Err(InputError::NoBlankLine);
    }

    // --- read image ---
    let mut image_strings: Vec<String> = Vec::new(); // need this to keep the Strings alive
    while let Some(line) = lines.next() {
        let text: String = line?;
        image_strings.push(text);
    }
    let image_lines: Vec<&str> = image_strings.iter().map(|x| x as &str).collect();
    let image: Image = Image::parse(&image_lines)?;

    // --- return result ---
    Ok((algo, image))
}






type Coord = i32;
type ImageIndex = u16;


struct EnhanceAlgo {
    data: [bool;512]
}
impl EnhanceAlgo {
    /// Constructor takes a line of 512 '.' and '#' characters.
    fn new(text: &str) -> Result<Self,InputError> {
        if text.chars().count() != 512 {
            return Err(InputError::InvalidAlgorithm)
        }
        let mut data: [bool;512] = [false;512];
        for (pos, char) in text.chars().enumerate() {
            data[pos] = match char {
                '#' => true,
                '.' => false,
                _ => return Err(InputError::InvalidCharacter),
            };
        }
        Ok(EnhanceAlgo{data})
    }

    fn eval(&self, index: ImageIndex) -> bool{
        self.data[index as usize]
    }
}


#[derive(Debug, Eq, PartialEq, Clone)]
struct Image {
    left: Coord,
    top: Coord,
    width: usize,
    height: usize,
    pixels: Vec<bool>
}

impl Image {
    fn right(&self) -> Coord {
        self.left + Coord::try_from(self.width).unwrap()
    }

    fn bottom(&self) -> Coord {
        self.top + Coord::try_from(self.height).unwrap()
    }

    fn get(&self, x: Coord, y: Coord) -> bool {
        if x < self.left || x >= self.right() || y < self.top || y >= self.bottom() {
            false
        } else {
            let x_index: usize = usize::try_from(x - self.left).unwrap();
            let y_index: usize = usize::try_from(y - self.top).unwrap();
            self.pixels[x_index + y_index * self.width]
        }
    }

    /// Gets the 9 values surrounding x,y in the form of a number
    fn get_neighborhood(&self, x: Coord, y: Coord) -> ImageIndex {
        let mut answer: ImageIndex = 0;
        for y_delta in -1..=1 {
            for x_delta in -1..=1 {
                let bit = self.get(x + x_delta, y + y_delta);
                answer *= 2;
                answer += if bit {1} else {0};
            }
        }
        answer
    }

    /// Parses from a Vec of lines in the usual format. The input must be "rectangular"
    /// or this raises an error. There must be at least one line with at least one character.
    fn parse(input: &Vec<&str>) -> Result<Self, InputError> {
        assert!(input.len() > 0);
        let height: usize = input.len();
        let width: usize = input[0].chars().count();
        assert!(width > 0);
        let top: Coord = -1 * Coord::try_from(height / 2).unwrap();
        let left: Coord = -1 * Coord::try_from(width / 2).unwrap();
        let mut pixels: Vec<bool> = Vec::with_capacity(width * height);
        for text in input.iter() {
            if text.chars().count() != width {
                return Err(InputError::NotRectangular);
            }
            for char in text.chars() {
                pixels.push(match char {
                    '#' => true,
                    '.' => false,
                    _ => return Err(InputError::InvalidCharacter),
                });
            }
        }
        Ok(Image{left, top, width, height, pixels})
    }

    /// Given an image, runs the "enhancement" algorithm on it to produce a new image.
    fn enhance(&self, algo: &EnhanceAlgo) -> Self {
        let left = &self.left - 1;
        let top = &self.top - 1;
        let width = self.width + 2;
        let height = self.height + 2;
        let mut pixels: Vec<bool> = Vec::with_capacity(width * height);
        for y_offset in 0..width {
            let y = top + Coord::try_from(y_offset).unwrap();
            for x_offset in 0..width {
                let x = left + Coord::try_from(x_offset).unwrap();
                let pixel = algo.eval(self.get_neighborhood(x,y));
                pixels.push(pixel);
            }
        }
        Image{left, top, width, height, pixels}
    }

    /// Returns the number of true pixels.
    fn count_lit(&self) -> usize {
        self.pixels.iter().filter(|x| **x).count()
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in self.top..self.bottom() {
            for x in self.left..self.right() {
                write!(f, "{}", if self.get(x,y) {"#"} else {"."})?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}


fn run() -> Result<(),InputError> {
    let (algo, image) = read_image_enhance_file()?;

    println!("{}", image);
    println!();
    let image_2 = image.enhance(&algo);
    println!("{}", image_2);
    println!();
    let image_3 = image_2.enhance(&algo);
    println!("{}", image_3);
    println!();
    println!("There are {} pixels lit.", image_3.count_lit());

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file() {
        let _ = read_image_enhance_file().unwrap();
    }

    #[test]
    fn read_image() {
        let img = Image::parse(&vec![
            "#..#.",
            "#....",
            "##..#",
            "..#..",
            "..###",
        ]).unwrap();
        assert_eq!(-2, img.left);
        assert_eq!(-2, img.top);
        assert_eq!(5, img.width);
        assert_eq!(5, img.height);
        assert_eq!(true, img.get(-2,-2));
        assert_eq!(false, img.get(0,0));
        assert_eq!(true, img.get(2,2));
        assert_eq!(false, img.get(-5,0));
        assert_eq!(false, img.get(2, 8));
    }

    #[test]
    fn get_neighborhood() {
        let img = Image::parse(&vec![
            "#..#.",
            "#....",
            "##..#",
            "..#..",
            "..###",
        ]).unwrap();
        assert_eq!(34, img.get_neighborhood(0,0));
    }

    #[test]
    fn enhance() {
        let img1 = Image::parse(&vec![
            "#..#.",
            "#....",
            "##..#",
            "..#..",
            "..###",
        ]).unwrap();
        let algo = EnhanceAlgo::new("..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#").unwrap();
        let img2 = img1.enhance(&algo);
        let expect2 = Image::parse(&vec![
            ".##.##.",
            "#..#.#.",
            "##.#..#",
            "####..#",
            ".#..##.",
            "..##..#",
            "...#.#.",
        ]).unwrap();
        assert_eq!(expect2, img2);
    }
}
