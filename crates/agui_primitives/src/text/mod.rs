use std::borrow::Cow;

use agui_core::{
    render::canvas::paint::Paint,
    unit::{FontStyle, Layout, Sizing},
    widget::{BuildContext, BuildResult, WidgetBuilder},
};

pub mod edit;
pub mod query;

#[derive(Debug, Default)]
pub struct Text {
    pub font: FontStyle,
    pub text: Cow<'static, str>,
}

impl WidgetBuilder for Text {
    fn build(&self, ctx: &mut BuildContext<Self>) -> BuildResult {
        ctx.set_layout(Layout {
            sizing: Sizing::Fill,
            min_sizing: Sizing::Axis {
                width: 0.0.into(),
                height: self.font.size.into(),
            },
            ..Layout::default()
        });

        ctx.on_draw(|ctx, canvas| {
            canvas.draw_text(
                &Paint {
                    color: ctx.font.color,
                    ..Paint::default()
                },
                ctx.font.clone(),
                Cow::clone(&ctx.text),
            );
        });

        BuildResult::None
    }
}
