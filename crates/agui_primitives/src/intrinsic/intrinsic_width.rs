use agui_core::{unit::Axis, widget::Widget};
use agui_elements::stateless::{StatelessBuildContext, StatelessWidget};
use agui_macros::{build, StatelessWidget};

use crate::intrinsic::IntrinsicAxis;

/// See [`IntrinsicAxis`] for more information.
#[derive(StatelessWidget, Debug)]
pub struct IntrinsicWidth {
    #[prop(into)]
    pub child: Option<Widget>,
}

impl StatelessWidget for IntrinsicWidth {
    fn build(&self, _: &mut StatelessBuildContext<Self>) -> Widget {
        build! {
            <IntrinsicAxis> {
                axis: Axis::Horizontal,
                child: self.child.clone(),
            }
        }
    }
}
