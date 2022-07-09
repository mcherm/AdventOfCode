use std::fs;
use std::io;
use json::JsonValue;


#[derive(Debug)]
enum InputError {
    IoError(io::Error),
    JsonError(json::JsonError),
}



impl From<io::Error> for InputError {
    fn from(err: io::Error) -> Self {
        InputError::IoError(err)
    }
}
impl From<json::JsonError> for InputError {
    fn from(err: json::JsonError) -> Self {
        InputError::JsonError(err)
    }
}


fn input() -> Result<JsonValue, InputError> {
    let s = fs::read_to_string("input/2015/12/input.txt")?;
    let js = json::parse(&s)?;
    Ok(js)
}


fn sum_up(js: &JsonValue) -> i32 {
    match js {
        JsonValue::Null => 0,
        JsonValue::Short(_) => 0,
        JsonValue::String(_) => 0,
        JsonValue::Number(_) => js.as_i32().unwrap(),
        JsonValue::Boolean(_) => 0,
        JsonValue::Object(_) => js.entries().map(|(_,val)| sum_up(val)).sum(),
        JsonValue::Array(_) => js.members().map(sum_up).sum(),
    }
}

fn is_red(js: &JsonValue) -> bool {
    match js {
        JsonValue::Object(_) => {
            js.entries().any(|(_,val)| match val {
                JsonValue::Short(_) => val.as_str().unwrap() == "red",
                JsonValue::String(_) => val.as_str().unwrap() == "red",
                _ => false
            })
        }
        _ => false
    }
}

fn sum_up_except_red(js: &JsonValue) -> i32 {
    match js {
        JsonValue::Null => 0,
        JsonValue::Short(_) => 0,
        JsonValue::String(_) => 0,
        JsonValue::Number(_) => js.as_i32().unwrap(),
        JsonValue::Boolean(_) => 0,
        JsonValue::Object(_) => if is_red(js) {0} else {js.entries().map(|(_,val)| sum_up_except_red(val)).sum()},
        JsonValue::Array(_) => js.members().map(sum_up_except_red).sum(),
    }
}

fn part_a(js: &JsonValue) -> Result<(), InputError> {
    let sum = sum_up(js);
    println!("Adding all numbers we get {}", sum);
    Ok(())
}

fn part_b(js: &JsonValue) -> Result<(), InputError> {
    let sum = sum_up_except_red(js);
    println!("Adding all numbers that aren't red, we get {}", sum);
    Ok(())
}

fn main() -> Result<(), InputError> {
    println!("Starting...");
    let data = input()?;
    part_a(&data)?;
    part_b(&data)?;
    Ok(())
}
