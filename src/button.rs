//! A clickable button widget.

use taffy::prelude::{AvailableSpace, Size};
use vita2d_rs::prelude::Drawing;

use crate::style::StyleSheet;
use crate::widget::{Rect, Widget, draw_rounded_rect, draw_rounded_rect_border};

/// A clickable button with a text label.
///
/// The button tracks its own visual state (idle, focused, pressed) and
/// calls `on_click` when the cross button is pressed while the button
/// is focused.
pub struct Button {
    /// The label text.
    pub label: String,
    /// Called when the button is clicked.
    pub on_click: Option<Box<dyn FnMut()>>,

    /// Current visual state.
    state: ButtonVisual,
    /// Whether this button is currently focused (navigated to).
    focused: bool,
    /// Custom width.  If `None`, auto-sizes based on label.
    pub width: Option<f32>,
    /// Custom height.  If `None`, uses default.
    pub height: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonVisual {
    Idle,
    Focused,
    Pressed,
}

impl Button {
    /// Create a new button with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
            state: ButtonVisual::Idle,
            focused: false,
            width: None,
            height: None,
        }
    }

    /// Set the click callback.
    pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Set a fixed width.
    pub fn with_width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    /// Set a fixed height.
    pub fn with_height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    /// Set whether this button is currently focused.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if !focused && self.state == ButtonVisual::Focused {
            self.state = ButtonVisual::Idle;
        }
    }

    /// Returns `true` if this button is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    fn handle_input(&mut self, input: &vita_input::ControllerInput) -> bool {
        if !self.focused {
            self.state = ButtonVisual::Idle;
            return false;
        }
        if input.is_pressed(vita_input::Button::Cross) {
            self.state = ButtonVisual::Pressed;
            if let Some(ref mut cb) = self.on_click {
                cb();
            }
            return true;
        } else {
            self.state = ButtonVisual::Focused;
            return self.focused;
        }
    }
}

impl Widget for Button {
    fn draw(&self, rect: Rect, draw: &Drawing, style: &StyleSheet) {
        let text_w = style.font.text_width(style.font_scale, &self.label) as f32;
        let line_h = style.line_height(style.font_scale);

        let w = rect.w;
        let h = rect.h;

        let bg = match self.state {
            ButtonVisual::Idle => style.button_bg,
            ButtonVisual::Focused => style.button_bg_focus,
            ButtonVisual::Pressed => style.button_bg_press,
        };

        // Background
        draw_rounded_rect(draw, rect.x, rect.y, w, h, style.corner_radius, bg);

        // Border
        draw_rounded_rect_border(
            draw,
            rect.x,
            rect.y,
            w,
            h,
            style.corner_radius,
            1.0,
            style.button_border,
        );

        // Label — centered using actual font metrics
        let tx = rect.x + (w - text_w) / 2.0;
        let ty = rect.y + (h - line_h) / 2.0 + line_h;
        style.font.draw_text(
            tx as i32,
            ty as i32,
            style.button_text,
            style.font_scale,
            &self.label,
            draw,
        );
    }

    fn measure(
        &self,
        style: &StyleSheet,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let text_w = style.font.text_width(style.font_scale, &self.label) as f32;
        let line_h = style.line_height(style.font_scale);

        let w = self.width.unwrap_or(text_w + style.padding * 2.0);
        let h = self.height.unwrap_or(line_h + style.padding * 2.0);

        let mut size = Size {
            width: w,
            height: h,
        };
        if let Some(w) = known_dimensions.width {
            size.width = w;
        }
        if let Some(h) = known_dimensions.height {
            size.height = h;
        }

        size
    }

    fn handle_input(&mut self, input: &vita_input::ControllerInput) -> bool {
        self.handle_input(input)
    }
}
