use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::AppState;

/// Input widget for text entry at the bottom of the right column
pub struct InputWidget;

impl InputWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let buffer = state.get_input();
        let is_focused = state.is_input_focused();

        // Get target agent name
        let target_name = state.selected_agent()
            .map(|a| a.abbreviated_path())
            .unwrap_or_else(|| "None".to_string());

        let title = format!(" Input → {} ", target_name);

        let border_color = if is_focused { Color::Green } else { Color::DarkGray };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        // Build content with cursor (only show cursor when focused)
        let lines: Vec<Line> = Self::build_lines_with_cursor(buffer, is_focused);

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);

        // Set cursor position for IME support (only when focused)
        if is_focused {
            Self::set_cursor_position(frame, area, buffer);
        }
    }

    /// Build lines with cursor indicator
    fn build_lines_with_cursor(buffer: &str, is_focused: bool) -> Vec<Line<'static>> {
        let cursor_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Green);
        let text_style = Style::default().fg(Color::White);
        let hint_style = Style::default().fg(Color::DarkGray);

        if buffer.is_empty() {
            if is_focused {
                return vec![Line::from(vec![
                    Span::styled("█", cursor_style),
                    Span::styled(" (Shift+Enter: newline, Enter: send, Esc: clear)",
                        hint_style),
                ])];
            } else {
                return vec![Line::from(vec![
                    Span::styled("← arrow key to input", hint_style),
                ])];
            }
        }

        let mut lines = Vec::new();
        let buffer_lines: Vec<&str> = buffer.split('\n').collect();

        for (i, line_text) in buffer_lines.iter().enumerate() {
            let is_last_line = i == buffer_lines.len() - 1;

            if is_last_line && is_focused {
                // Last line has cursor at end when focused
                lines.push(Line::from(vec![
                    Span::styled(line_text.to_string(), text_style),
                    Span::styled("█", cursor_style),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(line_text.to_string(), text_style),
                ]));
            }
        }

        lines
    }

    /// Set cursor position for IME (Input Method Editor) support
    fn set_cursor_position(frame: &mut Frame, area: Rect, buffer: &str) {
        // Calculate cursor position using display width (handles full-width chars)
        let lines: Vec<&str> = buffer.split('\n').collect();
        let last_line = lines.last().unwrap_or(&"");
        // Use unicode width for proper full-width character handling
        let last_line_width = last_line.width() as u16;

        let cursor_y = area.y + 1 + (lines.len().saturating_sub(1)) as u16;
        let cursor_x = area.x + 1 + last_line_width;

        // Ensure cursor is within bounds
        let cursor_x = cursor_x.min(area.x + area.width.saturating_sub(2));
        let cursor_y = cursor_y.min(area.y + area.height.saturating_sub(2));

        frame.set_cursor_position((cursor_x, cursor_y));
    }

    /// Calculate required height based on buffer content
    pub fn calculate_height(buffer: &str, max_height: u16) -> u16 {
        let line_count = buffer.split('\n').count() as u16;
        // Minimum 1 line, maximum max_height lines
        line_count.max(1).min(max_height)
    }
}
