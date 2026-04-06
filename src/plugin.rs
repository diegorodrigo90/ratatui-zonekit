//! Plugin trait — the contract between host and plugin.
//!
//! A [`ZonePlugin`] declares what zones it wants and how to render them.
//! The host calls `zones()` once at registration, then `render()` every
//! frame for each visible zone the plugin owns.
//!
//! Plugins receive a [`RenderContext`] with metadata — they never
//! access the `Frame` directly (Model B safety).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::zone::{ZoneId, ZoneRequest};

/// Context passed to a plugin during rendering.
///
/// Contains frame metadata and the base style the host wants the plugin
/// to use. Plugins should respect `base_style` for theme consistency,
/// but it's not enforced at the type level — this is a convention.
///
/// The context is **theme-agnostic**: the host sets `base_style` from
/// whatever styling system it uses (themekit, raw Style, custom).
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// Base style (background + foreground) from the host's theme.
    ///
    /// Plugins should apply this as the default style for their zone
    /// to maintain visual consistency with the rest of the application.
    pub base_style: Style,
    /// Whether this zone currently has keyboard focus.
    pub focused: bool,
    /// Terminal width (for responsive rendering decisions).
    pub terminal_width: u16,
    /// Terminal height.
    pub terminal_height: u16,
    /// Current tick count (for animations).
    pub tick: usize,
}

impl RenderContext {
    /// Creates a new render context with the given base style.
    #[must_use]
    pub fn new(base_style: Style, terminal_width: u16, terminal_height: u16) -> Self {
        Self {
            base_style,
            focused: false,
            terminal_width,
            terminal_height,
            tick: 0,
        }
    }

    /// Sets the focused state.
    #[must_use]
    pub fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the tick count.
    #[must_use]
    pub fn with_tick(mut self, tick: usize) -> Self {
        self.tick = tick;
        self
    }
}

/// A plugin that owns one or more zones in the TUI.
///
/// Implement this trait to contribute UI to a ratatui application.
/// The host calls methods in this order:
///
/// 1. `id()` + `zones()` — once at registration
/// 2. `on_register(zone_id)` — once per granted zone
/// 3. `render(zone_id, ctx, area, buf)` — every frame per visible zone
/// 4. `on_event(zone_id, event)` — when a relevant event occurs
///
/// # Theme Agnosticism
///
/// The `RenderContext.base_style` carries the host's theme as a plain
/// `ratatui::style::Style`. This works with any styling system:
///
/// - **ratatui-themekit**: host sets `base_style = theme.style_base()`
/// - **Raw styles**: host sets `base_style = Style::default().bg(Color::Rgb(...))`
/// - **Custom**: any `Style` value
///
/// Plugins that want to use themekit directly can depend on it
/// themselves — zonekit does not require or prevent this.
pub trait ZonePlugin: Send + Sync {
    /// Unique plugin identifier (e.g., `"official.bmad"`).
    fn id(&self) -> &str;

    /// Zone requests — what zones this plugin wants to own.
    ///
    /// Called once during registration. Each request is evaluated by
    /// the host, which may grant or deny based on available space.
    fn zones(&self) -> Vec<ZoneRequest> {
        vec![]
    }

    /// Called when the host grants a zone to this plugin.
    ///
    /// Uses `&self` — plugins that need to store the zone ID should
    /// use interior mutability (e.g., `Cell`, `Mutex`).
    fn on_register(&self, _zone_id: ZoneId) {}

    /// Renders the plugin's content into the granted zone.
    ///
    /// `area` is the allocated `Rect` — plugin MUST NOT write outside it.
    /// `buf` is the buffer slice for this area.
    /// `ctx` carries theme style, focus state, and terminal dimensions.
    ///
    /// Return `false` if the zone has nothing to show (host may hide it).
    fn render(&self, zone_id: ZoneId, ctx: &RenderContext, area: Rect, buf: &mut Buffer) -> bool;

    /// Called when a keyboard or mouse event occurs in this zone.
    ///
    /// Return `true` if the event was handled (prevents propagation).
    fn on_event(&self, _zone_id: ZoneId, _event: &ZoneEvent) -> bool {
        false
    }
}

/// Events delivered to zone plugins.
#[derive(Debug, Clone)]
pub enum ZoneEvent {
    /// A key was pressed while this zone was focused.
    Key {
        /// Key code.
        code: ratatui::crossterm::event::KeyCode,
        /// Key modifiers.
        modifiers: ratatui::crossterm::event::KeyModifiers,
    },
    /// Mouse click inside this zone.
    Click {
        /// Column relative to zone's left edge.
        x: u16,
        /// Row relative to zone's top edge.
        y: u16,
    },
    /// Mouse scroll inside this zone.
    Scroll {
        /// Positive = down, negative = up.
        delta: i8,
    },
    /// Zone was focused.
    FocusGained,
    /// Zone lost focus.
    FocusLost,
    /// Terminal was resized.
    Resize {
        /// New terminal width.
        width: u16,
        /// New terminal height.
        height: u16,
    },
}

#[cfg(test)]
#[allow(clippy::unnecessary_literal_bound)]
mod tests {
    use super::*;

    struct TestPlugin;

    impl ZonePlugin for TestPlugin {
        fn id(&self) -> &str {
            "test"
        }

        fn render(&self, _: ZoneId, ctx: &RenderContext, area: Rect, buf: &mut Buffer) -> bool {
            use ratatui::widgets::{Paragraph, Widget};
            let text = if ctx.focused { "FOCUSED" } else { "normal" };
            Paragraph::new(text).style(ctx.base_style).render(area, buf);
            true
        }
    }

    #[test]
    fn plugin_id_is_accessible() {
        let p = TestPlugin;
        assert_eq!(p.id(), "test");
    }

    #[test]
    fn default_zones_is_empty() {
        let p = TestPlugin;
        assert!(p.zones().is_empty());
    }

    #[test]
    fn render_context_builder() {
        let ctx = RenderContext::new(Style::default(), 120, 40)
            .with_focus(true)
            .with_tick(42);
        assert!(ctx.focused);
        assert_eq!(ctx.tick, 42);
        assert_eq!(ctx.terminal_width, 120);
    }

    #[test]
    fn plugin_renders_into_buffer() {
        let p = TestPlugin;
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        let ctx = RenderContext::new(Style::default(), 80, 24);
        let rendered = p.render(ZoneId::new(1), &ctx, area, &mut buf);
        assert!(rendered);
        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("normal"));
    }

    #[test]
    fn plugin_renders_focused() {
        let p = TestPlugin;
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        let ctx = RenderContext::new(Style::default(), 80, 24).with_focus(true);
        p.render(ZoneId::new(1), &ctx, area, &mut buf);
        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("FOCUSED"));
    }

    #[test]
    fn default_on_event_returns_false() {
        let p = TestPlugin;
        let handled = p.on_event(
            ZoneId::new(1),
            &ZoneEvent::Key {
                code: ratatui::crossterm::event::KeyCode::Char('a'),
                modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
            },
        );
        assert!(!handled);
    }
}
