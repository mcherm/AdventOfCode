
// ======= part_a =======

mod part_a {
    use std::fmt;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use regex::Regex;


    /// An error that we can encounter when reading the input.
    #[derive(Debug)]
    enum InputError {
        IoError(std::io::Error),
        BadInt(std::num::ParseIntError),
        InvalidStartPositionLine,
    }

    impl From<std::io::Error> for InputError {
        fn from(error: std::io::Error) -> Self {
            InputError::IoError(error)
        }
    }

    impl From<std::num::ParseIntError> for InputError {
        fn from(error: std::num::ParseIntError) -> Self {
            InputError::BadInt(error)
        }
    }

    impl fmt::Display for InputError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InputError::IoError(err) => write!(f, "{}", err),
                InputError::BadInt(err) => write!(f, "{}", err),
                InputError::InvalidStartPositionLine => write!(f, "Invalid starting position"),
            }
        }
    }

    /// Read in the input file.
    fn read_dice_game_file() -> Result<[usize;2], InputError> {
        // --- open file ---
        let filename = "input/2021/input_21.txt";
        let file = File::open(filename)?;
        let mut lines = BufReader::new(file).lines();

        // --- read start positions ---
        let start_position_line_regex = Regex::new(
            r"^Player [12] starting position: (\d+)$"
        ).unwrap();

        let text: String = lines.next().ok_or(InputError::InvalidStartPositionLine)??;
        let capture = start_position_line_regex.captures(&text).ok_or(InputError::InvalidStartPositionLine)?;
        let p1_pos: usize = capture.get(1).unwrap().as_str().parse()?;

        let text: String = lines.next().ok_or(InputError::InvalidStartPositionLine)??;
        let capture = start_position_line_regex.captures(&text).ok_or(InputError::InvalidStartPositionLine)?;
        let p2_pos: usize = capture.get(1).unwrap().as_str().parse()?;

        // --- return result ---
        Ok([p1_pos, p2_pos])
    }


    struct DeterministicDie {
        next: usize,
        times_rolled: usize,
    }

    struct GameState {
        die: DeterministicDie,
        positions: [usize;2],
        scores: [usize;2],
    }


    impl DeterministicDie {
        fn new() -> Self {
            DeterministicDie{next: 1, times_rolled: 0}
        }

        fn roll(&mut self) -> usize {
            let answer = self.next;
            self.next += 1;
            if self.next == 101 {
                self.next = 1;
            }
            self.times_rolled += 1;
            answer
        }
    }

    impl GameState {
        fn new(positions: [usize;2]) -> Self {
            GameState{die: DeterministicDie::new(), positions, scores: [0;2]}
        }

        fn take_turn(&mut self, player: usize) {
            let roll_sum = self.die.roll() + self.die.roll() + self.die.roll();
            self.positions[player] = (self.positions[player] + roll_sum - 1) % 10 + 1;
            self.scores[player] += self.positions[player];
        }

        fn game_not_over(&self) -> bool {
            self.scores[0] < 1000 && self.scores[1] < 1000
        }

        fn low_score(&self) -> usize {
            *self.scores.iter().min().unwrap()
        }
    }


    fn run() -> Result<(),InputError> {
        let starts = read_dice_game_file()?;
        println!("starts: ({},{})", starts[0], starts[1]);

        let mut game = GameState::new(starts);
        let mut player = 0;
        while game.game_not_over() {
            game.take_turn(player);
            player = (player + 1) % 2;
        }
        println!("After the game we had {} low score and {} rolls for a total of {}",
                 game.low_score(),
                 game.die.times_rolled,
                 game.low_score() * game.die.times_rolled
        );

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
            let _ = read_dice_game_file().unwrap();
        }
    }
}

// ======= part_b =======

mod part_b {
    use std::fmt;
    use std::fmt::Formatter;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use regex::Regex;
    use std::cmp;


    const BOARD_SIZE: usize = 10;
    const WINNING_SCORE: usize = 21;
    const POSSIBLE_SCORES: usize = WINNING_SCORE + 1;
    const NUM_GAMESTATES: usize = BOARD_SIZE * BOARD_SIZE * POSSIBLE_SCORES * POSSIBLE_SCORES;



    /// An error that we can encounter when reading the input.
    #[derive(Debug)]
    enum InputError {
        IoError(std::io::Error),
        BadInt(std::num::ParseIntError),
        InvalidStartPositionLine,
    }

    impl From<std::io::Error> for InputError {
        fn from(error: std::io::Error) -> Self {
            InputError::IoError(error)
        }
    }

