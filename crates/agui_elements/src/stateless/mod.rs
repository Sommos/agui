use agui_core::widget::Widget;

mod context;
mod instance;

pub use context::*;
pub use instance::*;

pub trait StatelessWidget: Sized + 'static {
    /// Called whenever this widget is rebuilt.
    ///
    /// This method may be called when any parent is rebuilt or when its internal state changes.
    fn build(&self, ctx: &mut StatelessBuildContext<Self>) -> Widget;
}
