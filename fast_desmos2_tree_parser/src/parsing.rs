use combine::{stream::SliceStream, Parser};
use fast_desmos2_tree::tree::EditorTreeSeq;

use crate::tree::{EvalKind, EvalNode, IdentStorer};

mod parser;

#[derive(Clone, Copy)]
struct ParseExtra<'a> {
    idents: &'a IdentStorer,
}

pub fn parse(tree: &EditorTreeSeq, idents: &IdentStorer) -> EvalNode {
    let input = ParseExtra { idents };
    let parsed = parser::parse_seq(input).parse(SliceStream(tree.children()));
    parsed.unwrap().0
}
