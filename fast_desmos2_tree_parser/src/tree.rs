use crate::builtins::Builtins;
use bitflags::bitflags;
use elsa::FrozenVec;
use std::cmp::Ordering;

pub struct EvalNode {
    kind: Box<EvalKind>,
}

impl EvalNode {
    pub fn new(kind: EvalKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }
}

#[derive(Clone, Default)]
pub struct IdentStorer {
    ids: FrozenVec<Box<str>>,
}

impl std::fmt::Debug for IdentStorer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot be debugged.")
    }
}

impl IdentStorer {
    pub fn convert_id(&self, ident: &str) -> IdentId {
        if let Some(id) = self.ids.iter().position(|x| x == ident) {
            IdentId(id)
        } else {
            let new_id = self.ids.len();
            self.ids.push(ident.to_string().into_boxed_str());
            IdentId(new_id)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IdentId(usize);

impl IdentId {
    pub fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum AddOrSub {
    #[default]
    Add,
    Sub,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SumOrProd {
    Sum,
    Prod,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Element {
    X,
    Y,
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct CompSet: u8 {
        const MORE = 0b100;
        const EQUAL = 0b010;
        const LESS = 0b001;
        const LESS_OR_EQUAL = 0b011;
        const MORE_OR_EQUAL = 0b110;
    }
}

impl From<Ordering> for CompSet {
    fn from(value: Ordering) -> Self {
        Self::from_ordering(value)
    }
}

impl CompSet {
    pub const fn from_ordering(ordering: Ordering) -> Self {
        match ordering {
            Ordering::Less => Self::LESS,
            Ordering::Equal => Self::EQUAL,
            Ordering::Greater => Self::MORE,
        }
    }

    pub const fn reference_char(self) -> char {
        const EQUAL: u8 = CompSet::EQUAL.bits();
        const MORE: u8 = CompSet::MORE.bits();
        const LESS: u8 = CompSet::LESS.bits();
        const MORE_OR_EQ: u8 = CompSet::MORE_OR_EQUAL.bits();
        const LESS_OR_EQ: u8 = CompSet::LESS_OR_EQUAL.bits();

        match self.bits() {
            EQUAL => '=',
            MORE => '>',
            LESS => '<',
            MORE_OR_EQ => 'G',
            LESS_OR_EQ => 'L',
            _ => '#',
        }
    }
}

pub struct Conditional {
    exprs: Vec<EvalNode>,
    comps: Vec<CompSet>,
}

pub enum EvalKind {
    Identifier(IdentId),
    Builtins(Builtins),
    Number(f64),
    Abs(EvalNode),
    Point(EvalNode, EvalNode),
    List(Vec<EvalNode>),
    SumProd {
        kind: SumOrProd,
        ident: IdentId,
        from: EvalNode,
        to: EvalNode,
        expr: EvalNode,
    },
    FunctionCall {
        ident: IdentId,
        power: Option<EvalNode>,
        params: Vec<EvalNode>,
    },
    Multiply(Vec<EvalNode>),
    AddSub(Vec<(AddOrSub, EvalNode)>),
    Frac {
        top: EvalNode,
        bottom: EvalNode,
    },
    Root {
        root: Option<EvalNode>,
        expr: EvalNode,
    },
    Exp {
        expr: EvalNode,
        exp: EvalNode,
    },
    For {
        expr: EvalNode,
        defs: Vec<(IdentId, EvalNode)>,
    },
    ListComp {
        expr: EvalNode,
        defs: Vec<EvalNode>,
    },
    ListRange {
        from: EvalNode,
        next: Option<EvalNode>,
        to: EvalNode,
    },
    IfElse {
        conds: Vec<Conditional>,
        yes: Option<EvalNode>,
        no: Option<EvalNode>,
    },
    ElemAccess {
        expr: EvalNode,
        element: Element,
    },
    ListIndexing {
        expr: EvalNode,
        index: EvalNode,
    },
    With {
        ident: IdentId,
        def: EvalNode,
        expr: EvalNode,
    },
}
