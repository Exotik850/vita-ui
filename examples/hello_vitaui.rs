//! A basic example showcasing the vita-ui framework with Taffy-powered layout.
//!
//! Build with:
//! ```sh
//! cargo vita build vpk -- --release --example hello_vitaui
//! ```

use std::borrow::Cow;

use vita_input::ControllerInput;
use vita_ui::{menu::MenuItem, prelude::*};

/// The application state.
struct AppState {
    /// The main menu (shared so it can be laid out and receive input).
    menu: Menu<'static>,
    /// A counter incremented by a button.
    counter: u32,
    /// Status message displayed below the menu.
    status: Cow<'static, str>,
    /// Whether to show the secondary panel.
    show_panel: bool,
}

enum Message {
    Increment,
    Toggle,
    Reset,
    Exit,
}

fn main() {
    let (tx, rx) = std::sync::mpsc::sync_channel(4);

    // TODO: Better way to send messages / hold state
    // maybe use dioxus state? something similar? 
    let menu_items = vec![
        MenuItem::new("Increment Counter").on_select({
            let tx = tx.clone();
            move || {
                let _ = tx.send(Message::Increment);
            }
        }),
        MenuItem::new("Toggle Panel").on_select({
            let tx = tx.clone();
            move || {
                let _ = tx.send(Message::Toggle);
            }
        }),
        MenuItem::new("Reset Counter").on_select({
            let tx = tx.clone();
            move || {
                let _ = tx.send(Message::Reset);
            }
        }),
        MenuItem::new("Exit (no-op)").on_select({
            let tx = tx.clone();
            move || {
                let _ = tx.send(Message::Exit);
            }
        }),
    ];

    let state = AppState {
        menu: Menu::new(menu_items).with_title("Main Menu"),
        counter: 0,
        status: "Ready.".into(),
        show_panel: false,
    };

    App::new()
        .with_state(state)
        .with_style(StyleSheet::default().with_font_scale(0.75))
        .with_config(
            AppConfig::new()
                .with_vsync(true)
                .with_target_fps(60)
                .with_block_input(InputHandling::Blocking),
        )
        .with_init(|state: &mut AppState| {
            state.status = "vita-ui initialised!".into();
        })
        .with_update(|_state: &mut AppState, _ctx: &FrameCtx| {
            // Any per-frame logic goes here.
        })
        .with_event_receiver(ControllerInput::read)
        .with_input(|state: &mut AppState, input: ControllerInput| {
            // Route input to the menu.
            state.menu.handle_input(&input);
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    Message::Increment => {
                        state.counter += 1;
                        state.status = format!("Counter incremented to {}.", state.counter).into();
                    }
                    Message::Toggle => {
                        state.show_panel = !state.show_panel;
                        state.status = if state.show_panel {
                            "Panel shown.".into()
                        } else {
                            "Panel hidden.".into()
                        };
                    }
                    Message::Reset => {
                        state.counter = 0;
                        state.status = "Counter reset.".into();
                    }
                    Message::Exit => {
                        state.status = "Exit selected (no-op).".into();
                    }
                }
            }
        })
        .with_render(|state: &mut AppState, ctx: RenderCtx| {
            let draw = ctx.draw;
            let style = &ctx.style;

            // Clear the screen.
            draw.clear_screen();

            // --- Build the layout tree ---
            // We rebuild each frame so the tree reflects current state.
            let mut tree = LayoutTree::new();
            

            let root = tree.flex_with(FlexDir::Column, |flex| {
                flex.padding(16.0).gap(12.0);

                // --- Title ---
                flex.add_widget(
                    Text::new("vita-ui Demo")
                        .with_color(Color::rgba(255, 220, 64, 255))
                        .with_scale(1.5),
                );

                // --- Menu and panel side-by-side ---
                flex.add_container(FlexDir::Row, |row| {
                    row.gap(24.0);
                    row.add_widget(&mut state.menu);

                    if state.show_panel {
                        row.add_container(FlexDir::Column, |panel| {
                            panel.padding(8.0).gap(8.0);

                            panel.add_widget(
                                Text::new("Info Panel")
                                    .with_color(Color::rgba(128, 200, 255, 255))
                                    .with_scale(1.0),
                            );
                            panel.add_widget(
                                Text::new("This panel can be toggled from the menu.")
                                    .with_color(Color::rgba(200, 200, 220, 255))
                                    .with_scale(0.7),
                            );
                            panel.add_widget(
                                vita_ui::button::Button::new("Click Me")
                                    .with_width(120.0)
                                    .with_height(30.0),
                            );
                        });
                    }
                });

                // --- Status text ---
                flex.add_widget(
                    Text::new(state.status.as_ref())
                        .with_color(Color::rgba(180, 180, 200, 255))
                        .with_scale(0.8),
                );

                // --- Counter display ---
                flex.add_widget(
                    Text::new(format!("Counter: {}", state.counter))
                        .with_color(Color::WHITE)
                        .with_scale(1.0),
                );

                // --- Flex spacer to push footer to bottom ---
                flex.add_flex_spacer(1.0);

                // --- Footer ---
                flex.add_widget(
                    Text::new("Use D-Pad to navigate, Cross to select.")
                        .with_color(Color::rgba(128, 128, 128, 255))
                        .with_scale(0.6),
                );
            });

            // Compute layout at Vita screen resolution.
            tree.compute(root, 960.0, 544.0, style);
            tree.draw(root, draw, style);
        })
        .run();
}
