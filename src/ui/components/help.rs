use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::ui::Layout;

/// Help popup widget
pub struct HelpWidget;

impl HelpWidget {
    pub fn render(frame: &mut Frame, area: Rect) {
        let popup_area = Layout::centered_popup(area, 60, 70);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        let key_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::White);
        let section_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let help_text = vec![
            Line::from(vec![Span::styled("Navigation", section_style)]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled("  j / ↓    ", key_style),
                Span::styled("Next agent", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  k / ↑    ", key_style),
                Span::styled("Previous agent", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Tab      ", key_style),
                Span::styled("Next agent (cycle)", desc_style),
            ]),
            Line::from(vec![]),
            Line::from(vec![Span::styled("Selection", section_style)]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled("  Space    ", key_style),
                Span::styled("Toggle selection of current agent", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+a   ", key_style),
                Span::styled("Select all agents", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Esc      ", key_style),
                Span::styled("Clear selection / Close subagent log", desc_style),
            ]),
            Line::from(vec![]),
            Line::from(vec![Span::styled("Actions", section_style)]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled("  y / Y    ", key_style),
                Span::styled("Approve pending request(s)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  n / N    ", key_style),
                Span::styled("Reject pending request(s)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  a / A    ", key_style),
                Span::styled("Approve all pending requests", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  1-9      ", key_style),
                Span::styled("Send number choice to agent", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  ← / →    ", key_style),
                Span::styled("Switch focus (Sidebar / Input)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  C-Enter  ", key_style),
                Span::styled("Send input to all selected agents", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  f / F    ", key_style),
                Span::styled("Focus on selected pane in tmux", desc_style),
            ]),
            Line::from(vec![]),
            Line::from(vec![Span::styled("View", section_style)]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled("  s / S    ", key_style),
                Span::styled("Toggle subagent log", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  t / T    ", key_style),
                Span::styled("Toggle TODO/Tools display", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Q        ", key_style),
                Span::styled("Toggle queue panel", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  C-u/C-d  ", key_style),
                Span::styled("Scroll preview up/down", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  PgUp/Dn  ", key_style),
                Span::styled("Scroll preview up/down", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  g        ", key_style),
                Span::styled("Scroll to bottom (latest)", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  < / >    ", key_style),
                Span::styled("Resize sidebar", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  r        ", key_style),
                Span::styled("Refresh / clear error", desc_style),
            ]),
            Line::from(vec![]),
            Line::from(vec![Span::styled("General", section_style)]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled("  h / ?    ", key_style),
                Span::styled("Toggle this help", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  q        ", key_style),
                Span::styled("Quit", desc_style),
            ]),
            Line::from(vec![]),
            Line::from(vec![Span::styled(
                "  Press any key to close this help",
                Style::default().fg(Color::DarkGray),
            )]),
        ];

        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let paragraph = Paragraph::new(help_text).block(block);

        frame.render_widget(paragraph, popup_area);
    }
}
