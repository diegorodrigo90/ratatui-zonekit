//! Zone registry — manages zone allocation and plugin ownership.
//!
//! The [`ZoneRegistry`] is the central coordinator. It processes
//! [`ZoneRequest`]s from plugins, allocates [`ZoneId`]s, and tracks
//! which plugin owns which zone. The host queries the registry
//! each frame to determine what to render where.

use std::collections::HashMap;
use std::sync::Arc;

use ratatui::layout::Rect;

use crate::plugin::ZonePlugin;
use crate::zone::{ZoneHint, ZoneId, ZoneSpec};

/// Registration result — whether a zone request was granted or denied.
#[derive(Debug)]
pub enum RegistrationResult {
    /// Zone was granted with this ID.
    Granted(ZoneId),
    /// Zone was denied with a reason.
    Denied(String),
}

/// Manages zone allocation and plugin ownership.
///
/// Create one per application. Register plugins at startup, then
/// query each frame for the current zone layout.
pub struct ZoneRegistry {
    next_id: u32,
    zones: Vec<ZoneSpec>,
    owners: HashMap<ZoneId, Arc<dyn ZonePlugin>>,
}

impl ZoneRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: 1,
            zones: Vec::new(),
            owners: HashMap::new(),
        }
    }

    /// Registers a plugin and processes its zone requests.
    ///
    /// Returns a vec of results (one per request, in order).
    #[allow(clippy::needless_pass_by_value)] // Arc::clone is cheap, by-value is ergonomic
    pub fn register(&mut self, plugin: Arc<dyn ZonePlugin>) -> Vec<RegistrationResult> {
        let requests = plugin.zones();
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            // Check for duplicate names
            if self.zones.iter().any(|z| z.name == request.name) {
                results.push(RegistrationResult::Denied(format!(
                    "zone '{}' already registered",
                    request.name
                )));
                continue;
            }

            let id = ZoneId::new(self.next_id);
            self.next_id += 1;

            self.zones.push(ZoneSpec {
                id,
                name: request.name.clone(),
                label: request.label,
                hint: request.hint,
                area: Rect::default(),
                visible: true,
                order: request.order,
            });

            self.owners.insert(id, Arc::clone(&plugin));
            results.push(RegistrationResult::Granted(id));
        }

        results
    }

    /// Returns all zones matching a hint, sorted by order.
    #[must_use]
    pub fn zones_by_hint(&self, hint: ZoneHint) -> Vec<&ZoneSpec> {
        let mut zones: Vec<&ZoneSpec> = self
            .zones
            .iter()
            .filter(|z| z.hint == hint && z.visible)
            .collect();
        zones.sort_by_key(|z| z.order);
        zones
    }

    /// Returns all tab zones, sorted by order.
    #[must_use]
    pub fn tabs(&self) -> Vec<&ZoneSpec> {
        self.zones_by_hint(ZoneHint::Tab)
    }

    /// Returns the plugin that owns a zone.
    #[must_use]
    pub fn owner(&self, zone_id: ZoneId) -> Option<&Arc<dyn ZonePlugin>> {
        self.owners.get(&zone_id)
    }

    /// Returns a zone spec by ID.
    #[must_use]
    pub fn zone(&self, zone_id: ZoneId) -> Option<&ZoneSpec> {
        self.zones.iter().find(|z| z.id == zone_id)
    }

    /// Returns a zone spec by name.
    #[must_use]
    pub fn zone_by_name(&self, name: &str) -> Option<&ZoneSpec> {
        self.zones.iter().find(|z| z.name == name)
    }

    /// Updates the area for a zone (called by the host each frame).
    pub fn update_area(&mut self, zone_id: ZoneId, area: Rect) {
        if let Some(zone) = self.zones.iter_mut().find(|z| z.id == zone_id) {
            zone.area = area;
        }
    }

    /// Sets visibility for a zone.
    pub fn set_visible(&mut self, zone_id: ZoneId, visible: bool) {
        if let Some(zone) = self.zones.iter_mut().find(|z| z.id == zone_id) {
            zone.visible = visible;
        }
    }

    /// Total number of registered zones.
    #[must_use]
    pub fn len(&self) -> usize {
        self.zones.len()
    }

    /// Whether the registry has no zones.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.zones.is_empty()
    }

    /// All zones (unfiltered).
    #[must_use]
    pub fn all_zones(&self) -> &[ZoneSpec] {
        &self.zones
    }
}

impl Default for ZoneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unnecessary_literal_bound)]
mod tests {
    use ratatui::buffer::Buffer;

    use super::*;
    use crate::plugin::RenderContext;
    use crate::zone::ZoneRequest;

