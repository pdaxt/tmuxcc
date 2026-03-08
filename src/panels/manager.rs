//! Panel Manager — focus, navigation, create, close, resize.

use std::path::PathBuf;
use super::graph::{PanelId, PanelKind, PanelLayout, PanelNode, SplitDirection, default_layout};

/// Manages the panel tree, focus, and panel operations.
#[derive(Debug)]
pub struct PanelManager {
    /// The root of the panel tree.
    root: PanelNode,
    /// Currently focused panel ID.
    focused: PanelId,
    /// Next ID to assign to a new panel.
    next_id: PanelId,
    /// Cached layout from last frame.
    cached_layouts: Vec<PanelLayout>,
}

impl PanelManager {
    /// Create a new PanelManager with the default IDE layout.
    pub fn new() -> Self {
        let root = default_layout();
        Self {
            focused: 1, // File tree by default
            next_id: 10, // IDs 1-9 reserved for defaults
            root,
            cached_layouts: Vec::new(),
        }
    }

    /// Get the root panel node.
    pub fn root(&self) -> &PanelNode {
        &self.root
    }

    /// Compute layout and cache it. Returns the panel layouts.
    pub fn compute_layout(&mut self, area: ratatui::layout::Rect) -> &[PanelLayout] {
        self.cached_layouts = self.root.layout(area);
        &self.cached_layouts
    }

    /// Get the cached layouts (from last compute_layout call).
    pub fn layouts(&self) -> &[PanelLayout] {
        &self.cached_layouts
    }

    /// Get the currently focused panel ID.
    pub fn focused(&self) -> PanelId {
        self.focused
    }

    /// Set focus to a specific panel.
    pub fn set_focus(&mut self, id: PanelId) {
        if self.root.find(id).is_some() {
            self.focused = id;
        }
    }

    /// Focus the next panel (cycle through leaves).
    pub fn focus_next(&mut self) {
        let ids = self.root.leaf_ids();
        if ids.is_empty() {
            return;
        }
        let current_idx = ids.iter().position(|&id| id == self.focused).unwrap_or(0);
        let next_idx = (current_idx + 1) % ids.len();
        self.focused = ids[next_idx];
    }

    /// Focus the previous panel.
    pub fn focus_prev(&mut self) {
        let ids = self.root.leaf_ids();
        if ids.is_empty() {
            return;
        }
        let current_idx = ids.iter().position(|&id| id == self.focused).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            ids.len() - 1
        } else {
            current_idx - 1
        };
        self.focused = ids[prev_idx];
    }

    /// Split the focused panel in a direction, inserting a new panel.
    pub fn split(&mut self, direction: SplitDirection, kind: PanelKind) -> PanelId {
        let new_id = self.next_id;
        self.next_id += 1;
        self.root
            .split_panel(self.focused, direction, new_id, kind, 0.5);
        new_id
    }

    /// Open a file in the editor. If there's an empty panel, use it. Otherwise split.
    pub fn open_file(&mut self, path: PathBuf) -> PanelId {
        // Look for an empty panel to replace
        for layout in &self.cached_layouts {
            if matches!(layout.kind, PanelKind::Empty) {
                // Replace the empty panel with an editor
                if let Some(node) = find_leaf_mut(&mut self.root, layout.id) {
                    *node = PanelNode::Leaf {
                        id: layout.id,
                        kind: PanelKind::Editor { path },
                    };
                    self.focused = layout.id;
                    return layout.id;
                }
            }
        }

        // No empty panel — split the focused panel
        let new_id = self.next_id;
        self.next_id += 1;
        self.root.split_panel(
            self.focused,
            SplitDirection::Vertical,
            new_id,
            PanelKind::Editor { path },
            0.5,
        );
        self.focused = new_id;
        new_id
    }

    /// Close the focused panel.
    pub fn close_focused(&mut self) {
        let ids = self.root.leaf_ids();
        if ids.len() <= 1 {
            return; // Don't close the last panel
        }

        let to_close = self.focused;
        // Focus the next panel before closing
        self.focus_next();
        self.root.remove_panel(to_close);
    }

    /// Close a specific panel by ID.
    pub fn close_panel(&mut self, id: PanelId) -> bool {
        let ids = self.root.leaf_ids();
        if ids.len() <= 1 {
            return false;
        }
        if self.focused == id {
            self.focus_next();
        }
        self.root.remove_panel(id)
    }

    /// Get the kind of the focused panel.
    pub fn focused_kind(&self) -> Option<&PanelKind> {
        if let Some(PanelNode::Leaf { kind, .. }) = self.root.find(self.focused) {
            Some(kind)
        } else {
            None
        }
    }

    /// Total number of panels.
    pub fn panel_count(&self) -> usize {
        self.root.leaf_count()
    }
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Find a mutable reference to a leaf node by ID.
fn find_leaf_mut(node: &mut PanelNode, target_id: PanelId) -> Option<&mut PanelNode> {
    match node {
        PanelNode::Leaf { id, .. } if *id == target_id => Some(node),
        PanelNode::Split { first, second, .. } => {
            find_leaf_mut(first, target_id).or_else(|| find_leaf_mut(second, target_id))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager() {
        let mgr = PanelManager::new();
        assert_eq!(mgr.panel_count(), 4); // File tree, editor, terminal, agent list
        assert_eq!(mgr.focused(), 1);
    }

    #[test]
    fn test_focus_cycling() {
        let mut mgr = PanelManager::new();
        assert_eq!(mgr.focused(), 1);
        mgr.focus_next();
        assert_eq!(mgr.focused(), 2);
        mgr.focus_next();
        assert_eq!(mgr.focused(), 3);
        mgr.focus_next();
        assert_eq!(mgr.focused(), 4);
        mgr.focus_next();
        assert_eq!(mgr.focused(), 1); // Wraps around
    }

    #[test]
    fn test_focus_prev() {
        let mut mgr = PanelManager::new();
        mgr.focus_prev();
        assert_eq!(mgr.focused(), 4); // Wraps to last
    }

    #[test]
    fn test_split_panel() {
        let mut mgr = PanelManager::new();
        mgr.set_focus(2);
        let new_id = mgr.split(SplitDirection::Vertical, PanelKind::Editor {
            path: PathBuf::from("test.rs"),
        });
        assert_eq!(mgr.panel_count(), 5);
        assert!(new_id >= 10);
    }

    #[test]
    fn test_open_file_replaces_empty() {
        let mut mgr = PanelManager::new();
        // Panel 2 is PanelKind::Empty in default layout
        let area = ratatui::layout::Rect::new(0, 0, 200, 50);
        mgr.compute_layout(area);
        let id = mgr.open_file(PathBuf::from("src/main.rs"));
        assert_eq!(id, 2); // Should replace the empty panel
        assert_eq!(mgr.panel_count(), 4); // No new panel created
    }

    #[test]
    fn test_close_panel() {
        let mut mgr = PanelManager::new();
        mgr.set_focus(2);
        mgr.close_focused();
        assert_eq!(mgr.panel_count(), 3);
        assert_ne!(mgr.focused(), 2);
    }

    #[test]
    fn test_cannot_close_last_panel() {
        let mut mgr = PanelManager::new();
        // Close 3 of 4 panels
        mgr.close_panel(1);
        mgr.close_panel(2);
        mgr.close_panel(3);
        assert_eq!(mgr.panel_count(), 1);
        // Try to close the last one — should fail
        mgr.close_focused();
        assert_eq!(mgr.panel_count(), 1);
    }
}
