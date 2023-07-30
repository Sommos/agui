use agui_core::{
    unit::Axis,
    widget::{BuildContext, Widget, WidgetBuild},
};
use agui_macros::{build, StatelessWidget};

use crate::IntrinsicAxis;

/// See [`IntrinsicAxis`] for more information.
#[derive(StatelessWidget, Debug, Default)]
pub struct IntrinsicWidth {
    pub child: Option<Widget>,
}

impl WidgetBuild for IntrinsicWidth {
    type Child = Widget;

    fn build(&self, _: &mut BuildContext<Self>) -> Self::Child {
        build! {
            IntrinsicAxis {
                axis: Axis::Horizontal,
                child: self.child.clone(),
            }
        }
    }
}