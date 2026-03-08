//! Panel Graph System — dynamic split layout for the terminal IDE.
//!
//! Replaces fixed layouts with a binary tree of splits.
//! Every view (editor, file tree, terminal, agent list) is a Panel.

pub mod graph;
pub mod manager;

pub use graph::{PanelId, PanelKind, PanelNode, SplitDirection};
pub use manager::PanelManager;
