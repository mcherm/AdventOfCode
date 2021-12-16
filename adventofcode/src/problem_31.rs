use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{AddAssign, MulAssign};


/// An error that we can encounter when reading the input.
enum InputError {
    IoError(std::io::Error),
    BadInt(std::num::ParseIntError),
    NoData,
    MultipleLines,
    InvalidCharacter,
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
            InputError::NoData => write!(f, "No data."),
            InputError::MultipleLines => write!(f, "Multiple lines."),
            InputError::InvalidCharacter => write!(f, "Invalid character."),
        }
    }
}


enum LengthTypeEncoding {
    TotalLength(usize),
    NumberOfSubpackets(usize),
}

struct OperatorPacketFields {
    length_type: LengthTypeEncoding,
    subpackets: Vec<Packet>,
}

enum PacketFields {
    LiteralValue(u64),
    OperatorPacket(OperatorPacketFields),
}

struct Packet {
    version: u32,
    type_id: u32,
    fields: PacketFields,
}
impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Packet{{{}, {}, {}}}", self.version, self.type_id, self.fields)
    }
}
impl fmt::Display for PacketFields {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PacketFields::LiteralValue(value) => {
                write!(f, "Literal: {}", value)
            },
            PacketFields::OperatorPacket(OperatorPacketFields{length_type, subpackets}) => {
                write!(f, "Operator: {} [", length_type)?;
                let mut first_one = true;
                for subpacket in subpackets {
                    if first_one {
                        first_one = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", subpacket)?;
                }
                write!(f, "]")
            },
        }
    }
}
impl fmt::Display for LengthTypeEncoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LengthTypeEncoding::TotalLength(size) => {
                write!(f, "({} bits)", size)
            },
            LengthTypeEncoding::NumberOfSubpackets(num) => {
                write!(f, "({} pkts)", num)
            },
        }
    }
}



#[derive(Debug)]
struct BinaryStream {
    data: Vec<bool>, // FIXME: Could be massively more efficient as an iterator
    pos: usize,
}

impl BinaryStream {
    fn new(hex: &str) -> BinaryStream {
        let data: Vec<bool> = hex.chars().flat_map(hex_to_bin).collect();
        BinaryStream{data, pos: 0}
    }

    // Pops off the first bit. Panics if the array is empty.
    fn pop(&mut self) -> bool {
        self.pos += 1;
        self.data.remove(0)
    }

    // Returns the position in the stream
    fn get_pos(&self) -> usize {
        self.pos
    }

    fn read_bits_as_number<T>(&mut self, num_bits: usize) -> T
        where T: MulAssign<T> + AddAssign<T> + From<u8>
    {
        let mut result: T = 0.into();
        for _ in 0..num_bits {
            result *= 2.into();
            match self.pop() {
                false => {},
                true => result += 1.into(),
            }
        }
        return result;
    }

    fn read_packet(&mut self) -> Packet {
        let version = self.read_bits_as_number(3);
        let type_id = self.read_bits_as_number(3);
        let fields: PacketFields = match type_id {
            4 => self.read_data_packet_fields(),
            _ => self.read_operator_packet_fields(),
        };
        Packet{version, type_id, fields}
    }

    fn read_data_packet_fields(&mut self) -> PacketFields {
        let mut value = 0;
        loop {
            value *= 16;
            let more_to_read_flag = self.pop();
            if self.pop() { value += 8 };
            if self.pop() { value += 4 };
            if self.pop() { value += 2 };
            if self.pop() { value += 1 };
            if !more_to_read_flag {
                break;
            }
        }
        PacketFields::LiteralValue(value)
    }

    fn read_operator_packet_fields(&mut self) -> PacketFields {
        let length_type: LengthTypeEncoding = match self.pop() {
            false => {
                let total_length: usize = self.read_bits_as_number(15);
                LengthTypeEncoding::TotalLength(total_length)
            },
            true => {
                let num_packets: usize = self.read_bits_as_number(11);
                LengthTypeEncoding::NumberOfSubpackets(num_packets)
            },
        };

        // --- Read child packets ---
        let subpackets = match length_type {
            LengthTypeEncoding::TotalLength(size) => {
                let start_pos = self.get_pos();
                let end_pos = start_pos + size;
                let mut packets: Vec<Packet> = Vec::new();
                while self.get_pos() < end_pos {
                    packets.push(self.read_packet());
                }
                packets
            }
            LengthTypeEncoding::NumberOfSubpackets(num) => {
                let mut packets: Vec<Packet> = Vec::new();
                for _ in 0..num {
                    packets.push(self.read_packet());
                }
                packets
            }
        };

        // --- Return the result ---
        PacketFields::OperatorPacket(OperatorPacketFields {
            length_type,
            subpackets,
        })
    }
}


