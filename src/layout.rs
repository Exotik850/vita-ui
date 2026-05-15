//! A Taffy-powered layout system for vita-ui.
//!
//! This module provides high-level layout widgets that use Taffy for
//! flexbox layout computation. Widgets are added to a [`LayoutTree`],
//! positioned by Taffy, and then drawn at their computed positions.
//!
//! # Architecture
//!
//! - [`LayoutTree`] wraps a [`taffy::TaffyTree`] and maps Taffy nodes to
//!   concrete widget instances.
//! - [`Flex`] is a builder for flex containers that can hold child widgets.
//! - [`Spacer`] is an invisible spacing widget.

use taffy::prelude::*;
use vita2d_rs::prelude::Drawing;

use crate::style::StyleSheet;
use crate::widget::{IntoWidget, Rect, Widget};

// ---------------------------------------------------------------------------
// LayoutTree — the main layout engine
// ---------------------------------------------------------------------------

/// Data stored alongside each Taffy node that has a widget.
type NodeData<'a> = Box<dyn Widget + 'a>;

/// The core layout tree.
///
/// Wraps a [`TaffyTree`] and stores widget handles keyed by [`NodeId`].
/// After building the tree and calling [`compute`](LayoutTree::compute),
/// widgets can be drawn at their Taffy-computed positions via
/// [`draw`](LayoutTree::draw).
pub struct LayoutTree<'a> {
    taffy: TaffyTree<NodeData<'a>>,
}

impl LayoutTree<'_> {
    /// Create a new, empty layout tree.
    pub fn new<'a>() -> LayoutTree<'a> {
        LayoutTree {
            taffy: TaffyTree::new(),
        }
    }
}

impl<'a> LayoutTree<'a> {
    /// Add a container (non-leaf) node with the given style and children.
    pub fn add_container(&mut self, style: Style, children: &[NodeId]) -> NodeId {
        self.taffy
            .new_with_children(style, children)
            .expect("failed to create container node")
    }

    /// Add a leaf widget node driven by Taffy's built-in leaf layout.
    ///
    /// The widget's [`Widget::measure`] is used by Taffy to determine
    /// the intrinsic size of the node.
    pub fn widget(&mut self, widget: impl IntoWidget<'a>, style: Style) -> NodeId {
        let node = self
            .taffy
            .new_leaf(style)
            .expect("failed to create widget node");
        self.taffy
            .set_node_context(node, Some(Box::new(widget.into_widget())))
            .expect("failed to set node context");
        node
    }

    /// Recompute layout for the subtree rooted at `root`.
    pub fn compute(&mut self, root: NodeId, width: f32, height: f32, style: &StyleSheet) {
        self.taffy
            .compute_layout_with_measure(
                root,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(height),
                },
                |known_dimensions, available_space, _node_id, node_context, _style| {
                    if let Some(widget) = node_context {
                        return widget.measure(style, known_dimensions, available_space);
                    }
                    Size::ZERO
                },
            )
            .expect("layout computation failed");
    }

    /// Draw all widgets in the subtree rooted at `root`.
    ///
    /// Walks the tree in pre-order, calling [`Widget::draw`] on each
    /// leaf that has an associated widget.
    pub fn draw(&self, root: NodeId, draw: &Drawing, style: &StyleSheet) {
        self.draw_at(root, draw, style, 0.0, 0.0);
    }

    /// Draw all widgets with a fixed offset applied to the layout.
    pub fn draw_at(&self, root: NodeId, draw: &Drawing, style: &StyleSheet, ox: f32, oy: f32) {
        self.draw_node(root, draw, style, ox, oy);
    }

    /// Route input to widgets in the subtree rooted at `root`.
    ///
    /// Walks children first (depth-first), then the node itself.
    /// Returns `true` if any widget consumed the event.
    pub fn handle_input(&mut self, root: NodeId, input: &crate::prelude::ControllerInput) -> bool {
        self.handle_input_at(root, input, 0.0, 0.0)
    }

    /// Route input with a fixed offset applied to the layout.
    pub fn handle_input_at(
        &mut self,
        root: NodeId,
        input: &crate::prelude::ControllerInput,
        ox: f32,
        oy: f32,
    ) -> bool {
        self.handle_input_node(root, input, ox, oy)
    }

    // --- internal helpers ---

    fn draw_node(&self, node: NodeId, draw: &Drawing, style: &StyleSheet, ox: f32, oy: f32) {
        let layout = self.taffy.layout(node).expect("layout missing for node");
        let rect = Rect::new(
            ox + layout.location.x,
            oy + layout.location.y,
            layout.size.width,
            layout.size.height,
        );

        // Draw our own widget (if any)
        if let Some(widget) = self.taffy.get_node_context(node) {
            widget.draw(rect, draw, style);
        }

        // Draw children
        if let Ok(children) = self.taffy.children(node) {
            for child in children {
                self.draw_node(child, draw, style, rect.x, rect.y);
            }
        }
    }

    fn handle_input_node(
        &mut self,
        node: NodeId,
        input: &crate::prelude::ControllerInput,
        ox: f32,
        oy: f32,
    ) -> bool {
        let layout = self.taffy.layout(node).expect("layout missing for node");
        let rect = Rect::new(
            ox + layout.location.x,
            oy + layout.location.y,
            layout.size.width,
            layout.size.height,
        );

        // Visit children first (depth-first)
        if let Ok(children) = self.taffy.children(node) {
            for child in children {
                if self.handle_input_node(child, input, rect.x, rect.y) {
                    return true;
                }
            }
        }

        // Then try this node
        if let Some(widget) = self.taffy.get_node_context_mut(node) {
            return widget.handle_input(input);
        }

        false
    }
}

