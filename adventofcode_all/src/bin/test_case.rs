
#[derive(Debug)]
enum Crab {
    BlueCrab { taste_factor: i32 },
    FiddlerCrab
}

impl Crab {
    /// This does nothing for FiddlerCrabs but it increases the taste_factor for a BlueCrab.
    fn sweeten(&mut self) {
        match self {
            Crab::BlueCrab{mut taste_factor} => {
                taste_factor += 1;
            }
            Crab::FiddlerCrab{} => {}
        }
    }
}

fn main() {
    let mut crab = Crab::BlueCrab{taste_factor: 1};
    crab.sweeten();
    println!("Sweetened crab is {:?}", crab);
}
