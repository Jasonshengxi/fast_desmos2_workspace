use crate::tree::SumProdIndex;

use super::{
    EditableIdent, EditorTree, EditorTreeFraction, EditorTreeKind, EditorTreePower, EditorTreeSeq,
    EditorTreeSumProd, EditorTreeTerminal, FractionIndex, SurroundIndex, SurroundsTreeSeq,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Up,
    Down,
    Left,
    Right,

    /// vim motion `w`
    Word,
    /// vim motion `b`
    Back,
    /// vim motion `^`
    First,
    /// vim motion `$`
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl From<Direction> for Motion {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => Self::Up,
            Direction::Down => Self::Down,
            Direction::Left => Self::Left,
            Direction::Right => Self::Right,
        }
    }
}

pub trait TreeMovable {
    fn apply_move(&mut self, movement: Motion) -> Option<Motion>;
    fn enter_from(&mut self, direction: Direction);
}

impl TreeMovable for EditorTreeSeq {
    fn apply_move(&mut self, movement: Motion) -> Option<Motion> {
        let movement = self
            .children
            .get_mut(self.cursor)
            .map_or(Some(movement), |child| child.apply_move(movement));

        match movement {
            Some(Motion::Left) => match self.cursor.checked_sub(1) {
                Some(left) => self.move_to(left, Direction::Right),
                None => return Some(Motion::Left),
            },
            Some(Motion::Right) => match self.cursor == self.children.len() {
                true => return Some(Motion::Right),
                false => self.move_right(1),
            },
            Some(Motion::Word) => todo!(),
            Some(Motion::Back) => todo!(),
            Some(Motion::First) => match self.cursor == 0 {
                true => return movement,
                false => self.move_to(0, Direction::Left),
            },
            Some(Motion::Last) => match self.cursor == self.children.len() {
                true => return movement,
                false => self.move_to(self.children.len(), Direction::Right),
            },
            None | Some(Motion::Up | Motion::Down) => return None,
        }
        None
    }

    fn enter_from(&mut self, direction: Direction) {
        match direction {
            Direction::Left => self.move_to(0, direction),
            Direction::Right => self.cursor = self.children.len(),
            Direction::Up | Direction::Down => {
                if let Some(child) = self.active_child_mut() {
                    child.enter_from(direction);
                }
            }
        }
    }
}

impl TreeMovable for EditorTreeTerminal {
    fn apply_move(&mut self, movement: Motion) -> Option<Motion> {
        Some(movement)
    }

    fn enter_from(&mut self, _direction: Direction) {}
}
impl TreeMovable for EditorTreePower {
    fn apply_move(&mut self, movement: Motion) -> Option<Motion> {
        self.power.apply_move(movement)
    }

    fn enter_from(&mut self, direction: Direction) {
        self.power.enter_from(direction);
    }
}
impl TreeMovable for EditorTreeFraction {
    fn apply_move(&mut self, movement: Motion) -> Option<Motion> {
        match self.cursor {
            FractionIndex::Bottom => match self.bottom.apply_move(movement) {
                Some(Motion::Up) => self.move_to(FractionIndex::Top, Direction::Down),
                Some(Motion::Left) => self.cursor = FractionIndex::Left,
                otherwise => return otherwise,
            },
            FractionIndex::Top => match self.top.apply_move(movement) {
                Some(Motion::Down) => self.move_to(FractionIndex::Bottom, Direction::Up),
                Some(Motion::Left) => self.move_to(FractionIndex::Left, Direction::Right),
                otherwise => return otherwise,
            },
            FractionIndex::Left => match movement {
                Motion::Right => self.move_to(FractionIndex::Top, Direction::Left),
                otherwise => return Some(otherwise),
            },
        }
        None
    }

    fn enter_from(&mut self, direction: Direction) {
        match direction {
            Direction::Down => self.move_to(FractionIndex::Bottom, direction),
            Direction::Up | Direction::Right => self.move_to(FractionIndex::Top, direction),
            Direction::Left => self.move_to(FractionIndex::Left, direction),
        }
    }
}

impl<T: SurroundsTreeSeq> TreeMovable for T {
    fn apply_move(&mut self, movement: Motion) -> Option<Motion> {
        match self.cursor() {
            SurroundIndex::Left => match movement {
                Motion::Right => {
                    self.set_cursor(SurroundIndex::Inside);
                    self.child_mut().enter_from(Direction::Left);
                }
                _ => return Some(movement), // for all intents and purposes, the left of parens is
                                            // outside the parens.
            },
            SurroundIndex::Inside => match self.child_mut().apply_move(movement) {
                Some(Motion::Left | Motion::Back) => self.set_cursor(SurroundIndex::Left),
                outcome @ (None
                | Some(
                    Motion::Down
                    | Motion::Up
                    | Motion::Right
                    | Motion::Word
                    | Motion::Last
                    | Motion::First,
                )) => return outcome,
            },
        }
        None
    }

    fn enter_from(&mut self, direction: Direction) {
        match direction {
            Direction::Left | Direction::Up => self.set_cursor(SurroundIndex::Left),
            Direction::Right | Direction::Down => {
                self.set_cursor(SurroundIndex::Inside);
                self.child_mut().enter_from(direction);
            }
        }
    }
}

