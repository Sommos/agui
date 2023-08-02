use agui_core::{
    unit::{Constraints, IntrinsicDimension, Size},
    widget::{BuildContext, IntrinsicSizeContext, LayoutContext, Widget, WidgetLayout},
};
use agui_macros::LayoutWidget;

#[derive(LayoutWidget, Debug, Default)]
pub struct Stack {
    pub children: Vec<Widget>,
}

impl WidgetLayout for Stack {
    type Children = Widget;

    fn build(&self, _: &mut BuildContext<Self>) -> Vec<Self::Children> {
        Vec::from_iter(self.children.iter().cloned())
    }

    // TODO: make this actually work properly
    fn intrinsic_size(&self, _: &mut IntrinsicSizeContext, _: IntrinsicDimension, _: f32) -> f32 {
        0.0
    }

    // TODO: make this actually work properly
    fn layout(&self, ctx: &mut LayoutContext, constraints: Constraints) -> Size {
        let mut children = ctx.iter_children_mut();

        let mut size = constraints.biggest();

        while let Some(mut child) = children.next() {
            size = child.compute_layout(constraints);
        }

        size
    }
}