//! Analytics panel — token usage, costs, per-project breakdown.

use crate::analytics::UsageTracker;
use crate::github::GitInfo;
use ratatui::{
    layout::{Constraint, Direction, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

pub struct AnalyticsWidget;

impl AnalyticsWidget {
    /// Render the analytics panel showing token usage and costs
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        tracker: &UsageTracker,
        git_info: &HashMap<String, GitInfo>,
    ) {
        // Split into 3 columns: session totals | per-project | git status
        let cols = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Percentage(30),
            ])
            .split(area);

        Self::render_session_totals(frame, cols[0], tracker);
        Self::render_project_breakdown(frame, cols[1], tracker);
        Self::render_git_status(frame, cols[2], git_info);
    }

    fn render_session_totals(frame: &mut Frame, area: Rect, tracker: &UsageTracker) {
        let totals = tracker.totals();
        let today = tracker.today_totals();

        let format_tokens = |n: u64| -> String {
            if n >= 1_000_000 {
                format!("{:.1}M", n as f64 / 1_000_000.0)
            } else if n >= 1_000 {
                format!("{:.1}k", n as f64 / 1_000.0)
            } else {
                n.to_string()
            }
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("SESSION", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("  Input:  "),
                Span::styled(
                    format_tokens(totals.total_input_tokens),
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  Output: "),
                Span::styled(
                    format_tokens(totals.total_output_tokens),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Cost:   "),
                Span::styled(
                    format!("${:.2}", totals.estimated_cost_usd),
                    cost_color(totals.estimated_cost_usd),
                ),
                Span::raw("  Tools:  "),
                Span::styled(
                    totals.tool_calls.to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("TODAY", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("  Input:  "),
                Span::styled(
                    format_tokens(today.input_tokens),
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  Output: "),
                Span::styled(
                    format_tokens(today.output_tokens),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Cost:   "),
                Span::styled(
                    format!("${:.2}", today.estimated_cost()),
                    cost_color(today.estimated_cost()),
                ),
                Span::raw("  Total:  "),
                Span::styled(
                    format_tokens(today.total_tokens()),
                    Style::default().fg(Color::White),
                ),
            ]),
        ];

        let block = Block::default()
            .title(" Token Usage ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn render_project_breakdown(frame: &mut Frame, area: Rect, tracker: &UsageTracker) {
        let by_project = tracker.today_by_project();
        let max_lines = (area.height as usize).saturating_sub(2);

        let format_tokens = |n: u64| -> String {
            if n >= 1_000_000 {
                format!("{:.1}M", n as f64 / 1_000_000.0)
            } else if n >= 1_000 {
                format!("{:.1}k", n as f64 / 1_000.0)
            } else {
                n.to_string()
            }
        };

        let lines: Vec<Line> = if by_project.is_empty() {
            vec![Line::from(Span::styled(
                "No data yet",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            by_project
                .iter()
                .take(max_lines)
                .map(|p| {
                    Line::from(vec![
                        Span::styled(
                            format!("{:<14}", truncate(&p.project, 14)),
                            Style::default().fg(Color::White),
                        ),
                        Span::styled(
                            format!("{:>6}", format_tokens(p.output_tokens)),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            format!("${:.2}", p.estimated_cost()),
                            cost_color(p.estimated_cost()),
                        ),
                    ])
                })
                .collect()
        };

        let block = Block::default()
            .title(" By Project (Today) ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta));

        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn render_git_status(
        frame: &mut Frame,
        area: Rect,
        git_info: &HashMap<String, GitInfo>,
    ) {
        let max_lines = (area.height as usize).saturating_sub(2);

        let lines: Vec<Line> = if git_info.is_empty() {
            vec![Line::from(Span::styled(
                "No repos tracked",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            let mut entries: Vec<_> = git_info.iter().collect();
            entries.sort_by_key(|(path, _)| (*path).clone());
            entries
                .iter()
                .take(max_lines)
                .map(|(path, info)| {
                    let project = path.rsplit('/').find(|s| !s.is_empty()).unwrap_or(path);
                    let mut spans = vec![
                        Span::styled(
                            format!("{:<10}", truncate(project, 10)),
                            Style::default().fg(Color::White),
                        ),
                    ];

                    // Branch
                    if !info.branch.is_empty() {
                        spans.push(Span::styled(
                            format!(" {}", truncate(&info.branch, 12)),
                            Style::default().fg(Color::Cyan),
                        ));
                    }

                    // Dirty indicator
                    if info.dirty_files > 0 {
                        spans.push(Span::styled(
                            format!(" *{}", info.dirty_files),
                            Style::default().fg(Color::Yellow),
                        ));
                    }

                    // PR status
                    if let Some(ref pr) = info.pr {
                        let (icon, color) = match pr.state.as_str() {
                            "OPEN" => ("PR", Color::Green),
                            "MERGED" => ("MG", Color::Magenta),
                            "CLOSED" => ("CL", Color::Red),
                            _ => ("PR", Color::White),
                        };
                        spans.push(Span::styled(
                            format!(" {}#{}", icon, pr.number),
                            Style::default().fg(color),
                        ));

                        if let Some(passing) = pr.checks_passing {
                            spans.push(Span::styled(
                                if passing { " \u{2713}" } else { " \u{2717}" },
                                Style::default().fg(if passing { Color::Green } else { Color::Red }),
                            ));
                        }
                    }

                    Line::from(spans)
                })
                .collect()
        };

        let block = Block::default()
            .title(" Git/PRs ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green));

        frame.render_widget(Paragraph::new(lines).block(block), area);
    }
}

fn cost_color(cost: f64) -> Style {
    if cost > 5.0 {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if cost > 1.0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}
