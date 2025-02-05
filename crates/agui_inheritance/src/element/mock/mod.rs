use std::{any::Any, cell::RefCell, rc::Rc};

use agui_core::{
    callback::CallbackId,
    element::{
        build::ElementBuild, widget::ElementWidget, ElementBuildContext, ElementBuilder,
        ElementCallbackContext, ElementMountContext, ElementType, ElementUnmountContext,
        ElementUpdate,
    },
    widget::{IntoWidget, Widget},
};

#[allow(clippy::disallowed_types)]
#[mockall::automock]
#[allow(clippy::needless_lifetimes)]
pub trait InheritedElement {
    fn widget_name(&self) -> &'static str;

    #[allow(unused_variables)]
    fn mount<'ctx>(&mut self, ctx: ElementMountContext<'ctx>);

    #[allow(unused_variables)]
    fn unmount<'ctx>(&mut self, ctx: ElementUnmountContext<'ctx>);

    fn update(&mut self, new_widget: &Widget) -> ElementUpdate;

    fn build<'ctx>(&mut self, ctx: ElementBuildContext<'ctx>) -> Widget;

    #[allow(unused_variables)]
    fn call<'ctx>(
        &mut self,
        ctx: ElementCallbackContext<'ctx>,
        callback_id: CallbackId,
        arg: Box<dyn Any>,
    ) -> bool;
}

#[derive(Default)]
pub struct MockInheritedWidget {
    pub mock: Rc<RefCell<MockInheritedElement>>,
}

impl MockInheritedWidget {
    pub fn new(mock: MockInheritedElement) -> Self {
        Self {
            mock: Rc::new(RefCell::new(mock)),
        }
    }
}

impl IntoWidget for MockInheritedWidget {
    fn into_widget(self) -> Widget {
        Widget::new(self)
    }
}

impl ElementBuilder for MockInheritedWidget {
    fn create_element(self: Rc<Self>) -> ElementType {
        ElementType::Widget(Box::new(MockElement::new(self)))
    }
}

struct MockElement {
    widget: Rc<MockInheritedWidget>,
}

impl MockElement {
    pub fn new(widget: Rc<MockInheritedWidget>) -> Self {
        Self { widget }
    }
}

impl ElementWidget for MockElement {
    fn widget_name(&self) -> &'static str {
        self.widget.mock.borrow().widget_name()
    }

    fn update(&mut self, new_widget: &Widget) -> ElementUpdate {
        self.widget.mock.borrow_mut().update(new_widget)
    }
}

impl ElementBuild for MockElement {
    fn build(&mut self, ctx: ElementBuildContext) -> Widget {
        self.widget.mock.borrow_mut().build(ctx)
    }

    fn call(
        &mut self,
        ctx: ElementCallbackContext,
        callback_id: CallbackId,
        arg: Box<dyn Any>,
    ) -> bool {
        self.widget.mock.borrow_mut().call(ctx, callback_id, arg)
    }
}
