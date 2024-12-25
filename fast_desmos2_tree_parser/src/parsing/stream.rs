use std::fmt::Debug;

use fast_desmos2_tree::tree::EditorTree;
use winnow::stream::{Offset, Stream, StreamIsPartial};

#[derive(Debug, Clone, Copy)]
pub struct ParseStream<'a> {
    slice: &'a [EditorTree],
}

impl<'a> ParseStream<'a> {
    pub fn new(slice: &'a [EditorTree]) -> Self {
        Self { slice }
    }
}

impl<'a> Offset<&'a [EditorTree]> for ParseStream<'a> {
    fn offset_from(&self, start: &&'a [EditorTree]) -> usize {
        self.slice.offset_from(start)
    }
}

impl<'a> StreamIsPartial for ParseStream<'a> {
    type PartialState = ();

    fn complete(&mut self) -> Self::PartialState {}
    fn restore_partial(&mut self, _: Self::PartialState) {}

    fn is_partial_supported() -> bool {
        false
    }
}

impl<'a> Stream for ParseStream<'a> {
    type Token = &'a EditorTree;

    type Slice = &'a [EditorTree];

    type IterOffsets = std::iter::Enumerate<std::slice::Iter<'a, EditorTree>>;

    type Checkpoint = &'a [EditorTree];

    fn iter_offsets(&self) -> Self::IterOffsets {
        self.slice.iter().enumerate()
    }

    fn eof_offset(&self) -> usize {
        self.slice.len()
    }

    fn next_token(&mut self) -> Option<Self::Token> {
        let (token, remaining) = self.slice.split_first()?;
        self.slice = remaining;
        Some(token)
    }

    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.slice.iter().position(predicate)
    }

    fn offset_at(&self, tokens: usize) -> Result<usize, winnow::error::Needed> {
        Ok(tokens)
    }

    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        let (tokens, remaining) = self.slice.split_at(offset);
        self.slice = remaining;
        tokens
    }

    fn checkpoint(&self) -> Self::Checkpoint {
        self.slice
    }

    fn reset(&mut self, checkpoint: &Self::Checkpoint) {
        self.slice = checkpoint;
    }

    fn raw(&self) -> &dyn Debug {
        &self.slice
    }
}
