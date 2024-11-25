//! Conversion of data from `FontRef` to more useful formats
//!
//! There are multiple coordinate systems in play:
//! - The source coordinate system, from the font file. Often ranges into the thousands.
//! - The converted coordinate system.

use crate::fonts::Verb;

use super::fonts::PointVerb;
use color_eyre::Result as EyreResult;
use fast_desmos2_utils::OptExt;
use glam::Vec2;
use skrifa::metrics::{GlyphMetrics, Metrics};
use skrifa::outline::DrawSettings;
use skrifa::prelude::{LocationRef, Size};
use skrifa::{FontRef, MetadataProvider};
use std::collections::HashMap;

pub fn new(data: &[u8]) -> EyreResult<(GpuGlyphData, CpuGlyphData)> {
    let font = FontRef::from_index(data, 0).unwrap();
    let outlines = font.outline_glyphs();
    let glyph_metrics = GlyphMetrics::new(&font, Size::unscaled(), LocationRef::default());
    let metrics = Metrics::new(&font, Size::unscaled(), LocationRef::default());

    // the maximum height of all glyphs.
    let glyph_height = metrics.ascent - metrics.descent;
    let scale = glyph_height.recip();

    let mut point_verb = PointVerb::new();
    let mut glyph_starts = Vec::new();
    let mut bounds = Vec::new();
    let mut glyph_info = HashMap::new();
    point_verb.set_modifier(move |x| x * scale);

    for (index, (char_id, glyph_id)) in font.charmap().mappings().enumerate() {
        glyph_starts.push(GlyphStarts {
            point_start: point_verb.points.len() as u32,
            verb_start: point_verb.verbs.len() as u32,
        });
        let outline_glyph = outlines.get(glyph_id).unwrap_unreach();
        let bbox = glyph_metrics.bounds(glyph_id).unwrap_unreach();
        let advance_width = glyph_metrics.advance_width(glyph_id).unwrap_unreach();
        outline_glyph
            .draw(DrawSettings::from(Size::unscaled()), &mut point_verb)
            .unwrap_unreach();

        bounds.push(BoundingBox::from(bbox.scale(scale)));
        glyph_info.insert(
            char::from_u32(char_id).unwrap_unreach(),
            GlyphInfo {
                glyph_id: index as u32,
                advance: advance_width * scale,
                bbox: bbox.scale(scale).into(),
            },
        );
    }

    // not a real glyph, but useful
    glyph_starts.push(GlyphStarts {
        point_start: point_verb.points.len() as u32,
        verb_start: point_verb.verbs.len() as u32,
    });

    Ok((GpuGlyphData {
        points: point_verb.points,
        verbs: point_verb.verbs,
        glyph_starts,
        bounds,
    }, CpuGlyphData {
        glyph_info,
        baseline: -metrics.descent * scale,
    }))
}

pub struct GlyphInfo {
    pub glyph_id: u32,
    pub advance: f32,
    pub bbox: BoundingBox,
}

pub struct GpuGlyphData {
    pub bounds: Vec<BoundingBox>,
    pub points: Vec<Vec2>,
    pub verbs: Vec<Verb>,
    pub glyph_starts: Vec<GlyphStarts>,
}

pub struct CpuGlyphData {
    baseline: f32,
    glyph_info: HashMap<char, GlyphInfo>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BoundingBox {
    pub size: Vec2,
    pub offset: Vec2,
}

impl BoundingBox {
    pub const ZERO: Self = Self {
        size: Vec2::ZERO,
        offset: Vec2::ZERO,
    };

    pub fn x_min(&self) -> f32 {
        self.offset.x
    }

    pub fn y_min(&self) -> f32 {
        self.offset.y
    }

    pub fn x_max(&self) -> f32 {
        self.x_min() + self.size.x
    }

    pub fn y_max(&self) -> f32 {
        self.y_min() + self.size.y
    }

    pub fn is_zero(&self) -> bool {
        self.size == Vec2::ZERO
    }

    pub fn union(self, other: Self) -> Self {
        if self.is_zero() {
            other
        } else if other.is_zero() {
            self
        } else {
            let min = self.offset.min(other.offset);
            let max = (self.offset + other.size).max(self.offset + other.size);
            Self {
                offset: min,
                size: max - min,
            }
        }
    }
}

impl From<skrifa::metrics::BoundingBox> for BoundingBox {
    fn from(value: skrifa::metrics::BoundingBox) -> Self {
        let min = Vec2::new(value.x_min, value.y_min);
        let max = Vec2::new(value.x_max, value.y_max);
        Self {
            offset: min,
            size: max - min,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GlyphStarts {
    pub point_start: u32,
    pub verb_start: u32,
}

impl CpuGlyphData {
    pub fn baseline(&self) -> f32 {
        self.baseline
    }

    pub fn get_info(&self, char: char) -> Option<&GlyphInfo> {
        self.glyph_info.get(&char)
    }

    pub fn get_advance(&self, char: char) -> Option<f32> {
        self.get_info(char).map(|x| x.advance)
    }

    pub fn get_bearing(&self, char: char) -> Option<f32> {
        self.get_info(char).map(|x| x.bbox.offset.x)
    }

    pub fn get_id(&self, char: char) -> Option<u32> {
        self.get_info(char).map(|x| x.glyph_id)
    }

    // pub fn layout<I: IntoIterator<Item = char>>(
    //     &self,
    //     text: I,
    //     size: f32,
    //     pos: Vec2,
    // ) -> LayoutIter<I::IntoIter> {
    //     LayoutIter {
    //         glyph: self,
    //         chars: text.into_iter(),
    //         size: Vec2::splat(size),
    //         pos,
    //     }
    // }
}

// pub struct LayoutIter<'a, T> {
//     glyph: &'a CpuGlyphData,
//     chars: T,
//     size: Vec2,
//     pos: Vec2,
// }

// impl<'a, T: Iterator<Item = char>> Iterator for LayoutIter<'a, T> {
//     type Item = GlyphInstance;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let char = self.chars.next()?;
//         let char_info = self.glyph.get_info(char).unwrap();
//
//         let result = GlyphInstance::new(self.pos, self.size, char_info.glyph_id);
//         self.pos.x += char_info.advance * self.size.x;
//         Some(result)
//     }
// }
