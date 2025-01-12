use std::fmt::Debug;

use fast_desmos2_tree::tree::{EditorTree, EditorTreeSeq};

use super::{error::ParseErrorKind, ParseError, ParseExtra};

#[derive(Debug, Clone, Copy)]
pub struct StreamIndex(pub usize);

impl StreamIndex {
    pub fn offset_from(self, start: Self) -> usize {
        start.0 - self.0
    }
}

#[derive(Clone, Copy)]
pub struct ParseStream<'a, S: EditorTreeSeq> {
    index: usize,
    slice: &'a [EditorTree<S>],

    extra: ParseExtra<'a>,
}

impl<'a, S: EditorTreeSeq> Debug for ParseStream<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParseStream")
            .field("index", &self.index)
            .finish()
    }
}

impl<'a, S: EditorTreeSeq> ParseStream<'a, S> {
    pub fn new(slice: &'a [EditorTree<S>], extra: ParseExtra<'a>) -> Self {
        Self {
            index: 0,
            slice,
            extra,
        }
    }

    pub fn derived(&self, slice: &'a [EditorTree<S>]) -> Self {
        Self::new(slice, self.extra())
    }

    pub fn index(&self) -> StreamIndex {
        StreamIndex(self.index)
    }

    pub fn offset_from(&self, start: StreamIndex) -> usize {
        self.index().offset_from(start)
    }

    pub fn extra(&self) -> ParseExtra<'a> {
        self.extra
    }

    pub fn iter_offsets(&self) -> impl Iterator<Item = (usize, &'a EditorTree<S>)> {
        self.slice[self.index..].iter().enumerate()
    }

    pub fn eof_offset(&self) -> usize {
        self.slice[self.index..].len()
    }

    pub fn peek(&self) -> Option<&'a EditorTree<S>> {
        self.slice.get(self.index)
    }

    pub fn advance(&mut self) {
        self.index += 1;
    }

    pub fn next_token(&mut self) -> Option<&'a EditorTree<S>> {
        let token = self.peek()?;
        self.advance();
        Some(token)
    }

    pub fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(&'a EditorTree<S>) -> bool,
    {
        self.slice[self.index..]
            .iter()
            .position(predicate)
            .map(|x| x + self.index)
    }

    pub fn next_slice(&mut self, offset: usize) -> &'a [EditorTree<S>] {
        let slice = &self.slice[self.index..self.index + offset];
        self.index += offset;
        slice
    }

    pub fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|tree| tree.is_terminal_and_eq(' ')) {
            self.advance();
        }
    }

    pub fn checkpoint(&self) -> StreamIndex {
        self.index()
    }

    #[track_caller]
    pub fn err(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(self.checkpoint(), kind)
    }

    #[track_caller]
    pub fn err_not_eof(&self, msg: &'static str) -> ParseError {
        self.err(ParseErrorKind::NotEof(msg))
    }

    #[track_caller]
    pub fn err_eof(&self) -> ParseError {
        self.err(ParseErrorKind::Eof)
    }

    #[track_caller]
    pub fn err_expected(&self, expect: &'static str) -> ParseError {
        self.err(ParseErrorKind::Expected(expect))
    }

    pub fn reset_to(&mut self, checkpoint: StreamIndex) {
        self.index = checkpoint.0;
    }
}
