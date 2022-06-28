use std::rc::Rc;

use downcast_rs::Downcast;

use crate::manager::Data;

use super::{BoxedWidget, BuildContext, BuildResult, IntoWidget, WidgetElement};

/// Implements the widget's `build()` method.
pub trait WidgetBuilder: std::fmt::Debug + Downcast + Sized {
    type State: Data + Default = ();

    /// Called whenever this widget is rebuilt.
    ///
    /// This method may be called when any parent is rebuilt or when its internal state changes.
    fn build(&self, ctx: &mut BuildContext<Self>) -> BuildResult;
}

impl<W, S> IntoWidget for W
where
    W: WidgetBuilder<State = S>,
    S: Data + Default,
{
    default fn into_widget(self: Rc<Self>) -> BoxedWidget {
        Box::new(WidgetElement::new(self))
    }
}

impl<W> IntoWidget for W
where
    W: WidgetBuilder<State = ()>,
{
    fn into_widget(self: Rc<Self>) -> BoxedWidget {
        Box::new(WidgetElement::new(self))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        manager::{context::Context, query::WidgetQueryExt, WidgetManager},
        widget::{BuildContext, BuildResult},
    };

    use super::WidgetBuilder;

    #[derive(Debug, Default, Clone, Copy)]
    struct TestGlobal(i32);

    #[derive(Debug, Default)]
    struct TestWidget {}

    impl WidgetBuilder for TestWidget {
        type State = u64;

        fn build(&self, ctx: &mut BuildContext<Self>) -> BuildResult {
            ctx.set_state(|state| {
                *state += 1;
            });

            BuildResult::None
        }
    }

    #[test]
    pub fn widget_build_can_set_state() {
        let mut manager = WidgetManager::with_root(TestWidget::default());

        manager.update();

        assert_eq!(
            *manager
                .query()
                .by_type::<TestWidget>()
                .next()
                .unwrap()
                .get_state(),
            1,
            "widget `u32` should be 1"
        );

        manager.update();

        assert_eq!(
            *manager
                .query()
                .by_type::<TestWidget>()
                .next()
                .unwrap()
                .get_state(),
            1,
            "widget `u32` should still be 1"
        );
    }
}