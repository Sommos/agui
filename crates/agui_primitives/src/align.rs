use agui_core::{
    unit::{Alignment, Constraints, Size},
    widget::{BuildContext, ContextWidgetLayout, LayoutContext, WidgetRef, WidgetView},
};
use agui_macros::StatelessWidget;

#[derive(StatelessWidget, Debug, Default)]
pub struct Align {
    pub alignment: Alignment,

    pub width_factor: Option<f32>,
    pub height_factor: Option<f32>,

    pub child: WidgetRef,
}

impl WidgetView for Align {
    type Child = WidgetRef;

    fn layout(&self, ctx: &mut LayoutContext<Self>, constraints: Constraints) -> Size {
        let children = ctx.get_children();

        let shrink_wrap_width =
            self.width_factor.is_some() || constraints.max_width == f32::INFINITY;

        let shrink_wrap_height =
            self.height_factor.is_some() || constraints.max_height == f32::INFINITY;

        if !children.is_empty() {
            let child_id = *children.first().unwrap();

            let child_size = ctx.compute_layout(child_id, constraints.loosen());

            let size = constraints.constrain(Size {
                width: shrink_wrap_width
                    .then(|| child_size.width * self.width_factor.unwrap_or(1.0))
                    .unwrap_or(f32::INFINITY),

                height: shrink_wrap_height
                    .then(|| child_size.height * self.height_factor.unwrap_or(1.0))
                    .unwrap_or(f32::INFINITY),
            });

            ctx.set_offset(0, self.alignment.along_size(size - child_size));

            size
        } else {
            constraints.constrain(Size {
                width: if shrink_wrap_width {
                    0.0
                } else {
                    f32::INFINITY
                },

                height: if shrink_wrap_height {
                    0.0
                } else {
                    f32::INFINITY
                },
            })
        }
    }

    fn build(&self, _: &mut BuildContext<Self>) -> Self::Child {
        self.child.clone()
    }
}
