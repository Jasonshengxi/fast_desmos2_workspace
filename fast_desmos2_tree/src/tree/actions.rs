use std::cmp::Ordering;

use crate::tree::{EditorTreeFraction, EditorTreeKind, FractionIndex};

use super::{EditorTree, EditorTreeSeq, PowerIndex, SurroundIndex, TreeMovable, TreeMove};

#[derive(Debug, Clone, Copy)]
pub enum TreeAction {
    Char(char),
    MakeFraction,
    MakePower,
    MakeParen,
    Delete,
}

impl TreeAction {
    pub const fn from_char(char: char) -> Self {
        match char {
            '/' => Self::MakeFraction,
            '^' => Self::MakePower,
            '(' => Self::MakeParen,
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
    CaptureCursor,
    Delegated,
    LeftNode(EditorTree),
    RightNode(EditorTree),
    Splice2 {
        first: Vec<EditorTree>,
        second: Vec<EditorTree>,
        put_cursor_first: bool,
    },
    Splice {
        children: Vec<EditorTree>,
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
                match action {
                    TreeAction::Char(ch) => {
                        self.children.push(EditorTree::terminal(ch));
                        self.cursor = 1;
                        None
                    }
                    TreeAction::MakeFraction | TreeAction::MakePower | TreeAction::Delete => {
                        Some(SeqActionOutcome::LeftOverflow)
                    }
                    TreeAction::MakeParen => todo!(),
                }
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

                if self.children.is_empty() {
                    return Some(SeqActionOutcome::Deleted);
                }
            }
            Some(ActionOutcome::Splice2 {
                first,
                second,
                put_cursor_first,
            }) => {
                let first_len = first.len();
                let second_len = second.len();
                self.children.splice(index..=index, second);
                self.children.splice(index..index, first);
                match index.cmp(&self.cursor) {
                    Ordering::Equal => {
                        if put_cursor_first {
                            self.children[index].enter_from(TreeMove::Left);
                        } else {
                            self.cursor += first_len;
                            self.children[self.cursor].enter_from(TreeMove::Left);
                        }
                    }
                    Ordering::Less => self.cursor += first_len + second_len - 1,
                    Ordering::Greater => {}
                }
            }
            Some(ActionOutcome::Splice { children }) => {
                let len = children.len();
                self.children.splice(index..=index, children);
                match index.cmp(&self.cursor) {
                    Ordering::Equal => self.children[index].enter_from(TreeMove::Left),
                    Ordering::Less => self.cursor += len - 1,
                    Ordering::Greater => {}
                }
            }
            Some(ActionOutcome::LeftNode(node)) => {
                self.children.insert(index, node);
                match index.cmp(&self.cursor) {
                    Ordering::Equal => {
                        self.cursor += 1;
                        self.children[self.cursor].enter_from(TreeMove::Left);
                    }
                    Ordering::Less => self.cursor += 1,
                    Ordering::Greater => {}
                }
            }
            Some(ActionOutcome::RightNode(node)) => {
                self.children.insert(index + 1, node);
                match index.cmp(&self.cursor) {
                    Ordering::Equal => {
                        self.cursor += 1;
                        self.children[self.cursor].enter_from(TreeMove::Left);
                    }
                    Ordering::Less => self.cursor += 1,
                    Ordering::Greater => {}
                }
            }
            Some(ActionOutcome::Delegated) => match action {
                TreeAction::Char(_) | TreeAction::Delete => unreachable!(),
                TreeAction::MakeFraction => todo!(),
                TreeAction::MakePower => todo!(),
                TreeAction::MakeParen => {
                    let useful = self.children.drain(index..).collect::<Vec<_>>();
                    let new_child =
                        EditorTree::paren(SurroundIndex::Inside, EditorTreeSeq::new(0, useful));
                    self.children.push(new_child);
                }
            },
            Some(ActionOutcome::CaptureCursor) => self.cursor = index,
            None => {}
        }
        None
    }

