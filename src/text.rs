//! A simple text label widget.

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

use vita2d_rs::prelude::{Color, Drawing, Pgf, VitaFont};

use crate::style::StyleSheet;
use crate::widget::{Rect, Widget};

/// A static text label.
///
/// The text is drawn using the system PGF font.  If no PGF font is
/// available (e.g., on a non-Vita host), drawing is silently skipped.
pub struct Text<'a> {
    /// The text content.
    pub content: Cow<'a, str>,
    /// Optional override color.  If `None`, uses `style.text_color`.
    pub color: Option<Color>,
    /// Font scale factor.
    pub scale: f32,
}

impl<'a> Text<'a> {
    /// Create a new text label.
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self {
            content: content.into(),
            color: None,
            scale: 1.0,
        }
    }

    /// Set the text color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the font scale.
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

impl Widget for Text<'_> {
    fn draw(&self, x: f32, y: f32, draw: &Drawing, style: &StyleSheet) {
        let color = self.color.unwrap_or(style.text_color);
        let scale = self.scale * style.font_scale;

        style
            .font
            .draw_text(x as i32, y as i32, color, scale, &self.content, draw);
    }

    fn bounds(&self, x: f32, y: f32, style: &StyleSheet) -> Rect {
        let scale = self.scale * style.font_scale;
        let (w, h) = style.font.text_dimensions(scale, &self.content);
        let w = w as f32;
        let h = h as f32;
        Rect::new(x, y, w, h)
    }
}
