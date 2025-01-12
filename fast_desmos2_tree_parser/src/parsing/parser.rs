use std::{marker::PhantomData, ops::RangeBounds};

use crate::{function_as_parser, parsing::combinator::Whitespace};
use fast_desmos2_eval::{
    builtins::Builtins, AddOrSub, CompSet, Conditional, Element, EvalNode, IdentId,
};
use fast_desmos2_tree::tree::{
    EditorTree, EditorTreeFraction, EditorTreeKind, EditorTreeSeq, EditorTreeTerminal,
};
use fast_desmos2_utils::ResExt;

use super::{
    combinator::{Aggregate, Alt, Chained, Filter, FilterMap, Todo},
    error::ParseErrorKind,
    MyParser, ParseError, ParseInput, ParseResult, ResExt as _,
};

pub fn expr_with_whitespace<S: EditorTreeSeq>() -> ExprWithWhitespace<S> {
    ExprWithWhitespace::new()
}

pub fn expr<S: EditorTreeSeq>() -> Expr<S> {
    Expr::new()
}

function_as_parser! {
    pub fn ExprWithWhitespace<S>(input) -> <EvalNode> {
        add_sub().surround_whitespace().parse_next(input)
    }

    pub fn Expr<S>(input) -> <EvalNode> {
        add_sub().parse_next(input)
    }
}

pub fn add_sub<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    (
        terminal_eq('-')
            .ignore_after(Whitespace::new())
            .opt()
            .map(|x| match x {
                Some(_) => AddOrSub::Sub,
                _ => AddOrSub::Add,
            }),
        multiply(),
    )
        .map(|x| vec![x])
        .then_fold_repeated(
            ..,
            (
                Alt::new((
                    terminal_eq('+').map(|_| AddOrSub::Add),
                    terminal_eq('-').map(|_| AddOrSub::Sub),
                ))
                .surround_whitespace(),
                multiply(),
            ),
            Vec::fold,
        )
        .map(EvalNode::add_sub)
}

pub fn multiply<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    postfix()
        .map(|x| vec![x])
        .then_fold_repeated(
            ..,
            (Whitespace::new(), terminal_eq('*').whitespace_after().opt()).preceding(postfix()),
            Vec::fold,
        )
        .map(EvalNode::multiply)
}

pub fn postfix<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    enum Postfix {
        Index(EvalNode),
        Power(EvalNode),
        Element(Element),
    }

    everything_else().then_fold_repeated(
        ..,
        Whitespace::new().preceding(Alt::new((
            brackets_chained(Alt::new((
                expr().then_eof("expr"),
                list_contents_then_eof(),
            )))
            .map(Postfix::Index),
            power_chained(Expr::new()).map(Postfix::Power),
            (
                terminal_eq('.'),
                Alt::new((
                    terminal_eq('x').map(|_| Element::X),
                    terminal_eq('y').map(|_| Element::Y),
                )),
            )
                .map(|x| Postfix::Element(x.1)),
        ))),
        |node, postfix| match postfix {
            Postfix::Index(index) => EvalNode::index(node, index),
            Postfix::Power(power) => EvalNode::power(node, power),
            Postfix::Element(element) => EvalNode::element(node, element),
        },
    )
}

pub fn everything_else<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    Alt::new((
        function_call(),
        ident(),
        number(),
        SumProd::new(),
        Fraction::new(),
        abs_chained(expr()).map(EvalNode::abs),
        parens_chained(Alt::new((
            (expr(), terminal_eq(','), expr()).map(|(x, _, y)| EvalNode::point((x, y))),
            expr(),
        ))),
        sqrt_chained(expr()).map(EvalNode::sqrt),
        brackets_chained(list_contents_then_eof()),
    ))
}

pub fn terminal_and_then<S: EditorTreeSeq, O>(
    and: impl Fn(char) -> Option<O>,
) -> impl MyParser<S, Output = O> {
    FilterMap::new(move |token| match token.kind() {
        EditorTreeKind::Terminal(term) => and(term.ch()),
        _ => None,
    })
}

