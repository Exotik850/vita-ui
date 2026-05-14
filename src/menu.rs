//! A vertical menu widget with selectable items.

use vita_input::Button;
use vita2d_rs::prelude::Drawing;

// use crate::input::{ButtonState, VitaInput};
use crate::style::StyleSheet;
use crate::widget::{Rect, Widget, draw_rounded_rect};

/// A single item in a [`Menu`].
pub struct MenuItem {
    /// The display label.
    pub label: String,
    /// Called when this item is activated.
    pub on_select: Option<Box<dyn FnMut()>>,
}

impl MenuItem {
    /// Create a new menu item.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            on_select: None,
        }
    }

    /// Set the selection callback.
    pub fn on_select(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }
}

/// A vertical menu.
///
/// The user navigates with the D-pad (up/down) and selects with Cross.
/// The menu draws a background panel and highlights the currently selected
/// item.
pub struct Menu {
    /// The menu items.
    pub items: Vec<MenuItem>,
    /// Index of the currently selected item.
    pub selected: usize,
    /// Optional title displayed above the items.
    pub title: Option<String>,
    /// Custom width.  If `None`, auto-sizes.
    pub width: Option<f32>,
}

impl Menu {
    /// Create a new menu with the given items.
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            selected: 0,
            title: None,
            width: None,
        }
    }

    /// Set the menu title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set a fixed width.
    pub fn with_width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    /// Move selection up.
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down.
    pub fn select_next(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        }
    }

    /// Activate the currently selected item.
    pub fn activate(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected) {
            if let Some(ref mut cb) = item.on_select {
                cb();
            }
        }
    }
}

impl Widget for Menu {
    fn draw(&self, x: f32, y: f32, draw: &Drawing, style: &StyleSheet) {
        // Compute line heights from actual font metrics (same as bounds).
        let item_line_h = style.line_height(style.font_scale);
        let item_h = item_line_h + style.padding * 2.0;

        let title_line_h = self.title.as_ref().map_or(0.0, |_| {
            style.line_height(style.font_scale * 0.9)
        });
        let title_h = if self.title.is_some() {
            title_line_h + style.padding * 2.0
        } else {
            0.0
        };
        let total_h = title_h + self.items.len() as f32 * item_h + style.padding * 2.0;

        // Compute auto-width from actual font metrics (same as bounds).
        let max_label_w = self
            .items
            .iter()
            .map(|i| style.font.text_width(style.font_scale, &i.label))
            .max()
            .unwrap_or(0) as f32;
        let title_w = self.title.as_ref().map_or(0.0, |t| {
            style.font.text_width(style.font_scale * 0.9, t) as f32
        });
        let content_w = max_label_w.max(title_w);
        let auto_w = content_w + style.padding * 4.0;
        let w = self.width.unwrap_or(auto_w);

        // Background panel
        draw_rounded_rect(draw, x, y, w, total_h, style.corner_radius, style.menu_bg);

        // Title
        let mut cy = y + style.padding;
        if let Some(ref title) = self.title {
            style.font.draw_text(
                (x + style.padding * 2.0) as i32,
                (cy + (title_h - title_line_h) / 2.0) as i32,
                style.menu_text,
                style.font_scale * 0.9,
                title,
                draw,
            );
            cy += title_h;
        }

        // Items
        for (i, item) in self.items.iter().enumerate() {
            let iy = cy + i as f32 * item_h;

            // Highlight selected
            if i == self.selected {
                draw_rounded_rect(
                    draw,
                    x + style.padding / 2.0,
                    iy,
                    w - style.padding,
                    item_h,
                    style.corner_radius,
                    style.menu_highlight,
                );
            }

            style.font.draw_text(
                (x + style.padding * 2.0) as i32,
                (iy + (item_h - item_line_h) / 2.0) as i32,
                style.menu_text,
                style.font_scale,
                &item.label,
                draw,
            );
        }
    }

    fn bounds(&self, x: f32, y: f32, style: &StyleSheet) -> Rect {
        let item_line_h = style.line_height(style.font_scale);
        let item_h = item_line_h + style.padding * 2.0;

        let max_w = self
            .items
            .iter()
            .map(|i| style.font.text_width(style.font_scale, &i.label))
            .max()
            .unwrap_or(0) as f32;
        let title_w =
            self.title
                .as_ref()
                .map_or(0, |t| style.font.text_width(style.font_scale * 0.9, t)) as f32;
        let content_w = max_w.max(title_w);
        let auto_w = content_w + style.padding * 4.0;
        let w = self.width.unwrap_or(auto_w);

        let title_line_h = self.title.as_ref().map_or(0.0, |_| {
            style.line_height(style.font_scale * 0.9)
        });
        let title_h = if self.title.is_some() {
            title_line_h + style.padding * 2.0
        } else {
            0.0
        };
        let total_h = title_h + self.items.len() as f32 * item_h + style.padding * 2.0;

        Rect::new(x, y, w, total_h)
    }

    fn handle_input(&mut self, input: &vita_input::ControllerInput) -> bool {
        let mut consumed = false;
        if input.is_pressed(Button::Up) {
            self.select_prev();
            consumed = true;
        }
        if input.is_pressed(Button::Down) {
            self.select_next();
            consumed = true;
        }
        if input.is_pressed(Button::Cross) {
            self.activate();
            consumed = true;
        }
        consumed
    }
}
