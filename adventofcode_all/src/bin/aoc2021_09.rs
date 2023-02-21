
// ======= part_a =======

mod part_a {
    use std::fmt;
    use std::num::ParseIntError;
    use std::fs::File;
    use std::io::{BufRead, BufReader};


    /// An error that we can encounter when reading the input.
    enum InputError {
        IoError(std::io::Error),
        BadInt(ParseIntError),
    }

    impl From<std::io::Error> for InputError {
        fn from(error: std::io::Error) -> Self {
            InputError::IoError(error)
        }
    }

    impl From<ParseIntError> for InputError {
        fn from(error: ParseIntError) -> Self {
            InputError::BadInt(error)
        }
    }

    impl fmt::Display for InputError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InputError::IoError(err)   => write!(f, "{}", err),
                InputError::BadInt(err)    => write!(f, "{}", err),
            }
        }
    }


    #[derive(Debug)]
    struct HeightMap {
        data: Vec<Vec<u8>>,
        width: usize,
        height: usize,
    }

    impl HeightMap {
        fn new(data: Vec<Vec<u8>>) -> HeightMap {
            let height = data.len();
            assert!(height > 0);
            let width = data[0].len();
            assert!(width > 0);
            for row in &data {
                assert!(row.len() == width);
            }
            HeightMap{data, width, height}
        }

        fn get(&self, x: usize, y: usize) -> u8 {
            self.data[y][x]
        }

        fn find_local_mins(&self) -> Vec<u8> {
            let mut mins: Vec<u8> = Vec::new();
            for y in 0..self.height {
                for x in 0..self.width {
                    let this_val = self.get(x,y);
                    let mut is_local_min = true; // assumed, so far
                    if x > 0 && self.get(x-1,y) <= this_val {
                        is_local_min = false;
                    }
                    if x < self.width-1 && self.get(x+1,y) <= this_val {
                        is_local_min = false;
                    }
                    if y > 0 && self.get(x,y-1) <= this_val {
                        is_local_min = false;
                    }
                    if y < self.height-1 && self.get(x,y+1) <= this_val {
                        is_local_min = false;
                    }
                    if is_local_min {
                        mins.push(this_val);
                    }
                }
            }
            mins
        }
    }


    /// Read in the input file.
    fn read_height_map_file() -> Result<HeightMap, InputError> {
        let filename = "input/2021/input_09.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        let mut data: Vec<Vec<u8>> = Vec::new();
        for line in lines {
            let text = line?;
            let mut row: Vec<u8> = Vec::new();
            let chars = text.chars();
            for c in chars {
                let val: u8 = c.to_string().parse::<u8>()?;
                row.push(val);
            }
            data.push(row);
        }

        return Ok(HeightMap::new(data));
    }


    pub fn main() {
        match read_height_map_file() {
            Ok(height_map) => {
                let local_mins = height_map.find_local_mins();
                let risk_level: u32 = local_mins.iter().map(|x| (x+1) as u32).sum();
                println!("risk_level: {:#?}", risk_level);
            },
            Err(err) => println!("Error: {}", err),
        }
    }
}

// ======= part_b =======

mod part_b {
    use std::fmt;
    use std::num::ParseIntError;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::collections::HashSet;


//
// I have a concern. They define a low point as being LOWER than any neighbor.
// But what if it's TIED with a neighbor? That makes it NOT a low point. For
// the moment, I'm assuming that there's a guarantee in our input that this
// will never happen. But really? That's quite a restriction on the height
// map.
//


    /// An error that we can encounter when reading the input.
    enum InputError {
        IoError(std::io::Error),
        BadInt(ParseIntError),
    }

    impl From<std::io::Error> for InputError {
        fn from(error: std::io::Error) -> Self {
            InputError::IoError(error)
        }
    }

    impl From<ParseIntError> for InputError {
        fn from(error: ParseIntError) -> Self {
            InputError::BadInt(error)
        }
    }