impl TreeMovable for EditableIdent {
    fn apply_move(&mut self, movement: Motion) -> Option<Motion> {
        match movement {
            Motion::Left => match self.cursor.checked_sub(1) {
                None => return Some(movement),
                Some(left) => self.cursor = left,
            },
            Motion::Right => match self.cursor == self.ident.len() - 1 {
                true => return Some(movement),
                false => self.cursor += 1,
            },
            Motion::First | Motion::Back => match self.cursor == 0 {
                true => return Some(movement),
                false => self.cursor = 0,
            },
            Motion::Last | Motion::Word | Motion::Up | Motion::Down => return Some(movement),
        }
        None
    }

    fn enter_from(&mut self, direction: Direction) {
        match direction {
            Direction::Up | Direction::Down => {}
            Direction::Left => self.cursor = 0,
            Direction::Right => unreachable!(),
        }
    }
}

impl TreeMovable for EditorTreeSumProd {
    fn apply_move(&mut self, movement: Motion) -> Option<Motion> {
        match self.cursor {
            SumProdIndex::BottomExpr => match self.bottom.apply_move(movement) {
                Some(Motion::Up) => self.move_to(SumProdIndex::Top, Direction::Down),
                Some(Motion::Left | Motion::Back) => {
                    self.move_to(SumProdIndex::BottomEq, Direction::Right)
                }
                Some(Motion::First) => self.move_to(SumProdIndex::BottomIdent, Direction::Left),
                outcome @ (None
                | Some(Motion::Down | Motion::Right | Motion::Word | Motion::Last)) => {
                    return outcome
                }
            },
            SumProdIndex::Left => {
                match movement {
                    Motion::Up | Motion::Right => self.move_to(SumProdIndex::Top, Direction::Left),
                    Motion::Down => self.move_to(SumProdIndex::BottomIdent, Direction::Left),
                    _ => return Some(movement), // also basically outside the sumprod
                }
            }
            SumProdIndex::BottomEq => match movement {
                Motion::Down => return Some(Motion::Down),
                Motion::Up => self.move_to(SumProdIndex::Top, Direction::Down),

                Motion::First | Motion::Back => {
                    self.move_to(SumProdIndex::BottomIdent, Direction::Left)
                }
                Motion::Left => self.move_to(SumProdIndex::BottomIdent, Direction::Right),

                Motion::Word | Motion::Right => {
                    self.move_to(SumProdIndex::BottomExpr, Direction::Left)
                }
                Motion::Last => self.move_to(SumProdIndex::BottomExpr, Direction::Right),
            },
            SumProdIndex::BottomIdent => match self.ident.apply_move(movement) {
                None => {}
                Some(Motion::Up) => self.move_to(SumProdIndex::Top, Direction::Down),
                Some(Motion::Left) => self.move_to(SumProdIndex::Left, Direction::Right),
                Some(Motion::Right | Motion::Word) => {
                    self.move_to(SumProdIndex::BottomEq, Direction::Left)
                }
                Some(Motion::Last) => self.move_to(SumProdIndex::BottomExpr, Direction::Right),
                outcome @ Some(Motion::First | Motion::Back | Motion::Down) => return outcome,
            },
            SumProdIndex::Top => match self.top.apply_move(movement) {
                None => {}
                Some(Motion::Down) => self.move_to(SumProdIndex::BottomExpr, Direction::Up),
                outcome @ Some(
                    Motion::Up
                    | Motion::First
                    | Motion::Last
                    | Motion::Back
                    | Motion::Word
                    | Motion::Left
                    | Motion::Right,
                ) => return outcome,
            },
        }
        None
    }

    fn enter_from(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.move_to(SumProdIndex::Top, Direction::Up),
            Direction::Down => self.move_to(SumProdIndex::BottomExpr, Direction::Down),
            Direction::Left => self.move_to(SumProdIndex::Left, Direction::Left),
            Direction::Right => self.move_to(SumProdIndex::Top, Direction::Right),
        }
    }
}

impl TreeMovable for EditorTree {
    fn enter_from(&mut self, direction: Direction) {
        match &mut self.kind {
            EditorTreeKind::Terminal(term) => term.enter_from(direction),
            EditorTreeKind::Power(power) => power.enter_from(direction),
            EditorTreeKind::Fraction(fraction) => fraction.enter_from(direction),
            EditorTreeKind::Sqrt(sqrt) => sqrt.enter_from(direction),
            EditorTreeKind::Paren(paren) => paren.enter_from(direction),
            EditorTreeKind::SumProd(sum_prod) => sum_prod.enter_from(direction),
        }
    }

    fn apply_move(&mut self, movement: Motion) -> Option<Motion> {
        match &mut self.kind {
            EditorTreeKind::Terminal(term) => term.apply_move(movement),
            EditorTreeKind::Power(power) => power.apply_move(movement),
            EditorTreeKind::Fraction(fraction) => fraction.apply_move(movement),
            EditorTreeKind::Sqrt(sqrt) => sqrt.apply_move(movement),
            EditorTreeKind::Paren(paren) => paren.apply_move(movement),
            EditorTreeKind::SumProd(sum_prod) => sum_prod.apply_move(movement),
        }
    }
}
