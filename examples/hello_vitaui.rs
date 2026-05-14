//! A basic example showcasing the vita-ui framework.
//!
//! Build with:
//! ```sh
//! cargo vita build vpk -- --release --example hello_vitaui
//! ```

use vita_input::{Button, ControllerInput};
use vita_ui::{menu::MenuItem, prelude::*};

/// The application state.
struct AppState {
    /// The main menu.
    menu: Menu,
    /// A counter incremented by a button.
    counter: u32,
    /// Status message displayed below the menu.
    status: String,
    /// Whether to show the secondary panel.
    show_panel: bool,
}

fn main() {
    // Build the menu items.
    let menu_items = vec![
        MenuItem::new("Increment Counter"),
        MenuItem::new("Toggle Panel"),
        MenuItem::new("Reset Counter"),
        MenuItem::new("Exit (no-op)"),
    ];

    let state = AppState {
        menu: Menu::new(menu_items).with_title("Main Menu"),
        counter: 0,
        status: String::from("Ready."),
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
            state.status = String::from("vita-ui initialised!");
        })
        .with_update(|state: &mut AppState, _ctx: &FrameCtx| {
            // Any per-frame logic goes here.
        })
        .with_input(|state: &mut AppState, input: ControllerInput| {
            // Route input to the menu.
            state.menu.handle_input(&input);

            // Handle menu actions based on selection + cross press.
            if input.is_pressed(Button::Cross) {
                match state.menu.selected {
                    0 => {
                        state.counter += 1;
                        state.status = format!("Counter: {}", state.counter);
                    }
                    1 => {
                        state.show_panel = !state.show_panel;
                        state.status = format!(
                            "Panel {}",
                            if state.show_panel { "shown" } else { "hidden" }
                        );
                    }
                    2 => {
                        state.counter = 0;
                        state.status = String::from("Counter reset.");
                    }
                    3 => {
                        state.status = String::from("Exit selected (no-op).");
                    }
                    _ => {}
                }
            }
        })
        .with_render(|state: &mut AppState, ctx: RenderCtx| {
            let draw = ctx.draw;
            let style = &ctx.style;

            // Clear the screen.
            draw.clear_screen();

            // --- Title ---
            let title = Text::new("vita-ui Demo")
                .with_color(Color::rgba(255, 220, 64, 255))
                .with_scale(1.5);
            title.draw(40.0, 20.0, draw, style);

            // --- Menu ---
            state.menu.draw(40.0, 60.0, draw, style);

            // --- Status text ---
            let status = Text::new(&state.status)
                .with_color(Color::rgba(180, 180, 200, 255))
                .with_scale(0.8);
            status.draw(40.0, 400.0, draw, style);

            // --- Counter display ---
            let counter_text = Text::new(format!("Counter: {}", state.counter))
                .with_color(Color::WHITE)
                .with_scale(1.0);
            counter_text.draw(40.0, 430.0, draw, style);

            // --- Secondary panel (toggled) ---
            if state.show_panel {
                let panel_x = 300.0;
                let panel_y = 60.0;

                // Panel background
                draw.draw_rectangle(panel_x, panel_y, 200.0, 200.0, Color::rgba(40, 40, 60, 255));

                let panel_title = Text::new("Info Panel")
                    .with_color(Color::rgba(128, 200, 255, 255))
                    .with_scale(1.0);
                panel_title.draw(panel_x + 10.0, panel_y + 10.0, draw, style);

                let info = Text::new("This panel can be\ntoggled from the menu.")
                    .with_color(Color::rgba(200, 200, 220, 255))
                    .with_scale(0.7);
                info.draw(panel_x + 10.0, panel_y + 40.0, draw, style);

                // A button inside the panel
                let btn = vita_ui::button::Button::new("Click Me")
                    .with_width(120.0)
                    .with_height(30.0);
                btn.draw(panel_x + 40.0, panel_y + 120.0, draw, style);
            }

            // --- Footer ---
            let footer = Text::new("Use D-Pad to navigate, Cross to select.")
                .with_color(Color::rgba(128, 128, 128, 255))
                .with_scale(0.6);
            footer.draw(40.0, 500.0, draw, style);
        })
        .run();
}
