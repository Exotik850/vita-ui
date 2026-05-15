#![feature(type_changing_struct_update)]
//! # vita-ui
//!
//! A modeless, immediate-mode-style UI framework for PlayStation Vita homebrew,
//! built on top of [`vita2d-rs`](https://crates.io/crates/vita2d-rs).
//!
//! ## Design
//!
//! `vita-ui` provides a thin, opt-in framework around the familiar builder
//! pattern. You bring your own state, and the library deals with drawing
//! and input routing for you. Every callback is a Rust closure or function
//! pointer — no virtual tables, no allocation beyond what you explicitly
//! create.
//!
//! ## Quick start
//!
//! ```no_run
#![doc = include_str!("../examples/hello_vitaui.rs")]
//! ```

// pub mod input;
pub mod button;
pub mod layout;
pub mod menu;
pub mod style;
pub mod text;
pub mod widget;

mod app;

pub use app::{
    App, AppConfig, AppEventReceiver, AppInit, AppInput, AppRender, AppUpdate, FrameCtx, NoOp,
    RenderCtx,
};

/// Convenience re-exports.
pub mod prelude {
    pub use crate::app::{
        App, AppConfig, AppEventReceiver, AppInit, AppInput, AppRender, AppUpdate, FrameCtx,
        InputHandling, RenderCtx,
    };
    pub use crate::button::Button;
    // pub use crate::input::{ButtonState, VitaInput};
    pub use crate::layout::{Flex, FlexDir, LayoutTree, Spacer};
    pub use crate::menu::Menu;
    pub use crate::style::{Style, StyleSheet};
    pub use crate::text::Text;
    pub use crate::widget::Widget;

    pub use vita_input::ControllerInput;
    pub use vita2d_rs::prelude::*;
}
