//! The [`Widget`] trait — the common interface for all UI elements.
//!
//! Every widget can draw itself and report its bounding box.  Widgets are
//! stateless in the sense that they don't own their own position — the
//! caller passes `x, y` at draw time.

use vita2d_rs::prelude::{Color, Drawing};

use crate::style::StyleSheet;

/// A rectangular region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    /// Create a new rect.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Check if a point is inside this rect.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
}

/// The common interface for all UI widgets.
///
/// Implementors must provide [`draw`](Widget::draw) and
/// [`bounds`](Widget::bounds).  The default [`handle_input`](Widget::handle_input)
/// does nothing.
pub trait Widget {
    /// Draw the widget at `(x, y)`.
    fn draw(&self, x: f32, y: f32, draw: &Drawing, style: &StyleSheet);

    /// Return the bounding box of the widget when drawn at `(x, y)`.
    fn bounds(&self, x: f32, y: f32, style: &StyleSheet) -> Rect;

    /// Handle an input event.  Returns `true` if the event was consumed.
    ///
    /// The default implementation does nothing and returns `false`.
    fn handle_input(
        &mut self,
        _input: &vita_input::ControllerInput,
    ) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// Helper drawing functions used by widgets
// ---------------------------------------------------------------------------

/// Draw a filled rounded rectangle.
pub(crate) fn draw_rounded_rect(
    draw: &Drawing,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: Color,
) {
    if radius <= 0.0 || w <= radius * 2.0 || h <= radius * 2.0 {
        // Fall back to sharp rectangle.
        draw.draw_rectangle(x, y, w, h, color);
        return;
    }

    let r = radius;

    // Center rectangle
    draw.draw_rectangle(x + r, y, w - r * 2.0, h, color);
    // Left and right strips
    draw.draw_rectangle(x, y + r, r, h - r * 2.0, color);
    draw.draw_rectangle(x + w - r, y + r, r, h - r * 2.0, color);

    // Four corner circles
    draw.draw_fill_circle(x + r, y + r, r, color);
    draw.draw_fill_circle(x + w - r, y + r, r, color);
    draw.draw_fill_circle(x + r, y + h - r, r, color);
    draw.draw_fill_circle(x + w - r, y + h - r, r, color);
}

/// Draw a border (outline) for a rounded rectangle.
pub(crate) fn draw_rounded_rect_border(
    draw: &Drawing,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    thickness: f32,
    color: Color,
) {
    // Top edge
    draw.draw_rectangle(x + radius, y, w - radius * 2.0, thickness, color);
    // Bottom edge
    draw.draw_rectangle(x + radius, y + h - thickness, w - radius * 2.0, thickness, color);
    // Left edge
    draw.draw_rectangle(x, y + radius, thickness, h - radius * 2.0, color);
    // Right edge
    draw.draw_rectangle(x + w - thickness, y + radius, thickness, h - radius * 2.0, color);

    // Corner arcs — approximate with small circles
    let t = thickness / 2.0;
    draw.draw_fill_circle(x + radius, y + radius, t, color);
    draw.draw_fill_circle(x + w - radius, y + radius, t, color);
    draw.draw_fill_circle(x + radius, y + h - radius, t, color);
    draw.draw_fill_circle(x + w - radius, y + h - radius, t, color);
}
