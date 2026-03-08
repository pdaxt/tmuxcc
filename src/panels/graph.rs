//! Binary split tree for panel layout.
//!
//! The UI is a tree where leaves are panels (editor, terminal, file tree, etc.)
//! and internal nodes are splits (horizontal or vertical).

use ratatui::layout::Rect;
use std::path::PathBuf;

/// Unique panel identifier.
pub type PanelId = u32;

/// What kind of content a panel displays.
#[derive(Debug, Clone)]
pub enum PanelKind {
    /// File explorer tree.
    FileTree,
    /// Text editor viewing/editing a file.
    Editor { path: PathBuf },
    /// Terminal PTY pane (running an agent or shell).
    Terminal { pane_id: u8 },
    /// Agent list sidebar.
    AgentList,
    /// Diff view for a file.
    Diff { path: PathBuf },
    /// Analytics/cost dashboard.
    Analytics,
    /// Empty/placeholder panel.
    Empty,
}

/// Direction of a split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    /// Left | Right
    Vertical,
    /// Top / Bottom
    Horizontal,
}

/// A node in the panel tree.
#[derive(Debug, Clone)]
pub enum PanelNode {
    /// A leaf panel that renders content.
    Leaf {
        id: PanelId,
        kind: PanelKind,
    },
    /// A split containing two child nodes.
    Split {
        direction: SplitDirection,
        /// Position of the split as a ratio (0.0 to 1.0).
        ratio: f32,
        first: Box<PanelNode>,
        second: Box<PanelNode>,
    },
}

/// A panel with its computed screen area.
#[derive(Debug, Clone)]
pub struct PanelLayout {
    pub id: PanelId,
    pub kind: PanelKind,
    pub area: Rect,
}

impl PanelNode {
    /// Create a new leaf panel.
    pub fn leaf(id: PanelId, kind: PanelKind) -> Self {
        PanelNode::Leaf { id, kind }
    }

