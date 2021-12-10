mod problem_1;
mod problem_2;
mod problem_3;
mod problem_4;
mod problem_5;
mod problem_6;
mod problem_7;
mod problem_8;
mod problem_9;
mod problem_10;
mod problem_11;
mod problem_12;
mod problem_13;
mod problem_14;
mod problem_15;
mod problem_16;
mod problem_17;
mod problem_18;
mod problem_19;
mod problem_20;


fn main() {
    match 20 {
        1 => problem_1::main(),
        2 => problem_2::main(),
        3 => problem_3::main(),
        4 => problem_4::main(),
        5 => problem_5::main(),
        6 => problem_6::main(),
        7 => problem_7::main(),
        8 => problem_8::main(),
        9 => problem_9::main(),
        10 => problem_10::main(),
        11 => problem_11::main(),
        12 => problem_12::main(),
        13 => problem_13::main(),
        14 => problem_14::main(),
        15 => problem_15::main(),
        16 => problem_16::main(),
        17 => problem_17::main(),
        18 => problem_18::main(),
        19 => problem_19::main(),
        20 => problem_20::main(),
        _ => panic!("Not a valid problem.")
    }
}
