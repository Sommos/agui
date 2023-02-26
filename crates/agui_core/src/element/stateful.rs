use std::rc::Rc;

use crate::{
    callback::CallbackId,
    render::canvas::Canvas,
    unit::{Data, Size},
    widget::{
        instance::{IntoElementWidget, WidgetInstance},
        BuildResult, LayoutResult, WidgetRef, WidgetView,
    },
};

use super::{context::ElementContext, ElementLifecycle};

pub struct StatefulElement {
    inner: Box<dyn IntoElementWidget>,
}

impl StatefulElement {
    pub fn new<W>(widget: Rc<W>) -> Self
    where
        W: WidgetView,
    {
        Self {
            inner: Box::new(WidgetInstance::new(widget)),
        }
    }
}

impl ElementLifecycle for StatefulElement {
    fn mount(&mut self, _ctx: ElementContext) {}

    fn unmount(&mut self, _ctx: ElementContext) {}

    fn update(&mut self, other: WidgetRef) -> bool {
        self.inner.update(other)
    }

    fn layout(&mut self, ctx: ElementContext) -> LayoutResult {
        self.inner.layout(ctx)
    }

    fn build(&mut self, ctx: ElementContext) -> BuildResult {
        self.inner.build(ctx)
    }

    fn paint(&self, size: Size) -> Option<Canvas> {
        self.inner.paint(size)
    }

    fn call(&mut self, ctx: ElementContext, callback_id: CallbackId, arg: &Box<dyn Data>) -> bool {
        self.inner.call(ctx, callback_id, arg)
    }
}

impl std::ops::Deref for StatefulElement {
    type Target = dyn WidgetLifecycle;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl std::fmt::Debug for StatefulElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatefulElement")
            .field("widget", &self.inner)
            .finish()
    }
}
