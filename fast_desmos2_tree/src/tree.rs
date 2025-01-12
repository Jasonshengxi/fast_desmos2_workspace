pub use actions::{ActionOutcome, TreeAction};
pub use movement::{Direction, Motion, TreeMovable};
use std::fmt::Debug;

use crate::Sealed;

mod actions;
mod movement;

pub trait EditorTreeSeq: Debug + Clone + PartialEq + Sized {
    fn children(&self) -> &[EditorTree<Self>];
    fn children_mut(&mut self) -> &mut Vec<EditorTree<Self>>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn cursor(&self) -> usize;
    fn active_child(&self) -> Option<&EditorTree<Self>>;
    fn active_child_mut(&mut self) -> Option<&mut EditorTree<Self>>;
    fn extend(&mut self, other: Self);
}

#[derive(Clone, PartialEq)]
pub struct EditorTreeSeqVisual {
    start_cursor: usize,
    now_cursor: usize,
    children: Vec<EditorTree<Self>>,
}

impl Debug for EditorTreeSeqVisual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DebugWrapper(self).fmt(f)
    }
}

impl EditorTreeSeq for EditorTreeSeqVisual {
    fn children(&self) -> &[EditorTree<Self>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<EditorTree<Self>> {
        &mut self.children
    }

    fn len(&self) -> usize {
        self.children.len()
    }

    fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    fn cursor(&self) -> usize {
        self.now_cursor
    }

    fn active_child(&self) -> Option<&EditorTree<Self>> {
        self.children.get(self.cursor())
    }

    fn active_child_mut(&mut self) -> Option<&mut EditorTree<Self>> {
        self.children.get_mut(self.now_cursor)
    }

    fn extend(&mut self, other: Self) {
        self.children.extend(other.children);
    }
}

#[derive(Clone, PartialEq)]
pub struct EditorTreeSeqNormal {
    cursor: usize,
    children: Vec<EditorTree<EditorTreeSeqNormal>>,
}

impl Debug for EditorTreeSeqNormal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DebugWrapper(self).fmt(f)
    }
}

