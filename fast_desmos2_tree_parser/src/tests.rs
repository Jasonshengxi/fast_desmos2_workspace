use fast_desmos2_tree::tree::{EditorTree, EditorTreeSeq, SurroundIndex};

use crate::builtins::{Builtins, MonadicPervasive};
use crate::parsing;
use crate::tree::{AddOrSub, EvalKind, EvalNode, IdentStorer};

fn parse(tree: impl Into<EditorTreeSeq>) -> (EvalNode, IdentStorer) {
    let idents = IdentStorer::default();
    let tree = tree.into();
    let parsed = parsing::parse(&tree, &idents);
    match parsed {
        Ok(parsed) => (parsed, idents),
        Err(err) => panic!("{err:#?}"),
    }
}

fn paren(child: impl Into<EditorTreeSeq>) -> EditorTree {
    EditorTree::complete_paren(SurroundIndex::Inside, child.into())
}

fn sqrt(child: impl Into<EditorTreeSeq>) -> EditorTree {
    EditorTree::sqrt(SurroundIndex::Inside, child.into())
}

fn brackets(child: impl Into<EditorTreeSeq>) -> EditorTree {
    EditorTree::complete_brackets(SurroundIndex::Inside, child.into())
}

fn abs(child: impl Into<EditorTreeSeq>) -> EditorTree {
    EditorTree::complete_abs(SurroundIndex::Inside, child.into())
}

fn seq(children: Vec<EditorTree>) -> EditorTreeSeq {
    EditorTreeSeq::new(0, children)
}

fn power(power: impl Into<EditorTreeSeq>) -> EditorTree {
    EditorTree::power(power.into())
}

fn one(child: EditorTree) -> EditorTreeSeq {
    EditorTreeSeq::one(child)
}

fn str(string: &str) -> EditorTreeSeq {
    EditorTreeSeq::str(string)
}

fn term(ch: char) -> EditorTree {
    EditorTree::terminal(ch)
}

fn adjoin(parts: Vec<EditorTreeSeq>) -> EditorTreeSeq {
    let mut result = EditorTreeSeq::empty();
    parts.into_iter().for_each(|part| result.extend(part));
    result
}

#[test]
fn test_number_integer() {
    let (parsed, _) = parse(str("12345"));
    assert_eq!(parsed, EvalNode::number(12345.0))
}

#[test]
fn test_number_float() {
    let (parsed, _) = parse(str("12345.78362"));
    assert_eq!(parsed, EvalNode::number(12345.78362))
}

#[test]
fn test_number_dot() {
    let (parsed, _) = parse(str(".78362"));
    assert_eq!(parsed, EvalNode::number(0.78362))
}

#[test]
fn test_parens_numbers() {
    let (parsed, _) = parse(paren(str("0.001")));
    assert_eq!(parsed, EvalNode::number(0.001))
}

#[test]
fn test_sqrt_numbers() {
    let (parsed, _) = parse(sqrt(str("0.31")));
    assert_eq!(parsed, EvalNode::sqrt(EvalNode::number(0.31)))
}

#[test]
fn test_abs_numbers() {
    let (parsed, _) = parse(abs(str("0.00")));
    assert_eq!(parsed, EvalNode::abs(EvalNode::number(0.)))
}

#[test]
fn test_identifier() {
    let (parsed, idents) = parse(str("xyz"));

    let true_id = idents.convert_id("xyz");
    assert_eq!(idents.len(), 1);
    let &EvalKind::Identifier(ident_id) = parsed.kind() else {
        panic!()
    };
    assert_eq!(ident_id, true_id);
}

#[test]
fn test_point_literal() {
    let (parsed, _) = parse(paren(str("1.0,2.0")));

    assert_eq!(
        parsed,
        EvalNode::point((EvalNode::number(1.0), EvalNode::number(2.0)))
    );
}

#[test]
fn test_list_literal() {
    let (parsed, _) = parse(brackets(str("1.0,2.0")));

    assert_eq!(
        parsed,
        EvalNode::list_literal(vec![EvalNode::number(1.0), EvalNode::number(2.0)])
    );
}

#[test]
fn test_builtins_call() {
    let (parsed, _) = parse(adjoin(vec![str("sin"), one(paren(str("1.0")))]));

    let EvalKind::BuiltinsCall {
        builtins,
        power,
        params,
    } = parsed.into_kind()
    else {
        panic!()
    };
    assert_eq!(builtins, Builtins::MonadicPervasive(MonadicPervasive::Sin));
    assert_eq!(power, None);
    assert_eq!(params, vec![EvalNode::number(1.0),])
}

#[test]
fn test_function_call() {
    let (parsed, idents) = parse(seq(vec![term('f'), paren(str("1.0"))]));

    let true_id = idents.convert_id("f");
    assert_eq!(idents.len(), 1);
    let EvalKind::FunctionCall {
        ident,
        power,
        params,
    } = parsed.into_kind()
    else {
        panic!()
    };
    assert_eq!(ident, true_id);
    assert_eq!(power, None);
    assert_eq!(params, vec![EvalNode::number(1.0),])
}

#[test]
fn test_product_parens() {
    let (parsed, _) = parse(seq(vec![paren(str("1.2")), paren(str("2.7"))]));

    assert_eq!(
        parsed,
        EvalNode::multiply(vec![EvalNode::number(1.2), EvalNode::number(2.7)])
    );
}

#[test]
fn test_power() {
    let (parsed, _) = parse(adjoin(vec![str("1.6"), one(power(str("7.3")))]));

    assert_eq!(
        parsed,
        EvalNode::power(EvalNode::number(1.6), EvalNode::number(7.3))
    )
}

#[test]
fn test_range_step() {
    let (parsed, _) = parse(brackets(str("1.2,2.3,...,7.2")));

    assert_eq!(
        parsed,
        EvalNode::list_range(
            EvalNode::number(1.2),
            Some(EvalNode::number(2.3)),
            EvalNode::number(7.2),
        )
    )
}

#[test]
fn test_range_simple() {
    let (parsed, _) = parse(brackets(str("0...1")));

    assert_eq!(
        parsed,
        EvalNode::list_range(EvalNode::number(0.0), None, EvalNode::number(1.0),)
    )
}

#[test]
fn test_add_sub() {
    let (parsed, _) = parse(str("0+1"));

    assert_eq!(
        parsed,
        EvalNode::add_sub(vec![
            (AddOrSub::Add, EvalNode::number(0.0)),
            (AddOrSub::Add, EvalNode::number(1.0)),
        ])
    )
}
