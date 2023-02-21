
// ======= part_a =======

mod part_a {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Error, ErrorKind};
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::fmt;


    #[derive(Debug)]
    pub enum Movement {
        Forward(isize),
        Down(isize),
        Up(isize),
    }

    fn invalid_movement_error() -> Result<Movement, Error> {
        Err(Error::new(ErrorKind::InvalidData, "Invalid Movement"))
    }

    pub enum MovementError {
        FlyingSubError,
    }

    impl fmt::Display for MovementError {
        // This trait requires `fmt` with this exact signature.
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                MovementError::FlyingSubError => write!(f, "FlyingSubError")
            }
        }
    }

    fn parse_movement(s: &str) -> Result<Movement, Error> {
        lazy_static! {
        static ref MOVEMENT_REGEX: Regex = Regex::new(
            r"^(forward|down|up) ([1-9][0-9]*)$"
        ).unwrap();
    }

        let maybe_captures = MOVEMENT_REGEX.captures(s);
        match maybe_captures {
            Some(captures) => {
                match captures.get(1) {
                    Some(direction) => {
                        match captures.get(2) {
                            Some(distance) => {
                                match distance.as_str().parse() {
                                    Ok(dist) => match direction.as_str() {
                                        "forward" => Ok(Movement::Forward(dist)),
                                        "down" => Ok(Movement::Down(dist)),
                                        "up" => Ok(Movement::Up(dist)),
                                        _ => invalid_movement_error()
                                    }
                                    Err(_) => invalid_movement_error(),
                                }
                            },
                            None => invalid_movement_error()
                        }
                    },
                    None => invalid_movement_error(),
                }
            },
            None => invalid_movement_error(),
        }
    }


    fn read_file_of_movements() -> Result<Vec<Movement>, Error>  {
        let filename = "input/2021/input_02.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();
        let mut movements = Vec::new();
        for line in lines {
            match line {
                Ok(text) => {
                    match parse_movement(&text) {
                        Ok(movement) => movements.push(movement),
                        Err(err) => return Err(err)
                    }
                }
                Err(err) => return Err(err)
            }
        }
        Ok(movements)
    }


    #[derive(Debug)]
    struct Position {
        x: isize,
        depth: isize,
    }

    impl Position {
        fn move_by(&self, movement: Movement) -> Result<Position, MovementError> {
            match movement {
                Movement::Forward(x) => Ok(Position{x: self.x + x, depth: self.depth}),
                Movement::Down(x) => Ok(Position{x: self.x, depth: self.depth + x}),
                Movement::Up(x) => if self.depth - x >= 0 {
                    Ok(Position{x: self.x, depth: self.depth - x})
                } else {
                    Err(MovementError::FlyingSubError)
                }
            }
        }
    }


    fn update_position(movements: Vec<Movement>, start: Position) -> Result<Position, MovementError> {
        let mut position = start;
        for movement in movements {
            position = position.move_by(movement)?
        }
        Ok(position)
    }


    pub fn main() {
        match read_file_of_movements() {
            Ok(movements) => {
                let end_position = update_position(movements, Position{x: 0, depth: 0});
                match end_position {
                    Ok(p) => println!("It ends up at: {:#?} which has 'area' of {}", p, p.x * p.depth),
                    Err(err) => println!("Error: {}", err),
                }
            },
            Err(err) => println!("Error: {}", err)
        };
    }
}

// ======= part_b =======

mod part_b {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::fmt;
    use std::num::ParseIntError;


    /// An error that can occur when a sub is moving.
    pub enum MovementError {
        FlyingSubError,
    }

