use std::{any::Any, rc::Rc};

mod context;
pub mod element;
mod inherited;
mod key;
mod layout;
mod paint;
mod stateful;
mod stateless;
#[allow(clippy::module_inception)]
mod widget;

pub use self::{
    context::*, inherited::*, key::*, layout::*, paint::*, stateful::*, stateless::*, widget::*,
};

pub trait ElementBuilder: 'static {
    fn create_element(self: Rc<Self>) -> Box<dyn self::element::WidgetElement>;
}

pub trait AnyWidget: ElementBuilder {
    fn as_any(self: Rc<Self>) -> Rc<dyn Any>;

    fn widget_name(&self) -> &'static str;
}

impl<T> AnyWidget for T
where
    T: ElementBuilder,
{
    fn as_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn widget_name(&self) -> &'static str {
        let type_name = std::any::type_name::<T>();

        type_name
            .split('<')
            .next()
            .unwrap_or(type_name)
            .split("::")
            .last()
            .unwrap_or(type_name)
    }
}

pub trait IntoWidget {
    fn into_widget(self) -> Widget;
}

impl<W> From<W> for Widget
where
    W: IntoWidget + ElementBuilder,
{
    fn from(widget: W) -> Self {
        Widget::new(widget)
    }
}

pub trait IntoChild {
    fn into_child(self) -> Option<Widget>;
}

impl IntoChild for () {
    fn into_child(self) -> Option<Widget> {
        None
    }
}

impl IntoChild for Widget {
    fn into_child(self) -> Option<Widget> {
        Some(self)
    }
}

impl IntoChild for Option<Widget> {
    fn into_child(self) -> Option<Widget> {
        self
    }
}

impl<W> IntoChild for Option<W>
where
    W: AnyWidget,
{
    fn into_child(self) -> Option<Widget> {
        self.map(Widget::new)
    }
}

impl<W> IntoChild for W
where
    W: AnyWidget,
{
    fn into_child(self) -> Option<Widget> {
        Some(Widget::new(self))
    }
}
