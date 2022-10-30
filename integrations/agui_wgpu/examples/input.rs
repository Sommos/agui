#![allow(clippy::needless_update)]

use agui::{
    prelude::*,
    widgets::{
        primitives::{Column, Text},
        App, TextInput,
    },
};
use agui_wgpu::AguiProgram;
use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;

fn main() {
    let filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::ERROR.into())
        .add_directive(format!("agui={}", LevelFilter::INFO).parse().unwrap());

    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::time())
        .with_level(true)
        .with_thread_names(false)
        .with_target(true)
        .with_env_filter(filter)
        .init();

    let mut ui = AguiProgram::new(
        "agui input",
        Size {
            width: 800.0,
            height: 600.0,
        },
    );

    // ui.register_default_plugins();
    // ui.register_default_globals();

    let deja_vu = ui
        .load_font_bytes(include_bytes!("./fonts/DejaVuSans.ttf"))
        .unwrap();

    ui.set_root(App {
        child: ExampleMain { font: deja_vu }.into(),
    });

    ui.run()
}

#[derive(StatefulWidget, PartialEq)]
struct ExampleMain {
    font: Font,
}

impl WidgetState for ExampleMain {
    type State = String;

    fn create_state(&self) -> Self::State {
        String::new()
    }
}

impl WidgetView for ExampleMain {
    fn layout(&self, _: &mut LayoutContext<Self>) -> LayoutResult {
        LayoutResult {
            layout_type: LayoutType::default(),

            layout: Layout {
                sizing: Sizing::Fill,
                ..Layout::default()
            },
        }
    }

    fn build(&self, ctx: &mut BuildContext<Self>) -> BuildResult {
        let on_value = ctx.callback::<String, _>(|ctx, input: &String| {
            ctx.set_state(|state| *state = input.clone());
        });

        BuildResult::new(build! {
            Column {
                layout: Layout {
                    sizing: Sizing::Axis {
                        width: Units::Stretch(1.0),
                        height: Units::Stretch(1.0)
                    },
                    margin: Margin::center()
                },
                spacing: Units::Pixels(8.0),
                children: [
                    TextInput {
                        layout: Layout {
                            sizing: Sizing::Axis {
                                width: Units::Stretch(1.0),
                                height: Units::Pixels(32.0),
                            }
                        },

                        font: self.font.styled(),
                        placeholder: "some text here",

                        on_value
                    },
                    Text {
                        font: self.font.styled().color(Color::from_rgb((1.0, 1.0, 1.0))),
                        text: ctx.state.clone(),
                    },
                ]
            }
        })
    }
}
