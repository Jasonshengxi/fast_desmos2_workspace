use std::fmt::Debug;

use fast_desmos2_tree::tree::EditorTree;
use winnow::stream::{Offset, Stream, StreamIsPartial};

#[derive(Debug, Clone, Copy)]
pub struct StreamIndex(pub usize);

impl<'a> Offset for StreamIndex {
    fn offset_from(&self, &start: &Self) -> usize {
        start.0 - self.0
    }
}

#[derive(Clone, Copy)]
pub struct ParseStream<'a> {
    index: usize,
    slice: &'a [EditorTree],
}

impl<'a> Debug for ParseStream<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParseStream")
            .field("index", &self.index)
            .finish()
    }
}

impl<'a> ParseStream<'a> {
    pub fn new(slice: &'a [EditorTree]) -> Self {
        Self { index: 0, slice }
    }

    pub fn index(&self) -> StreamIndex {
        StreamIndex(self.index)
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

impl<'a> Offset<StreamIndex> for ParseStream<'a> {
    fn offset_from(&self, start: &StreamIndex) -> usize {
        self.index().offset_from(start)
    }
}

impl<'a> Stream for ParseStream<'a> {
    type Token = &'a EditorTree;

    type Slice = &'a [EditorTree];

    type IterOffsets = std::iter::Enumerate<std::slice::Iter<'a, EditorTree>>;

    type Checkpoint = StreamIndex;

    fn iter_offsets(&self) -> Self::IterOffsets {
        self.slice[self.index..].iter().enumerate()
    }

    fn eof_offset(&self) -> usize {
        self.slice[self.index..].len()
    }

    fn next_token(&mut self) -> Option<Self::Token> {
        // let (token, remaining) = self.slice.split_first()?;
        // self.slice = remaining;
        // Some(token)
        let token = self.slice.get(self.index)?;
        self.index += 1;
        Some(token)
    }

    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.slice[self.index..]
            .iter()
            .position(predicate)
            .map(|x| x + self.index)
    }

    fn offset_at(&self, tokens: usize) -> Result<usize, winnow::error::Needed> {
        Ok(tokens)
    }

    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        // let (tokens, remaining) = self.slice.split_at(offset);
        // self.slice = remaining;
        // tokens
        let slice = &self.slice[self.index..self.index + offset];
        self.index += offset;
        slice
    }

    fn checkpoint(&self) -> Self::Checkpoint {
        self.index()
    }

    fn reset(&mut self, checkpoint: &Self::Checkpoint) {
        self.index = checkpoint.0;
    }

    fn raw(&self) -> &dyn Debug {
        &self.slice
    }
}
