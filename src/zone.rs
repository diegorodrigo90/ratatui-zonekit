//! Zone types — identifiers, hints, requests, and specs.
//!
//! A zone is a named rectangular area in the terminal that a plugin
//! can own and render into. Zones are created by the host application
//! in response to plugin requests.

use ratatui::layout::Rect;

/// Unique identifier for a zone, assigned by the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZoneId(u32);

impl ZoneId {
    /// Creates a new zone identifier.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw numeric identifier.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for ZoneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "zone:{}", self.0)
    }
}

/// Where a plugin wants its zone to appear.
///
/// The host maps these hints to actual layout positions. A hint is
/// a preference, not a guarantee — the host may ignore or reposition
/// based on available space and terminal tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneHint {
    /// Main content area (replaces default content when active).
    Tab,
    /// Sidebar column (Standard + Wide tiers).
    Sidebar,
    /// Control column (Wide tier only).
    Control,
    /// Floating overlay on top of all zones.
    Overlay,
    /// Status bar area (single line, bottom).
    StatusBar,
}

/// A plugin's request to create or own a zone.
///
/// Plugins submit requests during registration. The host evaluates
/// each request and either grants a [`ZoneId`] or denies it.
#[derive(Debug, Clone)]
pub struct ZoneRequest {
    /// Namespaced identifier: `"{plugin_id}.{local_name}"`.
    pub name: String,
    /// Where the plugin wants the zone.
    pub hint: ZoneHint,
    /// Display label (for tabs, panel headers).
    pub label: String,
    /// Preferred height in lines (0 = fill available). Ignored for tabs.
    pub preferred_height: u16,
    /// Preferred width in columns (0 = fill available). Ignored for tabs.
    pub preferred_width: u16,
    /// Minimum terminal width for this zone to appear (0 = always).
    pub min_terminal_width: u16,
    /// Display order within the zone hint (lower = first).
    pub order: u8,
}

impl ZoneRequest {
    /// Creates a tab zone request.
    #[must_use]
    pub fn tab(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hint: ZoneHint::Tab,
            label: label.into(),
            preferred_height: 0,
            preferred_width: 0,
            min_terminal_width: 0,
            order: 128,
        }
    }

    /// Creates a sidebar panel zone request.
    #[must_use]
    pub fn sidebar(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hint: ZoneHint::Sidebar,
            label: label.into(),
            preferred_height: 0,
            preferred_width: 0,
            min_terminal_width: 120,
            order: 128,
        }
    }

    /// Creates a floating overlay zone request.
    #[must_use]
    pub fn overlay(
        name: impl Into<String>,
        label: impl Into<String>,
        width: u16,
        height: u16,
    ) -> Self {
        Self {
            name: name.into(),
            hint: ZoneHint::Overlay,
            label: label.into(),
            preferred_height: height,
            preferred_width: width,
            min_terminal_width: 0,
            order: 0,
        }
    }

    /// Sets the display order.
    #[must_use]
    pub fn with_order(mut self, order: u8) -> Self {
        self.order = order;
        self
    }

    /// Sets the minimum terminal width.
    #[must_use]
    pub fn with_min_width(mut self, width: u16) -> Self {
        self.min_terminal_width = width;
        self
    }
}

/// A resolved zone — the host's response to a [`ZoneRequest`].
///
/// Contains the allocated area and metadata. Updated every frame
/// since terminal resize can change the allocated rect.
#[derive(Debug, Clone)]
pub struct ZoneSpec {
    /// Unique zone identifier.
    pub id: ZoneId,
    /// Original request name.
    pub name: String,
    /// Display label.
    pub label: String,
    /// Zone hint (where it was placed).
    pub hint: ZoneHint,
    /// Currently allocated area (updated per frame).
    pub area: Rect,
    /// Whether this zone is currently visible.
    pub visible: bool,
    /// Display order used for sorting.
    pub order: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_id_display() {
        assert_eq!(ZoneId::new(42).to_string(), "zone:42");
    }

    #[test]
    fn zone_id_equality() {
        assert_eq!(ZoneId::new(1), ZoneId::new(1));
        assert_ne!(ZoneId::new(1), ZoneId::new(2));
    }

    #[test]
    fn zone_request_tab() {
        let req = ZoneRequest::tab("bmad.sprint", "Sprint");
        assert_eq!(req.hint, ZoneHint::Tab);
        assert_eq!(req.name, "bmad.sprint");
        assert_eq!(req.label, "Sprint");
    }

    #[test]
    fn zone_request_sidebar() {
        let req = ZoneRequest::sidebar("github.pr", "Pull Requests");
        assert_eq!(req.hint, ZoneHint::Sidebar);
        assert_eq!(req.min_terminal_width, 120);
    }

    #[test]
    fn zone_request_overlay() {
        let req = ZoneRequest::overlay("finder.search", "Search", 60, 20);
        assert_eq!(req.hint, ZoneHint::Overlay);
        assert_eq!(req.preferred_width, 60);
        assert_eq!(req.preferred_height, 20);
    }

    #[test]
    fn zone_request_builder() {
        let req = ZoneRequest::tab("test.tab", "Test")
            .with_order(10)
            .with_min_width(140);
        assert_eq!(req.order, 10);
        assert_eq!(req.min_terminal_width, 140);
    }
}
