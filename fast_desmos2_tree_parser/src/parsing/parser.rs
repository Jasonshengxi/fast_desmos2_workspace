use std::{collections::VecDeque, marker::PhantomData};

use fast_desmos2_tree::tree::{EditorTree, EditorTreeKind, EditorTreeSeq};
use fast_desmos2_utils::OptExt;
use winnow::{
    combinator::{alt, opt, repeat, separated, separated_pair},
    error::{ErrMode, TreeError},
    prelude::*,
    stream::Accumulate,
    token::any,
    Stateful,
};

use crate::tree::{EvalNode, IdentId};

use super::{ParseExtra, ParseStream};

pub type ParseInput<'a> = Stateful<ParseStream<'a>, ParseExtra<'a>>;
pub type ParseResult<'a, T> = PResult<T, ParseError<'a>>;
pub type ParseError<'a> = TreeError<ParseInput<'a>>;

pub fn parse_seq<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    parse_add_sub(input)
}

fn parse_add_sub<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    parse_multiply(input)
}

fn parse_multiply<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    (
        parse_everything_else,
        repeat::<_, _, Vec<_>, _, _>(.., parse_everything_else),
    )
        .map(|(first, remaining)| {
            if remaining.is_empty() {
                first
            } else {
                let mut nodes = remaining;
                nodes.insert(0, first);
                EvalNode::multiply(nodes)
            }
        })
        .parse_next(input)
}

fn parse_everything_else<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    alt((
        parse_number,
        parse_function_call,
        parse_identifier,
        parse_point_literal,
        parse_parens,
        parse_sqrt,
        parse_abs,
        parse_list_literal,
    ))
    .parse_next(input)
}

fn parse_char<'a>(ch: char) -> impl Parser<ParseInput<'a>, (), ParseError<'a>> {
    any.verify(move |tree: &EditorTree| tree.is_terminal_and_eq(ch))
        .void()
}

fn parse_map_char<'a, T>(
    mut map: impl FnMut(char) -> Option<T> + 'static,
) -> impl Parser<ParseInput<'a>, T, ParseError<'a>> {
    any.verify_map(move |tree: &EditorTree| tree.is_terminal_and_then(|term| map(term.ch())))
}

fn parse_number<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    let unsigned_integer = || {
        repeat(
            1..,
            parse_map_char(|ch| ch.to_digit(10).map(|x| x as u8 + b'0')),
        )
    };

    let fractional_part = || (parse_char('.'), unsigned_integer());

    alt((
        (unsigned_integer(), opt(fractional_part())),
        fractional_part().map(|fract| (Vec::new(), Some(fract))),
    ))
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
    .parse_next(input)
}

fn parse_function_call<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    (
        parse_raw_ident,
        opt(parse_power_chained(parse_seq)),
        parse_parens_chained(separated(.., parse_seq, parse_char(','))),
    )
        .map(|(ident, power, params)| EvalNode::function_call(ident, power, params))
        .parse_next(input)
}

fn parse_raw_ident<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, IdentId> {
    repeat(
        1..,
        any.verify_map(|tree: &EditorTree| {
            tree.is_terminal_and_then(|term| term.ch().is_ascii_alphabetic().then_some(term.ch()))
        }),
    )
    .map(|output: Vec<_>| {
        let ident_str = output.into_iter().collect::<String>();
        input.state.idents.convert_id(&ident_str)
    })
    .parse_next(input)
}

fn parse_identifier<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    parse_raw_ident.map(EvalNode::ident).parse_next(input)
}

struct ChainParser<'a, M, P, O>
where
    M: FnMut(&EditorTreeKind) -> Option<&EditorTreeSeq>,
    P: Parser<ParseInput<'a>, O, ParseError<'a>>,
{
    matcher: M,
    inner: P,
    _phantom: PhantomData<(ParseInput<'a>, O, ParseError<'a>)>,
}

impl<'a, M, P, O> Parser<ParseInput<'a>, O, ParseError<'a>> for ChainParser<'a, M, P, O>
where
    M: FnMut(&EditorTreeKind) -> Option<&EditorTreeSeq>,
    P: Parser<ParseInput<'a>, O, ParseError<'a>>,
{
    fn parse_next(&mut self, input: &mut ParseInput<'a>) -> PResult<O, ParseError<'a>> {
        let stage = any
            .verify_map(|tree: &EditorTree| (self.matcher)(tree.kind()))
            .parse_next(input)?
            .children();
        let mut stream = Stateful {
            input: ParseStream::new(stage),
            state: input.state,
        };
        (self.inner).parse_next(&mut stream)
    }
}

fn parse_chained<'a, O>(
    matcher: impl FnMut(&EditorTreeKind) -> Option<&EditorTreeSeq>,
    inner: impl Parser<ParseInput<'a>, O, ParseError<'a>>,
) -> impl Parser<ParseInput<'a>, O, ParseError<'a>> {
    ChainParser {
        matcher,
        inner,
        _phantom: PhantomData,
    }
}

macro_rules! parser_chain {
    ($(fn $name: ident() {
        $p:pat => $e:expr $(,)?
    })*) => {
        $(fn $name<'a, O>(
            inner: impl Parser<ParseInput<'a>, O, ParseError<'a>>,
        ) -> impl Parser<ParseInput<'a>, O, ParseError<'a>> {
            parse_chained(
                |kind| match kind {
                    $p => $e,
                    _ => None,
                },
                inner,
            )
        })*
    };
}
parser_chain! {
    fn parse_parens_chained() {
        EditorTreeKind::Paren(paren) => Some(paren.child())
    }

    fn parse_power_chained() {
        EditorTreeKind::Power(power) => Some(power.power())
    }

    fn parse_abs_chained() {
        EditorTreeKind::Abs(abs) => Some(abs.child())
    }

    fn parse_sqrt_chained() {
        EditorTreeKind::Sqrt(sqrt) => Some(sqrt.child())
    }

    fn parse_brackets_chained() {
        EditorTreeKind::Bracket(bracket) => Some(bracket.child())
    }
}

fn parse_parens<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    parse_parens_chained(parse_seq).parse_next(input)
}

fn parse_point_literal<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    parse_parens_chained(separated_pair(parse_seq, parse_char(','), parse_seq))
        .map(EvalNode::point)
        .parse_next(input)
}

fn parse_abs<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    parse_abs_chained(parse_seq)
        .map(EvalNode::abs)
        .parse_next(input)
}

fn parse_sqrt<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    parse_sqrt_chained(parse_seq)
        .map(EvalNode::sqrt)
        .parse_next(input)
}

fn parse_list_literal<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
    parse_brackets_chained(separated(0.., parse_seq, parse_char(',')))
        .map(EvalNode::list_literal)
        .parse_next(input)
}
