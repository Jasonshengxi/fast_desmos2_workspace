use combine::{choice, many1, optional, satisfy, satisfy_map, stream::SliceStream, Parser};
use fast_desmos2_tree::tree::{EditorTree, EditorTreeKind, EditorTreeSeq};

use crate::tree::EvalNode;

use super::ParseExtra;

type ParseInput<'a> = SliceStream<'a, EditorTree>;

pub fn parse_seq<'a>(input: ParseExtra<'a>) -> impl Parser<ParseInput<'a>, Output = EvalNode> {
    parse_add_sub(input)
}

fn parse_add_sub<'a>(input: ParseExtra<'a>) -> impl Parser<ParseInput<'a>, Output = EvalNode> {
    parse_multiply(input)
}
fn parse_multiply<'a>(input: ParseExtra<'a>) -> impl Parser<ParseInput<'a>, Output = EvalNode> {
    parse_everything_else(input)
}
fn parse_everything_else<'a>(
    input: ParseExtra<'a>,
) -> impl Parser<ParseInput<'a>, Output = EvalNode> {
    choice!(
        parse_number(),
        parse_identifier(input),
        parse_parens(input),
        parse_sqrt(input),
        parse_abs(input)
    )
}

fn parse_number<'a>() -> impl Parser<ParseInput<'a>, Output = EvalNode> {
    let unsigned_integer = || {
        many1::<Vec<_>, _, _>(satisfy_map(|tree: &EditorTree| {
            tree.is_terminal_and_then(|term| term.ch().to_digit(10).map(|x| x as u8 + b'0'))
        }))
    };

    let fractional_part = || {
        (
            satisfy(|tree: &EditorTree| tree.is_terminal_and_eq('.')),
            unsigned_integer(),
        )
    };

    (unsigned_integer(), optional(fractional_part()))
        .or(fractional_part().map(|fract| (Vec::new(), Some(fract))))
        .map(|(int_part, frac_part)| {
            // TODO fix the amount of allocation here
            let int_part = String::from_utf8(int_part).unwrap();
            let frac_part = frac_part.map(|x| String::from_utf8(x.1).unwrap());
            let combined = match frac_part {
                Some(frac_part) => format!("{int_part}.{frac_part}"),
                None => int_part,
            };
            EvalNode::number(combined.parse().unwrap())
        })
}

fn parse_identifier<'a>(input: ParseExtra<'a>) -> impl Parser<ParseInput<'a>, Output = EvalNode> {
    many1(satisfy_map(|tree: &EditorTree| {
        tree.is_terminal_and_then(|term| term.ch().is_ascii_alphabetic().then_some(term.ch()))
    }))
    .map(|output: Vec<_>| {
        let ident_str = output.into_iter().collect::<String>();
        EvalNode::ident(input.idents.convert_id(&ident_str))
    })
}

macro_rules! parse_surrounds {
    ($(fn $name: ident() {
        $kind: ident ::$variant: ident (_) => |$input: ident| $expr: expr
    })*) => {
        $(fn $name<'a>(input: ParseExtra<'a>) -> impl Parser<ParseInput<'a>, Output = EvalNode> + 'a {
            satisfy_map::<ParseInput, _, &EditorTreeSeq>(|tree: &EditorTree| {
                match tree.kind() {
                    $kind::$variant(paren) => Some(paren.child()),
                    _ => None,
                }
            })
            .and_then(move |child| {
                parse_seq(input)
                    .parse(SliceStream(child.children()))
                    .map(|$input| $expr)
            })
        })*
    };
}

parse_surrounds! {
    fn parse_sqrt() {
        EditorTreeKind::Sqrt(_) => |x| EvalNode::sqrt(x.0)
    }

    fn parse_parens() {
        EditorTreeKind::Paren(_) => |x| x.0
    }

    fn parse_abs() {
        EditorTreeKind::Abs(_) => |x| EvalNode::abs(x.0)
    }
}
