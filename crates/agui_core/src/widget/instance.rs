use std::any::TypeId;

use downcast_rs::{impl_downcast, Downcast};

use crate::{
    callback::CallbackId,
    manager::{context::AguiContext, Data},
    render::canvas::painter::CanvasPainter,
    unit::{Layout, LayoutType, Rect},
};

use super::{BuildResult, WidgetKey};

pub trait WidgetInstance: std::fmt::Debug + Downcast {
    fn get_type_id(&self) -> TypeId;
    fn get_display_name(&self) -> String;

    fn get_key(&self) -> Option<WidgetKey>;
    fn set_key(&mut self, key: WidgetKey);

    fn get_layout_type(&self) -> Option<LayoutType>;
    fn get_layout(&self) -> Option<Layout>;

    fn set_rect(&mut self, rect: Option<Rect>);
    fn get_rect(&self) -> Option<Rect>;

    fn build(&mut self, ctx: AguiContext) -> BuildResult;

    fn call(&mut self, ctx: AguiContext, callback_id: CallbackId, arg: &dyn Data) -> bool;

    fn render(&self, canvas: &mut CanvasPainter);
}

impl_downcast!(WidgetInstance);