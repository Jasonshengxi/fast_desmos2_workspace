use fast_desmos2_fonts::layout::LayoutNode;
use fast_desmos2_tree::tree::{
    EditorTree, EditorTreeAbs, EditorTreeBracket, EditorTreeCurly, EditorTreeFraction,
    EditorTreeKind, EditorTreeParen, EditorTreeSeq, EditorTreeTerminal,
};

pub trait WithLayout {
    fn layout(&self) -> LayoutNode;
}

impl WithLayout for EditorTree {
    fn layout(&self) -> LayoutNode {
        match self.kind() {
            EditorTreeKind::Terminal(term) => term.layout(),
            EditorTreeKind::Fraction(frac) => frac.layout(),
            EditorTreeKind::Power(_) => todo!(),
            EditorTreeKind::Sqrt(_) => todo!(),
            EditorTreeKind::Paren(paren) => paren.layout(),
            EditorTreeKind::SumProd(_) => todo!(),
            EditorTreeKind::Abs(abs) => abs.layout(),
            EditorTreeKind::Bracket(brackets) => brackets.layout(),
            EditorTreeKind::Curly(curly) => curly.layout(),
        }
    }
}

impl WithLayout for EditorTreeSeq {
    fn layout(&self) -> LayoutNode {
        LayoutNode::horizontal(self.children().iter().map(WithLayout::layout).collect())
    }
}

impl WithLayout for EditorTreeTerminal {
    fn layout(&self) -> LayoutNode {
        LayoutNode::char(self.ch())
    }
}

impl WithLayout for EditorTreeParen {
    fn layout(&self) -> LayoutNode {
        LayoutNode::surround_horizontal('(', self.child().layout(), ')')
    }
}

impl WithLayout for EditorTreeBracket {
    fn layout(&self) -> LayoutNode {
        LayoutNode::surround_horizontal('[', self.child().layout(), ']')
    }
}

impl WithLayout for EditorTreeCurly {
    fn layout(&self) -> LayoutNode {
        LayoutNode::surround_horizontal('{', self.child().layout(), '}')
    }
}

impl WithLayout for EditorTreeAbs {
    fn layout(&self) -> LayoutNode {
        LayoutNode::surround_horizontal('|', self.child().layout(), '|')
    }
}

impl WithLayout for EditorTreeFraction {
    fn layout(&self) -> LayoutNode {
        LayoutNode::sandwich_vertical(self.top().layout(), self.bottom().layout())
    }
}
