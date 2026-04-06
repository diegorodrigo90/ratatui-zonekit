//! Safe renderer — delegates rendering to plugins with panic isolation.
//!
//! Wraps each plugin's `render()` call in `catch_unwind` so a crashing
//! plugin cannot take down the host application. If a plugin panics,
//! its zone shows an error message instead.

use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, Widget};

use crate::plugin::{RenderContext, ZonePlugin};
use crate::zone::ZoneId;

/// Safe renderer that isolates plugin panics.
///
/// Use this to render plugin zones instead of calling `plugin.render()`
/// directly. If a plugin panics, the zone shows a crash message and
/// the rest of the application continues normally.
pub struct SafeRenderer;

impl SafeRenderer {
    /// Renders a plugin's zone with panic isolation.
    ///
    /// Returns `true` if the plugin rendered successfully, `false` if
    /// it panicked or returned `false` (nothing to show).
    pub fn render(
        plugin: &Arc<dyn ZonePlugin>,
        zone_id: ZoneId,
        ctx: &RenderContext,
        area: Rect,
        buf: &mut Buffer,
    ) -> bool {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            plugin.render(zone_id, ctx, area, buf)
        }));

        if let Ok(rendered) = result {
            rendered
        } else {
            // Plugin panicked — render crash message in the zone
            let crash_msg = format!("[{} crashed]", plugin.id());
            let style = Style::default().fg(ratatui::style::Color::Red);
            Paragraph::new(crash_msg).style(style).render(area, buf);
            false
        }
    }
}

#[cfg(test)]
#[allow(clippy::unnecessary_literal_bound)]
mod tests {
    use super::*;

    struct GoodPlugin;

    impl ZonePlugin for GoodPlugin {
        fn id(&self) -> &str {
            "good"
        }

        fn render(&self, _: ZoneId, _: &RenderContext, area: Rect, buf: &mut Buffer) -> bool {
            Paragraph::new("ok").render(area, buf);
            true
        }
    }

    struct CrashPlugin;

    impl ZonePlugin for CrashPlugin {
        fn id(&self) -> &str {
            "crash"
        }

        fn render(&self, _: ZoneId, _: &RenderContext, _: Rect, _: &mut Buffer) -> bool {
            panic!("plugin bug");
        }
    }

    struct EmptyPlugin;

    impl ZonePlugin for EmptyPlugin {
        fn id(&self) -> &str {
            "empty"
        }

        fn render(&self, _: ZoneId, _: &RenderContext, _: Rect, _: &mut Buffer) -> bool {
            false
        }
    }

    #[test]
    fn good_plugin_renders() {
        let plugin: Arc<dyn ZonePlugin> = Arc::new(GoodPlugin);
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        let ctx = RenderContext::new(Style::default(), 80, 24);
        assert!(SafeRenderer::render(
            &plugin,
            ZoneId::new(1),
            &ctx,
            area,
            &mut buf
        ));
        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("ok"));
    }

    #[test]
    fn crash_plugin_is_caught() {
        let plugin: Arc<dyn ZonePlugin> = Arc::new(CrashPlugin);
        let area = Rect::new(0, 0, 30, 1);
        let mut buf = Buffer::empty(area);
        let ctx = RenderContext::new(Style::default(), 80, 24);
        // Should NOT panic — crash is caught
        let rendered = SafeRenderer::render(&plugin, ZoneId::new(1), &ctx, area, &mut buf);
        assert!(!rendered);
        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(content.contains("[crash crashed]"));
    }

    #[test]
    fn empty_plugin_returns_false() {
        let plugin: Arc<dyn ZonePlugin> = Arc::new(EmptyPlugin);
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        let ctx = RenderContext::new(Style::default(), 80, 24);
        assert!(!SafeRenderer::render(
            &plugin,
            ZoneId::new(1),
            &ctx,
            area,
            &mut buf
        ));
    }
}