impl<'a> Default for LayoutTree<'a> {
    fn default() -> Self {
        Self {
            taffy: TaffyTree::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Flex — builder for flex layouts
// ---------------------------------------------------------------------------

/// Direction for a flex container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDir {
    /// Items are laid out left-to-right.
    Row,
    /// Items are laid out top-to-bottom.
    Column,
}

/// A builder for flex container nodes.
///
/// Use [`LayoutTree::flex`] or the convenience methods
/// [`LayoutTree::flex_row`] / [`LayoutTree::flex_column`] to create one.
///
/// # Example
///
/// ```ignore
/// let root = tree.flex_row(|flex| {
///     flex.add_widget(Text::new("Left"));
///     flex.add_spacer(20.0);
///     flex.add_widget(Text::new("Right"));
/// });
/// ```
pub struct Flex<'a, 'tree> {
    tree: &'a mut LayoutTree<'tree>,
    children: Vec<NodeId>,
    direction: FlexDir,
    gap: f32,
    padding: f32,
    align_items: Option<AlignItems>,
    justify_content: Option<JustifyContent>,
}

impl<'a, 'tree> Flex<'a, 'tree> {
    fn new(tree: &'a mut LayoutTree<'tree>, direction: FlexDir) -> Self {
        Self {
            tree,
            children: Vec::new(),
            direction,
            gap: 8.0,
            padding: 8.0,
            align_items: None,
            justify_content: None,
        }
    }

    /// Set the gap between children (in pixels).
    pub fn gap(&mut self, gap: f32) -> &mut Self {
        self.gap = gap;
        self
    }

    /// Set the padding around all children (in pixels).
    pub fn padding(&mut self, padding: f32) -> &mut Self {
        self.padding = padding;
        self
    }

    /// Set cross-axis alignment for children.
    pub fn align_items(&mut self, align: AlignItems) -> &mut Self {
        self.align_items = Some(align);
        self
    }

    /// Set main-axis justification for children.
    pub fn justify_content(&mut self, justify: JustifyContent) -> &mut Self {
        self.justify_content = Some(justify);
        self
    }

    /// Add a widget child with default Taffy style.
    pub fn widget(&mut self, widget: impl IntoWidget<'tree>) -> NodeId {
        self.widget_styled(widget, Style::default())
    }

    /// Add a widget child with a custom Taffy style.
    pub fn widget_styled(&mut self, widget: impl IntoWidget<'tree>, style: Style) -> NodeId {
        let node = self.tree.widget(widget, style);
        self.children.push(node);
        node
    }

    /// Add an empty spacer child with the given fixed size on the main axis.
    ///
    /// For a row, this is the width; for a column, this is the height.
    pub fn spacer(&mut self, size: f32) -> NodeId {
        let spacer_style = match self.direction {
            FlexDir::Row => Style {
                size: Size {
                    width: Dimension::length(size),
                    height: Dimension::length(0.0),
                },
                ..Default::default()
            },
            FlexDir::Column => Style {
                size: Size {
                    width: Dimension::length(0.0),
                    height: Dimension::length(size),
                },
                ..Default::default()
            },
        };
        let node = self.tree.widget(Spacer, spacer_style);
        self.children.push(node);
        node
    }

    /// Add a flex-grow spacer that fills remaining space on the main axis.
    ///
    /// The `grow` value (typically 1.0) controls how much this spacer
    /// expands relative to siblings.
    pub fn flex_spacer(&mut self, grow: f32) -> NodeId {
        let spacer_style = Style {
            flex_grow: grow,
            flex_shrink: 0.0,
            ..Default::default()
        };
        let node = self.tree.widget(Spacer, spacer_style);
        self.children.push(node);
        node
    }

    /// Add a nested flex container as a child.
    ///
    /// The closure receives a [`Flex`] builder for the nested container.
    /// Returns the [`NodeId`] of the nested container.
    pub fn container(
        &mut self,
        direction: FlexDir,
        f: impl FnOnce(&mut Flex<'_, 'tree>),
    ) -> NodeId {
        let mut child_flex = Flex::new(self.tree, direction);
        f(&mut child_flex);
        let node = child_flex.build();
        self.children.push(node);
        node
    }

    /// Finish building and return the container [`NodeId`].
    pub fn build(self) -> NodeId {
        let mut style = Style {
            display: Display::Flex,
            flex_direction: match self.direction {
                FlexDir::Row => taffy::style::FlexDirection::Row,
                FlexDir::Column => taffy::style::FlexDirection::Column,
            },
            padding: taffy::geometry::Rect {
                left: LengthPercentage::length(self.padding),
                right: LengthPercentage::length(self.padding),
                top: LengthPercentage::length(self.padding),
                bottom: LengthPercentage::length(self.padding),
            },
            gap: Size {
                width: LengthPercentage::length(self.gap),
                height: LengthPercentage::length(self.gap),
            },
            ..Default::default()
        };

        if let Some(a) = self.align_items {
            style.align_items = Some(a);
        }
        if let Some(j) = self.justify_content {
            style.justify_content = Some(j);
        }

        self.tree.add_container(style, &self.children)
    }
}

// Convenience methods on LayoutTree
impl<'tree> LayoutTree<'tree> {
    /// Create a flex container with the given direction.
    pub fn flex<'a>(&'a mut self, direction: FlexDir) -> Flex<'a, 'tree> {
        Flex::new(self, direction)
    }

    /// Create a flex container with a closure for building children.
    /// Returns the container [`NodeId`].
    pub fn flex_with<'a>(
        &'a mut self,
        direction: FlexDir,
        f: impl FnOnce(&mut Flex<'a, 'tree>),
    ) -> NodeId {
        let mut flex = Flex::new(self, direction);
        f(&mut flex);
        flex.build()
    }

    /// Create a row-direction flex container.
    pub fn row<'a>(&'a mut self) -> Flex<'a, 'tree> {
        self.flex(FlexDir::Row)
    }

    /// Create a column-direction flex container.
    pub fn column<'a>(&'a mut self) -> Flex<'a, 'tree> {
        self.flex(FlexDir::Column)
    }
}

// ---------------------------------------------------------------------------
// Spacer — an invisible spacing widget
// ---------------------------------------------------------------------------

/// An invisible widget that occupies a fixed amount of space.
///
/// Typically created through [`Flex::add_spacer`] rather than directly.
pub struct Spacer;

impl Widget for Spacer {
    fn draw(&self, _rect: Rect, _draw: &Drawing, _style: &StyleSheet) {
        // Spacer is invisible.
    }

    fn measure(
        &self,
        _style: &StyleSheet,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        Size {
            width: known_dimensions.width.unwrap_or(0.0),
            height: known_dimensions.height.unwrap_or(0.0),
        }
    }
}
