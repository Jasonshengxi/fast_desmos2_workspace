use crate::builtins::Builtins;
use bitflags::bitflags;
use elsa::FrozenVec;
use fast_desmos2_tree::tree::SumOrProd;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub struct EvalNode {
    kind: Box<EvalKind>,
}

impl EvalNode {
    pub fn new(kind: EvalKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }

    pub fn kind(&self) -> &EvalKind {
        &self.kind
    }

    pub fn into_kind(self) -> EvalKind {
        *self.kind
    }

    pub fn number(x: f64) -> Self {
        Self::new(EvalKind::Number(x))
    }

    pub fn ident(ident: IdentId) -> Self {
        Self::new(EvalKind::Identifier(ident))
    }

    pub fn abs(node: EvalNode) -> Self {
        Self::new(EvalKind::Abs(node))
    }

    pub fn sqrt(node: EvalNode) -> Self {
        Self::new(EvalKind::Sqrt(node))
    }

    pub fn list_literal(nodes: Vec<Self>) -> Self {
        Self::new(EvalKind::List(nodes))
    }

    pub fn point((x, y): (Self, Self)) -> Self {
        Self::new(EvalKind::Point(x, y))
    }

    pub fn multiply(nodes: Vec<Self>) -> Self {
        Self::new(EvalKind::Multiply(nodes))
    }

    pub fn index(expr: Self, index: Self) -> Self {
        Self::new(EvalKind::ListIndexing { expr, index })
    }

    pub fn power(base: Self, power: Self) -> Self {
        Self::new(EvalKind::Power { base, power })
    }

    pub fn list_range(from: Self, next: Option<Self>, to: Self) -> Self {
        Self::new(EvalKind::ListRange { from, next, to })
    }

    pub fn add_sub(pairs: Vec<(AddOrSub, Self)>) -> Self {
        Self::new(EvalKind::AddSub(pairs))
    }

    pub fn if_else(conds: Vec<Conditional>, yes: Option<EvalNode>, no: Option<EvalNode>) -> Self {
        Self::new(EvalKind::IfElse { conds, yes, no })
    }

    pub fn sum_prod(
        kind: SumOrProd,
        ident: IdentId,
        from: EvalNode,
        to: EvalNode,
        expr: EvalNode,
    ) -> Self {
        Self::new(EvalKind::SumProd {
            kind,
            ident,
            from,
            to,
            expr,
        })
    }

    pub fn builtins_call(builtins: Builtins, power: Option<Self>, params: Vec<Self>) -> Self {
        Self::new(EvalKind::BuiltinsCall {
            builtins,
            power,
            params,
        })
    }

    pub fn function_call(ident: IdentId, power: Option<Self>, params: Vec<Self>) -> Self {
        Self::new(EvalKind::FunctionCall {
            ident,
            power,
            params,
        })
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

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
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
pub enum Element {
    X,
    Y,
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct VarDef {
    ident: IdentId,
    expr: EvalNode,
}

impl VarDef {
    pub fn new(ident: IdentId, expr: EvalNode) -> Self {
        Self { ident, expr }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conditional {
    expr: EvalNode,
    comps: Vec<(CompSet, EvalNode)>,
}

impl Conditional {
    pub fn new(expr: EvalNode, comps: Vec<(CompSet, EvalNode)>) -> Self {
        Self { expr, comps }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvalKind {
    Identifier(IdentId),
    BuiltinsCall {
        builtins: Builtins,
        power: Option<EvalNode>,
        params: Vec<EvalNode>,
    },
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
    Sqrt(EvalNode),
    Power {
        base: EvalNode,
        power: EvalNode,
    },
    For {
        expr: EvalNode,
        defs: Vec<VarDef>,
    },
    ListComp {
        expr: EvalNode,
        defs: Vec<VarDef>,
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
        expr: EvalNode,
        defs: Vec<VarDef>,
    },
}