    pub fn search_back(&self, index: usize) -> usize {
        let first = &self.children[index];
        match first.kind {
            EditorTreeKind::Fraction(_) => {}
            EditorTreeKind::Terminal(_) => {}
            EditorTreeKind::Power(_) => {}
            EditorTreeKind::Paren(_) => {}
            EditorTreeKind::Sqrt(_) => {}
        }
        todo!()
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
            EditorTreeKind::Terminal(_) => match action {
                TreeAction::Char(c) => Some(ActionOutcome::LeftNode(EditorTree::terminal(c))),
                TreeAction::Delete => Some(ActionOutcome::LeftOverflow),
                TreeAction::MakePower | TreeAction::MakeFraction | TreeAction::MakeParen => {
                    Some(ActionOutcome::Delegated)
                }
            },
            EditorTreeKind::Fraction(fraction) => match fraction.cursor() {
                FractionIndex::Top => {
                    let outcome = fraction.top.apply_action(action);
                    match outcome {
                        None => None,
                        Some(SeqActionOutcome::LeftOverflow) => match action {
                            TreeAction::Delete => {
                                let old_self = std::mem::replace(self, EditorTree::terminal('X'));
                                let EditorTreeKind::Fraction(EditorTreeFraction {
                                    cursor: _,
                                    top,
                                    bottom,
                                }) = old_self.kind
                                else {
                                    unreachable!()
                                };
                                Some(ActionOutcome::Splice2 {
                                    first: top.children,
                                    second: bottom.children,
                                    put_cursor_first: true,
                                })
                            }
                            _ => Some(ActionOutcome::LeftOverflow),
                        },
                        Some(SeqActionOutcome::Deleted) => None, // Keep node alive
                    }
                }
                FractionIndex::Bottom => {
                    let outcome = fraction.bottom.apply_action(action);
                    match outcome {
                        None => None,
                        Some(SeqActionOutcome::LeftOverflow) => match action {
                            TreeAction::Delete => {
                                let old_self = std::mem::replace(self, EditorTree::terminal('X'));
                                let EditorTreeKind::Fraction(EditorTreeFraction {
                                    cursor: _,
                                    top,
                                    bottom,
                                }) = old_self.kind
                                else {
                                    unreachable!()
                                };
                                Some(ActionOutcome::Splice2 {
                                    first: top.children,
                                    second: bottom.children,
                                    put_cursor_first: false,
                                })
                            }
                            _ => Some(ActionOutcome::LeftOverflow),
                        },
                        Some(SeqActionOutcome::Deleted) => None, // keep node alive
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
                            TreeAction::MakeParen => todo!(),
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
            EditorTreeKind::Sqrt(_) => todo!(),
            EditorTreeKind::Paren(paren) => match paren.cursor {
                SurroundIndex::Left => todo!(),
                SurroundIndex::Inside => {
                    let outcome = paren.child.apply_action(action);
                    match outcome {
                        Some(SeqActionOutcome::LeftOverflow) => match action {
                            TreeAction::Char(_) => unreachable!("LeftOverflow on Char"),
                            TreeAction::MakeFraction => None,
                            TreeAction::MakePower => None,
                            TreeAction::MakeParen => unreachable!("LeftOverflow on Paren"),
                            TreeAction::Delete => {
                                let old_self = std::mem::replace(self, EditorTree::terminal('X'));
                                let EditorTreeKind::Paren(paren) = old_self.kind else {
                                    unreachable!()
                                };
                                let children = paren.child.children;
                                Some(ActionOutcome::Splice { children })
                            }
                        },
                        Some(SeqActionOutcome::Deleted) => None,
                        None => None,
                    }
                }
            },
        }
    }

    pub fn apply_action_from_right(&mut self, action: TreeAction) -> Option<ActionOutcome> {
        match &mut self.kind {
            EditorTreeKind::Terminal(_) => match action {
                TreeAction::Delete => Some(ActionOutcome::Deleted),
                TreeAction::Char(ch) => Some(ActionOutcome::RightNode(EditorTree::terminal(ch))),
                TreeAction::MakeFraction => Some(ActionOutcome::Delegated),
                TreeAction::MakePower => Some(ActionOutcome::Delegated),
                TreeAction::MakeParen => Some(ActionOutcome::Delegated),
            },
            EditorTreeKind::Fraction(fraction) => match action {
                TreeAction::Char(ch) => Some(ActionOutcome::RightNode(EditorTree::terminal(ch))),
                TreeAction::Delete => {
                    fraction.enter_from(TreeMove::Right);
                    Some(ActionOutcome::CaptureCursor)
                }
                TreeAction::MakeFraction => todo!(),
                TreeAction::MakePower => todo!(),
                TreeAction::MakeParen => todo!(),
            },
            EditorTreeKind::Power(_) => todo!(),
            EditorTreeKind::Sqrt(_) => todo!(),
            EditorTreeKind::Paren(_) => todo!(),
        }
    }
}
