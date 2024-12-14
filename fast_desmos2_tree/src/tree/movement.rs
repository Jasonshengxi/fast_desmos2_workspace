use super::{
    EditorTree, EditorTreeFraction, EditorTreeKind, EditorTreePower, EditorTreeSeq,
    EditorTreeTerminal, FractionIndex, PowerIndex,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeMove {
    Up,
    Down,
    Left,
    Right,
}

pub trait TreeMovable {
    fn apply_move(&mut self, movement: TreeMove) -> Option<TreeMove>;
    fn enter_from(&mut self, direction: TreeMove);
}

impl TreeMovable for EditorTreeSeq {
    fn apply_move(&mut self, movement: TreeMove) -> Option<TreeMove> {
        let movement = self
            .children
            .get_mut(self.cursor)
            .map_or(Some(movement), |child| child.apply_move(movement));

        match movement {
            Some(TreeMove::Left) => {
                if self.cursor == 0 {
                    Some(TreeMove::Left)
                } else {
                    self.cursor -= 1;
                    self.children[self.cursor].enter_from(TreeMove::Right);
                    None
                }
            }
            Some(TreeMove::Right) => {
                if self.cursor >= self.children.len() {
                    Some(TreeMove::Right)
                } else {
                    self.cursor += 1;
                    if let Some(child) = self.children.get_mut(self.cursor) {
                        child.enter_from(TreeMove::Left);
                    }
                    None
                }
            }
            Some(up_or_down) => Some(up_or_down),
            None => None,
        }
    }

    fn enter_from(&mut self, direction: TreeMove) {
        match direction {
            TreeMove::Left => {
                self.cursor = 0;
                self.children[0].enter_from(direction);
            }
            TreeMove::Right => {
                self.cursor = self.children.len();
            }
            TreeMove::Up | TreeMove::Down => {
                if let Some(child) = self.active_child_mut() {
                    child.enter_from(direction);
                }
            }
        }
    }
}

impl TreeMovable for EditorTreeTerminal {
    fn apply_move(&mut self, movement: TreeMove) -> Option<TreeMove> {
        match movement {
            TreeMove::Right => {
                self.cursor += 1;
                (self.cursor >= self.string.len()).then_some(TreeMove::Right)
            }
            TreeMove::Left => {
                if self.cursor == 0 {
                    Some(TreeMove::Left)
                } else {
                    self.cursor -= 1;
                    None
                }
            }
            up_or_down => Some(up_or_down),
        }
    }

    fn enter_from(&mut self, direction: TreeMove) {
        self.cursor = match direction {
            TreeMove::Left | TreeMove::Up => 0,
            TreeMove::Right | TreeMove::Down => self.string.len() - 1,
        }
    }
}
impl TreeMovable for EditorTreePower {
    fn apply_move(&mut self, movement: TreeMove) -> Option<TreeMove> {
        match self.cursor {
            PowerIndex::Base => {
                let movement = self.base.apply_move(movement);
                match movement {
                    Some(TreeMove::Up | TreeMove::Right) => {
                        self.cursor = PowerIndex::Power;
                        self.power.enter_from(TreeMove::Left);
                        None
                    }
                    Some(left_or_down @ (TreeMove::Left | TreeMove::Down)) => Some(left_or_down),
                    None => None,
                }
            }
            PowerIndex::Power => {
                let movement = self.power.apply_move(movement);
                match movement {
                    Some(TreeMove::Left | TreeMove::Down) => {
                        self.cursor = PowerIndex::Base;
                        self.base.enter_from(TreeMove::Right);
                        None
                    }
                    Some(up_or_right @ (TreeMove::Up | TreeMove::Right)) => Some(up_or_right),
                    None => None,
                }
            }
        }
    }

    fn enter_from(&mut self, direction: TreeMove) {
        match direction {
            TreeMove::Left | TreeMove::Up => {
                self.cursor = PowerIndex::Base;
                self.base.enter_from(direction);
            }
            TreeMove::Down | TreeMove::Right => {
                self.cursor = PowerIndex::Power;
                self.power.enter_from(direction);
            }
        }
    }
}
impl TreeMovable for EditorTreeFraction {
    fn apply_move(&mut self, movement: TreeMove) -> Option<TreeMove> {
        match self.cursor {
            FractionIndex::Bottom => {
                let movement = self.bottom.apply_move(movement);
                match movement {
                    Some(TreeMove::Up) => {
                        self.cursor = FractionIndex::Top;
                        self.top.enter_from(TreeMove::Down);
                        None
                    }
                    Some(TreeMove::Left) => {
                        self.cursor = FractionIndex::Left;
                        None
                    }
                    otherwise => otherwise,
                }
            }
            FractionIndex::Top => {
                let movement = self.top.apply_move(movement);
                match movement {
                    Some(TreeMove::Down) => {
                        self.cursor = FractionIndex::Bottom;
                        self.bottom.enter_from(TreeMove::Up);
                        None
                    }
                    Some(TreeMove::Left) => {
                        self.cursor = FractionIndex::Left;
                        None
                    }
                    otherwise => otherwise,
                }
            }
            FractionIndex::Left => match movement {
                TreeMove::Right => {
                    self.cursor = FractionIndex::Top;
                    self.top.enter_from(TreeMove::Left);
                    None
                }
                otherwise => Some(otherwise),
            },
        }
    }

    fn enter_from(&mut self, direction: TreeMove) {
        match direction {
            TreeMove::Down => {
                self.cursor = FractionIndex::Bottom;
                self.bottom.enter_from(direction);
            }
            TreeMove::Up | TreeMove::Right => {
                self.cursor = FractionIndex::Top;
                self.top.enter_from(direction);
            }
            TreeMove::Left => {
                self.cursor = FractionIndex::Left;
            }
        }
    }
}

impl TreeMovable for EditorTree {
    fn enter_from(&mut self, direction: TreeMove) {
        match &mut self.kind {
            EditorTreeKind::Terminal(term) => term.enter_from(direction),
            EditorTreeKind::Power(power) => power.enter_from(direction),
            EditorTreeKind::Fraction(fraction) => fraction.enter_from(direction),
        }
    }

    fn apply_move(&mut self, movement: TreeMove) -> Option<TreeMove> {
        match &mut self.kind {
            EditorTreeKind::Terminal(term) => term.apply_move(movement),
            EditorTreeKind::Power(power) => power.apply_move(movement),
            EditorTreeKind::Fraction(fraction) => fraction.apply_move(movement),
        }
    }
}
