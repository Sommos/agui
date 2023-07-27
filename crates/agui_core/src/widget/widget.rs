use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

use super::{element::WidgetElement, AnyWidget, WidgetKey};

#[derive(Clone)]
pub struct Widget {
    key: Option<WidgetKey>,
    widget: Rc<dyn AnyWidget>,
}

impl Widget {
    pub fn new<W>(widget: W) -> Self
    where
        W: AnyWidget,
    {
        Self::new_with_key(None, widget)
    }

    pub fn new_with_key<W>(key: Option<WidgetKey>, widget: W) -> Self
    where
        W: AnyWidget,
    {
        Self {
            key,
            widget: Rc::new(widget),
        }
    }

    pub fn widget_name(&self) -> &str {
        (*self.widget).widget_name()
    }

    pub fn get_key(&self) -> Option<WidgetKey> {
        self.key
    }

    pub fn downcast<W>(&self) -> Option<Rc<W>>
    where
        W: AnyWidget,
    {
        Rc::clone(&self.widget).as_any().downcast::<W>().ok()
    }

    pub fn is<W>(&self) -> bool
    where
        W: AnyWidget,
    {
        Rc::clone(&self.widget).as_any().is::<W>()
    }

    pub(crate) fn create_element(&self) -> Box<dyn WidgetElement> {
        Rc::clone(&self.widget).create_element()
    }
}

impl PartialEq for Widget {
    fn eq(&self, other: &Self) -> bool {
        if self.key.is_some() || other.key.is_some() {
            return self.key == other.key;
        }

        // war crimes
        std::ptr::eq(
            Rc::as_ptr(&self.widget) as *const _ as *const (),
            Rc::as_ptr(&other.widget) as *const _ as *const (),
        )
    }
}

impl Eq for Widget {}

impl Hash for Widget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(key) = self.key {
            key.hash(state);

            return;
        }

        // more war crimes
        std::ptr::hash(Rc::as_ptr(&self.widget) as *const _ as *const (), state);
    }
}

impl std::fmt::Debug for Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.widget_name())?;

        if let Some(key) = self.key {
            f.write_str(" <key: ")?;
            key.fmt(f)?;
            f.write_str(">")?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.widget_name())?;

        if let Some(key) = self.key {
            f.write_str(" <key: ")?;
            key.fmt(f)?;
            f.write_str(">")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{ptr, rc::Rc};

    use agui_macros::StatelessWidget;

    use crate::widget::{BuildContext, IntoWidget, WidgetBuild};

    #[derive(StatelessWidget)]
    struct TestWidget;

    impl WidgetBuild for TestWidget {
        type Child = ();

        fn build(&self, _: &mut BuildContext<Self>) -> Self::Child {}
    }

    #[test]
    fn strip_fat_ptr_equality() {
        let widget1 = TestWidget.into_widget();
        let widget2 = TestWidget.into_widget();

        // These equality checks are theoretically unstable
        #[allow(clippy::vtable_address_comparisons)]
        {
            assert!(
                !Rc::ptr_eq(&widget1.widget, &widget2.widget),
                "Rc::ptr_eq(widget1, widget2) should never be equal, but is theoretically unstable"
            );

            assert!(
                Rc::ptr_eq(&widget1.widget, &widget1.widget),
                "Rc::ptr_eq(widget1, &widget1) should always be equal, but is theoretically unstable"
            );
        }

        // Black magic fuckery to remove the vtable from the pointer
        assert!(
            !ptr::eq(
                Rc::as_ptr(&widget1.widget) as *const _ as *const (),
                Rc::as_ptr(&widget2.widget) as *const _ as *const ()
            ),
            "ptr::eq(widget1, widget2) should never be equal"
        );

        assert!(
            ptr::eq(
                Rc::as_ptr(&widget1.widget) as *const _ as *const (),
                Rc::as_ptr(&widget1.widget) as *const _ as *const ()
            ),
            "ptr::eq(widget1, widget2) should always be equal"
        );

        // Therefore, this should be stable
        assert_ne!(
            widget1, widget2,
            "widget1 should should never be equal to widget2"
        );

        assert_eq!(
            widget1, widget1,
            "widget1 should should always be equal to itself"
        );
    }
}