    impl From<std::num::ParseIntError> for InputError {
        fn from(error: std::num::ParseIntError) -> Self {
            InputError::BadInt(error)
        }
    }

    impl fmt::Display for InputError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InputError::IoError(err) => write!(f, "{}", err),
                InputError::BadInt(err) => write!(f, "{}", err),
                InputError::InvalidStartPositionLine => write!(f, "Invalid starting position"),
            }
        }
    }

    /// Read in the input file.
    fn read_dice_game_file() -> Result<[usize;2], InputError> {
        // --- open file ---
        let filename = "input/2021/input_21.txt";
        let file = File::open(filename)?;
        let mut lines = BufReader::new(file).lines();

        // --- read start positions ---
        let start_position_line_regex = Regex::new(
            r"^Player [12] starting position: (\d+)$"
        ).unwrap();

        let text: String = lines.next().ok_or(InputError::InvalidStartPositionLine)??;
        let capture = start_position_line_regex.captures(&text).ok_or(InputError::InvalidStartPositionLine)?;
        let p1_pos: usize = capture.get(1).unwrap().as_str().parse()?;

        let text: String = lines.next().ok_or(InputError::InvalidStartPositionLine)??;
        let capture = start_position_line_regex.captures(&text).ok_or(InputError::InvalidStartPositionLine)?;
        let p2_pos: usize = capture.get(1).unwrap().as_str().parse()?;

        // --- return result ---
        Ok([p1_pos, p2_pos])
    }




    #[derive(Debug)]
    struct GameMetaState {
        /// If p0 and p1 are player positions (indexed from zero) and s0 and s1 are player
        /// scores (letting 21 mean "21 or higher") then positions[key(p0,p1,s0,s1)] is the
        /// number of universes with that set of positions and scores.
        universes: [usize;NUM_GAMESTATES],
    }


    /// Given a pair of positions (indexed from 0), and a pair of keys, return the key for it.
    /// This will always be a number from 0 to just-below 48400.
    fn key(p1: usize, p2: usize, s1: usize, s2: usize) -> usize {
        (((p1 * BOARD_SIZE) + p2) * POSSIBLE_SCORES + s1) * POSSIBLE_SCORES + s2
    }

    /// Given a key, return the pair of positions (indexed from 0) and scores in the order
    /// (p0,p1,s0,s1)
    fn unkey(key: usize) -> (usize, usize, usize, usize) {
        let s1 = key % POSSIBLE_SCORES;
        let remainder_1 = key / POSSIBLE_SCORES;
        let s0 = remainder_1 % POSSIBLE_SCORES;
        let remainder_2 = remainder_1 / POSSIBLE_SCORES;
        let p1 = remainder_2 % BOARD_SIZE;
        let p0 = remainder_2 / BOARD_SIZE;
        (p0,p1,s0,s1)
    }


    /// this represents all (roll, #-of-universes-it-happens-in) pairs
    const ROLL_PROBS: [(usize, usize);7] = [
        (3, 1),
        (4, 3),
        (5, 6),
        (6, 7),
        (7, 6),
        (8, 3),
        (9, 1),
    ];


    impl GameMetaState {
        /// This is passed positions with ONE-BASED indexing (not zero based) and creates a new
        /// game in just one universe with those starting positions and a score of 0,0.
        fn new(pos: [usize;2]) -> Self {
            let mut universes = [0; NUM_GAMESTATES];
            universes[key(pos[0] - 1, pos[1] - 1, 0, 0)] = 1;
            GameMetaState{universes}
        }

        /// Takes a turn, populating new universes.
        fn take_turn(&mut self, player: usize) {
            let mut new_universes = [0; NUM_GAMESTATES];
            for old_key in 0..NUM_GAMESTATES {
                let old_count = self.universes[old_key];
                let (p0_old, p1_old, s0_old, s1_old)  = unkey(old_key);

                if s0_old == WINNING_SCORE || s1_old == WINNING_SCORE {
                    // After someone wins, no one moves, no one scores, and no new universes are created
                    new_universes[old_key] += old_count;
                } else {
                    for (roll, weight) in ROLL_PROBS {
                        let new_p = |p_old| (p_old + roll) % BOARD_SIZE;
                        let new_score = |s_old, p_new| cmp::min(WINNING_SCORE, s_old + p_new + 1);
                        let p0_new;
                        let p1_new;
                        let s0_new;
                        let s1_new;
                        match player {
                            0 => {
                                p0_new = new_p(p0_old);
                                p1_new = p1_old;
                                s0_new = new_score(s0_old, p0_new);
                                s1_new = s1_old;
                            },
                            1 => {
                                p0_new = p0_old;
                                p1_new = new_p(p1_old);
                                s0_new = s0_old;
                                s1_new = new_score(s1_old, p1_new);
                            },
                            _ => panic!("Invalid player"),
                        }
                        let new_key = key(p0_new, p1_new, s0_new, s1_new);
                        new_universes[new_key] += weight * self.universes[old_key];
                    }
                }
            }
            self.universes = new_universes;
        }


        /// Returns the number of universes in which neither player has yet won.
        fn num_continuing_universes(&self) -> usize {
            let mut answer = 0;
            for key in 0..NUM_GAMESTATES {
                let (_, _, s0, s1) = unkey(key);
                if s0 != WINNING_SCORE && s1 != WINNING_SCORE {
                    answer += self.universes[key];
                }
            }
            answer
        }

        /// Returns the number of winning universes for [player1, player2]. If there is any
        /// universe in which both players won or neither player won then this panics.
        fn num_winning_universes(&self) -> [usize;2] {
            let mut p1_wins = 0;
            let mut p2_wins = 0;
            for key in 0..NUM_GAMESTATES {
                let count = self.universes[key];
                if count > 0 {
                    match unkey(key) {
                        (_, _, WINNING_SCORE, WINNING_SCORE) => panic!("Both players won a game."),
                        (_, _, WINNING_SCORE, _) => p1_wins += count,
                        (_, _, _, WINNING_SCORE) => p2_wins += count,
                        (_, _, _, _) => panic!("There was a game that no player won."),
                    }
                }
            }
            [p1_wins, p2_wins]
        }
    }

    impl fmt::Display for GameMetaState {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            for key in 0..NUM_GAMESTATES {
                let universes = self.universes[key];
                if universes > 0 {
                    let (p0,p1,s0,s1) = unkey(key);
                    writeln!(f, "There are {} universes with players at {} and {} and scores of {} and {}.",
                             universes, p0 + 1, p1 + 1, s0, s1)?;
                }
            }
            writeln!(f, "So that's {} universes still going.", self.num_continuing_universes())?;
            writeln!(f)
        }
    }


    fn run() -> Result<(),InputError> {
        let starts = read_dice_game_file()?;
        println!("Starting at : ({},{})", starts[0], starts[1]);

        let mut game = GameMetaState::new(starts);
        let mut player = 0;
        let mut turn = 0;
        println!("The game state is:\n{}", game);
        loop {
            turn += 1;
            println!("Beginning turn {}.", turn);
            game.take_turn(player);
            println!("The game state is now:\n{}", game);
            if game.num_continuing_universes() == 0 {
                break;
            }
            player = (player + 1) % 2;
        }
        let [p1_wins, p2_wins] = game.num_winning_universes();
        println!();
        println!("Player 1 wins in {} universes and player 2 in {}.", p1_wins, p2_wins);
        println!("The winner who won more won in {} universes.", cmp::max(p1_wins, p2_wins));

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
            let _ = read_dice_game_file().unwrap();
        }

        #[test]
        fn test_pos_key() {
            assert_eq!(0, key(0, 0, 0, 0));
            assert_eq!(WINNING_SCORE, key(0, 0, 0, WINNING_SCORE));
            assert_eq!(WINNING_SCORE + 1, key(0, 0, 1, 0));
            if WINNING_SCORE > 6 {
                assert_eq!(POSSIBLE_SCORES + 6, key(0, 0, 1, 6));
            }
            assert_eq!(POSSIBLE_SCORES * POSSIBLE_SCORES - 1, key(0, 0, WINNING_SCORE, WINNING_SCORE));
            assert_eq!(NUM_GAMESTATES - 1, key(BOARD_SIZE - 1, BOARD_SIZE - 1, WINNING_SCORE, WINNING_SCORE));
        }

        #[test]
        fn test_unpos_key() {
            assert_eq!((0,0,0,0), unkey(0));
            if WINNING_SCORE > 5 {
                assert_eq!((0,0,0,5), unkey(5));
                assert_eq!((0,0,5,0), unkey(POSSIBLE_SCORES * 5));
            }
            if BOARD_SIZE > 5 {
                assert_eq!((0,5,0,0), unkey(POSSIBLE_SCORES * POSSIBLE_SCORES * 5));
                assert_eq!((5,0,0,0), unkey(POSSIBLE_SCORES * POSSIBLE_SCORES * BOARD_SIZE * 5));
            }
            assert_eq!((BOARD_SIZE - 1, BOARD_SIZE - 1, WINNING_SCORE, WINNING_SCORE), unkey(NUM_GAMESTATES - 1));
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
