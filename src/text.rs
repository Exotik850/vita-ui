//! A simple text label widget.

use std::borrow::Cow;

use taffy::prelude::{AvailableSpace, Size};
use vita2d_rs::prelude::{Color, Drawing};

use crate::style::StyleSheet;
use crate::widget::{IntoWidget, Rect, Widget};

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

impl<'a> IntoWidget<'a> for &'a str {
    type WidgetType = Text<'a>;
    fn into_widget(self) -> Self::WidgetType {
        Text::new(self)
    }
}
impl<'a> IntoWidget<'a> for String {
    type WidgetType = Text<'a>;
    fn into_widget(self) -> Self::WidgetType {
        Text::new(self)
    }
}
impl<'a> IntoWidget<'a> for Cow<'a, str> {
    type WidgetType = Text<'a>;
    fn into_widget(self) -> Self::WidgetType {
        Text::new(self)
    }
}

impl Widget for Text<'_> {
    fn draw(&self, rect: Rect, draw: &Drawing, style: &StyleSheet) {
        let color = self.color.unwrap_or(style.text_color);
        let scale = self.scale * style.font_scale;
        let baseline_y = rect.y + style.line_height(scale);
        style.font.draw_text(
            rect.x as i32,
            baseline_y as i32,
            color,
            scale,
            &self.content,
            draw,
        );
    }

    fn measure(
        &self,
        style: &StyleSheet,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let scale = self.scale * style.font_scale;
        let (w, h) = style.font.text_dimensions(scale, &self.content);
        let mut size = Size {
            width: w as f32,
            height: h as f32,
        };
        if let Some(w) = known_dimensions.width {
            size.width = w;
        }
        if let Some(h) = known_dimensions.height {
            size.height = h;
        }

        size
    }
}
