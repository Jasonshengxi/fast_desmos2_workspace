use fast_desmos2_tree::tree::{EditorTree, EditorTreeSeq, SurroundIndex};

use crate::parsing;
use crate::tree::{EvalKind, EvalNode, IdentStorer};

fn parse(tree: EditorTreeSeq) -> (EvalNode, IdentStorer) {
    let idents = IdentStorer::default();
    let parsed = parsing::parse(&tree, &idents);
    (parsed.unwrap(), idents)
}

fn paren(child: EditorTreeSeq) -> EditorTree {
    EditorTree::complete_paren(SurroundIndex::Inside, child)
}

fn sqrt(child: EditorTreeSeq) -> EditorTree {
    EditorTree::sqrt(SurroundIndex::Inside, child)
}

fn brackets(child: EditorTreeSeq) -> EditorTree {
    EditorTree::complete_brackets(SurroundIndex::Inside, child)
}

fn abs(child: EditorTreeSeq) -> EditorTree {
    EditorTree::complete_abs(SurroundIndex::Inside, child)
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
    let (parsed, _) = parse(EditorTreeSeq::one(paren(EditorTreeSeq::str("0.001"))));
    assert_eq!(parsed, EvalNode::number(0.001))
}

#[test]
fn sqrt_numbers() {
    let (parsed, _) = parse(EditorTreeSeq::one(sqrt(EditorTreeSeq::str("0.31"))));
    assert_eq!(parsed, EvalNode::sqrt(EvalNode::number(0.31)))
}

#[test]
fn abs_numbers() {
    let (parsed, _) = parse(EditorTreeSeq::one(abs(EditorTreeSeq::str("0"))));
    assert_eq!(parsed, EvalNode::abs(EvalNode::number(0.)))
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

#[test]
fn point_literal() {
    let (parsed, _) = parse(EditorTreeSeq::one(paren(EditorTreeSeq::str("1.0,2.0"))));

    assert_eq!(
        parsed,
        EvalNode::point((EvalNode::number(1.0), EvalNode::number(2.0)))
    );
}

#[test]
fn list_literal() {
    let (parsed, _) = parse(EditorTreeSeq::one(brackets(EditorTreeSeq::str("1.0,2.0"))));

    assert_eq!(
        parsed,
        EvalNode::list_literal(vec![EvalNode::number(1.0), EvalNode::number(2.0)])
    );
}

#[test]
fn product_parens() {
    let (parsed, _) = parse(EditorTreeSeq::new(
        0,
        vec![
            paren(EditorTreeSeq::str("1.2")),
            paren(EditorTreeSeq::str("2.7")),
        ],
    ));

    assert_eq!(
        parsed,
        EvalNode::multiply(vec![EvalNode::number(1.2), EvalNode::number(2.7)])
    );
}
