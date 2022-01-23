// use glyph_brush_layout::{
//     BuiltInLineBreaker, FontId, GlyphPositioner, SectionGeometry, SectionText, ToSectionText,
// };

pub use glyph_brush_layout::ab_glyph::{Font, FontArc, ScaleFont};
pub use glyph_brush_layout::Layout as GlyphLayout;
pub use glyph_brush_layout::{HorizontalAlign, SectionGlyph, VerticalAlign};

use crate::unit::Color;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontId(pub(crate) usize);

impl FontId {
    pub fn styled(&self) -> FontStyle {
        FontStyle {
            font_id: *self,
            ..FontStyle::default()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FontStyle {
    pub font_id: FontId,
    pub font_size: f32,
    pub color: Color,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            font_id: FontId(0),
            font_size: 32.0,
            color: Color::Black,
        }
    }
}

impl FontStyle {
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}
