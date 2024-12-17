#![allow(clippy::should_implement_trait)]

mod dyadic_pervasive;
mod list_stat;
mod monadic_non_pervasive;
mod monadic_pervasive;

pub use dyadic_pervasive::DyadicPervasive;
pub use list_stat::ListStat;
pub use monadic_non_pervasive::MonadicNonPervasive;
pub use monadic_pervasive::MonadicPervasive;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Builtins {
    MonadicPervasive(MonadicPervasive),
    DyadicPervasive(DyadicPervasive),
    MonadicNonPervasive(MonadicNonPervasive),
    ListStat(ListStat),

    Join,   // variadic non-pervasive
    Sort,   // monadic/dyadic non-pervasive
    Random, // zero-adic / monadic non-pervasive / dyadic non-pervasive
}

macro_rules! try_options {
    (;maps: $($expr: expr => $func: expr;)* ;direct: $($simple_expr: expr => $value: expr;)*) => {
        $(if let Some(x) = $expr { Some($func(x)) } else)*
        $(if $simple_expr { Some($value) } else)*
        { None }
    };
}

impl Builtins {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MonadicPervasive(x) => x.as_str(),
            Self::DyadicPervasive(x) => x.as_str(),
            Self::MonadicNonPervasive(x) => x.as_str(),
            Self::ListStat(x) => x.as_str(),

            Self::Join => "join",
            Self::Sort => "sort",
            Self::Random => "random",
        }
    }

    pub fn from_str(input: &[u8]) -> Option<Self> {
        try_options! {
            ;maps:
                MonadicPervasive::from_str(input) => Self::MonadicPervasive;
                DyadicPervasive::from_str(input) => Self::DyadicPervasive;
                MonadicNonPervasive::from_str(input) => Self::MonadicNonPervasive;
                ListStat::from_str(input) => Self::ListStat;
            ;direct:
                input == b"join" => Self::Join;
                input == b"sort" => Self::Sort;
                input == b"random" => Self::Random;
        }
    }
}