    /// Create a vertical split (left | right).
    pub fn vsplit(ratio: f32, first: PanelNode, second: PanelNode) -> Self {
        PanelNode::Split {
            direction: SplitDirection::Vertical,
            ratio,
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Create a horizontal split (top / bottom).
    pub fn hsplit(ratio: f32, first: PanelNode, second: PanelNode) -> Self {
        PanelNode::Split {
            direction: SplitDirection::Horizontal,
            ratio,
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Compute layout rectangles for all leaf panels given the available area.
    pub fn layout(&self, area: Rect) -> Vec<PanelLayout> {
        let mut result = Vec::new();
        self.layout_inner(area, &mut result);
        result
    }

    fn layout_inner(&self, area: Rect, out: &mut Vec<PanelLayout>) {
        match self {
            PanelNode::Leaf { id, kind } => {
                out.push(PanelLayout {
                    id: *id,
                    kind: kind.clone(),
                    area,
                });
            }
            PanelNode::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let (area_first, area_second) = split_rect(area, *direction, *ratio);
                first.layout_inner(area_first, out);
                second.layout_inner(area_second, out);
            }
        }
    }

    /// Find a panel by ID.
    pub fn find(&self, target_id: PanelId) -> Option<&PanelNode> {
        match self {
            PanelNode::Leaf { id, .. } => {
                if *id == target_id {
                    Some(self)
                } else {
                    None
                }
            }
            PanelNode::Split { first, second, .. } => {
                first.find(target_id).or_else(|| second.find(target_id))
            }
        }
    }

    /// Collect all leaf panel IDs.
    pub fn leaf_ids(&self) -> Vec<PanelId> {
        match self {
            PanelNode::Leaf { id, .. } => vec![*id],
            PanelNode::Split { first, second, .. } => {
                let mut ids = first.leaf_ids();
                ids.extend(second.leaf_ids());
                ids
            }
        }
    }

    /// Count all leaf panels.
    pub fn leaf_count(&self) -> usize {
        match self {
            PanelNode::Leaf { .. } => 1,
            PanelNode::Split { first, second, .. } => {
                first.leaf_count() + second.leaf_count()
            }
        }
    }

    /// Replace a leaf panel with a split containing the original and a new panel.
    pub fn split_panel(
        &mut self,
        target_id: PanelId,
        direction: SplitDirection,
        new_id: PanelId,
        new_kind: PanelKind,
        ratio: f32,
    ) -> bool {
        match self {
            PanelNode::Leaf { id, .. } if *id == target_id => {
                let original = std::mem::replace(
                    self,
                    PanelNode::Leaf {
                        id: 0,
                        kind: PanelKind::Empty,
                    },
                );
                let new_leaf = PanelNode::Leaf {
                    id: new_id,
                    kind: new_kind,
                };
                *self = PanelNode::Split {
                    direction,
                    ratio,
                    first: Box::new(original),
                    second: Box::new(new_leaf),
                };
                true
            }
            PanelNode::Split { first, second, .. } => {
                first.split_panel(target_id, direction, new_id, new_kind.clone(), ratio)
                    || second.split_panel(target_id, direction, new_id, new_kind, ratio)
            }
            _ => false,
        }
    }

    /// Remove a panel by ID. Returns true if removed.
    /// If the panel is one half of a split, the other half replaces the split.
    pub fn remove_panel(&mut self, target_id: PanelId) -> bool {
        match self {
            PanelNode::Split { first, second, .. } => {
                // Check if first child is the target leaf
                if matches!(first.as_ref(), PanelNode::Leaf { id, .. } if *id == target_id) {
                    *self = *second.clone();
                    return true;
                }
                // Check if second child is the target leaf
                if matches!(second.as_ref(), PanelNode::Leaf { id, .. } if *id == target_id) {
                    *self = *first.clone();
                    return true;
                }
                // Recurse
                first.remove_panel(target_id) || second.remove_panel(target_id)
            }
            _ => false,
        }
    }
}

/// Split a rectangle into two parts based on direction and ratio.
fn split_rect(area: Rect, direction: SplitDirection, ratio: f32) -> (Rect, Rect) {
    let ratio = ratio.clamp(0.1, 0.9);

    match direction {
        SplitDirection::Vertical => {
            let left_width = (area.width as f32 * ratio) as u16;
            let right_width = area.width.saturating_sub(left_width);
            (
                Rect::new(area.x, area.y, left_width, area.height),
                Rect::new(area.x + left_width, area.y, right_width, area.height),
            )
        }
        SplitDirection::Horizontal => {
            let top_height = (area.height as f32 * ratio) as u16;
            let bottom_height = area.height.saturating_sub(top_height);
            (
                Rect::new(area.x, area.y, area.width, top_height),
                Rect::new(area.x, area.y + top_height, area.width, bottom_height),
            )
        }
    }
}

/// Create the default IDE layout.
pub fn default_layout() -> PanelNode {
    // File tree (left 20%) | Main area (middle 60%) | Agent list (right 20%)
    PanelNode::vsplit(
        0.18,
        PanelNode::leaf(1, PanelKind::FileTree),
        PanelNode::vsplit(
            0.78,
            // Main area: Editor (top 70%) / Terminal (bottom 30%)
            PanelNode::hsplit(
                0.70,
                PanelNode::leaf(2, PanelKind::Empty), // Editor opens here
                PanelNode::leaf(3, PanelKind::Terminal { pane_id: 0 }),
            ),
            PanelNode::leaf(4, PanelKind::AgentList),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_computation() {
        let tree = default_layout();
        let area = Rect::new(0, 0, 200, 50);
        let panels = tree.layout(area);

        // Should have 4 leaf panels
        assert_eq!(panels.len(), 4);

        // All panels should have non-zero area
        for p in &panels {
            assert!(p.area.width > 0);
            assert!(p.area.height > 0);
        }
    }

    #[test]
    fn test_leaf_ids() {
        let tree = default_layout();
        let ids = tree.leaf_ids();
        assert_eq!(ids, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_split_panel() {
        let mut tree = default_layout();
        let success = tree.split_panel(
            2,
            SplitDirection::Vertical,
            5,
            PanelKind::Editor {
                path: PathBuf::from("test.rs"),
            },
            0.5,
        );
        assert!(success);
        assert_eq!(tree.leaf_count(), 5);
    }

    #[test]
    fn test_remove_panel() {
        let mut tree = PanelNode::vsplit(
            0.5,
            PanelNode::leaf(1, PanelKind::FileTree),
            PanelNode::leaf(2, PanelKind::AgentList),
        );
        let removed = tree.remove_panel(1);
        assert!(removed);
        assert_eq!(tree.leaf_count(), 1);
    }

    #[test]
    fn test_find_panel() {
        let tree = default_layout();
        assert!(tree.find(1).is_some());
        assert!(tree.find(4).is_some());
        assert!(tree.find(99).is_none());
    }

    #[test]
    fn test_split_rect_vertical() {
        let area = Rect::new(0, 0, 100, 50);
        let (left, right) = split_rect(area, SplitDirection::Vertical, 0.3);
        assert_eq!(left.width, 30);
        assert_eq!(right.width, 70);
        assert_eq!(left.height, 50);
        assert_eq!(right.height, 50);
    }

    #[test]
    fn test_split_rect_horizontal() {
        let area = Rect::new(0, 0, 100, 50);
        let (top, bottom) = split_rect(area, SplitDirection::Horizontal, 0.6);
        assert_eq!(top.height, 30);
        assert_eq!(bottom.height, 20);
        assert_eq!(top.width, 100);
    }
}
