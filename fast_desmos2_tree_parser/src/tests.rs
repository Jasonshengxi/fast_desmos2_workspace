use pretty_assertions::assert_eq;

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use fast_desmos2_tree::tree::debug::Debugable;
use fast_desmos2_tree::tree::{EditorTree, EditorTreeSeq, SumOrProd, SumProdIndex, SurroundIndex};

use crate::builtins::{Builtins, MonadicPervasive};
use crate::parsing;
use crate::tree::{AddOrSub, CompSet, Conditional, EvalKind, EvalNode, IdentStorer};

struct IdentStorerGuard {
    used: Cell<bool>,
    idents: IdentStorer,
}

impl IdentStorerGuard {
    fn new(idents: IdentStorer) -> Self {
        Self {
            used: Cell::new(false),
            idents,
        }
    }
}

impl Deref for IdentStorerGuard {
    type Target = IdentStorer;

    fn deref(&self) -> &Self::Target {
        self.used.set(true);
        &self.idents
    }
}

impl DerefMut for IdentStorerGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.used.set(true);
        &mut self.idents
    }
}

impl Drop for IdentStorerGuard {
    fn drop(&mut self) {
        if !self.used.get() {
            assert!(self.idents.is_empty())
        }
    }
}

fn parse(tree: impl Into<EditorTreeSeq>) -> (EvalNode, IdentStorerGuard) {
    let idents = IdentStorer::default();
    let tree = tree.into();
    let parsed = parsing::parse(&tree, &idents);
    match parsed {
        Ok(parsed) => (parsed, IdentStorerGuard::new(idents)),
        Err(_err) => {
            let tree = tree.debug(false).render();
            eprintln!("{tree}");
            eprintln!("{_err:#?}");
            panic!("PARSING FAILURE");
        }
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

fn curly(child: impl Into<EditorTreeSeq>) -> EditorTree {
    EditorTree::complete_curly(SurroundIndex::Inside, child.into())
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

fn sum(top: EditorTreeSeq, bottom: EditorTreeSeq, ident: EditorTreeSeq) -> EditorTree {
    EditorTree::sum(SumProdIndex::Top, top, bottom, ident)
}

fn adjoin(parts: Vec<EditorTreeSeq>) -> EditorTreeSeq {
    let mut result = EditorTreeSeq::empty();
    parts.into_iter().for_each(|part| result.extend(part));
    result
}

#[test]
fn test_number_integer() {
    let (parsed, _) = parse(str("  12345 "));
    assert_eq!(parsed, EvalNode::number(12345.0))
}

#[test]
fn test_number_float() {
    let (parsed, _) = parse(str(" 12345.78362    "));
    assert_eq!(parsed, EvalNode::number(12345.78362))
}

#[test]
fn test_number_dot() {
    let (parsed, _) = parse(str("  .78362 "));
    assert_eq!(parsed, EvalNode::number(0.78362))
}

#[test]
fn test_parens_numbers() {
    let (parsed, _) = parse(seq(vec![term(' '), paren(str("  0.001 ")), term(' ')]));
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
fn test_index() {
    let (parsed, _) = parse(adjoin(vec![str("1.89"), one(brackets(str("8")))]));

    assert_eq!(
        parsed,
        EvalNode::index(EvalNode::number(1.89), EvalNode::number(8.0))
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

#[test]
fn test_if_else() {
    let (parsed, _) = parse(curly(str("1=2:3,4")));

    assert_eq!(
        parsed,
        EvalNode::if_else(
            vec![Conditional::new(
                EvalNode::number(1.0),
                vec![(CompSet::EQUAL, EvalNode::number(2.0))]
            )],
            Some(EvalNode::number(3.0)),
            Some(EvalNode::number(4.0)),
        )
    )
}

#[test]
fn test_sum() {
    let (parsed, idents) = parse(adjoin(vec![
        one(sum(str("10"), str("1"), str("x"))),
        str("7"),
    ]));

    let id = idents.convert_id("x");
    assert_eq!(idents.len(), 1);
    assert_eq!(
        parsed,
        EvalNode::sum_prod(
            SumOrProd::Sum,
            id,
            EvalNode::number(1.0),
            EvalNode::number(10.0),
            EvalNode::number(7.0),
        )
    )
}

#[test]
fn test_prod_hard() {
    let (parsed, idents) = parse(adjoin(vec![
        one(sum(str("10"), str("1"), str("x"))),
        one(paren(str("1"))),
        one(paren(str("2"))),
        str("7-8"),
    ]));

    let id = idents.convert_id("x");
    assert_eq!(idents.len(), 1);
    assert_eq!(
        parsed,
        EvalNode::add_sub(vec![
            (
                AddOrSub::Add,
                EvalNode::sum_prod(
                    SumOrProd::Sum,
                    id,
                    EvalNode::number(1.0),
                    EvalNode::number(10.0),
                    EvalNode::multiply(vec![
                        EvalNode::number(1.0),
                        EvalNode::number(2.0),
                        EvalNode::number(7.0)
                    ])
                )
            ),
            (AddOrSub::Sub, EvalNode::number(8.0),)
        ])
    )
}
