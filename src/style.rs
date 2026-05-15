//! Styling and theming for vita-ui widgets.
//!
//! A [`StyleSheet`] holds colors, font settings, and layout parameters
//! that all widgets reference.  Stylesheets are cheap to clone (they use
//! `Arc` internally for font handles).

use std::sync::Arc;

use vita2d_rs::{
    Pgf,
    prelude::{Color, VitaFont},
};

/// A complete visual theme for the UI.
///
/// All fields are public so you can construct one inline or modify it
/// after creation.
#[derive(Clone)]
pub struct StyleSheet {
    /// Background clear color.
    pub bg_color: Color,
    /// Default text color.
    pub text_color: Color,
    /// Button background color (idle).
    pub button_bg: Color,
    /// Button background color (hovered / focused).
    pub button_bg_focus: Color,
    /// Button background color (pressed).
    pub button_bg_press: Color,
    /// Button border color.
    pub button_border: Color,
    /// Button text color.
    pub button_text: Color,
    /// Menu background color.
    pub menu_bg: Color,
    /// Menu item highlight color.
    pub menu_highlight: Color,
    /// Menu item text color.
    pub menu_text: Color,
    /// Default font scale.
    pub font_scale: f32,
    /// Default padding (in pixels) inside widgets.
    pub padding: f32,
    /// Default corner radius for rounded rectangles (0 = sharp).
    pub corner_radius: f32,
    /// Default font for text widgets.
    pub font: Arc<dyn VitaFont>,
}

impl StyleSheet {
    /// Create a new stylesheet with the given font and default colors.
    pub fn new<F: VitaFont + 'static>(font: F) -> Self {
        let font = Arc::new(font);
        Self {
            font,
            bg_color: Color::rgba(24, 24, 32, 255),
            text_color: Color::WHITE,
            button_bg: Color::rgba(48, 48, 64, 255),
            button_bg_focus: Color::rgba(64, 64, 96, 255),
            button_bg_press: Color::rgba(96, 96, 128, 255),
            button_border: Color::rgba(128, 128, 160, 255),
            button_text: Color::WHITE,
            menu_bg: Color::rgba(32, 32, 48, 255),
            menu_highlight: Color::rgba(64, 64, 128, 255),
            menu_text: Color::WHITE,
            font_scale: 1.0,
            padding: 8.0,
            corner_radius: 4.0,
        }
    }

    pub fn with_font<F: VitaFont + 'static>(mut self, font: F) -> Self {
        self.font = Arc::new(font);
        self
    }

    pub fn with_font_scale(mut self, scale: f32) -> Self {
        self.font_scale = scale;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    pub fn with_text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn with_button_colors(mut self, idle: Color, focus: Color, press: Color) -> Self {
        self.button_bg = idle;
        self.button_bg_focus = focus;
        self.button_bg_press = press;
        self
    }

    pub fn with_button_border_color(mut self, color: Color) -> Self {
        self.button_border = color;
        self
    }

    pub fn with_button_text_color(mut self, color: Color) -> Self {
        self.button_text = color;
        self
    }

    pub fn with_menu_colors(mut self, bg: Color, highlight: Color, text: Color) -> Self {
        self.menu_bg = bg;
        self.menu_highlight = highlight;
        self.menu_text = text;
        self
    }

    /// Returns a stable line height for layout using a representative glyph pair.
    pub fn line_height(&self, scale: f32) -> f32 {
        self.font.text_height(scale, "Ag") as f32
    }
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self::new(Pgf::load_default().expect("Unable to load default font"))
    }
}

/// A lightweight style builder for one-off overrides.
#[derive(Debug, Clone)]
pub struct Style {
    /// Override background color.
    pub bg: Option<Color>,
    /// Override text color.
    pub text: Option<Color>,
    /// Override border color.
    pub border: Option<Color>,
    /// Override font scale.
    pub font_scale: Option<f32>,
    /// Override padding.
    pub padding: Option<f32>,
}

impl Style {
    /// Create an empty style (all fields `None`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the background color.
    pub fn with_bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Set the text color.
    pub fn with_text(mut self, color: Color) -> Self {
        self.text = Some(color);
        self
    }

    /// Set the border color.
    pub fn with_border(mut self, color: Color) -> Self {
        self.border = Some(color);
        self
    }

    /// Set the font scale.
    pub fn with_font_scale(mut self, scale: f32) -> Self {
        self.font_scale = Some(scale);
        self
    }

    /// Set the padding.
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            bg: None,
            text: None,
            border: None,
            font_scale: None,
            padding: None,
        }
    }
}
