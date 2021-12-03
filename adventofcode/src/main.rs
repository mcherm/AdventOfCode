mod problem_1;
mod problem_2;
mod problem_3;
mod problem_4;
mod problem_5;


fn main() {
    match 5 {
        1 => problem_1::main(),
        2 => problem_2::main(),
        3 => problem_3::main(),
        4 => problem_4::main(),
        5 => problem_5::main(),
        _ => panic!("Not a valid problem.")
    }
}
