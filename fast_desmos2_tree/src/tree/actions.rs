use std::cmp::Ordering;

use crate::tree::{
    CompletableSurrounds, EditorTreeKind, EditorTreeSeq, FractionIndex, SurroundsTreeSeq,
};

use super::{
    movement::Direction, EditorTree, EditorTreeSeqNormal, SumProdIndex, SurroundIndex, TreeMovable,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeAction {
    Char(char),
    MakeFraction,
    MakePower,
    MakeGroup(GroupKind),
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKind {
    Paren,
    Bracket,
    Abs,
    Curly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftAction {
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotLeftAction {
    Char(char),
    MakeGroup(GroupKind),
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
            TreeAction::MakeGroup(group) => Err(Self::Error::MakeGroup(group)),
            TreeAction::Char(ch) => Err(Self::Error::Char(ch)),
        }
    }
}

impl TreeAction {
    pub const fn from_char(char: char) -> Self {
        match char {
            '/' => Self::MakeFraction,
            '^' => Self::MakePower,
            '(' => Self::MakeGroup(GroupKind::Paren),
            '|' => Self::MakeGroup(GroupKind::Abs),
            '[' => Self::MakeGroup(GroupKind::Bracket),
            '{' => Self::MakeGroup(GroupKind::Curly),
            '*' => Self::Char('×'),
            otherwise => Self::Char(otherwise),
        }
    }
}

impl GroupKind {
    pub fn incomplete<S: EditorTreeSeq>(&self, cursor: SurroundIndex, child: S) -> EditorTree<S> {
        match self {
            GroupKind::Paren => EditorTree::incomplete_paren(cursor, child),
            GroupKind::Bracket => EditorTree::incomplete_brackets(cursor, child),
            GroupKind::Abs => EditorTree::incomplete_abs(cursor, child),
            GroupKind::Curly => EditorTree::incomplete_curly(cursor, child),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqActionOutcome {
    LeftOverflow(LeftAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcome {
    LeftOverflow(LeftAction),
    Delegated,
    Deleted,
    CaptureCursor,
    MoveRight,
    ToFraction(SplicedCursor),
    ExtractElements(GroupKind, usize),
    Unwrap(Unwrap),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplicedCursor {
    Left,
    Middle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unwrap {
    UnwrapPower,
    UnwrapSqrt,
    UnwrapGroup(GroupKind),
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

impl EditorTreeSeqNormal {
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
                                let contracted = self.check_and_contract(self.cursor);
                                if !contracted {
                                    self.cursor += 1;
                                }
                            }
                            NotLeftAction::MakeGroup(group) => {
                                self.children.push(group.incomplete(
                                    SurroundIndex::Inside,
                                    EditorTreeSeqNormal::empty(),
                                ))
                            }
                            NotLeftAction::MakeFraction => {
                                if let Ok(start_index) = self.search_back(self.cursor) {
                                    let section: Vec<_> =
                                        self.children.drain(start_index..self.cursor).collect();
                                    let new_node = EditorTree::fraction(
                                        FractionIndex::Bottom,
                                        EditorTreeSeqNormal::new(0, section),
                                        EditorTreeSeqNormal::empty(),
                                    );
                                    self.cursor = start_index;
                                    self.children.insert(start_index, new_node);
                                }
                            }
                            NotLeftAction::MakePower => {
                                self.children.push(EditorTree::power(
                                    SurroundIndex::Inside,
                                    EditorTreeSeqNormal::empty(),
                                ));
                            }
                        }
                        None
                    }
                }
            } else {
                match LeftAction::try_from(action) {
                    Ok(left_action) => return Some(SeqActionOutcome::LeftOverflow(left_action)),
                    Err(NotLeftAction::Char(ch)) => {
                        self.children.push(EditorTree::terminal(ch));
                        self.cursor = 1;
                    }
                    Err(NotLeftAction::MakeGroup(group)) => self.children.push(
                        group.incomplete(SurroundIndex::Inside, EditorTreeSeqNormal::empty()),
                    ),
                    Err(NotLeftAction::MakeFraction) => self.children.push(EditorTree::fraction(
                        FractionIndex::Top,
                        EditorTreeSeqNormal::empty(),
                        EditorTreeSeqNormal::empty(),
                    )),
                    Err(NotLeftAction::MakePower) => self.children.push(EditorTree::power(
                        SurroundIndex::Inside,
                        EditorTreeSeqNormal::empty(),
                    )),
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

        macro_rules! splice_extract {
            ($etk:ident :: $var:ident) => {{
                let this_node = self.children.remove(index);
                let $etk::$var(x) = this_node.kind else {
                    unreachable!()
                };
                x
            }};
        }
        macro_rules! splice_mut {
            ($etk:ident :: $var:ident) => {{
                let $etk::$var(x) = &mut self.children[index].kind else {
                    unreachable!()
                };
                x
            }};
        }

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
            Some(ActionOutcome::Delegated) => match action.to_tree_action() {
                TreeAction::Delete => unreachable!(),
                TreeAction::Char(ch) => {
                    let new_node = EditorTree::terminal(ch);
                    self.children.insert(index, new_node);

                    let contracted = self.check_and_contract(index);
                    if !contracted {
                        match index.cmp(&self.cursor) {
                            Ordering::Equal => self.move_right(1),
                            Ordering::Less => self.cursor += 1,
                            Ordering::Greater => {}
                        }
                    }
                }
                TreeAction::MakeFraction => {
                    if let Ok(start_index) = self.search_back(self.cursor) {
                        let section: Vec<_> =
                            self.children.drain(start_index..self.cursor).collect();
                        let new_node = EditorTree::fraction(
                            FractionIndex::Bottom,
                            EditorTreeSeqNormal::new(0, section),
                            EditorTreeSeqNormal::empty(),
                        );
                        self.cursor = start_index;
                        self.children.insert(start_index, new_node);
                    }
                }
                TreeAction::MakePower => {
                    let mut new_node =
                        EditorTree::power(SurroundIndex::Inside, EditorTreeSeqNormal::empty());
                    new_node.enter_from(Direction::Left);
                    self.children.insert(index, new_node);
                }
                TreeAction::MakeGroup(group) => {
                    let useful = self.children.drain(index..).collect::<Vec<_>>();
                    let new_child = group
                        .incomplete(SurroundIndex::Inside, EditorTreeSeqNormal::new(0, useful));
                    self.children.push(new_child);
                }
            },
            Some(ActionOutcome::CaptureCursor) => self.cursor = index,
            Some(ActionOutcome::MoveRight) => self.move_right(1),
            Some(ActionOutcome::ToFraction(cursor)) => {
                let fraction = splice_extract!(EditorTreeKind::Fraction);
                let [top, bottom] = [fraction.top, fraction.bottom].map(|x| x.children);
                let new_cursor = match cursor {
                    SplicedCursor::Left => index,
                    SplicedCursor::Middle => index + top.len(),
                };
                let mut nodes = top;
                nodes.extend(bottom);
                self.children.splice(index..index, nodes);
                if index == self.cursor {
                    self.cursor = new_cursor;
                }
            }
            Some(ActionOutcome::Unwrap(splice)) => {
                let nodes = match splice {
                    Unwrap::UnwrapPower => splice_extract!(EditorTreeKind::Power).child,
                    Unwrap::UnwrapGroup(GroupKind::Paren) => {
                        splice_extract!(EditorTreeKind::Paren).child
                    }
                    Unwrap::UnwrapGroup(GroupKind::Abs) => {
                        splice_extract!(EditorTreeKind::Abs).child
                    }
                    Unwrap::UnwrapGroup(GroupKind::Curly) => {
                        splice_extract!(EditorTreeKind::Curly).child
                    }
                    Unwrap::UnwrapGroup(GroupKind::Bracket) => {
                        splice_extract!(EditorTreeKind::Bracket).child
                    }
                    Unwrap::UnwrapSqrt => splice_extract!(EditorTreeKind::Sqrt).child,
                }
                .children;
                self.children.splice(index..index, nodes);
            }
            Some(ActionOutcome::ExtractElements(group, count)) => {
                fn extract<T, S>(tree: &mut T, count: usize) -> Vec<EditorTree<S>>
                where
                    T: CompletableSurrounds + SurroundsTreeSeq<Seq = S>,
                    S: EditorTreeSeq,
                {
                    let vec = tree.child_mut().children_mut();
                    let result = vec
                        .drain((vec.len() - count)..vec.len())
                        .collect::<Vec<_>>();
                    assert_eq!(result.len(), count);
                    result
                }

                let extracted = match group {
                    GroupKind::Paren => extract(splice_mut!(EditorTreeKind::Paren), count),
                    GroupKind::Bracket => extract(splice_mut!(EditorTreeKind::Bracket), count),
                    GroupKind::Abs => extract(splice_mut!(EditorTreeKind::Abs), count),
                    GroupKind::Curly => extract(splice_mut!(EditorTreeKind::Curly), count),
                };
                self.children.splice((index + 1)..(index + 1), extracted);
                if index == self.cursor {
                    self.move_right(1);
                }
            }
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

    pub fn check_and_contract(&mut self, index: usize) -> bool {
        fn at_index<S: EditorTreeSeq>(
            children: &[EditorTree<S>],
            index: usize,
            string: &str,
        ) -> bool {
            string.chars().rev().enumerate().all(|(offset, ch)| {
                children
                    .get(index - offset)
                    .is_some_and(|tree| tree.is_terminal_and_eq(ch))
            })
        }

        if at_index(self.children(), index, "sqrt") {
            const OFFSET: usize = "sqrt".len() - 1;
            let min_index = index - OFFSET;
            self.children.splice(
                min_index..=index,
                std::iter::once(EditorTree::sqrt(
                    SurroundIndex::Inside,
                    EditorTreeSeqNormal::empty(),
                )),
            );

            match index.cmp(&self.cursor) {
                Ordering::Equal => self.cursor = min_index,
                Ordering::Less => self.cursor -= OFFSET,
                Ordering::Greater => {}
            }
        } else if at_index(self.children(), index, "sum") {
            const OFFSET: usize = "sum".len() - 1;
            let min_index = index - OFFSET;
            self.children.splice(
                min_index..=index,
                std::iter::once(EditorTree::sum(
                    SumProdIndex::Top,
                    EditorTreeSeqNormal::str("10"),
                    EditorTreeSeqNormal::str("1"),
                    EditorTreeSeqNormal::str("n"),
                )),
            );

            match index.cmp(&self.cursor) {
                Ordering::Equal => self.move_to(min_index + 1, Direction::Left),
                Ordering::Less => self.cursor -= OFFSET,
                Ordering::Greater => {}
            }
        } else if at_index(self.children(), index, "prod") {
            const OFFSET: usize = "prod".len() - 1;
            let min_index = index - OFFSET;
            self.children.splice(
                min_index..=index,
                std::iter::once(EditorTree::prod(
                    SumProdIndex::Top,
                    EditorTreeSeqNormal::str("10"),
                    EditorTreeSeqNormal::str("1"),
                    EditorTreeSeqNormal::str("n"),
                )),
            );

            match index.cmp(&self.cursor) {
                Ordering::Equal => self.move_to(min_index + 1, Direction::Left),
                Ordering::Less => self.cursor -= OFFSET,
                Ordering::Greater => {}
            }
        } else {
            return false;
        }

        true
    }
}

impl EditorTree<EditorTreeSeqNormal> {
    pub fn apply_action(&mut self, action: TreeAction) -> Option<ActionOutcome> {
        fn apply_completable_surrounds<T>(
            this: &mut T,
            action: TreeAction,
            closure: char,
            group: GroupKind,
        ) -> Option<ActionOutcome>
        where
            T: CompletableSurrounds + SurroundsTreeSeq<Seq = EditorTreeSeqNormal>,
        {
            match this.cursor() {
                SurroundIndex::Left => match LeftAction::try_from(action) {
                    Ok(left_action) => Some(ActionOutcome::LeftOverflow(left_action)),
                    Err(_) => Some(ActionOutcome::Delegated),
                },
                SurroundIndex::Inside => {
                    // bracket completion
                    if action == TreeAction::Char(closure)
                        || (closure == '|' && action == TreeAction::MakeGroup(GroupKind::Abs))
                    {
                        let cursor = this.child().cursor();
                        let count = this.child().len() - cursor;
                        *this.is_complete_mut() = true;
                        Some(ActionOutcome::ExtractElements(group, count))
                    } else {
                        let outcome = this.child_mut().apply_action(action);
                        match outcome {
                            Some(SeqActionOutcome::LeftOverflow(left_action)) => {
                                match left_action {
                                    LeftAction::Delete => {
                                        Some(ActionOutcome::Unwrap(Unwrap::UnwrapGroup(group)))
                                    }
                                }
                            }
                            None => None,
                        }
                    }
                }
            }
        }

        match &mut self.kind {
            EditorTreeKind::Terminal(_) => match action {
                TreeAction::Delete => Some(ActionOutcome::LeftOverflow(LeftAction::Delete)),
                TreeAction::Char(_)
                | TreeAction::MakePower
                | TreeAction::MakeFraction
                | TreeAction::MakeGroup(_) => Some(ActionOutcome::Delegated),
            },
            EditorTreeKind::Fraction(fraction) => match fraction.cursor() {
                FractionIndex::Top => {
                    let outcome = fraction.top.apply_action(action);
                    match outcome? {
                        SeqActionOutcome::LeftDelete => {
                            Some(ActionOutcome::ToFraction(SplicedCursor::Left))
                        }
                    }
                }
                FractionIndex::Bottom => {
                    let outcome = fraction.bottom.apply_action(action);
                    match outcome? {
                        SeqActionOutcome::LeftDelete => {
                            Some(ActionOutcome::ToFraction(SplicedCursor::Middle))
                        }
                    }
                }
                FractionIndex::Left => match LeftAction::try_from(action) {
                    Ok(left_action) => Some(ActionOutcome::LeftOverflow(left_action)),
                    Err(action) => match action {
                        NotLeftAction::Char(_)
                        | NotLeftAction::MakePower
                        | NotLeftAction::MakeGroup(_)
                        | NotLeftAction::MakeFraction => Some(ActionOutcome::Delegated),
                    },
                },
            },
            EditorTreeKind::Power(power) => {
                let outcome = power.child.apply_action(action);
                match outcome? {
                    SeqActionOutcome::LeftDelete => {
                        Some(ActionOutcome::Unwrap(Unwrap::UnwrapPower))
                    }
                }
            }
            EditorTreeKind::SumProd(sum_prod) => {
                match sum_prod.cursor {
                    SumProdIndex::BottomExpr => match sum_prod.bottom.apply_action(action)? {
                        SeqActionOutcome::LeftDelete => {
                            sum_prod.move_to(SumProdIndex::BottomIdent, Direction::Right)
                        }
                    },
                    SumProdIndex::BottomIdent => match sum_prod.bottom.apply_action(action)? {
                        SeqActionOutcome::LeftDelete => return Some(ActionOutcome::Deleted),
                    },
                    SumProdIndex::Top => match sum_prod.top.apply_action(action)? {
                        SeqActionOutcome::LeftDelete => return Some(ActionOutcome::Deleted),
                    },
                    SumProdIndex::Left => {
                        return match LeftAction::try_from(action) {
                            Ok(left_action) => Some(ActionOutcome::LeftOverflow(left_action)),
                            Err(_) => Some(ActionOutcome::Delegated),
                        }
                    }
                };
                None
            }
            EditorTreeKind::Sqrt(sqrt) => match sqrt.cursor {
                SurroundIndex::Left => match LeftAction::try_from(action) {
                    Ok(left_action) => Some(ActionOutcome::LeftOverflow(left_action)),
                    Err(_) => Some(ActionOutcome::Delegated),
                },
                SurroundIndex::Inside => {
                    let outcome = sqrt.child.apply_action(action);
                    match outcome? {
                        SeqActionOutcome::LeftDelete => {
                            Some(ActionOutcome::Unwrap(Unwrap::UnwrapSqrt))
                        }
                    }
                }
            },
            EditorTreeKind::Paren(paren) => {
                apply_completable_surrounds(paren, action, ')', GroupKind::Paren)
            }
            EditorTreeKind::Abs(abs) => {
                apply_completable_surrounds(abs, action, '|', GroupKind::Abs)
            }
            EditorTreeKind::Curly(curly) => {
                apply_completable_surrounds(curly, action, '}', GroupKind::Curly)
            }
            EditorTreeKind::Bracket(bracket) => {
                apply_completable_surrounds(bracket, action, ']', GroupKind::Bracket)
            }
        }
    }

    pub fn apply_action_from_right(&mut self, action: LeftAction) -> Option<ActionOutcome> {
        fn completable_surrounds<T: CompletableSurrounds>(
            paren: &mut T,
            action: LeftAction,
        ) -> Option<ActionOutcome> {
            match action {
                LeftAction::Delete => {
                    *paren.is_complete_mut() = false;
                    Some(ActionOutcome::CaptureCursor)
                }
            }
        }
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
            EditorTreeKind::Power(power) => match action {
                LeftAction::Delete => {
                    power.enter_from(Direction::Right);
                    Some(ActionOutcome::CaptureCursor)
                }
            },
            EditorTreeKind::Sqrt(sqrt) => match action {
                LeftAction::Delete => {
                    sqrt.enter_from(Direction::Right);
                    Some(ActionOutcome::CaptureCursor)
                }
            },
            EditorTreeKind::Paren(paren) => completable_surrounds(paren, action),
            EditorTreeKind::Abs(abs) => completable_surrounds(abs, action),
            EditorTreeKind::Bracket(bracket) => completable_surrounds(bracket, action),
            EditorTreeKind::Curly(curly) => completable_surrounds(curly, action),
            EditorTreeKind::SumProd(sum_prod) => match action {
                LeftAction::Delete => {
                    sum_prod.move_to(SumProdIndex::Top, Direction::Right);
                    Some(ActionOutcome::CaptureCursor)
                }
            },
        }
    }
}
