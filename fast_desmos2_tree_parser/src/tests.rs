use fast_desmos2_tree::tree::{EditorTree, EditorTreeSeq, SurroundIndex};

use crate::parsing;
use crate::tree::{EvalKind, EvalNode, IdentStorer};

fn parse(tree: EditorTreeSeq) -> (EvalNode, IdentStorer) {
    let idents = IdentStorer::default();
    let parsed = parsing::parse(&tree, &idents);
    (parsed, idents)
}

#[test]
fn number_integer() {
    let (parsed, _) = parse(EditorTreeSeq::str("12345"));
    assert_eq!(parsed, EvalNode::number(12345.0))
}

#[test]
fn number_float() {
    let (parsed, _) = parse(EditorTreeSeq::str("12345.78362"));
    assert_eq!(parsed, EvalNode::number(12345.78362))
}

#[test]
fn number_dot() {
    let (parsed, _) = parse(EditorTreeSeq::str(".78362"));
    assert_eq!(parsed, EvalNode::number(0.78362))
}

#[test]
fn parens_numbers() {
    let (parsed, _) = parse(EditorTreeSeq::one(EditorTree::complete_paren(
        SurroundIndex::Inside,
        EditorTreeSeq::str("0.001"),
    )));
    assert_eq!(parsed, EvalNode::number(0.001))
}

#[test]
fn sqrt_numbers() {
    let (parsed, _) = parse(EditorTreeSeq::one(EditorTree::sqrt(
        SurroundIndex::Inside,
        EditorTreeSeq::str("0.31"),
    )));
    assert_eq!(parsed, EvalNode::sqrt(EvalNode::number(0.31)))
}

#[test]
fn abs_numbers() {
    let (parsed, _) = parse(EditorTreeSeq::one(EditorTree::complete_abs(
        SurroundIndex::Inside,
        EditorTreeSeq::str("0.111"),
    )));
    assert_eq!(parsed, EvalNode::abs(EvalNode::number(0.111)))
}

#[test]
fn identifier() {
    let (parsed, idents) = parse(EditorTreeSeq::str("xyz"));

    let true_id = idents.convert_id("xyz");
    assert_eq!(idents.len(), 1);
    let &EvalKind::Identifier(ident_id) = parsed.kind() else {
        panic!()
    };
    assert_eq!(ident_id, true_id);
}
