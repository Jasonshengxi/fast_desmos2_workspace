mod debug;

#[derive(Debug, PartialEq)]
pub struct EditorTreeSeq {
    cursor: usize,
    children: Vec<EditorTree>,
}

#[derive(Debug, PartialEq)]
pub struct EditorTree {
    cursor: usize,
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

    pub fn apply_move(&mut self, movement: TreeMove) -> Option<TreeMove> {
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

    pub fn enter_from(&mut self, direction: TreeMove) {
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

use EditorTreeKind as TK;
impl EditorTree {
    pub const FRACTION_LEFT: usize = 2;
    pub const FRACTION_BOTTOM: usize = 3;
    pub const FRACTION_TOP: usize = 4;

    pub const POWER_BASE: usize = 0;
    pub const POWER_POWER: usize = 1;

    pub fn str(content: &str) -> Self {
        Self::terminal(0, content.to_string())
    }

    pub fn terminal(cursor: usize, content: String) -> Self {
        assert!(cursor < content.len());
        Self {
            cursor,
            kind: EditorTreeKind::Terminal(content),
        }
    }

    pub fn power(cursor: usize, base: EditorTreeSeq, power: EditorTreeSeq) -> Self {
        assert!(cursor == Self::POWER_BASE || cursor == Self::POWER_POWER);
        Self {
            cursor,
            kind: EditorTreeKind::Power { base, power },
        }
    }

    pub fn fraction(cursor: usize, top: EditorTreeSeq, bottom: EditorTreeSeq) -> Self {
        assert!(cursor == Self::FRACTION_TOP || cursor == Self::FRACTION_BOTTOM);
        Self {
            cursor,
            kind: EditorTreeKind::Fraction { top, bottom },
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn active_child(&self) -> Option<&EditorTreeSeq> {
        Some(match &self.kind {
            TK::Terminal(_) => return None,
            TK::Fraction { top, bottom } => match self.cursor {
                Self::FRACTION_BOTTOM => bottom,
                Self::FRACTION_TOP => top,
                _ => unreachable!(),
            },
            TK::Power { base, power } => match self.cursor {
                Self::POWER_BASE => base,
                Self::POWER_POWER => power,
                _ => unreachable!(),
            },
        })
    }

    pub fn enter_from(&mut self, direction: TreeMove) {
        match &mut self.kind {
            TK::Terminal(content) => {
                self.cursor = match direction {
                    TreeMove::Left | TreeMove::Up => 0,
                    TreeMove::Right | TreeMove::Down => content.len() - 1,
                }
            }
            TK::Power { base, power } => match direction {
                TreeMove::Left | TreeMove::Up => {
                    self.cursor = Self::POWER_BASE;
                    base.enter_from(direction);
                }
                TreeMove::Down | TreeMove::Right => {
                    self.cursor = Self::POWER_POWER;
                    power.enter_from(direction);
                }
            },
            TK::Fraction { top, bottom } => match direction {
                TreeMove::Down | TreeMove::Right => {
                    self.cursor = Self::FRACTION_BOTTOM;
                    bottom.enter_from(direction);
                }
                TreeMove::Up => {
                    self.cursor = Self::FRACTION_TOP;
                    top.enter_from(direction);
                }
                TreeMove::Left => {
                    self.cursor = Self::FRACTION_LEFT;
                }
            },
        }
    }

    pub fn apply_move(&mut self, movement: TreeMove) -> Option<TreeMove> {
        match &mut self.kind {
            TK::Terminal(term) => match movement {
                TreeMove::Right => {
                    self.cursor += 1;
                    (self.cursor >= term.len()).then_some(TreeMove::Right)
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
            },
            TK::Power { base, power } => match self.cursor {
                Self::POWER_BASE => {
                    let movement = base.apply_move(movement);
                    match movement {
                        Some(TreeMove::Up | TreeMove::Right) => {
                            self.cursor = Self::POWER_POWER;
                            power.enter_from(TreeMove::Left);
                            None
                        }
                        Some(left_or_down @ (TreeMove::Left | TreeMove::Down)) => {
                            Some(left_or_down)
                        }
                        None => None,
                    }
                }
                Self::POWER_POWER => {
                    let movement = power.apply_move(movement);
                    match movement {
                        Some(TreeMove::Left | TreeMove::Down) => {
                            self.cursor = Self::POWER_BASE;
                            base.enter_from(TreeMove::Right);
                            None
                        }
                        Some(up_or_right @ (TreeMove::Up | TreeMove::Right)) => Some(up_or_right),
                        None => None,
                    }
                }
                _ => unreachable!(),
            },
            TK::Fraction { top, bottom } => match self.cursor {
                Self::FRACTION_BOTTOM => {
                    let movement = bottom.apply_move(movement);
                    match movement {
                        Some(TreeMove::Up) => {
                            self.cursor = Self::FRACTION_TOP;
                            top.enter_from(TreeMove::Down);
                            None
                        }
                        Some(TreeMove::Left) => {
                            self.cursor = Self::FRACTION_LEFT;
                            None
                        }
                        otherwise => otherwise,
                    }
                }
                Self::FRACTION_TOP => {
                    let movement = top.apply_move(movement);
                    match movement {
                        Some(TreeMove::Down) => {
                            self.cursor = Self::FRACTION_BOTTOM;
                            bottom.enter_from(TreeMove::Up);
                            None
                        }
                        Some(TreeMove::Left) => {
                            self.cursor = Self::FRACTION_LEFT;
                            None
                        }
                        otherwise => otherwise,
                    }
                }
                Self::FRACTION_LEFT => match movement {
                    TreeMove::Right => {
                        self.cursor = Self::FRACTION_TOP;
                        top.enter_from(TreeMove::Left);
                        None
                    }
                    otherwise => Some(otherwise),
                },
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EditorTreeKind {
    Terminal(String),
    Fraction {
        top: EditorTreeSeq,
        bottom: EditorTreeSeq,
    },
    Power {
        base: EditorTreeSeq,
        power: EditorTreeSeq,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeMove {
    Up,
    Down,
    Left,
    Right,
}
