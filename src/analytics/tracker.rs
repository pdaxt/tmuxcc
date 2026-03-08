//! Real-time usage tracker — aggregates token events per session.

use std::collections::HashMap;
use std::time::Instant;

use super::db::AnalyticsDb;

/// A token usage event
#[derive(Debug, Clone)]
pub struct TokenEvent {
    pub session_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read: u64,
    pub tool_name: Option<String>,
}

/// Per-session metrics
#[derive(Debug, Clone, Default)]
pub struct SessionMetrics {
    pub session_id: String,
    pub project: String,
    pub pane_num: u8,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read: u64,
    pub tool_calls: u64,
    pub estimated_cost_usd: f64,
    pub started_at: Option<Instant>,
    /// Token events per minute (for sparkline)
    pub tokens_per_minute: Vec<u64>,
    /// Tools used with counts
    pub tool_usage: HashMap<String, u64>,
}

impl SessionMetrics {
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens + self.total_output_tokens
    }

    fn recalculate_cost(&mut self) {
        self.estimated_cost_usd = (self.total_input_tokens as f64 * 3.0
            + self.total_output_tokens as f64 * 15.0
            + self.total_cache_read as f64 * 0.30)
            / 1_000_000.0;
    }
}

/// Tracks usage across all sessions in real-time
#[derive(Debug)]
pub struct UsageTracker {
    sessions: HashMap<String, SessionMetrics>,
    db: Option<AnalyticsDb>,
}

impl UsageTracker {
    pub fn new() -> Self {
        let db = AnalyticsDb::open().ok();
        if db.is_none() {
            tracing::warn!("Failed to open analytics DB, metrics won't persist");
        }
        Self {
            sessions: HashMap::new(),
            db,
        }
    }

    /// Register a new session
    pub fn register_session(
        &mut self,
        session_id: &str,
        pane_num: u8,
        project: &str,
        role: &str,
        theme: &str,
    ) {
        let metrics = SessionMetrics {
            session_id: session_id.to_string(),
            project: project.to_string(),
            pane_num,
            started_at: Some(Instant::now()),
            ..Default::default()
        };
        self.sessions.insert(session_id.to_string(), metrics);

        if let Some(ref db) = self.db {
            let _ = db.create_session(session_id, pane_num, project, role, theme);
        }
    }

    /// Record a token usage event
    pub fn record(&mut self, event: TokenEvent) {
        let metrics = self
            .sessions
            .entry(event.session_id.clone())
            .or_insert_with(|| SessionMetrics {
                session_id: event.session_id.clone(),
                ..Default::default()
            });

        metrics.total_input_tokens += event.input_tokens;
        metrics.total_output_tokens += event.output_tokens;
        metrics.total_cache_read += event.cache_read;
        metrics.tool_calls += 1;
        metrics.recalculate_cost();

        if let Some(ref tool) = event.tool_name {
            *metrics.tool_usage.entry(tool.clone()).or_insert(0) += 1;
        }

        // Persist to DB
        if let Some(ref db) = self.db {
            let _ = db.record_tokens(
                &event.session_id,
                event.input_tokens,
                event.output_tokens,
                event.cache_read,
                event.tool_name.as_deref(),
            );
        }
    }

    /// Get metrics for a session
    pub fn session_metrics(&self, session_id: &str) -> Option<&SessionMetrics> {
        self.sessions.get(session_id)
    }

    /// Get all session metrics
    pub fn all_metrics(&self) -> &HashMap<String, SessionMetrics> {
        &self.sessions
    }

    /// Get aggregate totals
    pub fn totals(&self) -> SessionMetrics {
        let mut total = SessionMetrics::default();
        for m in self.sessions.values() {
            total.total_input_tokens += m.total_input_tokens;
            total.total_output_tokens += m.total_output_tokens;
            total.total_cache_read += m.total_cache_read;
            total.tool_calls += m.tool_calls;
        }
        total.recalculate_cost();
        total
    }

    /// Total cost across all sessions in this DX Terminal instance
    pub fn session_cost(&self) -> f64 {
        self.sessions.values().map(|m| m.estimated_cost_usd).sum()
    }

    /// Total cost for today (includes previous sessions from DB)
    pub fn today_cost(&self) -> f64 {
        let db_total = self.today_totals();
        db_total.estimated_cost() + self.session_cost()
    }

    /// End a session
    pub fn end_session(&mut self, session_id: &str) {
        if let Some(ref db) = self.db {
            let _ = db.end_session(session_id);
        }
    }

    /// Get today's totals from DB (includes previous sessions)
    pub fn today_totals(&self) -> super::db::DayTotal {
        self.db
            .as_ref()
            .and_then(|db| db.today_totals().ok())
            .unwrap_or_default()
    }

    /// Get per-project totals for today
    pub fn today_by_project(&self) -> Vec<super::db::ProjectTotal> {
        self.db
            .as_ref()
            .and_then(|db| db.today_by_project().ok())
            .unwrap_or_default()
    }
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}
