/// Sometimes you want to build a simple enum where each value is parsed/output as a single
/// ascii character, and it does a bunch of basic enum stuff. Well, I got tired of typing
/// that out every time, so this module creates that.
///
/// Example:
///     use advent_lib::asciienum::AsciiEnum;
///     AsciiEnum!{
///         enum Rating {
///             Xtreme('x'),
///             Music('m'),
///             Aerodynamic('a'),
///             Shiny('s'),
///         }
///     }




#[macro_export]
macro_rules! AsciiEnum{
    (
        enum $type_name:ident {
            $($item_name:ident($ch:expr) ),* $(,)?
        }
    ) => {
        #[derive(Copy, Clone, Eq, PartialEq, Hash)]
        pub enum $type_name {
            $( $item_name, )*
        }

        impl TryFrom<char> for $type_name {
            type Error = anyhow::Error;

            fn try_from(value: char) -> Result<Self, Self::Error> {
                Ok(match value {
                    $( $ch => $type_name::$item_name, )*
                    _ => return Err(anyhow::anyhow!("invalid character '{}'", value))
                })
            }
        }

        impl From<$type_name> for char {
            fn from(value: $type_name) -> Self {
                match value {
                    $( $type_name::$item_name => $ch, )*
                }
            }
        }

        impl std::fmt::Debug for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let c: char = (*self).into();
                write!(f, "{}", c)
            }
        }

        impl std::fmt::Display for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl $type_name {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                nom::combinator::map(
                    nom::character::complete::one_of(&[ $( $ch, )*][..]),
                    |c: char| Self::try_from(c).expect("should already be a valid character")
                )(input)
            }
        }

    }
}

pub use AsciiEnum;