    impl fmt::Display for InputError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InputError::IoError(err)   => write!(f, "{}", err),
                InputError::BadInt(err)    => write!(f, "{}", err),
            }
        }
    }


    type Point = (usize,usize);


    #[derive(Debug)]
    struct HeightMap {
        data: Vec<Vec<u8>>,
        width: usize,
        height: usize,
    }

    impl HeightMap {
        fn new(data: Vec<Vec<u8>>) -> HeightMap {
            let height = data.len();
            assert!(height > 0);
            let width = data[0].len();
            assert!(width > 0);
            for row in &data {
                assert!(row.len() == width);
            }
            HeightMap{data, width, height}
        }

        fn get(&self, x: usize, y: usize) -> u8 {
            self.data[y][x]
        }

        fn get_at(&self, p: Point) -> u8 {
            self.get(p.0, p.1)
        }

        fn neighbors(&self, x: usize, y: usize) -> Vec<Point> {
            let mut result = Vec::new();
            if x > 0 {
                result.push((x-1,y));
            }
            if x < self.width-1  {
                result.push((x+1,y));
            }
            if y > 0 {
                result.push((x,y-1));
            }
            if y < self.height-1  {
                result.push((x,y+1));
            }
            result
        }

        fn find_local_mins(&self) -> Vec<(usize,usize)> {
            let mut mins: Vec<(usize,usize)> = Vec::new();
            for y in 0..self.height {
                for x in 0..self.width {
                    let this_val = self.get(x,y);
                    let mut is_local_min = true; // assumed, so far
                    for neighbor in self.neighbors(x,y) {
                        if self.get_at(neighbor) <= this_val {
                            is_local_min = false;
                        }
                    }
                    if is_local_min {
                        mins.push((x,y));
                    }
                }
            }
            mins
        }
    }


    /// Read in the input file.
    fn read_height_map_file() -> Result<HeightMap, InputError> {
        let filename = "input/2021/input_09.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();

        let mut data: Vec<Vec<u8>> = Vec::new();
        for line in lines {
            let text = line?;
            let mut row: Vec<u8> = Vec::new();
            let chars = text.chars();
            for c in chars {
                let val: u8 = c.to_string().parse::<u8>()?;
                row.push(val);
            }
            data.push(row);
        }

        return Ok(HeightMap::new(data));
    }



    #[derive(Debug)]
    struct Basin {
        points: HashSet<Point>,
    }

    impl Basin {

        /// Given a height_map and a starting point, this returns a Basin.
        fn new(height_map: &HeightMap, start: Point) -> Basin {
            let mut points: HashSet<Point> = HashSet::new();
            points.insert(start);
            loop { // loop will end because the size of points cannot grow forever, it is bounded by size of height_map
                let prev_size = points.len();
                // Do a pass of adding every neighbor of any current point
                // Copy current points into a list before iterating them because we'll modify the set while iterating
                let last_points: Vec<Point> = points.iter().map(|x: &Point| *x).collect::<Vec<Point>>();
                for p in last_points {
                    let p_val = height_map.get_at(p);
                    for neighbor in height_map.neighbors(p.0, p.1) {
                        let n_val = height_map.get_at(neighbor);
                        if n_val >= p_val && n_val != 9 {
                            points.insert(neighbor);
                        }
                    }
                }
                if points.len() == prev_size {
                    // It didn't grow, so we must be done discovering the basin
                    return Basin{points}
                }
            }
        }

        fn size(&self) -> usize {
            self.points.len()
        }
    }



    pub fn main() {
        match read_height_map_file() {
            Ok(height_map) => {
                let local_mins = height_map.find_local_mins();
                let mut basins: Vec<Basin> = Vec::new(); //local_mins.map(|p: Point| Basin::new(&height_map, p)).collect();
                for local_min in local_mins {
                    let basin = Basin::new(&height_map, local_min);
                    println!("local min at: {:?} has basin of size {}", local_min, basin.size());
                    basins.push(basin);
                }
                println!("There are {} basins.", basins.len());
                assert!(basins.len() >= 3);
                basins.sort_by_key(|x| x.size());
                let big_basins = &basins[(basins.len()-3)..];
                let mut product = 1;
                for basin in big_basins {
                    product *= basin.size();
                }
                println!("The product of the 3 largest is {}", product);
            },
            Err(err) => println!("Error: {}", err),
        }
    }
}


// ======= main() =======


fn main() -> Result<(), anyhow::Error> {
    println!("Starting...");
    println!("\nPart a:");
    part_a::main();
    println!("\nPart b:");
    part_b::main();
    Ok(())
}