fn sum_up_versions(packet: &Packet) -> u32 {
    let mut sum = 0;
    sum += packet.version;
    if let PacketFields::OperatorPacket(OperatorPacketFields{ref subpackets, ..}) = packet.fields {
        for subpacket in subpackets {
            sum += sum_up_versions(&subpacket)
        }
    }
    sum
}


fn hex_to_bin(c: char) -> [bool;4] {
    match c {
        '0' => [false, false, false, false],
        '1' => [false, false, false,  true],
        '2' => [false, false,  true, false],
        '3' => [false, false,  true,  true],
        '4' => [false,  true, false, false],
        '5' => [false,  true, false,  true],
        '6' => [false,  true,  true, false],
        '7' => [false,  true,  true,  true],
        '8' => [ true, false, false, false],
        '9' => [ true, false, false,  true],
        'A' => [ true, false,  true, false],
        'B' => [ true, false,  true,  true],
        'C' => [ true,  true, false, false],
        'D' => [ true,  true, false,  true],
        'E' => [ true,  true,  true, false],
        'F' => [ true,  true,  true,  true],
        _ => panic!("Not a hex character")
    }
}


/// Read in the input file.
fn read_packet_file() -> Result<String, InputError> {
    let filename = "data/2021/day/16/input.txt";
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    let first_line: String = lines.next().ok_or(InputError::NoData)??;
    if lines.next().is_some() {
        return Err(InputError::MultipleLines);
    }

    for char in first_line.chars() {
        match char {
            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'A'|'B'|'C'|'D'|'E'|'F' => {},
            _ => return Err(InputError::InvalidCharacter),
        }
    }
    Ok(first_line)
}



fn run() -> Result<(),InputError> {
    let hex_chars: String = read_packet_file()?;
    let mut binary_stream = BinaryStream::new(&hex_chars);
    let packet: Packet = binary_stream.read_packet();
    let sum = sum_up_versions(&packet);
    println!("packet = {}", packet);
    println!("sum = {}", sum);
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
    fn test_hex_to_bin() {
        assert_eq!(hex_to_bin('0'), [false, false, false, false]);
        assert_eq!(hex_to_bin('6'), [false, true, true, false]);
        assert_eq!(hex_to_bin('F'), [true, true, true, true]);
    }

    #[test]
    fn test_parse_literal_value() {
        let packet: Packet = BinaryStream::new("D2FE28").read_packet();
        assert!(matches!(packet, Packet{
            version: 6,
            type_id: 4,
            fields: PacketFields::LiteralValue(2021)
        }));
    }

    #[test]
    fn test_parse_operator_1() {
        let packet: Packet = BinaryStream::new("38006F45291200").read_packet();
        assert!(matches!(packet,
            Packet{
                version: 1,
                type_id: 6,
                fields: PacketFields::OperatorPacket(OperatorPacketFields{
                    length_type: LengthTypeEncoding::TotalLength(27),
                    subpackets,
                }),
            }
            if matches!(subpackets.as_slice(), [
                Packet{
                    version: 6,
                    type_id: 4,
                    fields: PacketFields::LiteralValue(10)
                },
                Packet{
                    version: 2,
                    type_id: 4,
                    fields: PacketFields::LiteralValue(20)
                },
            ])
        ));
    }

    #[test]
    fn test_parse_operator_2() {
        let packet: Packet = BinaryStream::new("EE00D40C823060").read_packet();
        assert!(matches!(packet,
            Packet{
                version: 7,
                type_id: 3,
                fields: PacketFields::OperatorPacket(OperatorPacketFields{
                    length_type: LengthTypeEncoding::NumberOfSubpackets(3),
                    subpackets,
                }),
            }
            if matches!(subpackets.as_slice(), [
                Packet{
                    version: 2,
                    type_id: 4,
                    fields: PacketFields::LiteralValue(1)
                },
                Packet{
                    version: 4,
                    type_id: 4,
                    fields: PacketFields::LiteralValue(2)
                },
                Packet{
                    version: 1,
                    type_id: 4,
                    fields: PacketFields::LiteralValue(3)
                },
            ])
        ));
    }

    #[test]
    fn test_sum_up_versions() {
        let test_cases = [
            ("8A004A801A8002F478", 16),
            ("620080001611562C8802118E34", 12),
            ("C0015000016115A2E0802F182340", 23),
            ("A0016C880162017C3686B18A3D4780", 31),
        ];
        for (hex, sum) in test_cases {
            let packet: Packet = BinaryStream::new(hex).read_packet();
            let version_sum = sum_up_versions(&packet);
            assert_eq!(version_sum, sum);
        }
    }
}
