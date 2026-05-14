//! Application runner — owns the vita2d context, drives the game loop,
//! and dispatches to user-provided callbacks.
//!
//! This module exports the builder-style [`App`] type and the core traits
//! (`AppInit`, `AppUpdate`, `AppRender`, `AppInput`, `AppEventReceiver`).

use std::marker::PhantomData;

use vita_input::ControllerInput;
use vita2d_rs::prelude::*;

use crate::style::StyleSheet;

#[derive(Debug, Clone, Copy)]
pub enum InputHandling {
    Polling,
    Blocking,
}

/// Configuration for the application loop.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// If `Some`, the loop will try to maintain this FPS.
    /// If `None`, runs as fast as the hardware allows.
    pub target_fps: Option<u32>,
    /// Enable or disable V-Sync (vertical blank synchronisation).
    pub vsync: bool,
    pub block_input: InputHandling, // If true, vita2d will block until the next input event instead of polling.
}

impl AppConfig {
    /// Create a new config with defaults (vsync on, no FPS cap).
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set a target FPS.
    pub fn with_target_fps(mut self, target_fps: u32) -> Self {
        self.target_fps = Some(target_fps);
        self
    }

    /// Builder: enable or disable V-Sync.
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Builder: enable or disable input blocking.
    pub fn with_block_input(mut self, block_input: InputHandling) -> Self {
        self.block_input = block_input;
        self
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            target_fps: Some(60),
            vsync: true,
            block_input: InputHandling::Polling,
        }
    }
}

// ---------------------------------------------------------------------------
// App — the main runner
// ---------------------------------------------------------------------------

/// The application runner.
///
/// `State` is your game/UI state.
/// The generic parameters `Init`, `Update`, `Render`, `Input`, and `F`
/// are callback types.  They default to [`NoOp`] so you only wire up what
/// you need.
///
/// # Example
///
/// ```no_run
/// use vita_ui::prelude::*;
///
/// struct MyState { counter: u32 }
///
/// let mut app = App::new()
///     .with_state(MyState { counter: 0 })
///     .with_update(|state: &mut MyState, _ctx: &FrameCtx| {
///         state.counter += 1;
///     })
///     .with_render(|state: &MyState, draw: &mut Drawing| {
///         draw.clear_screen();
///         // ... draw ...
///     });
/// app.run();
/// ```
pub struct App<
    State = (),
    Init = NoOp,
    Update = NoOp,
    Render = NoOp,
    Input = NoOpInput,
    F = NoOp,
    Event = vita_input::ControllerInput,
> {
    init: Option<Init>,
    update: Option<Update>,
    render: Option<Render>,
    input: Option<Input>,
    _event: PhantomData<Event>,
    event_receiver: Option<F>,
    state: State,
    config: AppConfig,
    style: StyleSheet,
    vita2d: Vita2d,
}

