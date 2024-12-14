use std::cmp::Ordering;

use crate::tree::{EditorTreeFraction, EditorTreeKind, FractionIndex};

use super::{EditorTree, EditorTreeSeq, PowerIndex, TreeMovable, TreeMove};

#[derive(Debug, Clone, Copy)]
pub enum TreeAction {
    Char(char),
    MakeFraction,
    MakePower,
    Delete,
}

impl TreeAction {
    pub const fn from_char(char: char) -> Self {
        match char {
            '/' => Self::MakeFraction,
            '^' => Self::MakePower,
            otherwise => Self::Char(otherwise),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeqActionOutcome {
    LeftOverflow,
    Deleted,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionOutcome {
    LeftOverflow,
    Deleted,
    LeftNode(EditorTree),
    Replaced2 {
        first: Vec<EditorTree>,
        second: Vec<EditorTree>,
        put_cursor_first: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HereOrRight {
    Here,
    Right,
}

impl EditorTreeSeq {
    pub fn apply_action(&mut self, action: TreeAction) -> Option<SeqActionOutcome> {
        if self.cursor < self.children.len() {
            self.apply_action_internal(self.cursor, action, HereOrRight::Here)
        } else {
            if let Some(last_index) = self.children.len().checked_sub(1) {
                self.apply_action_internal(last_index, action, HereOrRight::Right)
            } else {
                Some(SeqActionOutcome::LeftOverflow)
            }
        }
    }

    fn apply_action_internal(
        &mut self,
        index: usize,
        action: TreeAction,
        operation: HereOrRight,
    ) -> Option<SeqActionOutcome> {
        let Some(child) = self.children.get_mut(index) else {
            unreachable!()
        };

        let outcome = match operation {
            HereOrRight::Here => child.apply_action(action),
            HereOrRight::Right => child.apply_action_from_right(action),
        };

        match outcome {
            Some(ActionOutcome::LeftOverflow) => match index.checked_sub(1) {
                Some(left) => return self.apply_action_internal(left, action, HereOrRight::Right),
                None => return Some(SeqActionOutcome::LeftOverflow),
            },
            Some(ActionOutcome::Deleted) => {
                self.children.remove(index);
                match index.cmp(&self.cursor) {
                    Ordering::Equal => {
                        if let Some(child) = self.children.get_mut(index) {
                            child.enter_from(TreeMove::Left);
                        }
                    }
                    Ordering::Less => self.cursor -= 1,
                    Ordering::Greater => {}
                }
            }
            Some(ActionOutcome::Replaced2 {
                first,
                second,
                put_cursor_first,
            }) => {
                let first_len = first.len();
                self.children.splice(index..=index, second);
                self.children.splice(index..index, first);
                if index == self.cursor() {
                    if put_cursor_first {
                        self.children[index].enter_from(TreeMove::Left);
                    } else {
                        self.cursor += first_len;
                        self.children[self.cursor].enter_from(TreeMove::Left);
                    }
                }
            }
            Some(ActionOutcome::LeftNode(node)) => {
                self.children.insert(index, node);
                if index == self.cursor() {
                    self.cursor += 1;
                    self.children[self.cursor].enter_from(TreeMove::Left);
                }
            }
            None => {}
        }

        None
    }

    pub fn apply_action_from_right(&mut self, action: TreeAction) -> Option<SeqActionOutcome> {
        if let Some(last_index) = self.children.len().checked_sub(1) {
            self.apply_action_internal(last_index, action, HereOrRight::Right)
        } else {
            Some(SeqActionOutcome::LeftOverflow)
        }
    }
}

impl EditorTree {
    pub fn apply_action(&mut self, action: TreeAction) -> Option<ActionOutcome> {
        match &mut self.kind {
            EditorTreeKind::Terminal(term) => match action {
                TreeAction::Char(c) => {
                    term.insert_char(c);
                    None
                }
                TreeAction::Delete => {
                    let success = term.backspace_char();
                    match success {
                        true => term.is_empty().then_some(ActionOutcome::Deleted),
                        false => Some(ActionOutcome::LeftOverflow),
                    }
                }
                TreeAction::MakePower => todo!(),
                TreeAction::MakeFraction => todo!(),
            },
            EditorTreeKind::Fraction(fraction) => match fraction.cursor() {
                FractionIndex::Top => {
                    let outcome = fraction.top.apply_action(action);
                    match outcome {
                        None => None,
                        Some(SeqActionOutcome::LeftOverflow) => match action {
                            TreeAction::Delete => {
                                let old_self = std::mem::replace(
                                    self,
                                    EditorTree::str(
                                        "[ERR]: Replaced2 wasn't implemented correctly.",
                                    ),
                                );
                                let EditorTreeKind::Fraction(EditorTreeFraction {
                                    cursor: _,
                                    top,
                                    bottom,
                                }) = old_self.kind
                                else {
                                    unreachable!()
                                };
                                Some(ActionOutcome::Replaced2 {
                                    first: top.children,
                                    second: bottom.children,
                                    put_cursor_first: true,
                                })
                            }
                            _ => Some(ActionOutcome::LeftOverflow),
                        },
                        Some(SeqActionOutcome::Deleted) => todo!("Replace with placeholder"),
                    }
                }
                FractionIndex::Bottom => {
                    let outcome = fraction.bottom.apply_action(action);
                    match outcome {
                        None => None,
                        Some(SeqActionOutcome::LeftOverflow) => match action {
                            TreeAction::Delete => {
                                let old_self = std::mem::replace(
                                    self,
                                    EditorTree::str(
                                        "[ERR]: Replaced2 wasn't implemented correctly.",
                                    ),
                                );
                                let EditorTreeKind::Fraction(EditorTreeFraction {
                                    cursor: _,
                                    top,
                                    bottom,
                                }) = old_self.kind
                                else {
                                    unreachable!()
                                };
                                Some(ActionOutcome::Replaced2 {
                                    first: top.children,
                                    second: bottom.children,
                                    put_cursor_first: false,
                                })
                            }
                            _ => Some(ActionOutcome::LeftOverflow),
                        },
                        Some(SeqActionOutcome::Deleted) => todo!("Replace with placeholder"),
                    }
                }
                FractionIndex::Left => Some(ActionOutcome::LeftOverflow),
            },
            EditorTreeKind::Power(power) => match power.cursor() {
                PowerIndex::Base => {
                    let outcome = power.base.apply_action(action);
                    match outcome {
                        None => None,
                        Some(SeqActionOutcome::LeftOverflow) => match action {
                            TreeAction::Char(_) => unreachable!(),
                            TreeAction::MakeFraction => todo!(),
                            TreeAction::MakePower => todo!(),
                            TreeAction::Delete => todo!(),
                        },
                        Some(SeqActionOutcome::Deleted) => todo!("Remove and flatten"),
                    }
                }
                PowerIndex::Power => {
                    let outcome = power.power.apply_action(action);
                    match outcome {
                        None => None,
                        Some(SeqActionOutcome::LeftOverflow) => {
                            todo!("???")
                        }
                        Some(SeqActionOutcome::Deleted) => todo!("Remove and flatten"),
                    }
                }
            },
        }
    }

    pub fn apply_action_from_right(&mut self, action: TreeAction) -> Option<ActionOutcome> {
        match &mut self.kind {
            EditorTreeKind::Terminal(term) => match action {
                TreeAction::Delete => match term.pop() {
                    Some(_) => return (term.is_empty()).then_some(ActionOutcome::Deleted),
                    None => return Some(ActionOutcome::LeftOverflow),
                },
                TreeAction::Char(c) => term.push(c),
                _ => todo!(),
            },
            _ => todo!(),
        }
        None
    }
}
