use std::any::TypeId;

use crate::{
    widget::{BuildResult, Layout, Widget},
    WidgetContext,
};

pub struct Builder<F>
where
    F: Fn(&WidgetContext) -> BuildResult + 'static,
{
    func: F,
}

impl<F> Builder<F>
where
    F: Fn(&WidgetContext) -> BuildResult + 'static,
{
    pub fn new(func: F) -> Builder<F> {
        Builder { func }
    }
}

impl<F> Widget for Builder<F>
where
    F: Fn(&WidgetContext) -> BuildResult + 'static,
{
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn layout(&self) -> Option<&Layout> {
        None
    }

    fn build(&self, ctx: &WidgetContext) -> BuildResult {
        (self.func)(ctx)
    }
}

impl<F> From<Builder<F>> for Box<dyn Widget>
where
    F: Fn(&WidgetContext) -> BuildResult + 'static,
{
    fn from(builder: Builder<F>) -> Box<dyn Widget> {
        Box::new(builder)
    }
}

impl<F> From<Builder<F>> for Option<Box<dyn Widget>>
where
    F: Fn(&WidgetContext) -> BuildResult + 'static,
{
    fn from(builder: Builder<F>) -> Option<Box<dyn Widget>> {
        Some(Box::new(builder))
    }
}