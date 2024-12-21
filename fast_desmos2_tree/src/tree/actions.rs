use fast_desmos2_utils::ResExt;
use std::cmp::Ordering;

use crate::tree::{EditorTreeFraction, EditorTreeKind, FractionIndex};

use super::{movement::Direction, EditorTree, EditorTreeSeq, SurroundIndex, TreeMovable};

mod search_back;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeAction {
    Char(char),
    MakeFraction,
    MakePower,
    MakeParen,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftAction {
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotLeftAction {
    Char(char),
    MakeParen,
    MakeFraction,
    MakePower,
}

impl From<LeftAction> for TreeAction {
    fn from(value: LeftAction) -> Self {
        match value {
            LeftAction::Delete => Self::Delete,
        }
    }
}

impl TryFrom<TreeAction> for LeftAction {
    type Error = NotLeftAction;

    fn try_from(value: TreeAction) -> Result<Self, Self::Error> {
        match value {
            TreeAction::MakeFraction => Err(Self::Error::MakeFraction),
            TreeAction::MakePower => Err(Self::Error::MakePower),
            TreeAction::Delete => Ok(Self::Delete),
            TreeAction::MakeParen => Err(Self::Error::MakeParen),
            TreeAction::Char(ch) => Err(Self::Error::Char(ch)),
        }
    }
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
    LeftOverflow(LeftAction),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionOutcome {
    LeftOverflow(LeftAction),
    Delegated,
    Deleted,
    CaptureCursor,
    MoveRight,
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

macro_rules! left_overflows {
    ($name:ident) => {
        #[allow(non_upper_case_globals)]
        impl $name {
            pub const LeftDelete: Self = Self::LeftOverflow(LeftAction::Delete);
        }
    };
}
left_overflows!(ActionOutcome);
left_overflows!(SeqActionOutcome);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HereOrRight {
    Here(TreeAction),
    Right(LeftAction),
}

impl HereOrRight {
    fn to_tree_action(self) -> TreeAction {
        match self {
            HereOrRight::Here(action) => action,
            HereOrRight::Right(left_action) => left_action.into(),
        }
    }
}

impl EditorTreeSeq {
    pub fn apply_action(&mut self, action: TreeAction) -> Option<SeqActionOutcome> {
        if self.cursor < self.children.len() {
            self.apply_action_internal(self.cursor, HereOrRight::Here(action))
        } else {
            // the cursor is at the last element
            if let Some(last_index) = self.children.len().checked_sub(1) {
                // there is at least one element in this seq
                match LeftAction::try_from(action) {
                    Ok(left_action) => {
                        self.apply_action_internal(last_index, HereOrRight::Right(left_action))
                    }
                    Err(action) => {
                        match action {
                            NotLeftAction::Char(ch) => {
                                self.children.push(EditorTree::terminal(ch));
                                self.cursor += 1;
                            }
                            NotLeftAction::MakeParen => {
                                self.children.push(EditorTree::incomplete_paren(
                                    SurroundIndex::Inside,
                                    EditorTreeSeq::empty(),
                                ));
                            }
                            NotLeftAction::MakeFraction => {
                                if let Ok(start_index) = self.search_back(self.cursor) {
                                    let section: Vec<_> =
                                        self.children.drain(start_index..self.cursor).collect();
                                    let new_node = EditorTree::fraction(
                                        FractionIndex::Bottom,
                                        EditorTreeSeq::new(0, section),
                                        EditorTreeSeq::empty(),
                                    );
                                    self.cursor = start_index;
                                    self.children.insert(start_index, new_node);
                                }
                            }
                            NotLeftAction::MakePower => {
                                self.children.push(EditorTree::power(EditorTreeSeq::empty()));
                            }
                        }
                        None
                    }
                }
            } else {
                match action {
                    TreeAction::Char(ch) => {
                        self.children.push(EditorTree::terminal(ch));
                        self.cursor = 1;
                    }
                    TreeAction::MakeParen => self.children.push(EditorTree::incomplete_paren(
                        SurroundIndex::Inside,
                        EditorTreeSeq::empty(),
                    )),
                    TreeAction::MakeFraction | TreeAction::MakePower | TreeAction::Delete => {
                        return Some(SeqActionOutcome::LeftOverflow(
                            LeftAction::try_from(action).unwrap_unreach(),
                        ))
                    }
                }
                None
            }
        }
    }

    fn apply_action_internal(
        &mut self,
        index: usize,
        action: HereOrRight,
    ) -> Option<SeqActionOutcome> {
        let Some(child) = self.children.get_mut(index) else {
            unreachable!()
        };

        let outcome = match action {
            HereOrRight::Here(action) => child.apply_action(action),
            HereOrRight::Right(action) => child.apply_action_from_right(action),
        };

        match outcome {
            Some(ActionOutcome::LeftOverflow(left_action)) => match index.checked_sub(1) {
                Some(left) => {
                    return self.apply_action_internal(left, HereOrRight::Right(left_action))
                }
                None => return Some(SeqActionOutcome::LeftOverflow(left_action)),
            },
            Some(ActionOutcome::Deleted) => {
                self.children.remove(index);
                match index.cmp(&self.cursor) {
                    Ordering::Equal => self.move_to(self.cursor, Direction::Left),
                    Ordering::Less => self.cursor -= 1,
                    Ordering::Greater => {}
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
                    Ordering::Equal => match put_cursor_first {
                        true => self.move_to(self.cursor, Direction::Left),
                        false => self.move_right(first_len),
                    },
                    Ordering::Less => self.cursor += first_len + second_len - 1,
                    Ordering::Greater => {}
                }
            }
            Some(ActionOutcome::Splice { children }) => {
                let len = children.len();
                self.children.splice(index..=index, children);
                match index.cmp(&self.cursor) {
                    Ordering::Equal => self.children[index].enter_from(Direction::Left),
                    Ordering::Less => self.cursor += len - 1,
                    Ordering::Greater => {}
                }
            }
            Some(ActionOutcome::RightNode(node)) => {
                self.children.insert(index + 1, node);
                match index.cmp(&self.cursor) {
                    Ordering::Equal => self.move_right(1),
                    Ordering::Less => self.cursor += 1,
                    Ordering::Greater => {}
                }
            }
            Some(ActionOutcome::Delegated) => match action.to_tree_action() {
                TreeAction::Delete => unreachable!(),
                TreeAction::Char(ch) => {
                    let new_node = EditorTree::terminal(ch);
                    self.children.insert(index, new_node);
                    match index.cmp(&self.cursor) {
                        Ordering::Equal => self.move_right(1),
                        Ordering::Less => self.cursor += 1,
                        Ordering::Greater => {}
                    }
                }
                TreeAction::MakeFraction => {
                    if let Ok(start_index) = self.search_back(self.cursor) {
                        let section: Vec<_> =
                            self.children.drain(start_index..self.cursor).collect();
                        let new_node = EditorTree::fraction(
                            FractionIndex::Bottom,
                            EditorTreeSeq::new(0, section),
                            EditorTreeSeq::empty(),
                        );
                        self.cursor = start_index;
                        self.children.insert(start_index, new_node);
                    }
                }
                TreeAction::MakePower => {
                    let mut new_node = EditorTree::power(EditorTreeSeq::empty());
                    new_node.enter_from(Direction::Left);
                    self.children.insert(index, new_node);
                }
                TreeAction::MakeParen => {
                    let useful = self.children.drain(index..).collect::<Vec<_>>();
                    let new_child = EditorTree::incomplete_paren(
                        SurroundIndex::Inside,
                        EditorTreeSeq::new(0, useful),
                    );
                    self.children.push(new_child);
                }
            },
            Some(ActionOutcome::CaptureCursor) => self.cursor = index,
            Some(ActionOutcome::MoveRight) => self.move_right(1),
            None => {}
        }
        None
    }

    pub fn apply_action_from_right(&mut self, action: LeftAction) -> Option<SeqActionOutcome> {
        if let Some(last_index) = self.children.len().checked_sub(1) {
            self.apply_action_internal(last_index, HereOrRight::Right(action))
        } else {
            Some(SeqActionOutcome::LeftOverflow(action))
        }
    }
}

impl EditorTree {
    pub fn apply_action(&mut self, action: TreeAction) -> Option<ActionOutcome> {
        match &mut self.kind {
            EditorTreeKind::Terminal(_) => match action {
                TreeAction::Delete => Some(ActionOutcome::LeftOverflow(LeftAction::Delete)),
                TreeAction::Char(_)
                | TreeAction::MakePower
                | TreeAction::MakeFraction
                | TreeAction::MakeParen => Some(ActionOutcome::Delegated),
            },
            EditorTreeKind::Fraction(fraction) => match fraction.cursor() {
                FractionIndex::Top => {
                    let outcome = fraction.top.apply_action(action);
                    match outcome {
                        None => None,
                        Some(SeqActionOutcome::LeftOverflow(left_action)) => match left_action {
                            LeftAction::Delete => {
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
                        },
                    }
                }
                FractionIndex::Bottom => {
                    let outcome = fraction.bottom.apply_action(action);
                    match outcome {
                        None => None,
                        Some(SeqActionOutcome::LeftOverflow(left_action)) => match left_action {
                            LeftAction::Delete => {
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
                        },
                    }
                }
                FractionIndex::Left => match LeftAction::try_from(action) {
                    Ok(left_action) => Some(ActionOutcome::LeftOverflow(left_action)),
                    Err(action) => match action {
                        NotLeftAction::Char(_)
                        | NotLeftAction::MakeParen
                        | NotLeftAction::MakePower
                        | NotLeftAction::MakeFraction => Some(ActionOutcome::Delegated),
                    },
                },
            },
            EditorTreeKind::Power(power) => {
                let outcome = power.power.apply_action(action);
                match outcome {
                    Some(SeqActionOutcome::LeftOverflow(overflow)) => match overflow {
                        LeftAction::Delete => {
                            let old_self = std::mem::replace(self, EditorTree::terminal('X'));
                            let EditorTreeKind::Power(power) = old_self.kind else {
                                unreachable!()
                            };
                            let children = power.power.children;
                            Some(ActionOutcome::Splice { children })
                        }
                    },
                    None => todo!(),
                }
            }
            EditorTreeKind::Sqrt(_) => todo!(),
            EditorTreeKind::SumProd(_) => todo!(),
            EditorTreeKind::Paren(paren) => match paren.cursor {
                SurroundIndex::Left => match LeftAction::try_from(action) {
                    Ok(left_action) => Some(ActionOutcome::LeftOverflow(left_action)),
                    Err(
                        NotLeftAction::Char(_)
                        | NotLeftAction::MakeFraction
                        | NotLeftAction::MakePower
                        | NotLeftAction::MakeParen,
                    ) => Some(ActionOutcome::Delegated),
                },
                SurroundIndex::Inside => {
                    // bracket completion
                    if (action, paren.child.cursor()) == (TreeAction::Char(')'), paren.child.len()) {
                        paren.is_complete = true;
                        Some(ActionOutcome::MoveRight)
                    } else {
                        let outcome = paren.child.apply_action(action);
                        match outcome {
                            Some(SeqActionOutcome::LeftOverflow(left_action)) => match left_action {
                                LeftAction::Delete => {
                                    let old_self = std::mem::replace(self, EditorTree::terminal('X'));
                                    let EditorTreeKind::Paren(paren) = old_self.kind else {
                                        unreachable!()
                                    };
                                    let children = paren.child.children;
                                    Some(ActionOutcome::Splice { children })
                                }
                            },
                            None => None,
                        }
                    }
                }
            },
        }
    }

    pub fn apply_action_from_right(&mut self, action: LeftAction) -> Option<ActionOutcome> {
        match &mut self.kind {
            EditorTreeKind::Terminal(_) => match action {
                LeftAction::Delete => Some(ActionOutcome::Deleted),
            },
            EditorTreeKind::Fraction(fraction) => match action {
                LeftAction::Delete => {
                    fraction.enter_from(Direction::Right);
                    Some(ActionOutcome::CaptureCursor)
                }
            },
            EditorTreeKind::Power(_) => todo!(),
            EditorTreeKind::Sqrt(_) => todo!(),
            EditorTreeKind::Paren(paren) => match action {
                LeftAction::Delete => {
                    paren.is_complete = false;
                    Some(ActionOutcome::CaptureCursor)
                }
            },
            EditorTreeKind::SumProd(_) => todo!(),
        }
    }
}
