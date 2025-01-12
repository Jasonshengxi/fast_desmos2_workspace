use std::{fmt::Debug, panic::Location};

use super::{combinator::Aggregate, stream::StreamIndex};

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    Alt,
    Eof,
    NoInitial,
    NoSeparator,
    NotEnoughRepeat,
    Chained(ParseError),
    NotEof(&'static str),
    Context(&'static str),
    Expected(&'static str),
}

#[derive(Debug, Clone)]
pub struct ParseLayer {
    location: &'static Location<'static>,
    kind: ParseErrorKind,
}

impl ParseLayer {
    #[track_caller]
    pub fn new(kind: ParseErrorKind) -> Self {
        Self {
            location: Location::caller(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AltBehavior {
    Fatal,
    Backtrack,
}

#[derive(Clone)]
pub struct ParseError {
    alt_behavior: AltBehavior,
    index: StreamIndex,
    layers: Vec<ParseLayer>,
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParseError")
            .field("alt_behavior", &self.alt_behavior)
            .field("index", &self.index.0)
            .field("kind", &self.layers)
            .finish()
    }
}

impl ParseError {
    pub fn alt_behavior(&self) -> AltBehavior {
        self.alt_behavior
    }

    #[track_caller]
    pub fn new(index: StreamIndex, kind: ParseErrorKind) -> Self {
        Self {
            alt_behavior: AltBehavior::Backtrack,
            index,
            layers: vec![ParseLayer::new(kind)],
        }
    }

    #[track_caller]
    pub fn layered(self, layer: ParseErrorKind) -> Self {
        Self {
            layers: self.layers.fold(ParseLayer::new(layer)),
            ..self
        }
    }

    pub fn raw_layered(self, layer: ParseLayer) -> Self {
        Self {
            layers: self.layers.fold(layer),
            ..self
        }
    }

    pub fn fatal(self) -> Self {
        Self {
            alt_behavior: AltBehavior::Fatal,
            ..self
        }
    }
}

pub trait ResExt<'a> {
    fn fatal(self) -> Self;
    fn layered(self, layer: ParseErrorKind) -> Self;
}

impl<'a, T> ResExt<'a> for Result<T, ParseError> {
    fn fatal(self) -> Self {
        self.map_err(ParseError::fatal)
    }

    #[track_caller]
    fn layered(self, layer: ParseErrorKind) -> Self {
        let layer = ParseLayer::new(layer);
        self.map_err(|err| err.raw_layered(layer))
    }
}