impl App<()> {
    /// Create a new `App` with default (unit) state and no callbacks.
    /// You must at least call `.with_state()`, `.with_render()` before
    /// calling `.run()`.
    pub fn new() -> Self {
        Self {
            init: None,
            update: None,
            render: None,
            input: None,
            event_receiver: None,
            state: (),
            config: AppConfig::new(),
            vita2d: Vita2d::init().expect("failed to initialise vita2d"),
            style: StyleSheet::default(),
            _event: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Builder methods (available for all type states)
// ---------------------------------------------------------------------------

impl<State, Init, Update, Render, Input, F, Event>
    App<State, Init, Update, Render, Input, F, Event>
{
    /// Set application state.
    pub fn with_state<NewState>(
        self,
        state: NewState,
    ) -> App<NewState, Init, Update, Render, Input, F, Event> {
        App { state, ..self }
    }

    /// Set the config.
    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = config;
        self
    }

    /// Set a custom stylesheet.
    pub fn with_style(mut self, style: StyleSheet) -> Self {
        self.style = style;
        self
    }

    /// Register an init callback (called once before the first frame).
    pub fn with_init<NewInit: AppInit<State, Event>>(
        self,
        init: NewInit,
    ) -> App<State, NewInit, Update, Render, Input, F, Event> {
        App {
            init: Some(init),
            ..self
        }
    }

    /// Register an update callback (called every frame).
    pub fn with_update<NewUpdate: AppUpdate<State, Event>>(
        self,
        update: NewUpdate,
    ) -> App<State, Init, NewUpdate, Render, Input, F, Event> {
        App {
            update: Some(update),
            ..self
        }
    }

    /// Register a render callback (called every frame).
    pub fn with_render<NewRender: AppRender<State, Event>>(
        self,
        render: NewRender,
    ) -> App<State, Init, Update, NewRender, Input, F, Event> {
        App {
            render: Some(render),
            ..self
        }
    }

    /// Register an input callback (called per-event).
    pub fn with_input<NewInput: AppInput<State, Event>>(
        self,
        input: NewInput,
    ) -> App<State, Init, Update, Render, NewInput, F, Event> {
        App {
            input: Some(input),
            ..self
        }
    }

    /// Register a custom event source.
    pub fn with_event_receiver<NewF: AppEventReceiver<Event>>(
        self,
        event_receiver: NewF,
    ) -> App<State, Init, Update, Render, Input, NewF, Event> {
        App {
            event_receiver: Some(event_receiver),
            ..self
        }
    }
}

// ---------------------------------------------------------------------------
// run() — only available when Event = ControllerInput (the default)
// ---------------------------------------------------------------------------

/// Context passed to update and render callbacks.
pub struct FrameCtx {
    /// The global stylesheet.
    pub style: StyleSheet,
}

/// Context passed to render callbacks.
pub struct RenderCtx<'a, 'b> {
    /// The active drawing session.
    pub draw: &'a mut Drawing<'b>,
    /// The global stylesheet.
    pub style: &'a StyleSheet,
}

impl<
    'app,
    State,
    Init: AppInit<State, ControllerInput>,
    Update: AppUpdate<State, ControllerInput>,
    Render: AppRender<State, ControllerInput>,
    Input: AppInput<State, ControllerInput>,
    F: AppEventReceiver<ControllerInput>,
> App<State, Init, Update, Render, Input, F, ControllerInput>
{
    /// Run the application loop.
    ///
    /// This never returns. It initialises vita2d, enters the main loop,
    /// and tears down vita2d on drop.
    pub fn run(mut self) {
        let vita = self.vita2d;
        vita.set_vsync(self.config.vsync);
        println!("vita-ui: Starting main loop with config: {:?}", self.config);

        if let Some(init) = &mut self.init {
            init.init(&mut self.state);
        }

        let target_frame_time = self
            .config
            .target_fps
            .map(|fps| std::time::Duration::from_secs_f64(1.0 / fps as f64));

        loop {
            let frame_start = if target_frame_time.is_some() {
                Some(std::time::Instant::now())
            } else {
                None
            };

            vita.common_dialog_update();

            // --- Input processing ---
            // if let Some(event_receiver) = &mut self.event_receiver {
            //     while let Some(event) = event_receiver.receive() {
            //         input.merge(event);
            //     }
            // }

            if let Some(input_handler) = &mut self.input {
                let input = match self.config.block_input {
                    InputHandling::Polling => ControllerInput::poll(),
                    InputHandling::Blocking => ControllerInput::read(),
                };
                input_handler.input(&mut self.state, input);
                // for event in input.events() {
                //     input_handler.input(&mut self.state, event);
                // }
            }

            // --- Update ---
            if let Some(update) = &mut self.update {
                let ctx = FrameCtx {
                    style: self.style.clone(),
                };
                update.update(&mut self.state, &ctx);
            }

            // --- Render ---
            {
                let mut draw = vita.start_drawing();
                draw.set_clear_color(self.style.bg_color);

                if let Some(render) = &mut self.render {
                    {
                        let ctx = RenderCtx {
                            draw: &mut draw,
                            style: &self.style,
                        };
                        render.render(&mut self.state, ctx);
                    }
                } else {
                    draw.clear_screen();
                }
            }

            vita.wait_rendering_done();
            vita.swap_buffers();
            // `draw` is dropped here → vita2d_end_drawing() called

            // Frame pacing
            if let (Some(target), Some(start)) = (target_frame_time, frame_start) {
                let elapsed = start.elapsed();
                if elapsed < target {
                    // spin-wait — on Vita we don't have std::thread::sleep
                    // with good resolution, but the VBlank already paces us
                    // when vsync is on.
                    std::hint::spin_loop();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// Called once before the main loop starts.
pub trait AppInit<State, Event = ControllerInput> {
    fn init(&mut self, state: &mut State);
}

/// Called every frame for logic updates.
pub trait AppUpdate<State, Event = ControllerInput> {
    /// `ctx` provides read-only frame context.
    fn update(&mut self, state: &mut State, ctx: &FrameCtx);
}

/// Called every frame for drawing.
pub trait AppRender<State, Event = ControllerInput> {
    /// `ctx.draw` is the active drawing session. Drop happens
    /// automatically when this method returns.
    fn render(&mut self, state: &mut State, ctx: RenderCtx<'_, '_>);
}

/// Called for each input event.
pub trait AppInput<State, Event = ControllerInput> {
    fn input(&mut self, state: &mut State, event: Event);
}

/// A source that produces events (e.g., polling controller state).
pub trait AppEventReceiver<Event> {
    fn receive(&mut self) -> Option<Event>;
}

// ---------------------------------------------------------------------------
// Blanket impls for closures
// ---------------------------------------------------------------------------

impl<State, Event, F: FnMut(&mut State)> AppInit<State, Event> for F {
    fn init(&mut self, state: &mut State) {
        self(state);
    }
}

impl<State, Event, F: FnMut(&mut State, &FrameCtx)> AppUpdate<State, Event> for F {
    fn update(&mut self, state: &mut State, ctx: &FrameCtx) {
        self(state, ctx);
    }
}

impl<State, Event, F: FnMut(&mut State, RenderCtx<'_, '_>)> AppRender<State, Event> for F {
    fn render(&mut self, state: &mut State, ctx: RenderCtx<'_, '_>) {
        self(state, ctx);
    }
}

impl<State, Event, F: FnMut(&mut State, Event)> AppInput<State, Event> for F {
    fn input(&mut self, state: &mut State, event: Event) {
        self(state, event);
    }
}

impl<Event, F: FnMut() -> Option<Event>> AppEventReceiver<Event> for F {
    fn receive(&mut self) -> Option<Event> {
        self()
    }
}

// ---------------------------------------------------------------------------
// NoOp — default "do nothing" callbacks
// ---------------------------------------------------------------------------

/// A no-op type used as the default for all callback slots.
pub struct NoOp;

/// A no-op type specifically for the input callback slot.
pub struct NoOpInput;

impl<State, Event> AppInit<State, Event> for NoOp {
    fn init(&mut self, _state: &mut State) {}
}

impl<State, Event> AppUpdate<State, Event> for NoOp {
    fn update(&mut self, _state: &mut State, _ctx: &FrameCtx) {}
}

impl<State, Event> AppRender<State, Event> for NoOp {
    fn render(&mut self, _state: &mut State, _ctx: RenderCtx<'_, '_>) {}
}

impl<State, Event> AppInput<State, Event> for NoOpInput {
    fn input(&mut self, _state: &mut State, _event: Event) {}
}

impl<Event> AppEventReceiver<Event> for NoOp {
    fn receive(&mut self) -> Option<Event> {
        None
    }
}
