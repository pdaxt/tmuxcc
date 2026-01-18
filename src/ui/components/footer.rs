use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;

/// Footer widget showing available keybindings
pub struct FooterWidget;

impl FooterWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let key_style = Style::default().fg(Color::Yellow);
        let text_style = Style::default().fg(Color::White);
        let sep_style = Style::default().fg(Color::DarkGray);
        let input_style = Style::default().fg(Color::Green).add_modifier(Modifier::BOLD);

        // Different display based on focus
        let spans = if state.is_input_focused() {
            // Input focused - show input-specific hints
            vec![
                Span::styled(" INPUT ", input_style),
                Span::styled("│", sep_style),
                Span::styled(" [Enter]", key_style),
                Span::styled(" Send ", text_style),
                Span::styled("[S-Enter]", key_style),
                Span::styled(" Newline ", text_style),
                Span::styled("[Esc]", key_style),
                Span::styled(" Back ", text_style),
            ]
        } else {
            // Normal mode display
            let mut spans = vec![
                Span::styled(" [Y]", key_style),
                Span::styled(" Approve ", text_style),
                Span::styled("[N]", key_style),
                Span::styled(" Reject ", text_style),
                Span::styled("[A]", key_style),
                Span::styled(" All ", text_style),
                Span::styled("│", sep_style),
                Span::styled(" [F]", key_style),
                Span::styled(" Focus ", text_style),
                Span::styled("[→]", key_style),
                Span::styled(" Input ", text_style),
                Span::styled("│", sep_style),
                Span::styled(" [Space]", key_style),
                Span::styled(" Select ", text_style),
                Span::styled("[1-9]", key_style),
                Span::styled(" Choice ", text_style),
            ];

            // Show selection count if any
            if !state.selected_agents.is_empty() {
                spans.push(Span::styled("│", sep_style));
                spans.push(Span::styled(
                    format!(" {} selected ", state.selected_agents.len()),
                    Style::default().fg(Color::Cyan),
                ));
            }

            // Add error message if present
            if let Some(error) = &state.last_error {
                spans.push(Span::styled("│", sep_style));
                spans.push(Span::styled(
                    format!(" Error: {} ", error),
                    Style::default().fg(Color::Red),
                ));
            }

            spans
        };

        let line = Line::from(spans);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        let paragraph = Paragraph::new(line).block(block);

        frame.render_widget(paragraph, area);
    }
}
