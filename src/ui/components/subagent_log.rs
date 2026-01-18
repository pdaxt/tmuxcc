use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

use crate::agents::SubagentStatus;
use crate::app::AppState;

/// Widget for displaying subagent activity log
pub struct SubagentLogWidget;

impl SubagentLogWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let agent = state.selected_agent();

        let title = if let Some(agent) = agent {
            format!(" Subagent Log: {} ", agent.target)
        } else {
            " Subagent Log ".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Gray));

        let items: Vec<ListItem> = if let Some(agent) = agent {
            if agent.subagents.is_empty() {
                vec![ListItem::new(Line::from(vec![Span::styled(
                    "  No subagent activity detected",
                    Style::default().fg(Color::DarkGray),
                )]))]
            } else {
                agent
                    .subagents
                    .iter()
                    .map(|subagent| {
                        let (indicator, style) = match subagent.status {
                            SubagentStatus::Running => ("▶", Style::default().fg(Color::Cyan)),
                            SubagentStatus::Completed => ("✓", Style::default().fg(Color::Green)),
                            SubagentStatus::Failed => ("✗", Style::default().fg(Color::Red)),
                            SubagentStatus::Unknown => ("?", Style::default().fg(Color::DarkGray)),
                        };

                        let duration = subagent.duration_str();

                        let line = Line::from(vec![
                            Span::raw("  "),
                            Span::styled(indicator, style),
                            Span::raw(" "),
                            Span::styled(
                                subagent.subagent_type.display_name(),
                                Style::default().fg(Color::White),
                            ),
                            Span::raw("  "),
                            Span::styled(&subagent.description, Style::default().fg(Color::Gray)),
                            Span::raw("  "),
                            Span::styled(
                                format!("[{}]", duration),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]);

                        ListItem::new(line)
                    })
                    .collect()
            }
        } else {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "  No agent selected",
                Style::default().fg(Color::DarkGray),
            )]))]
        };

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
