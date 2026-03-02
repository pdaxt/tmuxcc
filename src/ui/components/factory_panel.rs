use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;

pub struct FactoryPanelWidget;

impl FactoryPanelWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let block = Block::default()
            .title(" Factory Pipeline ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        if state.factory_requests.is_empty() {
            let hint = Paragraph::new(Line::from(vec![Span::styled(
                "  No factory requests. Press : to submit one.",
                Style::default().fg(Color::DarkGray),
            )]))
            .block(block);
            frame.render_widget(hint, area);
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        for req in &state.factory_requests {
            // Request header
            let status_color = match req.status.as_str() {
                "complete" => Color::Green,
                "running" => Color::Cyan,
                "pending" => Color::Yellow,
                "error" => Color::Red,
                _ => Color::White,
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", req.status.to_uppercase()),
                    Style::default().fg(Color::Black).bg(status_color),
                ),
                Span::raw(" "),
                Span::styled(
                    truncate_str(&req.request, 60),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]));

            // Classification
            if !req.classification.project.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("   → ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&req.classification.project, Style::default().fg(Color::Cyan)),
                    Span::styled(" / ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&req.classification.role, Style::default().fg(Color::Magenta)),
                    Span::styled(" / ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&req.classification.req_type, Style::default().fg(Color::Yellow)),
                ]));
            }

            // Pipeline stages
            for task in &req.tasks {
                let (icon, color) = match task.status.as_str() {
                    "done" => ("●", Color::Green),
                    "running" => ("◐", Color::Cyan),
                    "pending" => ("○", Color::DarkGray),
                    "error" => ("✗", Color::Red),
                    _ => ("○", Color::DarkGray),
                };

                let pane_info = task
                    .pane
                    .map(|p| format!(" [P{}]", p))
                    .unwrap_or_else(|| " [--]".to_string());

                lines.push(Line::from(vec![
                    Span::styled(format!("   {} ", icon), Style::default().fg(color)),
                    Span::styled(
                        format!("{:<4}", task.stage.to_uppercase()),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(pane_info, Style::default().fg(Color::DarkGray)),
                    Span::raw(" "),
                    Span::styled(&task.role, Style::default().fg(Color::White)),
                    Span::raw("  "),
                    Span::styled(
                        task.status.to_string(),
                        Style::default().fg(color),
                    ),
                ]));
            }

            lines.push(Line::raw(""));
        }

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
