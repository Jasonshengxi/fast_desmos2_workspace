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

impl EditorTreeSeq {
    pub fn new(cursor: usize, children: Vec<EditorTree>) -> Self {
        assert!(cursor <= children.len());
        Self { cursor, children }
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

    pub fn apply_movement(&mut self, movement: TreeMov) -> Option<TreeMov> {
        let movement = self
            .children
            .get_mut(self.cursor)
            .map_or(Some(movement), |child| child.apply_movement(movement));

        match movement {
            Some(TreeMov::Left) => {
                if self.cursor == 0 {
                    Some(TreeMov::Left)
                } else {
                    self.cursor -= 1;
                    self.children[self.cursor].enter_from(SeqTreeMov::Right);
                    None
                }
            }
            Some(TreeMov::Right) => {
                if self.cursor >= self.children.len() {
                    Some(TreeMov::Right)
                } else {
                    self.cursor += 1;
                    if let Some(child) = self.children.get_mut(self.cursor) {
                        child.enter_from(SeqTreeMov::Left);
                    }
                    None
                }
            }
            Some(up_or_down) => Some(up_or_down),
            None => None,
        }
    }

    pub fn enter_from(&mut self, direction: SeqTreeMov) {
        match direction {
            SeqTreeMov::Left => {
                self.cursor = 0;
                self.children[0].enter_from(direction);
            }
            SeqTreeMov::Right => {
                self.cursor = self.children.len();
            }
        }
    }
}

impl EditorTree {
    pub fn terminal(cursor: usize, content: String) -> Self {
        assert!(cursor < content.len());
        Self {
            cursor,
            kind: EditorTreeKind::Terminal(content),
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn enter_from(&mut self, direction: SeqTreeMov) {
        use EditorTreeKind as TK;
        match &mut self.kind {
            TK::Terminal(content) => {
                self.cursor = match direction {
                    SeqTreeMov::Left => 0,
                    SeqTreeMov::Right => content.len(),
                }
            }
            TK::Power { base, power } => match direction {
                SeqTreeMov::Left => {
                    self.cursor = 0;
                    base.enter_from(SeqTreeMov::Left);
                }
                SeqTreeMov::Right => {
                    self.cursor = 1;
                    power.enter_from(SeqTreeMov::Right);
                }
            },
            TK::Fraction { top, bottom: _ } => {
                self.cursor = 0;
                top.enter_from(direction);
            }
        }
    }

    pub fn apply_movement(&mut self, movement: TreeMov) -> Option<TreeMov> {
        use EditorTreeKind as TK;
        match &mut self.kind {
            TK::Terminal(term) => match movement {
                TreeMov::Right => {
                    self.cursor += 1;
                    (self.cursor >= term.len()).then_some(TreeMov::Right)
                }
                TreeMov::Left => None,
                up_or_down => Some(up_or_down),
            },
            TK::Power { base, power } => match self.cursor {
                0 => {
                    let movement = base.apply_movement(movement);
                    match movement {
                        Some(TreeMov::Up | TreeMov::Right) => {
                            self.cursor = 1;
                            power.enter_from(SeqTreeMov::Left);
                            None
                        }
                        Some(left_or_down @ (TreeMov::Left | TreeMov::Down)) => Some(left_or_down),
                        None => None,
                    }
                }
                1 => {
                    let movement = power.apply_movement(movement);
                    match movement {
                        Some(TreeMov::Left | TreeMov::Down) => {
                            self.cursor = 0;
                            base.enter_from(SeqTreeMov::Right);
                            None
                        }
                        Some(up_or_right @ (TreeMov::Up | TreeMov::Right)) => Some(up_or_right),
                        None => None,
                    }
                }
                _ => unreachable!(),
            },
            TK::Fraction { top, bottom } => match self.cursor {
                0 => {
                    let movement = bottom.apply_movement(movement);
                    match movement {
                        Some(TreeMov::Up) => {
                            self.cursor = 1;
                            top.enter_from(SeqTreeMov::Left);
                            None
                        }
                        otherwise => otherwise,
                    }
                }
                1 => {
                    let movement = top.apply_movement(movement);

                    match movement {
                        Some(TreeMov::Down) => {
                            self.cursor = 0;
                            bottom.enter_from(SeqTreeMov::Left);
                            None
                        }
                        otherwise => otherwise,
                    }
                }
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
pub enum TreeMov {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqTreeMov {
    Left,
    Right,
}

impl TryFrom<TreeMov> for SeqTreeMov {
    type Error = TreeMov;

    fn try_from(value: TreeMov) -> Result<Self, Self::Error> {
        match value {
            TreeMov::Left => Ok(Self::Left),
            TreeMov::Right => Ok(Self::Right),
            _ => Err(value),
        }
    }
}

impl From<SeqTreeMov> for TreeMov {
    fn from(value: SeqTreeMov) -> Self {
        match value {
            SeqTreeMov::Left => Self::Left,
            SeqTreeMov::Right => Self::Right,
        }
    }
}
