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


fn main() {
    match 10 {
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
        _ => panic!("Not a valid problem.")
    }
}
