use fast_desmos2_utils::ResExt;
use std::fmt::Display;

use thiserror::Error;

use crate::tree::{EditorTree, EditorTreeKind, EditorTreeSeq};

#[derive(Debug, Clone, Copy)]
pub enum ExpectCategory {
    NumberDecimal,
    NumberInteger,
    Identifier,
}

impl Display for ExpectCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectCategory::NumberDecimal => write!(f, "decimal part of number"),
            ExpectCategory::NumberInteger => write!(f, "integer part of number"),
            ExpectCategory::Identifier => write!(f, "an identifier"),
        }
    }
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("SumProd was found first")]
    FoundSumProdFirst,
    #[error("Character `{:?}` was found, which is unknown", .0)]
    UnknownChar(char),
    #[error("Expected {}", .expect)]
    WrongChar { expect: ExpectCategory },
    #[error("No elements left")]
    RanOut,

    #[deprecated]
    #[error("Generic Error")]
    Generic,
}
pub type SearchResult<T> = Result<T, SearchError>;

struct SearchState<'a> {
    seq: &'a EditorTreeSeq,
    index: usize,
}

impl SearchState<'_> {
    fn peek(&self) -> Option<&EditorTree> {
        self.index
            .checked_sub(1)
            .and_then(|left| self.seq.children.get(left))
    }

    fn advance(&mut self) {
        self.index -= 1;
    }

    fn advance_number(&mut self) -> SearchResult<()> {
        let original_index = self.index;

        let mut none_consumed = true;
        while self
            .peek()
            .is_some_and(|tree| tree.is_terminal_and(|term| term.ch.is_ascii_digit()))
        {
            self.advance();
            none_consumed = false;
        }
        if none_consumed {
            self.index = original_index;
            return Err(SearchError::WrongChar {
                expect: ExpectCategory::NumberDecimal,
            });
        }
        if self.peek().is_some_and(|tree| tree.is_terminal_and_eq('.')) {
            self.advance();
            none_consumed = true;
            while self
                .peek()
                .is_some_and(|tree| tree.is_terminal_and(|term| term.ch.is_ascii_digit()))
            {
                self.advance();
                none_consumed = false;
            }
            if none_consumed {
                self.index = original_index;
                return Err(SearchError::WrongChar {
                    expect: ExpectCategory::NumberInteger,
                });
            }
        }
        Ok(())
    }

    fn advance_ident(&mut self) -> SearchResult<()> {
        let mut none_consumed = true;
        while self
            .peek()
            .is_some_and(|tree| tree.is_terminal_and(|term| term.ch.is_ascii_alphabetic()))
        {
            self.advance();
            none_consumed = false;
        }
        if none_consumed {
            Err(SearchError::WrongChar {
                expect: ExpectCategory::Identifier,
            })
        } else {
            Ok(())
        }
    }

    /// Advances (backwards) by an item.
    ///
    /// an "item" is a word I just came up with that describes something with value that has
    /// precedence >= multiplication.
    ///
    /// Thus, a^b is an item, (1+2) is an item, while a*b is not an item.
    fn advance_item(&mut self) -> SearchResult<()> {
        match &self.peek().ok_or(SearchError::RanOut)?.kind {
            EditorTreeKind::Terminal(term) => match term.ch {
                'a'..='z' | 'A'..='Z' => self.advance_ident().assert_ok(),
                '0'..='9' => self.advance_number().assert_ok(),
                ch => Err(SearchError::UnknownChar(ch)),
            },
            EditorTreeKind::Power(_) => {
                self.advance();
                self.advance_item()
            }
            EditorTreeKind::SumProd(_) => Err(SearchError::FoundSumProdFirst),
            EditorTreeKind::Fraction(_) | EditorTreeKind::Sqrt(_) | EditorTreeKind::Paren(_) => {
                self.advance();
                Ok(())
            }
        }
    }

    /// Advances (backwards) by a pack.
    ///
    /// a "pack" is a word I just came up with that describes some consecutive items grouped by the
    /// fact that they're consecutive with no operator.
    ///
    /// Something like (1+2)(3+4) should be this.
    fn advance_pack(&mut self) -> SearchResult<()> {
        self.advance_item()?;
        while self.advance_item().is_ok() {}
        Ok(())
    }
}

impl EditorTreeSeq {
    pub fn search_back(&self, start: usize) -> SearchResult<usize> {
        let mut state = SearchState {
            seq: self,
            index: start,
        };

        state.advance_pack()?;

        Ok(state.index)
    }
}
