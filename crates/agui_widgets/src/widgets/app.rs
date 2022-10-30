use agui_core::{
    unit::{Layout, LayoutType, Sizing, Units},
    widget::{BuildContext, BuildResult, LayoutContext, LayoutResult, WidgetRef, WidgetView},
};
use agui_macros::StatelessWidget;

use crate::state::window::WindowSize;

#[derive(StatelessWidget, Default, PartialEq)]
pub struct App {
    pub child: WidgetRef,
}

impl WidgetView for App {
    fn layout(&self, _ctx: &mut LayoutContext<Self>) -> LayoutResult {
        let window_size = WindowSize {
            width: 800.0,
            height: 600.0,
        }; //ctx.get_global::<WindowSize>();

        // let window_size = window_size.borrow();

        LayoutResult {
            layout_type: LayoutType::default(),

            layout: Layout {
                sizing: Sizing::Axis {
                    width: Units::Pixels(window_size.width),
                    height: Units::Pixels(window_size.height),
                },

                ..Layout::default()
            },
        }
    }

    fn build(&self, _: &mut BuildContext<Self>) -> BuildResult {
        BuildResult::from(&self.child)
    }
}
