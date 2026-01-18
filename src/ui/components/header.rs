use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph},
    Frame,
};
use chrono::Local;
use crate::app::AppState;

pub struct HeaderWidget;

impl HeaderWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let total = state.agents.root_agents.len();
        let processing = state.agents.processing_count();
        let pending = state.agents.active_count();
        let time = Local::now().format("%H:%M").to_string();

        let mut spans = vec![
            Span::styled(" TmuxCC ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" {} agents ", total), Style::default().fg(Color::White)),
        ];

        // Processing count
        if processing > 0 {
            spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(format!(" {} {} working ", state.spinner_frame(), processing), Style::default().fg(Color::Yellow)));
        }

        // Pending count
        spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
        if pending > 0 {
            spans.push(Span::styled(format!(" ⚠ {} pending ", pending), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
        } else {
            spans.push(Span::styled(" ✓ ready ", Style::default().fg(Color::Green)));
        }

        // Time
        spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(format!(" {} ", time), Style::default().fg(Color::DarkGray)));

        let line = Line::from(spans);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Gray));

        let paragraph = Paragraph::new(line).block(block);
        frame.render_widget(paragraph, area);
    }
}
