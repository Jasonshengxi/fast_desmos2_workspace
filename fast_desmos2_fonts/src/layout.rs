use std::marker::PhantomData;

use crate::glyph_data::CpuGlyphData;
use glam::Vec2;

pub trait GlyphInstance {
    fn new(pos: Vec2, size: Vec2, id: u32) -> Self;
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

