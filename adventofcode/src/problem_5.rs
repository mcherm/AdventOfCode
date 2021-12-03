use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt;
use std::cmp::Ordering;


const DIAGNOSTIC_LENGTH: usize = 12;

type ReportLine = [u8; DIAGNOSTIC_LENGTH];

/// An error that we can encounter when reading the input.
pub enum InputError {
    IoError(std::io::Error),
    BadLine(isize),
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
        }
    }
}


pub enum DiagnosticError {
    UnclearReading,
}

impl fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiagnosticError::UnclearReading => write!(f, "Unclear Reading"),
        }
    }
}



/// This reads a line from a diagnostic report, returning an InputError if
fn read_report_line(s: &str, line_num: &isize) -> Result<ReportLine, InputError> {
    let mut report_line: ReportLine = [0; DIAGNOSTIC_LENGTH];

    let mut count = 0;
    for c in s.chars() {
        match c {
            '0' => report_line[count] = 0,
            '1' => report_line[count] = 1,
            _   => return Err(InputError::BadLine(*line_num)),
        }
        count += 1;
        if count > DIAGNOSTIC_LENGTH {
            return Err(InputError::BadLine(*line_num));
        }
    }
    if count < DIAGNOSTIC_LENGTH {
        return Err(InputError::BadLine(*line_num));
    }
    Ok(report_line)
}


/// This reads the file containing a diagnostic report and returns it as a vector of Strings,
/// or returns an error.
fn read_diagnostic_report_file() -> Result<Vec<ReportLine>, InputError>  {
    let filename = "data/2021/day/3/input.txt";
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines();
    let mut diagnostic_report = Vec::new();
    let mut line_num: isize = 0;
    for line in lines {
        line_num += 1;
        let report_line = read_report_line(&line?, &line_num)?;
        diagnostic_report.push(report_line);
    }
    Ok(diagnostic_report)
}


struct DiagnosticData {
    gamma: u32,
    epsilon: u32,
}


// This reads a diagnostic report and returns a DiagnosticData.
fn analyze_diagnostics(diagnostic_report: Vec<ReportLine>) -> Result<DiagnosticData,DiagnosticError>  {
    let lines = diagnostic_report.len();
    let mut count_of_ones: [usize; DIAGNOSTIC_LENGTH] = [0; DIAGNOSTIC_LENGTH];
    for report_line in diagnostic_report {
        for bit in 0..DIAGNOSTIC_LENGTH {
            count_of_ones[bit] += usize::from(report_line[bit]);
        }
    }
    let mut gamma_string = String::from("");
    let mut epsilon_string = String::from("");
    for bit in 0..DIAGNOSTIC_LENGTH {
        let double_the_count = count_of_ones[bit] * 2;
        let comparison = double_the_count.cmp(&lines);
        match comparison {
            Ordering::Less => {
                gamma_string.push('0');
                epsilon_string.push('1');
            },
            Ordering::Greater => {
                gamma_string.push('1');
                epsilon_string.push('0');
            },
            Ordering::Equal => return Err(DiagnosticError::UnclearReading),
        }
    }
    let gamma = u32::from_str_radix(&gamma_string, 2).unwrap();
    let epsilon = u32::from_str_radix(&epsilon_string, 2).unwrap();
    Ok(DiagnosticData{gamma: gamma, epsilon: epsilon})
}


pub fn main() {
    let diagnostic_report = read_diagnostic_report_file();
    match diagnostic_report {
        Ok(diagnostic_report) => {
            let data = analyze_diagnostics(diagnostic_report);
            match data {
                Ok(data) => {
                    println!("Gamma: {}, Epsilon: {}, Product: {}",
                             data.gamma, data.epsilon, data.gamma * data.epsilon
                    )
                },
                Err(err) => println!("Error: {}", err)
            }
        },
        Err(err) => println!("Error: {}", err),
    }
}