    impl fmt::Display for MovementError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                MovementError::FlyingSubError => write!(f, "FlyingSubError")
            }
        }
    }


    /// An error that we can encounter when reading the input.
    pub enum InputError {
        IoError(std::io::Error),
        BadLine(isize),
        BadDirection(isize),
        BadInt(ParseIntError),
    }

    impl From<ParseIntError> for InputError {
        fn from(error: ParseIntError) -> Self {
            InputError::BadInt(error)
        }
    }

    impl From<std::io::Error> for InputError {
        fn from(error: std::io::Error) -> Self {
            InputError::IoError(error)
        }
    }

    impl fmt::Display for InputError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InputError::IoError(err)           => write!(f, "{}", err),
                InputError::BadLine(line_num)      => write!(f, "Invalid line on line {}", line_num),
                InputError::BadDirection(line_num) => write!(f, "Invalid direction on line {}", line_num),
                InputError::BadInt(err)            => write!(f, "Invalid number: {}", err),
            }
        }
    }


    /// Object that represents a movement that can be made.
    #[derive(Debug)]
    pub enum Movement {
        Forward(isize),
        Down(isize),
        Up(isize),
    }

    /// This creates a Movement object from the string description of it. It accepts
    /// a line_num argument which will be used in error messages if the Movement
    /// cannot be created.
    fn parse_movement(s: &str, line_num: &isize) -> Result<Movement, InputError> {
        lazy_static! {
        static ref MOVEMENT_REGEX: Regex = Regex::new(
            r"^([a-z]+) ([1-9][0-9]*)$"
        ).unwrap();
    }

        let captures: regex::Captures = MOVEMENT_REGEX.captures(s).ok_or_else(|| InputError::BadLine(*line_num))?;
        let direction: &str = captures.get(1).unwrap().as_str(); // unwrap() is OK because the regex guarantees there is a direction
        let distance: isize = captures.get(2).unwrap().as_str().parse()?; // unwrap() is OK because the regex guarantees there is a distance
        match direction {
            "forward" => Ok(Movement::Forward(distance)),
            "down"    => Ok(Movement::Down(distance)),
            "up"      => Ok(Movement::Up(distance)),
            _         => Err(InputError::BadDirection(*line_num))
        }
    }


    /// This reads the file of movements and returns it as a vector of Movement objects.
    fn read_file_of_movements() -> Result<Vec<Movement>, InputError>  {
        let filename = "input/2021/input_02.txt";
        let file = File::open(filename)?;
        let lines = BufReader::new(file).lines();
        let mut movements = Vec::new();
        let mut line_num: isize = 0;
        for line in lines {
            line_num += 1;
            let movement = parse_movement(&line?, &line_num)?;
            movements.push(movement);
        }
        Ok(movements)
    }


    /// An object to track the current state of the submarine. It is immutable.
    #[derive(Debug)]
    struct SubmarineState {
        x: isize,
        depth: isize,
        aim: isize,
    }

    // Applies a movement and returns the new SubmarineState obtained by applying
    /// that movement.
    impl SubmarineState {
        fn move_by(&self, movement: Movement) -> Result<SubmarineState, MovementError> {
            match movement {
                Movement::Forward(x) => {
                    let new_depth = self.depth + self.aim * x;
                    if new_depth < 0 {
                        Err(MovementError::FlyingSubError)
                    } else {
                        Ok(SubmarineState{x: self.x + x, depth: new_depth, ..*self})
                    }
                },
                Movement::Down(x) => Ok(SubmarineState{aim: self.aim + x, ..*self}),
                Movement::Up(x) => Ok(SubmarineState{aim: self.aim - x, ..*self}),
            }
        }
    }


    /// Applies a whole vector of movements. Could return an error in cases
    /// where the submarine attempts to fly.
    fn apply_movements(movements: Vec<Movement>, start: SubmarineState) -> Result<SubmarineState, MovementError> {
        let mut position = start;
        for movement in movements {
            position = position.move_by(movement)?
        }
        Ok(position)
    }


    pub fn main() {
        match read_file_of_movements() {
            Ok(movements) => {
                let submarine_state_result = apply_movements(movements, SubmarineState{x: 0, depth: 0, aim: 0});
                match submarine_state_result {
                    Ok(p) => println!("It ends up at: {:#?} which has 'area' of {}", p, p.x * p.depth),
                    Err(err) => println!("Error: {}", err),
                }
            },
            Err(err) => println!("Error: {}", err)
        };
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