pub fn terminal_and<S: EditorTreeSeq>(
    and: impl Fn(char) -> bool,
) -> impl MyParser<S, Output = char> {
    FilterMap::new(move |token| match token.kind() {
        EditorTreeKind::Terminal(term) => and(term.ch()).then_some(term.ch()),
        _ => None,
    })
}

pub fn terminal_eq<S: EditorTreeSeq>(ch: char) -> impl MyParser<S, Output = ()> {
    Filter::new(move |token| match token.kind() {
        EditorTreeKind::Terminal(term) => term.ch() == ch,
        _ => false,
    })
}

pub fn raw_raw_ident<S: EditorTreeSeq>() -> impl MyParser<S, Output = String> {
    terminal_and(|ch| ch.is_ascii_alphabetic()).repeat_collected(1..)
}

pub fn raw_ident<S: EditorTreeSeq>() -> impl MyParser<S, Output = IdentId> {
    raw_raw_ident().map_with_extra(|ident, extra| extra.idents.convert_id(&ident))
}

pub fn function_call<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    (
        raw_raw_ident().whitespace_after(),
        power_chained(expr()).whitespace_after().opt(),
        parens_chained(comma_separated_exprs()),
    )
        .map_with_extra(|(ident, power, params), extra| {
            match Builtins::from_str(ident.as_bytes()) {
                Some(builtins) => EvalNode::builtins_call(builtins, power, params),
                None => EvalNode::function_call(extra.idents.convert_id(&ident), power, params),
            }
        })
}

pub fn ident<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    raw_ident().map(EvalNode::ident)
}

pub fn num_string<S: EditorTreeSeq>() -> impl MyParser<S, Output = String> {
    terminal_and(|ch| ch.is_ascii_digit()).repeat_collected(1..)
}

pub fn raw_number<S: EditorTreeSeq>() -> impl MyParser<S, Output = f64> {
    Alt::new((
        (
            num_string().map(Some),
            (terminal_eq('.'), num_string()).map(|x| x.1).opt(),
        ),
        (terminal_eq('.'), num_string())
            .map(|x| x.1)
            .map(Some)
            .map(|frac| (None, frac)),
    ))
    .map(|(int, frac)| {
        let mut joined = int.unwrap_or_else(String::new);
        if let Some(frac) = frac {
            joined.push('.');
            joined.extend(frac.chars());
        }
        joined.parse().unwrap_unreach()
    })
}

pub fn number<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    raw_number().map(EvalNode::number)
}

// fn parse_if_else<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
//     fn parse_conditionals<S: EditorTreeSeq>() -> impl MyParser<> {
//         separated(1.., parse_conditional, parse_char(','))
//     }
//
//     fn parse_if_else_content<'a>(input: &mut ParseInput<'a>) -> ParseResult<'a, EvalNode> {
//         (
//             parse_conditionals,
//             opt((
//                 preceded(parse_char(':'), parse_seq),
//                 opt(preceded(
//                     parse_char(','),
//                     alt((parse_seq, parse_if_else_content)),
//                 )),
//             )),
//         )
//             .map(|(conds, options)| {
//                 let (yes, no) = match options {
//                     Some((yes, no)) => (Some(yes), no),
//                     None => (None, None),
//                 };
//                 EvalNode::if_else(conds, yes, no)
//             })
//             .parse_next(input)
//     }
//
//     parse_curly_chained(parse_if_else_content).parse_next(input)
// }

fn parse_conditional<S: EditorTreeSeq>() -> impl MyParser<S, Output = Conditional> {
    (
        expr().whitespace_after(),
        (
            Alt::new((
                terminal_eq('=').map(|_| CompSet::EQUAL),
                (
                    Alt::new((
                        terminal_eq('<').map(|_| CompSet::LESS),
                        terminal_eq('>').map(|_| CompSet::MORE),
                    )),
                    terminal_eq('=').opt(),
                )
                    .map(|(normal, equal)| match equal {
                        Some(_) => normal.union(CompSet::EQUAL),
                        None => normal,
                    }),
            )),
            expr(),
        )
            .repeat_collected(1..),
    )
        .map(|(first, remaining): (_, Vec<_>)| Conditional::new(first, remaining))
}

