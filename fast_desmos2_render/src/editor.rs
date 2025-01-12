use fast_desmos2_fonts::layout::LayoutNode;
use fast_desmos2_tree::tree::{
    EditorTree, EditorTreeAbs, EditorTreeBracket, EditorTreeCurly, EditorTreeFraction, EditorTreeKind, EditorTreeParen, EditorTreeSeq, EditorTreeSeqNormal, EditorTreeSeqVisual, EditorTreeTerminal
};

pub trait WithLayout {
    fn layout(&self) -> LayoutNode;
}

impl<S: EditorTreeSeq + WithLayout> WithLayout for EditorTree<S> {
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

impl WithLayout for EditorTreeSeqNormal {
    fn layout(&self) -> LayoutNode {
        LayoutNode::horizontal(self.children().iter().map(WithLayout::layout).collect())
    }
}

impl WithLayout for EditorTreeSeqVisual {
    fn layout(&self) -> LayoutNode {
        LayoutNode::horizontal(self.children().iter().map(WithLayout::layout).collect())
    }
}

impl WithLayout for EditorTreeTerminal {
    fn layout(&self) -> LayoutNode {
        LayoutNode::char(self.ch())
    }
}

impl<S: EditorTreeSeq + WithLayout> WithLayout for EditorTreeParen<S> {
    fn layout(&self) -> LayoutNode {
        LayoutNode::surround_horizontal('(', self.child().layout(), ')')
    }
}

impl<S: EditorTreeSeq + WithLayout> WithLayout for EditorTreeBracket<S> {
    fn layout(&self) -> LayoutNode {
        LayoutNode::surround_horizontal('[', self.child().layout(), ']')
    }
}

impl<S: EditorTreeSeq + WithLayout> WithLayout for EditorTreeCurly<S> {
    fn layout(&self) -> LayoutNode {
        LayoutNode::surround_horizontal('{', self.child().layout(), '}')
    }
}

impl<S: EditorTreeSeq + WithLayout> WithLayout for EditorTreeAbs<S> {
    fn layout(&self) -> LayoutNode {
        LayoutNode::surround_horizontal('|', self.child().layout(), '|')
    }
}

impl<S: EditorTreeSeq + WithLayout> WithLayout for EditorTreeFraction<S> {
    fn layout(&self) -> LayoutNode {
        LayoutNode::sandwich_vertical(self.top().layout(), self.bottom().layout())
    }
}
