pub use actions::{ActionOutcome, TreeAction};
pub use movement::{Direction, Motion, TreeMovable};

mod actions;
pub mod debug;
mod movement;

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeSeq {
    cursor: usize,
    children: Vec<EditorTree>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTree {
    kind: EditorTreeKind,
}

impl From<EditorTree> for EditorTreeSeq {
    fn from(value: EditorTree) -> Self {
        EditorTreeSeq::one(value)
    }
}

impl EditorTreeSeq {
    pub fn new(cursor: usize, children: Vec<EditorTree>) -> Self {
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

    pub fn one(child: EditorTree) -> Self {
        Self::new(0, vec![child])
    }

    pub fn first(children: Vec<EditorTree>) -> Self {
        Self::new(0, children)
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn active_child(&self) -> Option<&EditorTree> {
        self.children.get(self.cursor)
    }

    pub fn active_child_mut(&mut self) -> Option<&mut EditorTree> {
        self.children.get_mut(self.cursor)
    }

    pub fn extend(&mut self, other: Self) {
        self.children.extend(other.children);
    }

    pub fn move_to(&mut self, to: usize, from: Direction) {
        assert!(to <= self.children.len());
        self.cursor = to;
        if let Some(child) = self.active_child_mut() {
            child.enter_from(from);
        }
    }

    pub fn move_right(&mut self, by: usize) {
        self.move_to(self.cursor + by, Direction::Left);
    }

    pub fn move_left(&mut self, by: usize) {
        self.move_to(self.cursor - by, Direction::Right);
    }
}

impl EditorTree {
    pub fn terminal(ch: char) -> Self {
        Self {
            kind: EditorTreeKind::Terminal(EditorTreeTerminal::new(ch)),
        }
    }

    pub fn complete_paren(cursor: SurroundIndex, child: EditorTreeSeq) -> Self {
        Self {
            kind: EditorTreeKind::Paren(EditorTreeParen::complete(cursor, child)),
        }
    }

    pub fn incomplete_paren(cursor: SurroundIndex, child: EditorTreeSeq) -> Self {
        Self {
            kind: EditorTreeKind::Paren(EditorTreeParen::incomplete(cursor, child)),
        }
    }

    pub fn power(power: EditorTreeSeq) -> Self {
        Self {
            kind: EditorTreeKind::Power(EditorTreePower::new(power)),
        }
    }

    pub fn fraction(cursor: FractionIndex, top: EditorTreeSeq, bottom: EditorTreeSeq) -> Self {
        Self {
            kind: EditorTreeKind::Fraction(EditorTreeFraction::new(cursor, top, bottom)),
        }
    }

    pub fn cursor(&self) -> CombinedCursor {
        match &self.kind {
            EditorTreeKind::Power(_) => CombinedCursor::Power,
            EditorTreeKind::Fraction(fraction) => CombinedCursor::Fraction(fraction.cursor()),
            EditorTreeKind::Terminal(_) => CombinedCursor::Terminal,
            EditorTreeKind::Sqrt(sqrt) => CombinedCursor::Sqrt(sqrt.cursor()),
            EditorTreeKind::Paren(paren) => CombinedCursor::Paren(paren.cursor()),
            EditorTreeKind::SumProd(sum_prod) => CombinedCursor::SumProd(sum_prod.cursor()),
        }
    }

    pub fn active_child(&self) -> Option<&EditorTreeSeq> {
        match &self.kind {
            EditorTreeKind::Terminal(_) => None,
            EditorTreeKind::Fraction(fraction) => fraction.active_child(),
            EditorTreeKind::Power(power) => Some(power.power()),
            EditorTreeKind::Sqrt(sqrt) => sqrt.active_child(),
            EditorTreeKind::Paren(paren) => paren.active_child(),
            EditorTreeKind::SumProd(sum_prod) => sum_prod.active_child(),
        }
    }

    pub fn is_terminal_and_eq(&self, other: char) -> bool {
        self.is_terminal_and(|x| x.ch == other)
    }

    pub fn is_terminal_and(&self, func: impl FnOnce(&EditorTreeTerminal) -> bool) -> bool {
        if let EditorTreeKind::Terminal(term) = &self.kind {
            func(term)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorTreeKind {
    Terminal(EditorTreeTerminal),
    Fraction(EditorTreeFraction),
    Power(EditorTreePower),
    Sqrt(EditorTreeSqrt),
    Paren(EditorTreeParen),
    SumProd(EditorTreeSumProd),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombinedCursor {
    Fraction(FractionIndex),
    Power,
    Terminal,
    Sqrt(SurroundIndex),
    Paren(SurroundIndex),
    SumProd(SumProdIndex),
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

trait SurroundsTreeSeq {
    fn cursor(&self) -> SurroundIndex;
    fn cursor_mut(&mut self) -> &mut SurroundIndex;
    fn child(&self) -> &EditorTreeSeq;
    fn child_mut(&mut self) -> &mut EditorTreeSeq;

    fn active_child(&self) -> Option<&EditorTreeSeq> {
        (self.cursor() == SurroundIndex::Inside).then_some(self.child())
    }
    fn active_child_mut(&mut self) -> Option<&mut EditorTreeSeq> {
        (self.cursor() == SurroundIndex::Inside).then_some(self.child_mut())
    }
    fn set_cursor(&mut self, cursor: SurroundIndex) {
        *self.cursor_mut() = cursor;
    }
}

macro_rules! impl_surrounds_tree_seq {
    ($name: ident) => {
        impl SurroundsTreeSeq for $name {
            fn child(&self) -> &EditorTreeSeq {
                &self.child
            }
            fn child_mut(&mut self) -> &mut EditorTreeSeq {
                &mut self.child
            }
            fn cursor(&self) -> SurroundIndex {
                self.cursor
            }
            fn cursor_mut(&mut self) -> &mut SurroundIndex {
                &mut self.cursor
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeSqrt {
    cursor: SurroundIndex,
    child: EditorTreeSeq,
}
impl_surrounds_tree_seq!(EditorTreeSqrt);

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeParen {
    is_complete: bool,
    cursor: SurroundIndex,
    child: EditorTreeSeq,
}
impl_surrounds_tree_seq!(EditorTreeParen);
impl EditorTreeParen {
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    pub fn incomplete(cursor: SurroundIndex, child: EditorTreeSeq) -> Self {
        Self {
            is_complete: false,
            cursor,
            child,
        }
    }

    pub fn complete(cursor: SurroundIndex, child: EditorTreeSeq) -> Self {
        Self {
            is_complete: false,
            cursor,
            child,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeTerminal {
    ch: char,
}

impl EditorTreeTerminal {
    pub fn new(ch: char) -> Self {
        Self { ch }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum FractionIndex {
    Left,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeFraction {
    cursor: FractionIndex,
    top: EditorTreeSeq,
    bottom: EditorTreeSeq,
}

impl EditorTreeFraction {
    pub const fn new(cursor: FractionIndex, top: EditorTreeSeq, bottom: EditorTreeSeq) -> Self {
        Self {
            cursor,
            top,
            bottom,
        }
    }

    pub const fn cursor(&self) -> FractionIndex {
        self.cursor
    }

    pub const fn top(&self) -> &EditorTreeSeq {
        &self.top
    }

    pub const fn bottom(&self) -> &EditorTreeSeq {
        &self.bottom
    }

    pub const fn active_child(&self) -> Option<&EditorTreeSeq> {
        match self.cursor {
            FractionIndex::Left => None,
            FractionIndex::Top => Some(&self.top),
            FractionIndex::Bottom => Some(&self.bottom),
        }
    }

    pub fn active_child_mut(&mut self) -> Option<&mut EditorTreeSeq> {
        match self.cursor {
            FractionIndex::Left => None,
            FractionIndex::Top => Some(&mut self.top),
            FractionIndex::Bottom => Some(&mut self.bottom),
        }
    }

    fn move_to(&mut self, to: FractionIndex, from: Direction) {
        self.cursor = to;
        match to {
            FractionIndex::Left => {}
            FractionIndex::Top => self.top.enter_from(from),
            FractionIndex::Bottom => self.bottom.enter_from(from),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreePower {
    power: EditorTreeSeq,
}

impl EditorTreePower {
    pub const fn new(power: EditorTreeSeq) -> Self {
        Self { power }
    }

    pub const fn power(&self) -> &EditorTreeSeq {
        &self.power
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SumProdIndex {
    BottomExpr,
    BottomEq,
    BottomIdent,
    Top,
    Left,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeSumProd {
    cursor: SumProdIndex,
    top: EditorTreeSeq,
    bottom: EditorTreeSeq,
    ident: EditableIdent,
}

#[derive(Debug, Clone, PartialEq)]
struct EditableIdent {
    cursor: usize,
    ident: Vec<char>,
}

impl From<Vec<char>> for EditableIdent {
    fn from(value: Vec<char>) -> Self {
        Self::new(0, value)
    }
}

impl EditableIdent {
    pub fn new(cursor: usize, ident: Vec<char>) -> Self {
        assert!(cursor < ident.len());
        Self { cursor, ident }
    }
}

impl EditorTreeSumProd {
    pub fn default_counter() -> Self {
        Self {
            cursor: SumProdIndex::Top,
            top: EditorTreeSeq::one(EditorTree::terminal('5')),
            bottom: EditorTreeSeq::one(EditorTree::terminal('0')),
            ident: EditableIdent::from(vec!['n']),
        }
    }

    pub const fn cursor(&self) -> SumProdIndex {
        self.cursor
    }

    pub const fn active_child(&self) -> Option<&EditorTreeSeq> {
        match self.cursor {
            SumProdIndex::BottomExpr => Some(&self.bottom),
            SumProdIndex::Top => Some(&self.top),
            SumProdIndex::Left | SumProdIndex::BottomEq | SumProdIndex::BottomIdent => None,
        }
    }

    pub fn active_child_mut(&mut self) -> Option<&mut EditorTreeSeq> {
        match self.cursor {
            SumProdIndex::BottomExpr => Some(&mut self.bottom),
            SumProdIndex::Top => Some(&mut self.top),
            SumProdIndex::Left | SumProdIndex::BottomEq | SumProdIndex::BottomIdent => None,
        }
    }

    fn move_to(&mut self, to: SumProdIndex, from: Direction) {
        self.cursor = to;
        match to {
            SumProdIndex::BottomExpr => self.bottom.enter_from(from),
            SumProdIndex::Top => self.top.enter_from(from),
            SumProdIndex::BottomIdent => todo!(),
            SumProdIndex::BottomEq => {}
            SumProdIndex::Left => {}
        }
    }
}