    struct FakePlugin {
        id: &'static str,
        requests: Vec<ZoneRequest>,
    }

    impl ZonePlugin for FakePlugin {
        fn id(&self) -> &str {
            self.id
        }

        fn zones(&self) -> Vec<ZoneRequest> {
            self.requests.clone()
        }

        fn render(&self, _: ZoneId, _: &RenderContext, _: Rect, _: &mut Buffer) -> bool {
            true
        }
    }

    fn plugin_with_tab(id: &'static str, name: &str, label: &str) -> Arc<dyn ZonePlugin> {
        Arc::new(FakePlugin {
            id,
            requests: vec![ZoneRequest::tab(name, label)],
        })
    }

    fn plugin_with_sidebar(id: &'static str, name: &str, label: &str) -> Arc<dyn ZonePlugin> {
        Arc::new(FakePlugin {
            id,
            requests: vec![ZoneRequest::sidebar(name, label)],
        })
    }

    #[test]
    fn empty_registry() {
        let reg = ZoneRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert!(reg.tabs().is_empty());
    }

    #[test]
    fn register_plugin_grants_zone() {
        let mut reg = ZoneRegistry::new();
        let results = reg.register(plugin_with_tab("bmad", "bmad.sprint", "Sprint"));
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], RegistrationResult::Granted(_)));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn duplicate_name_is_denied() {
        let mut reg = ZoneRegistry::new();
        reg.register(plugin_with_tab("a", "shared.name", "Tab A"));
        let results = reg.register(plugin_with_tab("b", "shared.name", "Tab B"));
        assert!(matches!(results[0], RegistrationResult::Denied(_)));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn zones_by_hint_filters_correctly() {
        let mut reg = ZoneRegistry::new();
        reg.register(plugin_with_tab("a", "a.tab", "Tab A"));
        reg.register(plugin_with_sidebar("b", "b.side", "Side B"));
        assert_eq!(reg.zones_by_hint(ZoneHint::Tab).len(), 1);
        assert_eq!(reg.zones_by_hint(ZoneHint::Sidebar).len(), 1);
        assert_eq!(reg.zones_by_hint(ZoneHint::Overlay).len(), 0);
    }

    #[test]
    fn tabs_returns_tab_zones_sorted() {
        let mut reg = ZoneRegistry::new();
        reg.register(Arc::new(FakePlugin {
            id: "b",
            requests: vec![ZoneRequest::tab("b.tab", "B").with_order(20)],
        }));
        reg.register(Arc::new(FakePlugin {
            id: "a",
            requests: vec![ZoneRequest::tab("a.tab", "A").with_order(10)],
        }));
        let tabs = reg.tabs();
        assert_eq!(tabs[0].label, "A");
        assert_eq!(tabs[1].label, "B");
    }

    #[test]
    fn owner_returns_plugin() {
        let mut reg = ZoneRegistry::new();
        let plugin = plugin_with_tab("test", "test.tab", "Test");
        let results = reg.register(plugin);
        if let RegistrationResult::Granted(id) = &results[0] {
            let owner = reg.owner(*id).unwrap();
            assert_eq!(owner.id(), "test");
        }
    }

    #[test]
    fn zone_by_name() {
        let mut reg = ZoneRegistry::new();
        reg.register(plugin_with_tab("x", "x.tab", "X"));
        assert!(reg.zone_by_name("x.tab").is_some());
        assert!(reg.zone_by_name("nonexistent").is_none());
    }

    #[test]
    fn update_area() {
        let mut reg = ZoneRegistry::new();
        let results = reg.register(plugin_with_tab("x", "x.tab", "X"));
        if let RegistrationResult::Granted(id) = &results[0] {
            let new_area = Rect::new(10, 20, 80, 40);
            reg.update_area(*id, new_area);
            assert_eq!(reg.zone(*id).unwrap().area, new_area);
        }
    }

    #[test]
    fn set_visible_hides_zone() {
        let mut reg = ZoneRegistry::new();
        let results = reg.register(plugin_with_tab("x", "x.tab", "X"));
        if let RegistrationResult::Granted(id) = &results[0] {
            reg.set_visible(*id, false);
            assert!(reg.tabs().is_empty(), "hidden tab should not appear");
        }
    }

    #[test]
    fn multiple_plugins_get_unique_ids() {
        let mut reg = ZoneRegistry::new();
        let r1 = reg.register(plugin_with_tab("a", "a.tab", "A"));
        let r2 = reg.register(plugin_with_tab("b", "b.tab", "B"));
        if let (RegistrationResult::Granted(id1), RegistrationResult::Granted(id2)) =
            (&r1[0], &r2[0])
        {
            assert_ne!(id1, id2);
        }
    }
}
