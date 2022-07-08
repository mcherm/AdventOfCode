
pub use nom::branch::alt;
pub use nom::character::complete::i8 as parse_i8;
pub use nom::character::complete::u8 as parse_u8;
pub use nom::character::complete::i16 as parse_i16;
pub use nom::character::complete::u16 as parse_u16;
pub use nom::character::complete::i32 as parse_i32;
pub use nom::character::complete::u32 as parse_u32;
pub use nom::character::complete::i64 as parse_i64;
pub use nom::character::complete::u64 as parse_u64;
pub use nom::character::complete::i128 as parse_i128;
pub use nom::character::complete::u128 as parse_u128;
pub use nom::character::complete::newline;
pub use nom::combinator::opt;
pub use nom::sequence::{delimited, tuple};
pub use nom::multi::{many0, many1, separated_list0, separated_list1};


use nom::character::complete::space0 as nom_space0;
use nom::character::complete::space1 as nom_space1;
use nom::bytes::complete::tag as nom_tag;


pub type Result<'a, T> = nom::IResult<&'a str, T>;


/// If I have a Result that returns an &str, this changes it to a Result that returns a
/// String. The reason is that when I return an &str I get some kind of lifetime error
/// that I don't comprehend, but I can make it work by returning a String instead.
fn convert_result_to_string<'a,'b>(str_result: nom::IResult<&'a str, &'b str>) -> nom::IResult<&'a str, String> {
    match str_result {
        Ok((rest, body)) => Ok((rest, body.to_string())),
        Err(err) => Err(err),
    }
}


/// We won't be able to use "tag" from nom because when we try to use it we get complaints
/// that I don't comprehend dealing with lifetimes. Instead, this is our own version of
/// tag which works similarily but returns a String instead of an &str. I also don't fully
/// understand why THIS works.
pub fn fixed(tag: &str) -> impl Fn(&str) -> Result<String> + '_ {
    move |i: &str| {
        convert_result_to_string(nom_tag(tag)(i))
    }
}


pub fn space0(input: &str) -> Result<String> {
    convert_result_to_string(nom_space0(input))
}

pub fn space1(input: &str) -> Result<String> {
    convert_result_to_string(nom_space1(input))
}



/// This trait represents an object which can be parsed from a unicode string. (Typically objects
/// will also implement Display to serialize to a string.) Implementing objects must provide
/// two methods: recognize() and build().
pub trait Parseable<TParsed> where Self: Sized {
    /// This takes in a string reference and returns a Result.
    fn recognize(input: &str) -> nom::IResult<&str, TParsed>;

    fn build(turn: TParsed) -> Self;

    fn parse(input: &str) -> nom::IResult<&str, Self> {
        type_builder(Self::recognize, Self::build)(input)
    }
}


//#[deprecated]
pub fn type_builder<'a, FRecog, FBuild, TParsed, TOut>(recognizer: FRecog, builder: FBuild)
    -> impl Fn(&'a str) -> nom::IResult<&'a str, TOut>
    where
        FRecog: Fn(&'a str) -> nom::IResult<&'a str, TParsed>,
        FBuild: Fn(TParsed) -> TOut,
{
    let parse = move |s: &'a str| {
        recognizer(s).map(|(rest, v)| (rest, builder(v)))
    };
    parse
}




#[cfg(test)]
mod test {
    use super::*;

    use nom::character::complete::alpha1 as nom_alpha1;

    #[derive(Debug, Eq, PartialEq)]
    enum Input {
        Const(String),
    }

    #[test]
    fn test_type_builder() {
        let alpha1 = nom_alpha1; // dangerously fragile: I have no idea why this line is needed
        let parse = type_builder(
            alpha1,
            |s| Input::Const(s.to_string())
        );
        let input: &str = &"ab ";
        let extra: &str = &" ";
        assert_eq!(Ok((extra, Input::Const("ab".to_string()))), parse(input));
        assert!(matches!(
            parse(input),
            Ok((" ", Input::Const(_)))
        ));
    }

}
