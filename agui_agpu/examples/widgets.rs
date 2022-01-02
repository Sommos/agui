#![allow(clippy::needless_update)]

use agpu::Features;
use agui::{
    context::WidgetContext,
    layout::Layout,
    macros::{build, functional_widget},
    unit::{Callback, Color, Margin, Sizing, Units},
    widget::{BuildResult, WidgetRef},
    widgets::{
        plugins::{hovering::HoveringPlugin, provider::ProviderExt},
        primitives::{Builder, Column, Padding, Quad, QuadStyle, Spacing, Text},
        state::{
            hovering::Hovering,
            keyboard::{Keyboard, KeyboardInput},
            mouse::{Mouse, Scroll},
            theme::Theme,
        },
        App, Button, ButtonStyle,
    },
    Ref,
};
use agui_agpu::UI;

fn main() -> Result<(), agpu::BoxError> {
    let program = agpu::GpuProgram::builder("agui widgets")
        .with_gpu_features(Features::POLYGON_MODE_LINE)
        .with_framerate(60.0)
        .build()?;

    let mut ui = UI::with_default(&program);

    ui.get_context().init_plugin(HoveringPlugin::default);

    ui.get_context().init_global(Keyboard::default);
    ui.get_context().init_global(KeyboardInput::default);

    ui.get_context().init_global(Mouse::default);
    ui.get_context().init_global(Scroll::default);

    ui.get_context().init_global(Hovering::default);

    let dejavu_font = ui.load_font_bytes(include_bytes!("./fonts/DejaVuSans.ttf"));

    ui.set_root(build! {
        App {
            child: Column {
                layout: Layout {
                    sizing: Sizing::Axis {
                        width: Units::Stretch(1.0),
                        height: Units::Stretch(1.0)
                    },
                    margin: Margin::center()
                },
                spacing: Units::Pixels(16.0),
                children: [
                    Text::is(dejavu_font, 64.0, "A Title".into()).color(Color::White),
                    Spacing::vertical(32.0.into()),
                    Button {
                        child: Padding {
                            padding: Margin::All(10.0.into()),
                            child: Text::is(dejavu_font, 32.0, "A Button".into())
                        },
                        on_pressed: Callback::from(|()| {
                            println!("Pressed 1");
                        })
                    },
                    Button {
                        child: Padding {
                            padding: Margin::All(10.0.into()),
                            child: Text::is(dejavu_font, 32.0, "Another Button".into())
                        },
                        on_pressed: Callback::from(|()| {
                            println!("Pressed 1");
                        })
                    },
                    Button {
                        child: Padding {
                            padding: Margin::All(10.0.into()),
                            child: Text::is(dejavu_font, 32.0, "Also a Button".into())
                        },
                        on_pressed: Callback::from(|()| {
                            println!("Pressed 2");
                        })
                    },
                    Builder::new(move |ctx| {
                        let theme = ctx.init_state(|| {
                            let mut theme = Theme::new();

                            theme.set(ButtonStyle {
                                normal: QuadStyle {
                                    color: Color::Red,
                                },

                                hover: QuadStyle {
                                    color: Color::Green,
                                },

                                pressed: QuadStyle {
                                    color: Color::Blue,
                                },
                            });

                            theme
                        });

                        theme.provide(ctx);

                        build!{
                            Button {
                                child: Padding {
                                    padding: Margin::All(10.0.into()),
                                    child: Text::is(dejavu_font, 32.0, "Beuton".into()).color(Color::White)
                                },
                                on_pressed: Callback::from(|()| {
                                    println!("Pressed 3");
                                })
                            }
                        }
                    })
                ]
            }
        }
    });

    let main = ExampleMain::default();

    ui.run(program)
}

#[functional_widget]
fn example_main(
    ctx: &WidgetContext,
    layout: Ref<Layout>,
    color: Color,
    child: WidgetRef,
) -> BuildResult {
    ctx.set_layout(Ref::clone(&layout));

    build! {
        Quad {
            layout: Layout {
                sizing: Sizing::Fill
            },
            child
        }
    }
}
