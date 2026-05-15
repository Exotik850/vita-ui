//! A vertical menu widget with selectable items.

use std::borrow::Cow;

use taffy::prelude::{AlignItems, AvailableSpace, Size};
use vita_input::Button;
use vita2d_rs::prelude::Drawing;

use crate::layout::{FlexDir, LayoutTree};
use crate::style::StyleSheet;
use crate::widget::{Rect, Widget, draw_rounded_rect};

/// A single item in a [`Menu`].
pub struct MenuItem<'a> {
    /// The display label.
    pub label: Cow<'a, str>,
    /// Called when this item is activated.
    pub on_select: Option<Box<dyn FnMut()>>,
}

impl<'a> MenuItem<'a> {
    /// Create a new menu item.
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
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
pub struct Menu<'a> {
    /// The menu items.
    pub items: Vec<MenuItem<'a>>,
    /// Index of the currently selected item.
    pub selected: usize,
    /// Optional title displayed above the items.
    title: Option<MenuTitle<'a>>,
    /// Custom width.  If `None`, auto-sizes.
    pub width: Option<f32>,
}

struct MenuTitle<'a> {
    label: Cow<'a, str>,
}

impl<'a> Widget for MenuTitle<'a> {
    fn draw(&self, rect: Rect, draw: &Drawing, style: &StyleSheet) {
        let scale = style.font_scale * 0.9;
        let line_h = style.line_height(scale);
        let tx = rect.x + style.padding;
        let ty = rect.y + (rect.h - line_h) / 2.0 + line_h;
        style.font.draw_text(
            tx as i32,
            ty as i32,
            style.menu_text,
            scale,
            self.label.as_ref(),
            draw,
        );
    }

    fn measure(
        &self,
        style: &StyleSheet,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let scale = style.font_scale * 0.9;
        let text_w = style.font.text_width(scale, self.label.as_ref()) as f32;
        let line_h = style.line_height(scale);

        let mut size = Size {
            width: text_w + style.padding * 2.0,
            height: line_h + style.padding * 2.0,
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

struct MenuEntry<'a> {
    label: &'a str,
    selected: bool,
}

impl<'a> Widget for MenuEntry<'a> {
    fn draw(&self, rect: Rect, draw: &Drawing, style: &StyleSheet) {
        if self.selected {
            draw_rounded_rect(
                draw,
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                style.corner_radius,
                style.menu_highlight,
            );
        }

        let line_h = style.line_height(style.font_scale);
        let tx = rect.x + style.padding;
        let ty = rect.y + (rect.h - line_h) / 2.0 + line_h;
        style.font.draw_text(
            tx as i32,
            ty as i32,
            style.menu_text,
            style.font_scale,
            self.label,
            draw,
        );
    }

    fn measure(
        &self,
        style: &StyleSheet,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let text_w = style.font.text_width(style.font_scale, self.label) as f32;
        let line_h = style.line_height(style.font_scale);
        let mut size = Size {
            width: text_w + style.padding * 2.0,
            height: line_h + style.padding * 2.0,
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

impl<'a> Menu<'a> {
    /// Create a new menu with the given items.
    pub fn new(items: Vec<MenuItem<'a>>) -> Self {
        Self {
            items,
            selected: 0,
            title: None,
            width: None,
        }
    }

    /// Set the menu title.
    pub fn with_title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(MenuTitle {
            label: title.into(),
        });
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

impl<'a> Widget for Menu<'a> {
    fn draw(&self, rect: Rect, draw: &Drawing, style: &StyleSheet) {
        // Background panel
        draw_rounded_rect(
            draw,
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            style.corner_radius,
            style.menu_bg,
        );

        let mut tree = LayoutTree::new();
        let root = tree.flex_with(FlexDir::Column, |flex| {
            flex.padding(style.padding)
                .gap(style.padding)
                .align_items(AlignItems::Stretch);

            if let Some(ref title) = self.title {
                flex.add_widget(title);
            }

            for (i, item) in self.items.iter().enumerate() {
                flex.add_widget(MenuEntry {
                    label: item.label.as_ref(),
                    selected: i == self.selected,
                });
            }
        });

        tree.compute(root, rect.w, rect.h, style);
        tree.draw_at(root, draw, style, rect.x, rect.y);
    }

    fn measure(
        &self,
        style: &StyleSheet,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let max_w = self
            .items
            .iter()
            .map(|i| {
                style.font.text_width(style.font_scale, i.label.as_ref()) as f32
                    + style.padding * 2.0
            })
            .fold(0.0, f32::max);
        let title_w = self.title.as_ref().map_or(0.0, |t| {
            style
                .font
                .text_width(style.font_scale * 0.9, t.label.as_ref()) as f32
                + style.padding * 2.0
        });
        let content_w = max_w.max(title_w);
        let auto_w = content_w + style.padding * 2.0;
        let w = self.width.unwrap_or(auto_w);

        let item_line_h = style.line_height(style.font_scale);
        let item_h = item_line_h + style.padding * 2.0;
        let title_h = if self.title.is_some() {
            style.line_height(style.font_scale * 0.9) + style.padding * 2.0
        } else {
            0.0
        };
        let count = self.items.len() + if self.title.is_some() { 1 } else { 0 };
        let gaps = if count > 1 {
            (count - 1) as f32 * style.padding
        } else {
            0.0
        };
        let total_h = style.padding * 2.0 + title_h + self.items.len() as f32 * item_h + gaps;

        let mut size = Size {
            width: w,
            height: total_h,
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
