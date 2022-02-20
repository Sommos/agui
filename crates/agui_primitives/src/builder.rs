use agui_core::widget::{BuildResult, WidgetBuilder, BuildContext, WidgetRef};
use agui_macros::Widget;

#[derive(Widget)]
#[widget(into = false)]
pub struct Builder<F>
where
    F: Fn(&mut BuildContext) -> BuildResult + 'static,
{
    func: F,
}

impl<F> std::fmt::Debug for Builder<F>
where
    F: Fn(&mut BuildContext) -> BuildResult + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builder").finish()
    }
}

impl<F> Builder<F>
where
    F: Fn(&mut BuildContext) -> BuildResult + 'static,
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> WidgetBuilder for Builder<F>
where
    F: Fn(&mut BuildContext) -> BuildResult + 'static,
{
    fn build(&self, ctx: &mut BuildContext) -> BuildResult {
        (self.func)(ctx)
    }
}

impl<F> From<Builder<F>> for WidgetRef
where
    F: Fn(&mut BuildContext) -> BuildResult + 'static,
{
    fn from(builder: Builder<F>) -> Self {
        Self::new(builder)
    }
}
