//! Extensible zone and plugin rendering system for [ratatui](https://ratatui.rs).
//!
//! `ratatui-zonekit` lets plugins **own** UI zones in a ratatui application.
//! Plugins declare what they need (tabs, panels, overlays), and the host
//! application decides where and how to render them.
//!
//! # Design Principles
//!
//! - **Request, don't mutate**: plugins request zones, the host approves.
//! - **Safe delegation**: plugin renders are wrapped in `catch_unwind`.
//! - **Theme-agnostic**: works with any styling system (themekit, raw Style, custom).
//! - **Model B**: plugins declare intent, host renders. Plugins never touch `Frame`.
//!
//! # Quick Start
//!
//! ```rust
//! use ratatui_zonekit::{ZonePlugin, ZoneId, RenderContext};
//! use ratatui::buffer::Buffer;
//! use ratatui::layout::Rect;
//!
//! struct MyPlugin;
//!
//! impl ZonePlugin for MyPlugin {
//!     fn id(&self) -> &str { "my-plugin" }
//!     fn render(&self, _zone_id: ZoneId, _ctx: &RenderContext, area: Rect, buf: &mut Buffer) -> bool {
//!         use ratatui::widgets::{Paragraph, Widget};
//!         Paragraph::new("Hello from plugin!").render(area, buf);
//!         true
//!     }
//! }
//! ```

mod plugin;
mod registry;
mod render;
mod zone;

pub use plugin::{RenderContext, ZoneEvent, ZonePlugin};
pub use registry::{RegistrationResult, ZoneRegistry};
pub use render::SafeRenderer;
pub use zone::{ZoneHint, ZoneId, ZoneRequest, ZoneSpec};
