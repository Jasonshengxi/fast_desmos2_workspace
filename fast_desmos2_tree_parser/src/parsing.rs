use fast_desmos2_tree::tree::{
    EditorTree, EditorTreeFraction, EditorTreePower, EditorTreeSeq, EditorTreeTerminal,
};

use crate::tree::{EvalKind, EvalNode, IdentStorer};

#[derive(Clone, Copy)]
struct ParseInput<'a> {
    idents: &'a IdentStorer,
}

trait Parseable {
    fn parse<'a>(&self, input: ParseInput<'a>) -> EvalNode;
}

impl Parseable for EditorTreeSeq {
    fn parse<'a>(&self, input: ParseInput<'a>) -> EvalNode {
        todo!()
    }
}

impl Parseable for EditorTreeTerminal {
    fn parse<'a>(&self, input: ParseInput<'a>) -> EvalNode {
        todo!()
    }
}

impl Parseable for EditorTreeFraction {
    fn parse<'a>(&self, input: ParseInput<'a>) -> EvalNode {
        EvalNode::new(EvalKind::Frac {
            top: self.top().parse(input),
            bottom: self.bottom().parse(input),
        })
    }
}

impl Parseable for EditorTreePower {
    fn parse<'a>(&self, input: ParseInput<'a>) -> EvalNode {
        EvalNode::new(EvalKind::Exp {
            expr: self.base().parse(input),
            exp: self.power().parse(input),
        })
    }
}
