use agui_core::{
    unit::{Axis, ClipBehavior, TextDirection},
    widget::{IntoWidget, Widget},
};
use agui_macros::WidgetProps;

use crate::flex::{
    child::FlexChild, CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize, VerticalDirection,
};

#[derive(Debug, WidgetProps)]
#[props(default)]
pub struct Row {
    pub main_axis_size: MainAxisSize,

    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub vertical_direction: VerticalDirection,

    pub text_direction: Option<TextDirection>,

    pub clip_behavior: ClipBehavior,

    #[prop(into, transform = |widgets: impl IntoIterator<Item = Widget>| widgets.into_iter().map(FlexChild::from).collect())]
    pub children: Vec<FlexChild>,
}

impl IntoWidget for Row {
    fn into_widget(self) -> Widget {
        Flex {
            direction: Axis::Vertical,

            main_axis_size: self.main_axis_size,

            main_axis_alignment: self.main_axis_alignment,
            cross_axis_alignment: self.cross_axis_alignment,
            vertical_direction: self.vertical_direction,

            text_direction: self.text_direction,

            clip_behavior: self.clip_behavior,

            children: self.children.clone(),
        }
        .into_widget()
    }
}
