# ratatui-zonekit

[![Crates.io](https://img.shields.io/crates/v/ratatui-zonekit.svg)](https://crates.io/crates/ratatui-zonekit)
[![docs.rs](https://docs.rs/ratatui-zonekit/badge.svg)](https://docs.rs/ratatui-zonekit)
[![CI](https://github.com/diegorodrigo90/ratatui-zonekit/actions/workflows/ci.yml/badge.svg)](https://github.com/diegorodrigo90/ratatui-zonekit/actions)
[![codecov](https://codecov.io/gh/diegorodrigo90/ratatui-zonekit/graph/badge.svg)](https://codecov.io/gh/diegorodrigo90/ratatui-zonekit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Extensible zone and plugin rendering system for [ratatui](https://ratatui.rs).**

Let plugins **own** UI zones in your TUI application. Plugins declare what they need (tabs, panels, overlays), and the host decides where and how to render them — safely.

## The Problem

Building an extensible TUI means plugins need to render their own content. But giving plugins raw `Frame` access is unsafe — a misbehaving plugin can draw anywhere, crash the app, or ignore your theme.

## The Solution

```rust
use ratatui_zonekit::{ZonePlugin, ZoneId, ZoneRequest, RenderContext};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

struct SprintPlugin;

impl ZonePlugin for SprintPlugin {
    fn id(&self) -> &str { "bmad.sprint" }

    fn zones(&self) -> Vec<ZoneRequest> {
        vec![
            ZoneRequest::tab("bmad.sprint", "Sprint").with_order(10),
            ZoneRequest::sidebar("bmad.hooks", "Hooks").with_order(20),
        ]
    }

    fn render(&self, _zone_id: ZoneId, ctx: &RenderContext, area: Rect, buf: &mut Buffer) -> bool {
        use ratatui::widgets::{Paragraph, Widget};
        Paragraph::new("Sprint status here")
            .style(ctx.base_style)
            .render(area, buf);
        true
    }
}
```

The host renders plugins safely:

```rust
use ratatui_zonekit::{ZoneRegistry, SafeRenderer};
use std::sync::Arc;

let mut registry = ZoneRegistry::new();
let plugin = Arc::new(SprintPlugin);
registry.register(plugin);

// In the render loop, for each zone:
// SafeRenderer::render(&plugin, zone_id, &ctx, area, buf);
// If the plugin panics, the zone shows "[plugin crashed]" instead.
```

## Features

### Zone Types

| Zone | Description | Example |
|------|-------------|---------|
| **Tab** | Replaces main content when active | Sprint board, file tree, logs |
| **Sidebar** | Panel in sidebar column | Plugin status, hooks, config |
| **Control** | Panel in control column (wide) | Agent controls, actions |
| **Overlay** | Floating popup on top | Search, picker, modal |
| **StatusBar** | Single line at bottom | Plugin status text |

### Safe Rendering

Every plugin render is wrapped in `catch_unwind`. A crashing plugin shows an error message in its zone — the rest of the application continues normally.

### Theme-Agnostic

Zonekit passes `RenderContext.base_style` (a plain `ratatui::style::Style`) to plugins. Works with:

- **ratatui-themekit**: `ctx.base_style = theme.style_base()`
- **Raw styles**: `ctx.base_style = Style::default().bg(Color::Rgb(...))`
- **Any theme system**: just set the Style

Plugins that want richer theme access can depend on a theme crate directly — zonekit doesn't require or prevent this.

### Event Routing

Plugins receive events for their zones:

```rust
fn on_event(&mut self, zone_id: ZoneId, event: &ZoneEvent) -> bool {
    match event {
        ZoneEvent::Key { code, .. } => { /* handle key */ true }
        ZoneEvent::Click { x, y } => { /* handle click */ true }
        _ => false
    }
}
```

## Design Principles

- **Request, don't mutate**: plugins request zones, the host approves
- **Safe delegation**: `catch_unwind` isolates plugin panics
- **Theme-agnostic**: works with any styling system
- **Model B**: plugins declare intent, host renders
- **Zero opinion on layout**: zonekit manages zones, not the overall layout

## License

MIT
