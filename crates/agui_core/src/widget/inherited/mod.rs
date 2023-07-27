mod instance;

pub use instance::*;

use super::{AnyWidget, WidgetChild};

pub trait InheritedWidget: WidgetChild {
    #[allow(unused_variables)]
    fn should_notify(&self, old_widget: &Self) -> bool {
        true
    }
}

pub trait ContextInheritedMut {
    fn depend_on_inherited_widget<I>(&mut self) -> Option<&I>
    where
        I: AnyWidget + InheritedWidget;
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use agui_macros::{InheritedWidget, StatelessWidget};

    use crate::{
        manager::WidgetManager,
        widget::{BuildContext, InheritedWidget, IntoWidget, WidgetBuild, WidgetRef},
    };

    use super::ContextInheritedMut;

    #[derive(Default)]
    struct TestResult {
        root_child: WidgetRef,

        inherited_data: Option<usize>,
    }

    thread_local! {
        static TEST_HOOK: RefCell<TestResult> = RefCell::default();
    }

    #[derive(Default, StatelessWidget)]
    struct TestRootWidget;

    impl WidgetBuild for TestRootWidget {
        type Child = WidgetRef;

        fn build(&self, _: &mut BuildContext<Self>) -> Self::Child {
            TEST_HOOK.with(|result| result.borrow().root_child.clone())
        }
    }

    #[derive(Default, InheritedWidget)]
    struct TestInheritedWidget {
        data: usize,

        #[child]
        pub child: WidgetRef,
    }

    impl InheritedWidget for TestInheritedWidget {}

    #[derive(Default, InheritedWidget)]
    struct TestOtherInheritedWidget {
        data: usize,

        #[child]
        pub child: WidgetRef,
    }

    impl InheritedWidget for TestOtherInheritedWidget {}

    #[derive(StatelessWidget, Default)]
    struct TestDependingWidget;

    impl WidgetBuild for TestDependingWidget {
        type Child = WidgetRef;

        fn build(&self, ctx: &mut BuildContext<Self>) -> Self::Child {
            let widget = ctx.depend_on_inherited_widget::<TestInheritedWidget>();

            TEST_HOOK.with(|result| {
                result.borrow_mut().inherited_data = widget.map(|w| w.data);
            });

            WidgetRef::None
        }
    }

    fn set_root_child(child: impl IntoWidget) {
        TEST_HOOK.with(|result| {
            result.borrow_mut().root_child = child.into_widget();
        });
    }

    fn assert_inherited_data(data: usize, message: &'static str) {
        TEST_HOOK.with(|result| {
            assert_eq!(result.borrow().inherited_data, Some(data), "{}", message);
        });
    }

    // TODO: Test cases:
    // - [x] Child can retrieve inherited widget ancestor
    // - [x] With multiple nested inherited widgets, the child can retrieve the nearest one
    // - [x] Child receives updates when the inherited widget changes
    // - [] When the inherited widget is removed from the tree, the child is updated
    // - [] When the inherited widget is moved in the tree but not removed, the child is updated
    // - [] When the child is keyed and reparented, it detects if its inherited widget has changed and updates if necessary
    // - [] When the child is reparented to a different inherited widget, it detects the change and updates if necessary

    #[test]
    pub fn updates_scoped_children() {
        let mut manager = WidgetManager::new();

        manager.set_root(TestRootWidget);

        let depending_widget = TestDependingWidget.into_widget();

        set_root_child(TestInheritedWidget {
            data: 7,
            child: depending_widget.clone(),
        });

        manager.update();

        assert_inherited_data(7, "should have retrieved the inherited widget");

        set_root_child(TestInheritedWidget {
            data: 9,
            child: depending_widget.clone(),
        });

        manager.mark_dirty(manager.get_root().unwrap());
        manager.update();

        assert_inherited_data(9, "should have updated the child widget");
    }

    #[test]
    pub fn updates_nested_scope_children() {
        let mut manager = WidgetManager::new();

        manager.set_root(TestRootWidget);

        let nested_scope = TestOtherInheritedWidget {
            data: 3,

            child: TestDependingWidget.into_widget(),
        }
        .into_widget();

        set_root_child(TestInheritedWidget {
            data: 7,
            child: nested_scope.clone(),
        });

        manager.update();

        assert_inherited_data(7, "should have retrieved the inherited widget");

        set_root_child(TestInheritedWidget {
            data: 9,
            child: nested_scope.clone(),
        });

        manager.mark_dirty(manager.get_root().unwrap());
        manager.update();

        assert_inherited_data(9, "should have updated the child widget");
    }
}
