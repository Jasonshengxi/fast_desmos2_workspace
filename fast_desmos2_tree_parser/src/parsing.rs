use fast_desmos2_eval::{EvalNode, IdentStorer};
pub use combinator::MyParser;
pub use error::{ParseError, ResExt};
use fast_desmos2_tree::tree::EditorTreeSeq;
use stream::ParseStream;

mod combinator;
mod error;
mod parser;
mod stream;

pub type ParseInput<'a, S> = ParseStream<'a, S>;
pub type ParseResult<'a, T> = Result<T, ParseError>;
// pub type ParseError<'a> = TreeError<ParseInput<'a>>;
// pub type ParseError<'a, S> = TreeError<ParseInput<'a, S>>;
// pub type ParseError<'a> = ContextError;

#[derive(Debug, Clone, Copy)]
pub struct ParseExtra<'a> {
    idents: &'a IdentStorer,
}

pub fn parse<'a, S: EditorTreeSeq>(
    tree: &'a S,
    idents: &'a IdentStorer,
) -> ParseResult<'a, EvalNode> {
    let state = ParseExtra { idents };
    let input = ParseInput::new(tree.children(), state);
    parser::Expr::new().parse_whole("whole input", input)
}
