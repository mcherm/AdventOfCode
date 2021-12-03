use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fmt;
use std::cmp::Ordering;


const DIAGNOSTIC_LENGTH: usize = 12;


#[derive(Debug, Copy, Clone)]
struct ReportLine {
    values: [u8; DIAGNOSTIC_LENGTH],
}

impl ReportLine {
    fn new_blank() -> ReportLine {
        ReportLine{values: [0; DIAGNOSTIC_LENGTH]}
    }

    fn get_bit(&self, position: usize) -> u8 {
        if position >= DIAGNOSTIC_LENGTH {
            panic!("Attempt to get a bit beyond the size of a ReportLine")
        }
        self.values[position]
    }

    fn set_bit(&mut self, position: usize, value: u8) {
        if position >= DIAGNOSTIC_LENGTH {
            panic!("Attempt to set a bit beyond the size of a ReportLine")
        }
        match value {
            0 => self.values[position] = 0,
            1 => self.values[position] = 1,
            _ => panic!("Attempt to set a bit to an invalid value in ReportLine")
        }
    }

    fn as_str(&self) -> String {
        let mut result = String::from("");
        for bit in 0..DIAGNOSTIC_LENGTH {
            result.push(match self.values[bit] {
                0 => '0',
                1 => '1',
                _ => panic!("Invalid value in ReportLine"),
            })
        }
        result
    }

    fn as_num(&self) -> u32 {
        u32::from_str_radix(&self.as_str(), 2).unwrap()
    }
}


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
    let mut report_line: ReportLine = ReportLine::new_blank();

    let mut count = 0;
    for c in s.chars() {
        match c {
            '0' => report_line.set_bit(count, 0),
            '1' => report_line.set_bit(count, 1),
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
    oxygen: u32,
    co2_scrubber: u32,
}


// This reads a diagnostic report and returns a DiagnosticData.
fn analyze_diagnostics(diagnostic_report: Vec<ReportLine>) -> Result<DiagnosticData,DiagnosticError>  {

    fn find_diagnostic(seek_value: u8, report_lines: &[ReportLine], bit_to_consider: usize) -> Result<u32,DiagnosticError> {
        let report_count = report_lines.len();
        let mut count_of_ones: usize = 0;
        for report_line in report_lines {
            count_of_ones += usize::from(report_line.get_bit(bit_to_consider));
        }
        let double_the_count = count_of_ones * 2;
        let comparison = double_the_count.cmp(&report_count);
        let value_to_keep = match comparison {
            Ordering::Less => 1-seek_value,
            Ordering::Greater => seek_value,
            Ordering::Equal => seek_value,
        };

        let mut filtered_lines: Vec<ReportLine> = Vec::new();
        for report_line in report_lines {
            if report_line.get_bit(bit_to_consider) == value_to_keep {
                filtered_lines.push(*report_line);
            }
        }

        let filtered_count = filtered_lines.len();
        if filtered_count == 1 {
            // found a single line
            return Ok(filtered_lines[0].as_num());
        } else {
            let next_bit = bit_to_consider + 1;
            if next_bit < DIAGNOSTIC_LENGTH {
                return find_diagnostic(seek_value, &filtered_lines, next_bit);
            } else {
                // multiple rows match the criteria
                return Err(DiagnosticError::UnclearReading)
            }
        }
    }

    let oxygen = find_diagnostic(1, &diagnostic_report, 0)?;
    let co2_scrubber = find_diagnostic(0, &diagnostic_report, 0)?;

    Ok(DiagnosticData{oxygen, co2_scrubber})
}


pub fn main() {
    let diagnostic_report = read_diagnostic_report_file();
    match diagnostic_report {
        Ok(diagnostic_report) => {
            let data = analyze_diagnostics(diagnostic_report);
            match data {
                Ok(data) => {
                    println!("oxygen: {}, co2_scrubber: {}, Product: {}",
                             data.oxygen, data.co2_scrubber, data.oxygen * data.co2_scrubber
                    )
                },
                Err(err) => println!("Error: {}", err)
            }
        },
        Err(err) => println!("Error: {}", err),
    }
}
