//! Conversion of data from `FontRef` to more useful formats
//!
//! There are multiple coordinate systems in play:
//! - The source coordinate system, from the font file. Often ranges into the thousands.
//! - The converted coordinate system.

use crate::fonts::Verb;
use crate::layout::GlyphInstance;

use super::fonts::PointVerb;
use color_eyre::Result as EyreResult;
use fast_desmos2_utils::OptExt;
use glam::Vec2;
use skrifa::metrics::{GlyphMetrics, Metrics};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::prelude::{LocationRef, Size};
use skrifa::{FontRef, MetadataProvider};
use std::collections::HashMap;

pub fn new(data: &[u8]) -> EyreResult<(GpuGlyphData, CpuGlyphData)> {
    let font = FontRef::from_index(data, 0).unwrap();
    let outlines = font.outline_glyphs();
    let glyph_metrics = GlyphMetrics::new(&font, Size::unscaled(), LocationRef::default());
    let metrics = Metrics::new(&font, Size::unscaled(), LocationRef::default());
    // println!("metrics: {metrics:?}");

    // the maximum height of all glyphs.
    // ljt glyph_height = metrics.ascent - metrics.descent;
    // let scale = glyph_height.recip();

    const DPI: f32 = 96.0;
    let scale = DPI / (72.0 * metrics.units_per_em as f32);
    // println!("Scale used: {scale}");

    let mut point_verb = PointVerb::new();
    let mut glyph_starts = Vec::new();
    let mut bounds = Vec::new();
    let mut glyph_info = HashMap::new();

    // A default rectangle glyph
    {
        glyph_starts.push(GlyphStarts {
            point_start: 0,
            verb_start: 0,
        });
        point_verb.move_to(0., 0.);
        point_verb.line_to(0., 1.);
        point_verb.line_to(1., 1.);
        point_verb.line_to(1., 0.);
        point_verb.close();

        let bbox = BoundingBox {
            offset: Vec2::ZERO,
            size: Vec2::ONE,
        };
        bounds.push(bbox);
        glyph_info.insert(
            CpuGlyphData::RECT_CHAR,
            GlyphInfo {
                bbox,
                glyph_id: 0,
                advance: 1.0,
            },
        );
    }
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
                glyph_id: index as u32 + 1,
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

    Ok((
        GpuGlyphData {
            points: point_verb.points,
            verbs: point_verb.verbs,
            glyph_starts,
            bounds,
        },
        (CpuGlyphData {
            glyph_info,
            leading: metrics.leading * scale,
            baseline: -metrics.descent * scale,
            descent: metrics.descent * scale,
            ascent: metrics.ascent * scale,
        }),
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    pub glyph_id: u32,
    pub advance: f32,
    pub bbox: BoundingBox,
}

impl GlyphInfo {
    pub fn create_instance<I: GlyphInstance>(&self, offset: Vec2, size: Vec2) -> (BoundingBox, I) {
        (
            self.bbox.transformed_alt(offset, size),
            I::new(offset, size, self.glyph_id),
        )
    }
}

#[derive(Debug)]
pub struct GpuGlyphData {
    pub bounds: Vec<BoundingBox>,
    pub points: Vec<Vec2>,
    pub verbs: Vec<Verb>,
    pub glyph_starts: Vec<GlyphStarts>,
}

#[derive(Debug)]
pub struct CpuGlyphData {
    baseline: f32,
    leading: f32,
    ascent: f32,
    descent: f32,
    glyph_info: HashMap<char, GlyphInfo>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    pub size: Vec2,
    // pub scaling_offset: Vec2,
    pub offset: Vec2,
}

impl BoundingBox {
    pub const ZERO: Self = Self {
        size: Vec2::ZERO,
        // scaling_offset: Vec2::ZERO,
        offset: Vec2::ZERO,
    };

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn offset(&self) -> Vec2 {
        self.offset // + self.scaling_offset
    }

    pub fn min_pos(&self) -> Vec2 {
        self.offset()
    }

    pub fn max_pos(&self) -> Vec2 {
        self.offset() + self.size
    }

    pub fn center(&self) -> Vec2 {
        self.offset() + 0.5 * self.size
    }

    pub fn is_zero(&self) -> bool {
        self.size.x == 0.0 || self.size.y == 0.0
    }

    /// Scale and offset are applied separately
    pub fn transformed(self, offset: Vec2, scale: Vec2) -> Self {
        Self {
            offset: self.offset + offset,
            // scaling_offset: self.scaling_offset * scale,
            size: self.size * scale,
        }
    }

    /// Scale the entire box around the origin by `scale`, then translate by `offset`
    pub fn transformed_alt(self, offset: Vec2, scale: Vec2) -> Self {
        Self {
            offset: self.offset * scale + offset,
            size: self.size * scale,
        }
    }

    pub fn union(self, other: Self) -> Self {
        if self.is_zero() {
            other
        } else if other.is_zero() {
            self
        } else {
            let min = self.min_pos().min(other.min_pos());
            let max = self.max_pos().max(other.max_pos());
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
#[derive(Debug, Copy, Clone)]
pub struct GlyphStarts {
    pub point_start: u32,
    pub verb_start: u32,
}

impl CpuGlyphData {
    /// The first reserved character from the PUA of Unicode.
    pub const RECT_CHAR: char = '\u{f8ff}';

    pub fn baseline(&self) -> f32 {
        self.baseline
    }

    pub fn leading(&self) -> f32 {
        self.leading
    }

    pub fn ascent(&self) -> f32 {
        self.ascent
    }

    pub fn descent(&self) -> f32 {
        self.descent
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
}