struct DebugWrapper<'a, T>(&'a T);
impl<'a, T: EditorTreeSeq> Debug for DebugWrapper<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let all_str = self
            .0
            .children()
            .iter()
            .all(|child| matches!(child.kind(), EditorTreeKind::Terminal(_)));
        if all_str {
            let mut string = String::with_capacity(self.0.children().len());

            for child in self.0.children().iter() {
                let EditorTreeKind::Terminal(term) = child.kind() else {
                    unreachable!()
                };
                string.push(term.ch);
            }

            f.debug_struct("EditorTreeSeq")
                .field("string", &string)
                .finish()
        } else {
            f.debug_struct("EditorTreeSeq")
                .field("cursor", &self.0.cursor())
                .field("children", &self.0.children())
                .finish()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTree<S: EditorTreeSeq> {
    kind: EditorTreeKind<S>,
}

impl From<EditorTree<EditorTreeSeqNormal>> for EditorTreeSeqNormal {
    fn from(value: EditorTree<EditorTreeSeqNormal>) -> Self {
        EditorTreeSeqNormal::one(value)
    }
}

impl EditorTreeSeqNormal {
    pub fn new(cursor: usize, children: Vec<EditorTree<EditorTreeSeqNormal>>) -> Self {
        assert!(cursor <= children.len());
        Self { cursor, children }
    }

    pub fn str(string: &str) -> Self {
        Self::new(
            0,
            string.chars().map(|ch| EditorTree::terminal(ch)).collect(),
        )
    }

    pub fn empty() -> Self {
        Self::new(0, Vec::new())
    }

    pub fn one(child: EditorTree<EditorTreeSeqNormal>) -> Self {
        Self::new(0, vec![child])
    }

    pub fn first(children: Vec<EditorTree<EditorTreeSeqNormal>>) -> Self {
        Self::new(0, children)
    }

    fn move_to(&mut self, to: usize, from: Direction) {
        assert!(to <= self.children.len());
        self.cursor = to;
        if let Some(child) = self.active_child_mut() {
            child.enter_from(from);
        }
    }

    fn move_right(&mut self, by: usize) {
        self.move_to(self.cursor + by, Direction::Left);
    }

    fn move_left(&mut self, by: usize) {
        self.move_to(self.cursor - by, Direction::Right);
    }
}

impl EditorTreeSeq for EditorTreeSeqNormal {
    fn children(&self) -> &[EditorTree<EditorTreeSeqNormal>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<EditorTree<Self>> {
        &mut self.children
    }

    fn len(&self) -> usize {
        self.children.len()
    }

    fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn active_child(&self) -> Option<&EditorTree<EditorTreeSeqNormal>> {
        self.children.get(self.cursor)
    }

    fn active_child_mut(&mut self) -> Option<&mut EditorTree<EditorTreeSeqNormal>> {
        self.children.get_mut(self.cursor)
    }

    fn extend(&mut self, other: Self) {
        self.children.extend(other.children);
    }
}

impl<S: EditorTreeSeq> EditorTree<S> {
    pub fn new(kind: EditorTreeKind<S>) -> Self {
        Self { kind }
    }

    pub fn terminal(ch: char) -> Self {
        Self::new(EditorTreeKind::Terminal(EditorTreeTerminal::new(ch)))
    }

    pub fn sqrt(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Sqrt(EditorTreeSqrt { cursor, child }))
    }

    pub fn power(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Power(EditorTreePower { cursor, child }))
    }

    pub fn complete_paren(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Paren(EditorTreeParen::complete(
            cursor, child,
        )))
    }

    pub fn incomplete_paren(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Paren(EditorTreeParen::incomplete(
            cursor, child,
        )))
    }

    pub fn complete_abs(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Abs(EditorTreeAbs::complete(cursor, child)))
    }

    pub fn incomplete_abs(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Abs(EditorTreeAbs::incomplete(
            cursor, child,
        )))
    }

    pub fn complete_curly(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Curly(EditorTreeCurly::complete(
            cursor, child,
        )))
    }

    pub fn incomplete_curly(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Curly(EditorTreeCurly::incomplete(
            cursor, child,
        )))
    }

    pub fn complete_brackets(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Bracket(EditorTreeBracket::complete(
            cursor, child,
        )))
    }

    pub fn incomplete_brackets(cursor: SurroundIndex, child: S) -> Self {
        Self::new(EditorTreeKind::Bracket(EditorTreeBracket::incomplete(
            cursor, child,
        )))
    }

    pub fn sum(cursor: SumProdIndex, top: S, bottom: S, ident: S) -> Self {
        Self::new(EditorTreeKind::SumProd(EditorTreeSumProd::new(
            SumOrProd::Sum,
            cursor,
            top,
            bottom,
            ident,
        )))
    }

    pub fn prod(cursor: SumProdIndex, top: S, bottom: S, ident: S) -> Self {
        Self::new(EditorTreeKind::SumProd(EditorTreeSumProd::new(
            SumOrProd::Prod,
            cursor,
            top,
            bottom,
            ident,
        )))
    }

    pub fn fraction(cursor: FractionIndex, top: S, bottom: S) -> Self {
        Self::new(EditorTreeKind::Fraction(EditorTreeFraction::new(
            cursor, top, bottom,
        )))
    }

    pub fn kind(&self) -> &EditorTreeKind<S> {
        &self.kind
    }

    pub fn into_take(self) -> EditorTreeKind<S> {
        self.kind
    }

    pub fn cursor(&self) -> CombinedCursor {
        match &self.kind {
            EditorTreeKind::Power(_) => CombinedCursor::Power,
            EditorTreeKind::Fraction(fraction) => CombinedCursor::Fraction(fraction.cursor()),
            EditorTreeKind::Terminal(_) => CombinedCursor::Terminal,
            EditorTreeKind::Sqrt(sqrt) => CombinedCursor::Sqrt(sqrt.cursor()),
            EditorTreeKind::SumProd(sum_prod) => CombinedCursor::SumProd(sum_prod.cursor()),
            EditorTreeKind::Paren(paren) => CombinedCursor::Paren(paren.cursor()),
            EditorTreeKind::Abs(abs) => CombinedCursor::Abs(abs.cursor()),
            EditorTreeKind::Bracket(bracket) => CombinedCursor::Bracket(bracket.cursor()),
            EditorTreeKind::Curly(curly) => CombinedCursor::Curly(curly.cursor()),
        }
    }

    pub fn active_child(&self) -> Option<&S> {
        match &self.kind {
            EditorTreeKind::Terminal(_) => None,
            EditorTreeKind::Fraction(fraction) => fraction.active_child(),
            EditorTreeKind::Power(power) => power.active_child(),
            EditorTreeKind::Sqrt(sqrt) => sqrt.active_child(),
            EditorTreeKind::SumProd(sum_prod) => sum_prod.active_child(),
            EditorTreeKind::Paren(paren) => paren.active_child(),
            EditorTreeKind::Abs(abs) => abs.active_child(),
            EditorTreeKind::Bracket(bracket) => bracket.active_child(),
            EditorTreeKind::Curly(curly) => curly.active_child(),
        }
    }

    pub fn is_terminal_and_eq(&self, other: char) -> bool {
        self.is_terminal_and(|x| x.ch == other)
    }

    pub fn is_terminal_and(&self, func: impl FnOnce(&EditorTreeTerminal) -> bool) -> bool {
        match self.kind() {
            EditorTreeKind::Terminal(term) => func(term),
            _ => false,
        }
    }

    pub fn is_terminal_then<T>(&self, func: impl FnOnce(&EditorTreeTerminal) -> T) -> Option<T> {
        match self.kind() {
            EditorTreeKind::Terminal(term) => Some(func(term)),
            _ => None,
        }
    }

    pub fn is_terminal_and_then<T>(
        &self,
        func: impl FnOnce(&EditorTreeTerminal) -> Option<T>,
    ) -> Option<T> {
        match self.kind() {
            EditorTreeKind::Terminal(term) => func(term),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorTreeKind<S: EditorTreeSeq> {
    Terminal(EditorTreeTerminal),
    Fraction(EditorTreeFraction<S>),
    Power(EditorTreePower<S>),
    Sqrt(EditorTreeSqrt<S>),
    Paren(EditorTreeParen<S>),
    SumProd(EditorTreeSumProd<S>),
    Abs(EditorTreeAbs<S>),
    Bracket(EditorTreeBracket<S>),
    Curly(EditorTreeCurly<S>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombinedCursor {
    Terminal,
    Fraction(FractionIndex),
    Power,
    Sqrt(SurroundIndex),
    Paren(SurroundIndex),
    SumProd(SumProdIndex),
    Abs(SurroundIndex),
    Bracket(SurroundIndex),
    Curly(SurroundIndex),
}

impl CombinedCursor {
    pub const TOP: Self = Self::Fraction(FractionIndex::Top);
    pub const BOTTOM: Self = Self::Fraction(FractionIndex::Bottom);
    pub const LEFT: Self = Self::Fraction(FractionIndex::Left);
}

impl From<FractionIndex> for CombinedCursor {
    fn from(value: FractionIndex) -> Self {
        Self::Fraction(value)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum SurroundIndex {
    Left,
    Inside,
}

#[expect(private_bounds)]
pub trait SurroundsTreeSeq: Sealed {
    type Seq: EditorTreeSeq;

    fn cursor(&self) -> SurroundIndex;
    fn cursor_mut(&mut self) -> &mut SurroundIndex;
    fn child(&self) -> &Self::Seq;
    fn child_mut(&mut self) -> &mut Self::Seq;

    fn active_child(&self) -> Option<&Self::Seq> {
        (self.cursor() == SurroundIndex::Inside).then_some(self.child())
    }
    fn active_child_mut(&mut self) -> Option<&mut Self::Seq> {
        (self.cursor() == SurroundIndex::Inside).then_some(self.child_mut())
    }
    fn set_cursor(&mut self, cursor: SurroundIndex) {
        *self.cursor_mut() = cursor;
    }
}

pub trait CompletableSurrounds: SurroundsTreeSeq {
    fn is_complete(&self) -> bool;
    fn is_complete_mut(&mut self) -> &mut bool;
}

macro_rules! impl_surrounds_tree_seq {
    ($name: ident) => {
        impl<S: EditorTreeSeq> Sealed for $name<S> {}
        impl<S: EditorTreeSeq> SurroundsTreeSeq for $name<S> {
            type Seq = S;

            fn child(&self) -> &S {
                &self.child
            }
            fn child_mut(&mut self) -> &mut S {
                &mut self.child
            }
            fn cursor(&self) -> SurroundIndex {
                self.cursor
            }
            fn cursor_mut(&mut self) -> &mut SurroundIndex {
                &mut self.cursor
            }
        }

        impl<S: EditorTreeSeq> $name<S> {
            pub fn child(&self) -> &S {
                &self.child
            }
            pub fn child_mut(&mut self) -> &mut S {
                &mut self.child
            }
            pub fn cursor(&self) -> SurroundIndex {
                self.cursor
            }
            pub fn cursor_mut(&mut self) -> &mut SurroundIndex {
                &mut self.cursor
            }
        }
    };
}

macro_rules! completable_surrounds {
    ($name: ident) => {
        impl<S: EditorTreeSeq> CompletableSurrounds for $name<S> {
            fn is_complete(&self) -> bool {
                self.is_complete
            }
            fn is_complete_mut(&mut self) -> &mut bool {
                &mut self.is_complete
            }
        }
        impl<S: EditorTreeSeq> $name<S> {
            pub fn is_complete(&self) -> bool {
                self.is_complete
            }
            pub fn new(is_complete: bool, cursor: SurroundIndex, child: S) -> Self {
                Self {
                    is_complete,
                    cursor,
                    child,
                }
            }
            pub fn incomplete(cursor: SurroundIndex, child: S) -> Self {
                Self::new(false, cursor, child)
            }
            pub fn complete(cursor: SurroundIndex, child: S) -> Self {
                Self::new(true, cursor, child)
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeSqrt<S: EditorTreeSeq> {
    cursor: SurroundIndex,
    child: S,
}
impl_surrounds_tree_seq!(EditorTreeSqrt);

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreePower<S: EditorTreeSeq> {
    cursor: SurroundIndex,
    child: S,
}
impl_surrounds_tree_seq!(EditorTreePower);

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeParen<S: EditorTreeSeq> {
    is_complete: bool,
    cursor: SurroundIndex,
    child: S,
}
impl_surrounds_tree_seq!(EditorTreeParen);
completable_surrounds!(EditorTreeParen);

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeBracket<S: EditorTreeSeq> {
    is_complete: bool,
    cursor: SurroundIndex,
    child: S,
}
impl_surrounds_tree_seq!(EditorTreeBracket);
completable_surrounds!(EditorTreeBracket);

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeCurly<S: EditorTreeSeq> {
    is_complete: bool,
    cursor: SurroundIndex,
    child: S,
}
impl_surrounds_tree_seq!(EditorTreeCurly);
completable_surrounds!(EditorTreeCurly);

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeAbs<S: EditorTreeSeq> {
    is_complete: bool,
    cursor: SurroundIndex,
    child: S,
}
impl_surrounds_tree_seq!(EditorTreeAbs);
completable_surrounds!(EditorTreeAbs);

#[derive(Clone, PartialEq)]
pub struct EditorTreeTerminal {
    ch: char,
}

impl Debug for EditorTreeTerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EditorTreeTerminal").field(&self.ch).finish()
    }
}

impl EditorTreeTerminal {
    pub fn new(ch: char) -> Self {
        Self { ch }
    }

    pub fn ch(&self) -> char {
        self.ch
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum FractionIndex {
    Left,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeFraction<S: EditorTreeSeq> {
    cursor: FractionIndex,
    top: S,
    bottom: S,
}

impl<S: EditorTreeSeq> EditorTreeFraction<S> {
    pub const fn new(cursor: FractionIndex, top: S, bottom: S) -> Self {
        Self {
            cursor,
            top,
            bottom,
        }
    }

    pub const fn cursor(&self) -> FractionIndex {
        self.cursor
    }

    pub const fn top(&self) -> &S {
        &self.top
    }

    pub const fn bottom(&self) -> &S {
        &self.bottom
    }

    pub const fn active_child(&self) -> Option<&S> {
        match self.cursor {
            FractionIndex::Left => None,
            FractionIndex::Top => Some(&self.top),
            FractionIndex::Bottom => Some(&self.bottom),
        }
    }

    pub fn active_child_mut(&mut self) -> Option<&mut S> {
        match self.cursor {
            FractionIndex::Left => None,
            FractionIndex::Top => Some(&mut self.top),
            FractionIndex::Bottom => Some(&mut self.bottom),
        }
    }
}

impl<S: EditorTreeSeq + TreeMovable> EditorTreeFraction<S> {
    fn move_to(&mut self, to: FractionIndex, from: Direction) {
        self.cursor = to;
        match to {
            FractionIndex::Left => {}
            FractionIndex::Top => self.top.enter_from(from),
            FractionIndex::Bottom => self.bottom.enter_from(from),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SumProdIndex {
    BottomExpr,
    BottomIdent,
    Top,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SumOrProd {
    Sum,
    Prod,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeSumProd<S: EditorTreeSeq> {
    sum_or_prod: SumOrProd,
    cursor: SumProdIndex,
    top: S,
    bottom: S,
    ident: S,
}

impl<S: EditorTreeSeq> EditorTreeSumProd<S> {
    pub fn new(sum_or_prod: SumOrProd, cursor: SumProdIndex, top: S, bottom: S, ident: S) -> Self {
        Self {
            sum_or_prod,
            cursor,
            top,
            bottom,
            ident,
        }
    }

    pub const fn sum_or_prod(&self) -> SumOrProd {
        self.sum_or_prod
    }

    pub const fn cursor(&self) -> SumProdIndex {
        self.cursor
    }

    pub const fn top(&self) -> &S {
        &self.top
    }

    pub const fn bottom(&self) -> &S {
        &self.bottom
    }

    pub const fn ident(&self) -> &S {
        &self.ident
    }

    pub const fn active_child(&self) -> Option<&S> {
        match self.cursor {
            SumProdIndex::BottomExpr => Some(&self.bottom),
            SumProdIndex::BottomIdent => Some(&self.ident),
            SumProdIndex::Top => Some(&self.top),
            SumProdIndex::Left => None,
        }
    }

    pub fn active_child_mut(&mut self) -> Option<&mut S> {
        match self.cursor {
            SumProdIndex::BottomExpr => Some(&mut self.bottom),
            SumProdIndex::BottomIdent => Some(&mut self.ident),
            SumProdIndex::Top => Some(&mut self.top),
            SumProdIndex::Left => None,
        }
    }
}

impl<S: EditorTreeSeq + TreeMovable> EditorTreeSumProd<S> {
    fn move_to(&mut self, to: SumProdIndex, from: Direction) {
        self.cursor = to;
        match to {
            SumProdIndex::BottomExpr => self.bottom.enter_from(from),
            SumProdIndex::Top => self.top.enter_from(from),
            SumProdIndex::BottomIdent => self.ident.enter_from(from),
            SumProdIndex::Left => {}
        }
    }
}
