use std::{any::TypeId, collections::HashMap};

use agpu::{BindGroup, Buffer, Frame, GpuProgram, RenderPipeline};
use agui::{
    unit::{Color, Rect},
    widget::WidgetId,
    WidgetManager,
};
use generational_arena::{Arena, Index as GenerationalIndex};

use super::{RenderContext, WidgetRenderPass};

pub struct BoundingRenderPass {
    bind_group: BindGroup,

    pipeline: RenderPipeline,
    buffer: Buffer,

    locations: Arena<WidgetId>,
    widgets: HashMap<WidgetId, GenerationalIndex>,
}

const RECT_BUFFER_SIZE: u64 = std::mem::size_of::<[f32; 4]>() as u64;
const COLOR_BUFFER_SIZE: u64 = std::mem::size_of::<[f32; 4]>() as u64;
const QUAD_BUFFER_SIZE: u64 = RECT_BUFFER_SIZE + COLOR_BUFFER_SIZE;

const PREALLOCATE: u64 = QUAD_BUFFER_SIZE * 16;

// Make room for extra quads when we reach the buffer size, so we have to resize less often
const EXPAND_ALLOCATE: u64 = QUAD_BUFFER_SIZE * 8;

const UNCHANGED_COLOR: [f32; 4] = Color::Green.as_rgba();
// const CHANGED_COLOR: [f32; 4] = Color::Red.as_rgba();

impl BoundingRenderPass {
    pub fn new(program: &GpuProgram, ctx: &RenderContext) -> Self {
        let bindings = &[ctx.bind_app_settings()];

        let bind_group = program.gpu.create_bind_group(bindings);

        let pipeline = program
            .gpu
            .new_pipeline("agui_bounding_pipeline")
            .with_vertex(include_bytes!("shader/bounding.vert.spv"))
            .with_fragment(include_bytes!("shader/rect.frag.spv"))
            .with_vertex_layouts(&[agpu::wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 8]>() as u64,
                step_mode: agpu::wgpu::VertexStepMode::Instance,
                attributes: &agpu::wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4],
            }])
            .wireframe()
            .with_bind_groups(&[&bind_group.layout])
            .create();

        Self {
            bind_group,
            pipeline,

            buffer: program
                .gpu
                .new_buffer("BoundingRenderPass")
                .as_vertex_buffer()
                .allow_copy()
                .create_uninit(PREALLOCATE),

            locations: Arena::default(),
            widgets: HashMap::default(),
        }
    }
}

impl WidgetRenderPass for BoundingRenderPass {
    fn added(
        &mut self,
        _ctx: &RenderContext,
        _manager: &WidgetManager,
        _type_id: &TypeId,
        widget_id: &WidgetId,
    ) {
        let index = self.locations.insert(*widget_id);
        self.widgets.insert(*widget_id, index);
    }

    fn layout(
        &mut self,
        ctx: &RenderContext,
        _manager: &WidgetManager,
        _type_id: &TypeId,
        widget_id: &WidgetId,
        rect: &Rect,
    ) {
        let index = match self.widgets.get(widget_id) {
            Some(widget) => widget,
            None => return,
        };

        let index = index.into_raw_parts().0 as u64;

        let index = index * QUAD_BUFFER_SIZE;

        let rect = rect.to_slice();

        let rect = bytemuck::cast_slice(&rect);
        let rgba = bytemuck::cast_slice(&UNCHANGED_COLOR);

        if (self.buffer.size() as u64) < index + QUAD_BUFFER_SIZE {
            self.buffer
                .resize((self.buffer.size() as u64) + EXPAND_ALLOCATE);
        }

        ctx.gpu.queue.write_buffer(&self.buffer, index, rect);
        ctx.gpu
            .queue
            .write_buffer(&self.buffer, index + RECT_BUFFER_SIZE, rgba);
    }

    fn removed(
        &mut self,
        _ctx: &RenderContext,
        _manager: &WidgetManager,
        _type_id: &TypeId,
        widget_id: &WidgetId,
    ) {
        if let Some(index) = self.widgets.remove(widget_id) {
            self.locations.remove(index);
        }
    }

    fn render(&self, _ctx: &RenderContext, frame: &mut Frame) {
        let mut r = frame
            .render_pass("bounding render pass")
            .with_pipeline(&self.pipeline)
            .begin();

        r.set_bind_group(0, &self.bind_group, &[]);

        for (index, _) in self.locations.iter() {
            let index = (index.into_raw_parts().0 as u64) * QUAD_BUFFER_SIZE;

            r.set_vertex_buffer(0, self.buffer.slice(index..(index + QUAD_BUFFER_SIZE)))
                .draw(0..6, 0..1);
        }
    }
}