pub fn fraction<'a, S: EditorTreeSeq>(input: &mut ParseInput<'a, S>) -> ParseResult<'a, EvalNode> {
    let next_token = input.peek().ok_or(input.err_eof())?;
    let EditorTreeKind::Fraction(fraction) = next_token.kind() else {
        return Err(input.err_expected("a fraction"));
    };
    input.advance();

    let [top, bottom] = [fraction.top(), fraction.bottom()].map(|x| input.derived(x.children()));

    let top = Expr::new().parse_whole("fraction top expr", top).fatal()?;
    let bottom = Expr::new()
        .parse_whole("fraction bottom expr", bottom)
        .fatal()?;

    Ok(EvalNode::fraction(top, bottom))
}

pub fn sum_prod<'a, S: EditorTreeSeq>(input: &mut ParseInput<'a, S>) -> ParseResult<'a, EvalNode> {
    let next_token = input.peek().ok_or(input.err_eof())?;
    let EditorTreeKind::SumProd(sum_prod) = next_token.kind() else {
        return Err(input.err_expected("a sum or product"));
    };
    input.advance();

    let kind = sum_prod.sum_or_prod();
    let expr = multiply().fatal().parse_next(input)?;

    let [top, bottom, ident] =
        [sum_prod.top(), sum_prod.bottom(), sum_prod.ident()].map(|x| input.derived(x.children()));

    let top = Expr::new().parse_whole("sum prod top expr", top).fatal()?;
    let bottom = Expr::new()
        .parse_whole("sum prod bottom expr", bottom)
        .fatal()?;
    let ident = raw_ident().parse_whole("sum prod ident", ident).fatal()?;

    Ok(EvalNode::sum_prod(kind, ident, bottom, top, expr))
}

pub fn list_contents_then_eof<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    Alt::new((
        list_range().surround_whitespace().then_eof("list range"),
        list_literal()
            .surround_whitespace()
            .then_eof("list literal"),
    ))
}

pub fn comma_separated_exprs<S: EditorTreeSeq>() -> impl MyParser<S, Output = Vec<EvalNode>> {
    expr().separated(
        terminal_eq(',').surround_whitespace(),
        ..,
        Vec::new,
        Vec::fold,
    )
}

pub fn list_literal<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    comma_separated_exprs().map(EvalNode::list_literal)
}

pub fn ellipsis<S: EditorTreeSeq>() -> impl MyParser<S, Output = ()> {
    (terminal_eq('.'), terminal_eq('.'), terminal_eq('.')).map(|_| ())
}

pub fn list_range<S: EditorTreeSeq>() -> impl MyParser<S, Output = EvalNode> {
    (
        expr().whitespace_after(),
        terminal_eq(',').preceding(expr()).whitespace_after().opt(),
        terminal_eq(',').whitespace_after().opt(),
        ellipsis().ignore_after(Whitespace::new()),
        terminal_eq(',').whitespace_after().opt(),
        expr(),
    )
        .map(|(from, next, _, _, _, to)| EvalNode::list_range(from, next, to))
}

function_as_parser! {
    pub fn Fraction<S>(input) -> <EvalNode> { fraction(input) }
    pub fn SumProd<S>(input) -> <EvalNode> { sum_prod(input) }
}

macro_rules! chained_parsers {
    ($($name:ident $msg:literal : $etk:ident :: $variant:ident ($x:ident) => $ex:expr,)*) =>{
        $(#[allow(unused)] fn $name<S: EditorTreeSeq, P: MyParser<S>>(parser: P) -> impl MyParser<S, Output = P::Output> {
            Chained::new(|node|
                Some(match node.kind() {
                    $etk::$variant($x) => $ex,
                    _ => return None,
                }),
                parser.then_eof($msg).fatal()
            )
        })*
    };
}

chained_parsers! {
    parens_chained "parens expr": EditorTreeKind::Paren(parens) => parens.child(),
    brackets_chained "brackets expr": EditorTreeKind::Bracket(brackets) => brackets.child(),
    power_chained "power expr": EditorTreeKind::Power(power) => power.child(),
    abs_chained "abs expr": EditorTreeKind::Abs(abs) => abs.child(),
    sqrt_chained "sqrt expr": EditorTreeKind::Sqrt(sqrt) => sqrt.child(),
}
