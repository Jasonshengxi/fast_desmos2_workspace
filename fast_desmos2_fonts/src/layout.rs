use std::marker::PhantomData;

use crate::glyph_data::{BoundingBox, CpuGlyphData};
use fast_desmos2_utils::OptExt;
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
        self.for_each_inst(&mut |x| result.push(x), &mut |_, _| {}, 0);
        result
    }

    pub fn collect_vec_debug(self) -> (Vec<I>, Vec<(usize, BoundingBox)>) {
        let mut result = Vec::new();
        let mut bboxes = Vec::new();
        self.for_each_inst(&mut |x| result.push(x), &mut |x, y| bboxes.push((x, y)), 0);
        (result, bboxes)
    }

    pub fn for_each_inst(
        self,
        func: &mut impl FnMut(I),
        bbox_debug: &mut impl FnMut(usize, BoundingBox),
        depth: usize,
    ) {
        bbox_debug(depth, self.bbox());
        match self.kind {
            InstTreeKind::Node(node) => func(node.offset_by(self.offset)),
            InstTreeKind::Nodes(nodes) => nodes
                .into_iter()
                .for_each(|node| func(node.offset_by(self.offset))),
            InstTreeKind::Children(children) => children.into_iter().for_each(|mut child| {
                child.offset += self.offset;
                child.for_each_inst(func, bbox_debug, depth + 1);
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

    pub fn new_children(children: Vec<InstTree<I>>) -> Self {
        let bbox = children
            .iter()
            .fold(BoundingBox::ZERO, |bbox, child| child.bbox().union(bbox));
        Self::new(bbox, InstTreeKind::Children(children))
    }

    pub fn new_children_bbox(bbox: BoundingBox, children: Vec<InstTree<I>>) -> Self {
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
                            .transformed_alt(Vec2::new(current_x, 0.0), size_vec),
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

                let bbox = char_info.bbox.transformed_alt(Vec2::ZERO, size_vec);
                let result = GlyphInstance::new(Vec2::ZERO, size_vec, char_info.glyph_id);

                NodeRenderOutcome {
                    advance: Vec2::new(char_info.advance * size, -1.2 * size),
                    instances: InstTree::new_node(bbox, result),
                }
            }
            LayoutKind::Horizontal(ref nodes) => {
                let mut current_x = 0f32;
                let mut advance_y = 0f32;
                let mut children = Vec::with_capacity(nodes.len());

                for node in nodes {
                    let mut outcome = node.render(glyph_data, size);
                    outcome.instances.offset.x = current_x;

                    current_x += outcome.advance.x;
                    advance_y = advance_y.min(outcome.advance.y);

                    children.push(outcome.instances);
                }

                NodeRenderOutcome {
                    advance: Vec2::new(current_x, advance_y),
                    instances: InstTree::new_children(children),
                }
            }
            LayoutKind::Vertical(ref nodes) => {
                let mut current_y = 0f32;
                let mut max_x = 0f32;
                let mut children = Vec::with_capacity(nodes.len());
                let mut advances = Vec::with_capacity(nodes.len());

                for node in nodes {
                    let mut outcome = node.render(glyph_data, size);
                    outcome.instances.offset.y = current_y;
                    advances.push(outcome.advance.x);

                    current_y += outcome.advance.y;
                    max_x = max_x.max(outcome.instances.bbox().max_pos().x);

                    children.push(outcome.instances);
                }

                let bbox_middle = max_x / 2.0;
                for node in children.iter_mut() {
                    let node_middle = node.bbox().center().x;
                    node.offset.x += bbox_middle - node_middle;
                }

                let advance_x = children
                    .iter()
                    .zip(advances.iter())
                    .fold(0.0f32, |acc, (child, &advance)| {
                        acc.max(child.offset.x + advance)
                    });

                NodeRenderOutcome {
                    advance: Vec2::new(advance_x, current_y),
                    instances: InstTree::new_children(children),
                }
            }
            LayoutKind::SandwichVertical {
                ref top,
                ref bottom,
            } => {
                let top_inst = top.render::<I>(glyph_data, size);
                let bottom_inst = bottom.render::<I>(glyph_data, size);

                let max_advance_x = top_inst.advance.x.max(bottom_inst.advance.x);
                let big_box = top_inst
                    .instances
                    .bbox()
                    .union(bottom_inst.instances.bbox());
                let x_center = big_box.center().x;

                let mid_info = glyph_data
                    .get_info(CpuGlyphData::RECT_CHAR)
                    .unwrap_unreach();
                // let mid_offset = Vec2::new(big_box.x_min(), top_inst.instances.bbox().y_min());
                // let mid_bbox = mid_info.bbox.transformed(mid_offset, mid_scale);
                // let mid_instance = I::new(mid_offset, mid_scale, mid_info.glyph_id);

                const MIDDLE_HEIGHT: f32 = 0.1;
                const MIDDLE_MARGIN: f32 = 0.05;
                let top_bbox = top_inst.instances.bbox();
                let mid_scale = Vec2::new(big_box.size().x, MIDDLE_HEIGHT * size);
                let mid_offset = Vec2::new(0.0, top_bbox.min_pos().y - mid_scale.y - MIDDLE_MARGIN);
                let mid_bbox = mid_info.bbox.transformed_alt(mid_offset, mid_scale);
                let mid_instance = I::new(mid_offset, mid_scale, mid_info.glyph_id);

                let top_offset = Vec2::new(x_center - top_inst.instances.bbox().center().x, 0.0);
                let bottom_offset = Vec2::new(
                    x_center - bottom_inst.instances.bbox().center().x,
                    top_inst.advance.y - mid_scale.y - 2.0 * MIDDLE_MARGIN,
                );
                let total_advance = bottom_offset.y + bottom_inst.advance.y;

                let mut top_inst = top_inst.instances;
                let mut bottom_inst = bottom_inst.instances;
                top_inst.offset += top_offset;
                bottom_inst.offset += bottom_offset;

                let bbox = top_inst.bbox().union(bottom_inst.bbox());

                NodeRenderOutcome {
                    advance: Vec2::new(max_advance_x, bottom_offset.y + total_advance),
                    instances: InstTree::new_children_bbox(
                        bbox,
                        vec![
                            top_inst,
                            InstTree::new_node(mid_bbox, mid_instance),
                            bottom_inst,
                        ],
                    ),
                }
            }
            LayoutKind::SurroundHorizontal {
                left,
                ref middle,
                right,
            } => {
                let left_info = glyph_data.get_info(left).unwrap_unreach();
                let right_info = glyph_data.get_info(right).unwrap_unreach();

                let mut mid_inst = middle.render::<I>(glyph_data, size);
                mid_inst.instances.offset.x += left_info.advance * size;
                let target_bbox = mid_inst.instances.bbox();

                let target_height = target_bbox.size().y;
                let target_offset = target_bbox.offset().y;

                let left_scale = target_height / left_info.bbox.size().y;
                let left_shift_y = target_offset - (left_info.bbox.offset().y * left_scale);
                let (left_bbox, left_inst) = left_info
                    .create_instance(Vec2::new(0.0, left_shift_y), Vec2::new(size, left_scale));

                let right_scale = target_height / right_info.bbox.size().y;
                let right_shift_y = target_offset - (right_info.bbox.offset().y * right_scale);
                let (right_bbox, right_inst) = right_info.create_instance(
                    Vec2::new(left_info.advance * size + mid_inst.advance.x, right_shift_y),
                    Vec2::new(size, right_scale),
                );

                NodeRenderOutcome {
                    advance: Vec2::new(left_info.advance + right_info.advance, 0.0) * size
                        + mid_inst.advance,
                    instances: InstTree::new_children(vec![
                        InstTree::new_node(left_bbox, left_inst),
                        mid_inst.instances,
                        InstTree::new_node(right_bbox, right_inst),
                    ]),
                }
            }
        }
    }

    pub fn new(kind: LayoutKind<'a>) -> Self {
        Self { kind }
    }

    pub fn str(str: &'a str) -> Self {
        // Self::new(LayoutKind::Str(str))
        Self::horizontal(str.chars().map(Self::char).collect())
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

    pub fn sandwich_vertical(top: Self, bottom: Self) -> Self {
        Self::new(LayoutKind::SandwichVertical {
            top: Box::new(top),
            bottom: Box::new(bottom),
        })
    }

    pub fn surround_horizontal(left: char, middle: Self, right: char) -> Self {
        Self::new(LayoutKind::SurroundHorizontal {
            left,
            middle: Box::new(middle),
            right,
        })
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
        bottom: Box<LayoutNode<'a>>,
    },
}
