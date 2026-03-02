use ratatui::layout::{Constraint, Direction, Rect};

/// Layout manager for the application
pub struct Layout;

impl Layout {
    /// Creates the main layout with header, content, optional queue, and footer
    pub fn main_layout(area: Rect) -> Vec<Rect> {
        Self::main_layout_with_queue(area, true)
    }

    /// Creates the main layout with configurable queue and dashboard visibility
    pub fn main_layout_with_queue(area: Rect, show_queue: bool) -> Vec<Rect> {
        Self::main_layout_full(area, show_queue, false)
    }

    /// Creates the main layout with all optional panels
    pub fn main_layout_full(area: Rect, show_queue: bool, show_dashboard: bool) -> Vec<Rect> {
        Self::main_layout_all(area, show_queue, show_dashboard, false)
    }

    /// Creates the main layout with all optional panels including factory
    pub fn main_layout_all(
        area: Rect,
        show_queue: bool,
        show_dashboard: bool,
        show_factory: bool,
    ) -> Vec<Rect> {
        let queue_height = if show_queue { 8 } else { 0 };
        let dashboard_height = if show_dashboard { 12 } else { 0 };
        let factory_height = if show_factory { 10 } else { 0 };
        ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),                 // Header
                Constraint::Min(10),                   // Content area (agents + preview)
                Constraint::Length(queue_height),       // Queue panel
                Constraint::Length(dashboard_height),   // Dashboard panel
                Constraint::Length(factory_height),     // Factory panel
                Constraint::Length(1),                  // Footer
            ])
            .split(area)
            .to_vec()
    }

    /// Splits the content area into 2 columns: agent list (left) and preview (right)
    pub fn content_layout(area: Rect, sidebar_width: u16) -> (Rect, Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(sidebar_width),
                Constraint::Percentage(100 - sidebar_width),
            ])
            .split(area);
        (chunks[0], chunks[1])
    }

    /// Splits the content area with summary, preview, and input
    /// Returns (sidebar, summary, preview, input)
    pub fn content_layout_with_input(
        area: Rect,
        sidebar_width: u16,
        input_height: u16,
        show_summary: bool,
    ) -> (Rect, Rect, Rect, Rect) {
        let columns = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(sidebar_width),
                Constraint::Percentage(100 - sidebar_width),
            ])
            .split(area);

        let summary_height = if show_summary { 15 } else { 0 };

        let right_side = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(summary_height), // Summary (TODO + activity) - 2 columns
                Constraint::Min(5),                 // Preview (pane content)
                Constraint::Length(input_height + 2), // Input area (+ border)
            ])
            .split(columns[1]);

        (columns[0], right_side[0], right_side[1], right_side[2])
    }

    /// Splits the content area with subagent log (2 columns, right side split vertically)
    pub fn content_layout_with_log(area: Rect, sidebar_width: u16) -> (Rect, Rect, Rect) {
        let columns = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(sidebar_width),
                Constraint::Percentage(100 - sidebar_width),
            ])
            .split(area);

        let right_side = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Preview
                Constraint::Percentage(40), // Subagent log
            ])
            .split(columns[1]);

        (columns[0], right_side[0], right_side[1])
    }

    /// Creates a centered popup area
    pub fn centered_popup(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
        let vertical = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - height_percent) / 2),
                Constraint::Percentage(height_percent),
                Constraint::Percentage((100 - height_percent) / 2),
            ])
            .split(area);

        ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - width_percent) / 2),
                Constraint::Percentage(width_percent),
                Constraint::Percentage((100 - width_percent) / 2),
            ])
            .split(vertical[1])[1]
    }
}
