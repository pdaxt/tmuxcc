use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
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
        let lines: Vec<Line> = if state.is_input_focused() {
            // Input focused - show input-specific hints
            vec![
                Line::from(vec![
                    Span::styled(" INPUT ", input_style),
                    Span::styled("│", sep_style),
                    Span::styled(" [Enter]", key_style),
                    Span::styled(" Send ", text_style),
                    Span::styled("[S-Enter]", key_style),
                    Span::styled(" Newline ", text_style),
                    Span::styled("[←→]", key_style),
                    Span::styled(" Move cursor ", text_style),
                ]),
                Line::from(vec![
                    Span::styled(" [Esc]", key_style),
                    Span::styled(" Back to sidebar ", text_style),
                    Span::styled("[Home/End]", key_style),
                    Span::styled(" Jump ", text_style),
                ]),
            ]
        } else {
            // Normal mode - 2 lines
            let mut line1 = vec![
                Span::styled(" [Y]", key_style),
                Span::styled(" Approve ", text_style),
                Span::styled("[N]", key_style),
                Span::styled(" Reject ", text_style),
                Span::styled("[A]", key_style),
                Span::styled(" All ", text_style),
                Span::styled("│", sep_style),
                Span::styled(" [1-9]", key_style),
                Span::styled(" Choice ", text_style),
                Span::styled("│", sep_style),
                Span::styled(" [Space]", key_style),
                Span::styled(" Select ", text_style),
            ];

            // Show selection count on line 1
            if !state.selected_agents.is_empty() {
                line1.push(Span::styled(
                    format!("({})", state.selected_agents.len()),
                    Style::default().fg(Color::Cyan),
                ));
            }

            let mut line2 = vec![
                Span::styled(" [→]", key_style),
                Span::styled(" Input ", text_style),
                Span::styled("[F]", key_style),
                Span::styled(" Focus pane ", text_style),
                Span::styled("[S]", key_style),
                Span::styled(" Subagents ", text_style),
                Span::styled("│", sep_style),
                Span::styled(" [?]", key_style),
                Span::styled(" Help ", text_style),
                Span::styled("[Q]", key_style),
                Span::styled(" Quit ", text_style),
            ];

            // Add error message on line 2
            if let Some(error) = &state.last_error {
                line2.push(Span::styled("│", sep_style));
                line2.push(Span::styled(
                    format!(" ✗ {} ", truncate_error(error, 30)),
                    Style::default().fg(Color::Red),
                ));
            }

            vec![Line::from(line1), Line::from(line2)]
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Gray));

        let paragraph = Paragraph::new(lines).block(block);

        frame.render_widget(paragraph, area);
    }
}

fn truncate_error(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max_len - 1).collect::<String>())
    }
}
