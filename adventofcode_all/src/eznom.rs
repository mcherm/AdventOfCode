
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
