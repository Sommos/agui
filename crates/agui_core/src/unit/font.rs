use glyph_brush_layout::{
    ab_glyph::{Font as GlyphFont, FontArc, GlyphId, ScaleFont},
    BuiltInLineBreaker, FontId as GlyphFontId, GlyphPositioner, Layout as GlyphLayout,
    SectionGeometry, SectionGlyph, SectionText,
};

use crate::unit::{Color, Rect};

#[derive(Debug, Clone, Default)]
pub struct Font(pub(crate) usize, pub(crate) Option<FontArc>);

impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Font {
    pub fn get(&self) -> Option<&FontArc> {
        self.1.as_ref()
    }

    pub fn styled(&self) -> FontStyle {
        FontStyle {
            font: self.clone(),
            color: Color {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },

            ..FontStyle::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontStyle {
    pub font: Font,

    pub size: f32,
    pub color: Color,

    pub h_align: HorizontalAlign,
    pub v_align: VerticalAlign,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            font: Font(0, None),
            size: 16.0,
            color: Color {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },

            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Top,
        }
    }
}

impl FontStyle {
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn h_align(mut self, h_align: HorizontalAlign) -> Self {
        self.h_align = h_align;
        self
    }

    pub fn v_align(mut self, v_align: VerticalAlign) -> Self {
        self.v_align = v_align;
        self
    }

    pub fn get(&self) -> Option<&FontArc> {
        self.font.get()
    }

    pub fn h_advance(&self, glyph_id: GlyphId) -> f32 {
        self.get()
            .map(|font| font.as_scaled(self.size).h_advance(glyph_id))
            .unwrap_or(0.0)
    }

    pub fn v_advance(&self, glyph_id: GlyphId) -> f32 {
        self.get()
            .map(|font| font.as_scaled(self.size).v_advance(glyph_id))
            .unwrap_or(0.0)
    }

    pub fn get_glyphs(&self, mut rect: Rect, text: &str) -> Vec<SectionGlyph> {
        if text.is_empty() {
            return Vec::new();
        }

        self.font.get().map_or_else(Vec::default, |font| {
            let glyphs_layout = GlyphLayout::Wrap {
                line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                h_align: match self.h_align {
                    HorizontalAlign::Left => glyph_brush_layout::HorizontalAlign::Left,
                    HorizontalAlign::Center => {
                        rect.left += rect.width / 2.0;

                        glyph_brush_layout::HorizontalAlign::Center
                    }

                    HorizontalAlign::Right => {
                        rect.left += rect.width;

                        glyph_brush_layout::HorizontalAlign::Right
                    }
                },
                v_align: match self.v_align {
                    VerticalAlign::Top => glyph_brush_layout::VerticalAlign::Top,
                    VerticalAlign::Center => {
                        rect.top += rect.height / 2.0;

                        glyph_brush_layout::VerticalAlign::Center
                    }

                    VerticalAlign::Bottom => {
                        rect.top += rect.height;

                        glyph_brush_layout::VerticalAlign::Bottom
                    }
                },
            };

            glyphs_layout.calculate_glyphs(
                &[font],
                &SectionGeometry {
                    screen_position: (rect.left, rect.top),
                    bounds: (rect.width, rect.height),
                },
                &[SectionText {
                    text,
                    scale: self.size.into(),
                    font_id: GlyphFontId(0),
                }],
            )
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

impl Default for HorizontalAlign {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

impl Default for VerticalAlign {
    fn default() -> Self {
        Self::Top
    }
}
