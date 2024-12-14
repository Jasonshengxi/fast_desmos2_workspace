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

use EditorTreeKind as TK;
impl EditorTree {
    pub fn str(string: &str) -> Self {
        Self::terminal(0, string.to_string())
    }

    pub fn terminal(cursor: usize, string: String) -> Self {
        Self {
            kind: EditorTreeKind::Terminal(EditorTreeTerminal::new(cursor, string)),
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
            EditorTreeKind::Terminal(terminal) => CombinedCursor::Terminal(terminal.cursor()),
        }
    }

    pub fn active_child(&self) -> Option<&EditorTreeSeq> {
        match &self.kind {
            TK::Terminal(_) => None,
            TK::Fraction(fraction) => fraction.active_child(),
            TK::Power(power) => power.active_child(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorTreeKind {
    Terminal(EditorTreeTerminal),
    Fraction(EditorTreeFraction),
    Power(EditorTreePower),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombinedCursor {
    Terminal(usize),
    Fraction(FractionIndex),
    Power(PowerIndex),
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

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTreeTerminal {
    cursor: usize,
    string: String,
}

impl EditorTreeTerminal {
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn new(cursor: usize, string: String) -> Self {
        assert!(cursor < string.len());
        Self { cursor, string }
    }

    pub fn to_byte_cursor(&self, cursor: usize) -> Option<usize> {
        self.string.char_indices().nth(cursor).map(|x| x.0)
    }

    pub fn byte_cursor(&self) -> usize {
        self.to_byte_cursor(self.cursor).unwrap()
    }

    pub fn char_at(&self) -> char {
        self.string.chars().nth(self.cursor).unwrap()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.string.insert(self.byte_cursor(), ch);
        self.cursor += 1;
    }

    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }

    pub fn pop(&mut self) -> Option<char> {
        self.string.pop()
    }

    pub fn push(&mut self, ch: char) {
        self.string.push(ch)
    }

    /// Returns success.
    /// - `true` indicates character was successfully removed
    /// - `false` indicates character wasn't removed
    pub fn backspace_char(&mut self) -> bool {
        match self.cursor.checked_sub(1) {
            Some(leftwards) => {
                let index = self.to_byte_cursor(leftwards).unwrap();
                self.string.remove(index);
                self.cursor -= 1;
                true
            }
            None => false,
        }
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
