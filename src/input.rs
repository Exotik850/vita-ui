//! Vita controller input abstraction.
//!
//! Provides [`VitaInput`] — a snapshot of the current controller state —
//! and [`ButtonState`] for tracking individual button transitions.

use std::time::Instant;

/// The state of a single button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Button was just pressed this frame.
    Pressed,
    /// Button is being held down.
    Held,
    /// Button was just released this frame.
    Released,
    /// Button is not pressed.
    Up,
}

impl ButtonState {
    /// Returns `true` if the button is currently down (Pressed or Held).
    pub fn is_down(self) -> bool {
        matches!(self, ButtonState::Pressed | ButtonState::Held)
    }

    /// Returns `true` if the button was just pressed this frame.
    pub fn is_pressed(self) -> bool {
        matches!(self, ButtonState::Pressed)
    }

    /// Returns `true` if the button was just released this frame.
    pub fn is_released(self) -> bool {
        matches!(self, ButtonState::Released)
    }
}

/// A snapshot of the Vita's physical controls for one frame.
///
/// Each button reports a [`ButtonState`] that captures edge transitions
/// (Pressed / Released) as well as sustained state (Held / Up).
#[derive(Debug, Clone)]
pub struct VitaInput {
    /// Frame timestamp.
    pub time: Instant,

    // Face buttons
    pub cross: ButtonState,
    pub circle: ButtonState,
    pub triangle: ButtonState,
    pub square: ButtonState,

    // D-pad
    pub dpad_up: ButtonState,
    pub dpad_down: ButtonState,
    pub dpad_left: ButtonState,
    pub dpad_right: ButtonState,

    // Shoulder
    pub l1: ButtonState,
    pub r1: ButtonState,

    // System
    pub start: ButtonState,
    pub select: ButtonState,

    // Analog sticks (raw -128..127)
    pub left_stick_x: i8,
    pub left_stick_y: i8,
    pub right_stick_x: i8,
    pub right_stick_y: i8,

    // Touch (front panel)
    /// `Some((x, y))` if the front touch panel is being touched.
    pub touch: Option<(u16, u16)>,

    // Rear touch
    /// `Some((x, y))` if the rear touch panel is being touched.
    pub rear_touch: Option<(u16, u16)>,
}

impl VitaInput {
    /// Create an empty input snapshot (nothing pressed).
    pub fn empty() -> Self {
        Self {
            time: Instant::now(),
            cross: ButtonState::Up,
            circle: ButtonState::Up,
            triangle: ButtonState::Up,
            square: ButtonState::Up,
            dpad_up: ButtonState::Up,
            dpad_down: ButtonState::Up,
            dpad_left: ButtonState::Up,
            dpad_right: ButtonState::Up,
            l1: ButtonState::Up,
            r1: ButtonState::Up,
            start: ButtonState::Up,
            select: ButtonState::Up,
            left_stick_x: 0,
            left_stick_y: 0,
            right_stick_x: 0,
            right_stick_y: 0,
            touch: None,
            rear_touch: None,
        }
    }

    /// Merge another input snapshot into this one (OR of button states).
    pub fn merge(&mut self, other: VitaInput) {
        self.cross = merge_button(self.cross, other.cross);
        self.circle = merge_button(self.circle, other.circle);
        self.triangle = merge_button(self.triangle, other.triangle);
        self.square = merge_button(self.square, other.square);
        self.dpad_up = merge_button(self.dpad_up, other.dpad_up);
        self.dpad_down = merge_button(self.dpad_down, other.dpad_down);
        self.dpad_left = merge_button(self.dpad_left, other.dpad_left);
        self.dpad_right = merge_button(self.dpad_right, other.dpad_right);
        self.l1 = merge_button(self.l1, other.l1);
        self.r1 = merge_button(self.r1, other.r1);
        self.start = merge_button(self.start, other.start);
        self.select = merge_button(self.select, other.select);
        if other.left_stick_x != 0 {
            self.left_stick_x = other.left_stick_x;
        }
        if other.left_stick_y != 0 {
            self.left_stick_y = other.left_stick_y;
        }
        if other.right_stick_x != 0 {
            self.right_stick_x = other.right_stick_x;
        }
        if other.right_stick_y != 0 {
            self.right_stick_y = other.right_stick_y;
        }
        if other.touch.is_some() {
            self.touch = other.touch;
        }
        if other.rear_touch.is_some() {
            self.rear_touch = other.rear_touch;
        }
    }

    /// Iterate over all button events in this snapshot.
    pub fn events(&self) -> impl Iterator<Item = VitaInput> + '_ {
        // Return clones of self for each active button — this lets the
        // input handler pattern-match on individual events.
        // For simplicity, we just yield the whole snapshot once.
        std::iter::once(self.clone())
    }
}

fn merge_button(a: ButtonState, b: ButtonState) -> ButtonState {
    match (a, b) {
        (ButtonState::Pressed, _) | (_, ButtonState::Pressed) => ButtonState::Pressed,
        (ButtonState::Held, _) | (_, ButtonState::Held) => ButtonState::Held,
        (ButtonState::Released, _) | (_, ButtonState::Released) => ButtonState::Released,
        _ => ButtonState::Up,
    }
}

/// Poll the Vita controller and return a snapshot.
///
/// This is a stub that returns an empty snapshot. On real hardware, this
/// would call `sceCtrlPeekBufferPositive` or similar.
///
/// # Platform note
///
/// When targeting `armv7-sony-vita-newlibeabihf`, replace this with actual
/// `sceCtrl` calls. The stub allows the library to compile on the host for
/// testing.
#[cfg(not(target_os = "vita"))]
pub fn poll() -> VitaInput {
    VitaInput::empty()
}

/// Real Vita controller polling.
#[cfg(target_os = "vita")]
pub fn poll() -> VitaInput {
    // We use the vitasdk-sys or psvita-sys crate for sceCtrl.
    // For now, return empty — users can provide their own event receiver.
    VitaInput::empty()
}
