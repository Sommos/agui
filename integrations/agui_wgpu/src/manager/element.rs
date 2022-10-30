use agui::{element::ElementId, render::canvas::Canvas, unit::Point};
use glyph_brush_draw_cache::ab_glyph::FontArc;
use wgpu::RenderPass;

use crate::{
    context::RenderContext,
    render::{canvas::RenderCanvas, layer::RenderLayer},
};

#[derive(Default)]
pub(crate) struct RenderElement {
    /// This is the layer that this render element belongs to
    pub head_target: Option<ElementId>,

    pub head: Option<RenderCanvas>,
    pub children: Vec<RenderLayer>,
    pub tail: Option<RenderLayer>,
}

impl RenderElement {
    pub fn update(
        &mut self,
        ctx: &mut RenderContext,
        fonts: &[FontArc],
        pos: Point,
        canvas: Canvas,
    ) {
        if canvas.head.is_empty() {
            self.head = None;
        } else if let Some(head) = &mut self.head {
            head.update(ctx, fonts, pos, &canvas.head);
        } else {
            self.head = Some(RenderCanvas::new(ctx, fonts, pos, &canvas.head));
        }
    }

    pub fn clear(&mut self) {
        self.head = None;
        self.children.clear();
        self.tail = None;
    }

    pub fn render<'r>(&'r self, r: &mut RenderPass<'r>) {
        if let Some(head) = &self.head {
            head.render(r);
        }

        // for child in &self.children {
        //     child.render(r);
        // }

        // if let Some(tail) = &self.tail {
        //     tail.render(r);
        // }
    }
}
