pub use actions::{ActionOutcome, TreeAction};
pub use movement::{TreeMovable, TreeMove};

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
}

impl EditorTree {
    pub fn terminal(ch: char) -> Self {
        Self {
            kind: EditorTreeKind::Terminal(EditorTreeTerminal::new(ch)),
        }
    }

    pub fn paren(cursor: SurroundIndex, child: EditorTreeSeq) -> Self {
        Self {
            kind: EditorTreeKind::Paren(EditorTreeParen::new(cursor, child)),
        }
    }

    pub fn power(cursor: PowerIndex, base: EditorTreeSeq, power: EditorTreeSeq) -> Self {
        Self {
            kind: EditorTreeKind::Power(EditorTreePower::new(cursor, base, power)),
        }
    }

    pub fn fraction(cursor: FractionIndex, top: EditorTreeSeq, bottom: EditorTreeSeq) -> Self {
        Self {
            kind: EditorTreeKind::Fraction(EditorTreeFraction::new(cursor, top, bottom)),
        }
    }

    pub fn cursor(&self) -> CombinedCursor {
        match &self.kind {
            EditorTreeKind::Power(power) => CombinedCursor::Power(power.cursor()),
            EditorTreeKind::Fraction(fraction) => CombinedCursor::Fraction(fraction.cursor()),
            EditorTreeKind::Terminal(_) => CombinedCursor::Terminal,
            EditorTreeKind::Sqrt(sqrt) => CombinedCursor::Sqrt(sqrt.cursor()),
            EditorTreeKind::Paren(paren) => CombinedCursor::Paren(paren.cursor()),
        }
    }

    pub fn active_child(&self) -> Option<&EditorTreeSeq> {
        match &self.kind {
            EditorTreeKind::Terminal(_) => None,
            EditorTreeKind::Fraction(fraction) => fraction.active_child(),
            EditorTreeKind::Power(power) => power.active_child(),
            EditorTreeKind::Sqrt(sqrt) => sqrt.active_child(),
            EditorTreeKind::Paren(paren) => paren.active_child(),
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombinedCursor {
    Fraction(FractionIndex),
    Power(PowerIndex),
    Terminal,
    Sqrt(SurroundIndex),
    Paren(SurroundIndex),
}

impl CombinedCursor {
    pub const TOP: Self = Self::Fraction(FractionIndex::Top);
    pub const BOTTOM: Self = Self::Fraction(FractionIndex::Bottom);
    pub const LEFT: Self = Self::Fraction(FractionIndex::Left);
    pub const BASE: Self = Self::Power(PowerIndex::Base);
    pub const POWER: Self = Self::Power(PowerIndex::Power);
}

impl From<FractionIndex> for CombinedCursor {
    fn from(value: FractionIndex) -> Self {
        Self::Fraction(value)
    }
}

impl From<PowerIndex> for CombinedCursor {
    fn from(value: PowerIndex) -> Self {
        Self::Power(value)
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
    cursor: SurroundIndex,
    child: EditorTreeSeq,
}
impl_surrounds_tree_seq!(EditorTreeParen);
impl EditorTreeParen {
    pub fn new(cursor: SurroundIndex, child: EditorTreeSeq) -> Self {
        Self { cursor, child }
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
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum PowerIndex {
    Base,
    Power,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreePower {
    cursor: PowerIndex,
    base: EditorTreeSeq,
    power: EditorTreeSeq,
}

impl EditorTreePower {
    pub const fn new(cursor: PowerIndex, base: EditorTreeSeq, power: EditorTreeSeq) -> Self {
        Self {
            cursor,
            base,
            power,
        }
    }

    pub const fn cursor(&self) -> PowerIndex {
        self.cursor
    }

    pub const fn base(&self) -> &EditorTreeSeq {
        &self.base
    }

    pub const fn power(&self) -> &EditorTreeSeq {
        &self.power
    }

    pub const fn active_child(&self) -> Option<&EditorTreeSeq> {
        match self.cursor {
            PowerIndex::Base => Some(&self.base),
            PowerIndex::Power => Some(&self.power),
        }
    }

    pub fn active_child_mut(&mut self) -> Option<&mut EditorTreeSeq> {
        match self.cursor {
            PowerIndex::Base => Some(&mut self.base),
            PowerIndex::Power => Some(&mut self.power),
        }
    }
}
