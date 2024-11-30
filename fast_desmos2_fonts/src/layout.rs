use std::marker::PhantomData;

use crate::glyph_data::{BoundingBox, CpuGlyphData};
use glam::Vec2;

pub trait GlyphInstance {
    fn new(pos: Vec2, size: Vec2, id: u32) -> Self;
    fn offset_by(self, offset: Vec2) -> Self;
}

impl CpuGlyphData {
    pub fn layout<I: IntoIterator<Item = char>, G: GlyphInstance>(
        &self,
        text: I,
        size: f32,
        pos: Vec2,
    ) -> LayoutIter<I::IntoIter, G> {
        LayoutIter {
            glyph: self,
            chars: text.into_iter(),
            size: Vec2::splat(size),
            init_x: pos.x,
            pos,
            _phantom: PhantomData,
        }
    }
}

pub struct LayoutIter<'a, T, I: GlyphInstance> {
    glyph: &'a CpuGlyphData,
    chars: T,
    size: Vec2,
    init_x: f32,
    pos: Vec2,
    _phantom: PhantomData<I>,
}

impl<'a, T: Iterator<Item = char>, G: GlyphInstance> Iterator for LayoutIter<'a, T, G> {
    type Item = G;

    fn next(&mut self) -> Option<Self::Item> {
        let char = self.chars.next()?;
        if char == '\n' {
            self.pos.x = self.init_x;
            self.pos.y -= 1.2 * self.size.y;

            let char_info = self.glyph.get_info(' ').unwrap();
            Some(GlyphInstance::new(self.pos, self.size, char_info.glyph_id))
        } else {
            let char_info = self.glyph.get_info(char).unwrap();

            let result = GlyphInstance::new(self.pos, self.size, char_info.glyph_id);
            self.pos.x += char_info.advance * self.size.x;
            Some(result)
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstTree<I: GlyphInstance> {
    bbox: BoundingBox,
    pub offset: Vec2,
    kind: InstTreeKind<I>,
}

#[derive(Debug, Clone)]
pub enum InstTreeKind<I: GlyphInstance> {
    Children(Vec<InstTree<I>>),
    Nodes(Vec<I>),
    Node(I),
}

impl<I: GlyphInstance> InstTree<I> {
    pub fn bbox(&self) -> BoundingBox {
        self.bbox.transformed(self.offset, Vec2::ONE)
    }

    pub fn collect_vec(self) -> Vec<I> {
        let mut result = Vec::new();
        self.for_each_inst(&mut |x| result.push(x));
        result
    }

    pub fn for_each_inst(self, func: &mut impl FnMut(I)) {
        match self.kind {
            InstTreeKind::Node(node) => func(node.offset_by(self.offset)),
            InstTreeKind::Nodes(nodes) => nodes
                .into_iter()
                .for_each(|node| func(node.offset_by(self.offset))),
            InstTreeKind::Children(children) => children.into_iter().for_each(|mut child| {
                child.offset += self.offset;
                child.for_each_inst(func);
            }),
        }
    }

    pub fn new(bbox: BoundingBox, kind: InstTreeKind<I>) -> Self {
        Self {
            bbox,
            offset: Vec2::ZERO,
            kind,
        }
    }

    pub fn new_children(bbox: BoundingBox, children: Vec<InstTree<I>>) -> Self {
        Self::new(bbox, InstTreeKind::Children(children))
    }

    pub fn new_nodes(bbox: BoundingBox, nodes: Vec<I>) -> Self {
        Self::new(bbox, InstTreeKind::Nodes(nodes))
    }

    pub fn new_node(bbox: BoundingBox, node: I) -> Self {
        Self::new(bbox, InstTreeKind::Node(node))
    }
}

#[derive(Debug, Clone)]
pub struct NodeRenderOutcome<I: GlyphInstance> {
    advance: Vec2,
    instances: InstTree<I>,
}

impl<I: GlyphInstance> NodeRenderOutcome<I> {
    pub fn into_instances(self) -> InstTree<I> {
        self.instances
    }
}

pub struct LayoutNode<'a> {
    kind: LayoutKind<'a>,
}

impl<'a> LayoutNode<'a> {
    pub fn render<I: GlyphInstance>(
        &self,
        glyph_data: &CpuGlyphData,
        size: f32,
    ) -> NodeRenderOutcome<I> {
        match self.kind {
            LayoutKind::Str(s) => {
                let size_vec = Vec2::splat(size);

                let mut bbox = BoundingBox::ZERO;
                let mut current_x = 0.0;
                let mut instances = Vec::new();

                for char in s.chars() {
                    let char_info = glyph_data.get_info(char).unwrap();

                    bbox = bbox.union(
                        char_info
                            .bbox
                            .transformed(Vec2::new(current_x, 0.0), size_vec),
                    );
                    let result =
                        GlyphInstance::new(Vec2::new(current_x, 0.0), size_vec, char_info.glyph_id);
                    instances.push(result);

                    let advance = char_info.advance * size_vec.x;
                    current_x += advance;
                }

                NodeRenderOutcome {
                    advance: Vec2::new(current_x, -1.2 * size),
                    instances: InstTree::new_nodes(bbox, instances),
                }
            }
            LayoutKind::OneChar(c) => {
                let size_vec = Vec2::splat(size);
                let char_info = glyph_data.get_info(c).unwrap();

                let bbox = char_info.bbox.transformed(Vec2::ZERO, size_vec);
                let result = GlyphInstance::new(Vec2::ZERO, size_vec, char_info.glyph_id);

                NodeRenderOutcome {
                    advance: Vec2::new(char_info.advance * size, -1.2 * size),
                    instances: InstTree::new_node(bbox, result),
                }
            }
            LayoutKind::Horizontal(ref nodes) => {
                let mut bbox = BoundingBox::ZERO;
                let mut current_x = 0f32;
                let mut advance_y = 0f32;
                let mut children = Vec::with_capacity(nodes.len());

                for node in nodes {
                    let mut outcome = node.render(glyph_data, size);
                    outcome.instances.offset.x = current_x;

                    bbox = bbox.union(outcome.instances.bbox());
                    current_x += outcome.advance.x;
                    advance_y = advance_y.min(outcome.advance.y);

                    children.push(outcome.instances);
                }

                NodeRenderOutcome {
                    advance: Vec2::new(current_x, advance_y),
                    instances: InstTree::new_children(bbox, children),
                }
            }
            LayoutKind::Vertical(ref nodes) => {
                let mut current_y = 0f32;
                let mut advance_x = 0f32;
                let mut max_x = 0f32;
                let mut children = Vec::with_capacity(nodes.len());

                for node in nodes {
                    let mut outcome = node.render(glyph_data, size);
                    outcome.instances.offset.y = current_y;


                    current_y += outcome.advance.y;
                    advance_x = advance_x.max(outcome.advance.x);
                    max_x = max_x.max(outcome.instances.bbox().x_max());

                    children.push(outcome.instances);
                }

                let bbox_middle = max_x / 2.0;
                for node in children.iter_mut() {
                    let node_middle = node.bbox().x_center();
                    node.offset.x += bbox_middle - node_middle;
                }

                let bbox = children
                    .iter()
                    .fold(BoundingBox::ZERO, |bbox, node| bbox.union(node.bbox()));

                NodeRenderOutcome {
                    advance: Vec2::new(advance_x, current_y),
                    instances: InstTree::new_children(bbox, children),
                }
            }
            LayoutKind::SandwichVertical {
                ref top,
                middle,
                ref bottom,
            } => todo!(),
            LayoutKind::SurroundHorizontal {
                left,
                ref middle,
                right,
            } => todo!(),
        }
    }

    pub fn new(kind: LayoutKind<'a>) -> Self {
        Self { kind }
    }

    pub fn str(str: &'a str) -> Self {
        Self::new(LayoutKind::Str(str))
    }

    pub fn char(char: char) -> Self {
        Self::new(LayoutKind::OneChar(char))
    }

    pub fn horizontal(nodes: Vec<Self>) -> Self {
        Self::new(LayoutKind::Horizontal(nodes))
    }

    pub fn vertical(nodes: Vec<Self>) -> Self {
        Self::new(LayoutKind::Vertical(nodes))
    }
}

pub enum LayoutKind<'a> {
    Str(&'a str),
    OneChar(char),
    Horizontal(Vec<LayoutNode<'a>>),
    Vertical(Vec<LayoutNode<'a>>),
    SurroundHorizontal {
        left: char,
        middle: Box<LayoutNode<'a>>,
        right: char,
    },
    SandwichVertical {
        top: Box<LayoutNode<'a>>,
        middle: char,
        bottom: Box<LayoutNode<'a>>,
    },
}
