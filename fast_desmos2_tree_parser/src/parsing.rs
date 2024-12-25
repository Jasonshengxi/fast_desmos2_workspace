use fast_desmos2_tree::tree::EditorTreeSeq;
use winnow::Stateful;

use crate::tree::{EvalNode, IdentStorer};
use stream::ParseStream;

mod parser;
mod stream;

#[derive(Debug, Clone, Copy)]
pub struct ParseExtra<'a> {
    idents: &'a IdentStorer,
}

pub fn parse<'a>(
    tree: &'a EditorTreeSeq,
    idents: &'a IdentStorer,
) -> parser::ParseResult<'a, EvalNode> {
    let state = ParseExtra { idents };
    let mut input = Stateful {
        input: ParseStream::new(tree.children()),
        state,
    };
    // let parsed = parser::parse_seq(input).parse(SliceStream(tree.children()));
    parser::parse_seq(&mut input)
}
