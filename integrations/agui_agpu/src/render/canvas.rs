use agpu::{Buffer, RenderPass};
use agui::{
    render::{canvas::command::CanvasCommand, texture::TextureId},
    unit::{Point, Rect},
};
use glyph_brush_draw_cache::ab_glyph::FontArc;

use crate::context::RenderContext;

use super::{
    builder::{shape::LayerShapeBuilder, text::TextDrawCallBuilder, DrawCallBuilder},
    draw_call::DrawCall,
};

pub(crate) struct RenderCanvas {
    pub rect: Rect,

    pub pos: Buffer,
    pub draw_calls: Vec<DrawCall>,
}

impl RenderCanvas {
    pub fn new(
        ctx: &mut RenderContext,
        fonts: &[FontArc],
        rect: Rect,
        commands: &[CanvasCommand],
    ) -> Self {
        let mut canvas = Self {
            rect,

            pos: ctx
                .gpu
                .new_buffer("agui canvas position buffer")
                .as_vertex_buffer()
                .create(&[rect.x, rect.y]),

            draw_calls: Vec::default(),
        };

        canvas.build(ctx, fonts, commands);

        canvas
    }

    pub fn update(
        &mut self,
        ctx: &mut RenderContext,
        fonts: &[FontArc],
        rect: Rect,
        commands: &[CanvasCommand],
    ) {
        // Update the position if necessary
        if Point::from(self.rect) != Point::from(rect) {
            self.rect = rect;

            self.pos = ctx
                .gpu
                .new_buffer("agui canvas position buffer")
                .as_vertex_buffer()
                .create(&[rect.x, rect.y]);
        }

        self.build(ctx, fonts, commands);
    }

    fn build(&mut self, ctx: &mut RenderContext, fonts: &[FontArc], commands: &[CanvasCommand]) {
        self.draw_calls.clear();

        let mut draw_call_builder: Option<Box<dyn DrawCallBuilder>> = None;

        for cmd in commands {
            // Check if the current layer builder can process the command, and finalize the build if not
            if let Some(builder) = draw_call_builder.as_ref() {
                if !builder.can_process(cmd) {
                    // Add the draw call to the current layer

                    if let Some(draw_call) = builder.build(ctx) {
                        self.draw_calls.push(draw_call);
                    }

                    draw_call_builder = None;
                }
            }

            match cmd {
                CanvasCommand::Shape { .. } => {
                    if draw_call_builder.is_none() {
                        draw_call_builder =
                            Some(Box::new(LayerShapeBuilder::new(TextureId::default())));
                    }
                }

                CanvasCommand::Texture { texture_id, .. } => {
                    if draw_call_builder.is_none() {
                        draw_call_builder = Some(Box::new(LayerShapeBuilder::new(*texture_id)));
                    }
                }

                CanvasCommand::Text { .. } => {
                    if draw_call_builder.is_none() {
                        draw_call_builder = Some(Box::new(TextDrawCallBuilder {
                            fonts,

                            ..TextDrawCallBuilder::default()
                        }));
                    }
                }

                cmd => {
                    tracing::error!("unknown command: {:?}", cmd);

                    continue;
                }
            }

            draw_call_builder.as_mut().unwrap().process(cmd);
        }

        if let Some(builder) = draw_call_builder.take() {
            self.draw_calls.extend(builder.build(ctx));
        }
    }

    pub fn render<'pass>(&'pass self, r: &mut RenderPass<'pass>) {
        r.set_vertex_buffer(0, self.pos.slice(..));

        for draw_call in &self.draw_calls {
            draw_call.render(r);
        }
    }
}
