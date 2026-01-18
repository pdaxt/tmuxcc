use ratatui::style::{Color, Modifier, Style};

/// Central style definitions for the application
pub struct Styles;

impl Styles {
    // Status colors
    pub fn idle() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn processing() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn awaiting_approval() -> Style {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    }

    pub fn error() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn unknown() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    // UI element styles
    pub fn header() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn normal() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn dimmed() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn highlight() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border() -> Style {
        Style::default().fg(Color::Gray)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(Color::Cyan)
    }

    // Agent type colors
    pub fn claude_code() -> Style {
        Style::default().fg(Color::Magenta)
    }

    pub fn opencode() -> Style {
        Style::default().fg(Color::Blue)
    }

    pub fn codex_cli() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn gemini_cli() -> Style {
        Style::default().fg(Color::Yellow)
    }

    // Subagent styles
    pub fn subagent_running() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn subagent_completed() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn subagent_failed() -> Style {
        Style::default().fg(Color::Red)
    }

    // Footer/Help styles
    pub fn footer_key() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn footer_text() -> Style {
        Style::default().fg(Color::White)
    }
}
