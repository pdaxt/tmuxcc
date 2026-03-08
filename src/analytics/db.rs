//! SQLite persistence for analytics data.

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// Analytics database
pub struct AnalyticsDb {
    conn: Connection,
}

impl std::fmt::Debug for AnalyticsDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyticsDb").finish()
    }
}

impl AnalyticsDb {
    /// Open or create the analytics database
    pub fn open() -> Result<Self> {
        let db_path = Self::db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&db_path)
            .context(format!("Failed to open analytics DB at {}", db_path.display()))?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing)
    #[cfg(test)]
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn db_path() -> Result<PathBuf> {
        let dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("dx-terminal");
        Ok(dir.join("analytics.db"))
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                pane_num INTEGER NOT NULL,
                project TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'developer',
                theme TEXT NOT NULL DEFAULT '',
                started_at TEXT NOT NULL,
                ended_at TEXT,
                total_input_tokens INTEGER NOT NULL DEFAULT 0,
                total_output_tokens INTEGER NOT NULL DEFAULT 0,
                total_cache_read INTEGER NOT NULL DEFAULT 0,
                total_cost_usd REAL NOT NULL DEFAULT 0.0,
                tool_calls INTEGER NOT NULL DEFAULT 0,
                git_commits INTEGER NOT NULL DEFAULT 0,
                files_changed INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS token_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                cache_read INTEGER NOT NULL DEFAULT 0,
                tool_name TEXT,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );

            CREATE TABLE IF NOT EXISTS daily_summary (
                date TEXT NOT NULL,
                project TEXT NOT NULL,
                total_input_tokens INTEGER NOT NULL DEFAULT 0,
                total_output_tokens INTEGER NOT NULL DEFAULT 0,
                total_cache_read INTEGER NOT NULL DEFAULT 0,
                total_cost_usd REAL NOT NULL DEFAULT 0.0,
                session_count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (date, project)
            );

            CREATE INDEX IF NOT EXISTS idx_token_events_session ON token_events(session_id);
            CREATE INDEX IF NOT EXISTS idx_token_events_timestamp ON token_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project);
            CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at);
            "
        )?;
        Ok(())
    }

    /// Record a new session
    pub fn create_session(
        &self,
        id: &str,
        pane_num: u8,
        project: &str,
        role: &str,
        theme: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO sessions (id, pane_num, project, role, theme, started_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, pane_num, project, role, theme, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Record a token usage event
    pub fn record_tokens(
        &self,
        session_id: &str,
        input_tokens: u64,
        output_tokens: u64,
        cache_read: u64,
        tool_name: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO token_events (session_id, timestamp, input_tokens, output_tokens, cache_read, tool_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![session_id, now, input_tokens, output_tokens, cache_read, tool_name],
        )?;

        // Update session totals
        self.conn.execute(
            "UPDATE sessions SET total_input_tokens = total_input_tokens + ?1, total_output_tokens = total_output_tokens + ?2, total_cache_read = total_cache_read + ?3 WHERE id = ?4",
            params![input_tokens, output_tokens, cache_read, session_id],
        )?;

        Ok(())
    }

    /// End a session
    pub fn end_session(&self, session_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), session_id],
        )?;
        Ok(())
    }

    /// Get today's total usage
    pub fn today_totals(&self) -> Result<DayTotal> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), COALESCE(SUM(cache_read), 0) FROM token_events WHERE timestamp LIKE ?1 || '%'"
        )?;
        let row = stmt.query_row(params![today], |row| {
            Ok(DayTotal {
                input_tokens: row.get::<_, i64>(0)? as u64,
                output_tokens: row.get::<_, i64>(1)? as u64,
                cache_read: row.get::<_, i64>(2)? as u64,
            })
        })?;
        Ok(row)
    }

    /// Get per-project totals for today
    pub fn today_by_project(&self) -> Result<Vec<ProjectTotal>> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let mut stmt = self.conn.prepare(
            "SELECT s.project, COALESCE(SUM(e.input_tokens), 0), COALESCE(SUM(e.output_tokens), 0), COALESCE(SUM(e.cache_read), 0) FROM token_events e JOIN sessions s ON e.session_id = s.id WHERE e.timestamp LIKE ?1 || '%' GROUP BY s.project ORDER BY SUM(e.output_tokens) DESC"
        )?;
        let rows = stmt.query_map(params![today], |row| {
            Ok(ProjectTotal {
                project: row.get(0)?,
                input_tokens: row.get::<_, i64>(1)? as u64,
                output_tokens: row.get::<_, i64>(2)? as u64,
                cache_read: row.get::<_, i64>(3)? as u64,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Get usage over last N days
    pub fn daily_history(&self, days: u32) -> Result<Vec<DayTotal>> {
        let mut stmt = self.conn.prepare(
            "SELECT DATE(timestamp) as day, COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), COALESCE(SUM(cache_read), 0) FROM token_events WHERE timestamp >= DATE('now', ?1) GROUP BY day ORDER BY day DESC"
        )?;
        let offset = format!("-{} days", days);
        let rows = stmt.query_map(params![offset], |row| {
            Ok(DayTotal {
                input_tokens: row.get::<_, i64>(1)? as u64,
                output_tokens: row.get::<_, i64>(2)? as u64,
                cache_read: row.get::<_, i64>(3)? as u64,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

#[derive(Debug, Clone, Default)]
pub struct DayTotal {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read: u64,
}

impl DayTotal {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    /// Estimated cost in USD
    pub fn estimated_cost(&self) -> f64 {
        (self.input_tokens as f64 * 3.0
            + self.output_tokens as f64 * 15.0
            + self.cache_read as f64 * 0.30)
            / 1_000_000.0
    }
}

#[derive(Debug, Clone)]
pub struct ProjectTotal {
    pub project: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read: u64,
}

impl ProjectTotal {
    pub fn estimated_cost(&self) -> f64 {
        (self.input_tokens as f64 * 3.0
            + self.output_tokens as f64 * 15.0
            + self.cache_read as f64 * 0.30)
            / 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_query() {
        let db = AnalyticsDb::open_memory().unwrap();
        db.create_session("s1", 1, "dataxlr8", "developer", "cyan").unwrap();
        db.record_tokens("s1", 1000, 2000, 500, Some("Edit")).unwrap();
        db.record_tokens("s1", 500, 1000, 250, Some("Bash")).unwrap();

        let totals = db.today_totals().unwrap();
        assert_eq!(totals.input_tokens, 1500);
        assert_eq!(totals.output_tokens, 3000);
        assert_eq!(totals.cache_read, 750);
    }

    #[test]
    fn test_per_project() {
        let db = AnalyticsDb::open_memory().unwrap();
        db.create_session("s1", 1, "dataxlr8", "developer", "cyan").unwrap();
        db.create_session("s2", 2, "bskiller", "developer", "green").unwrap();
        db.record_tokens("s1", 1000, 2000, 0, None).unwrap();
        db.record_tokens("s2", 500, 1000, 0, None).unwrap();

        let by_project = db.today_by_project().unwrap();
        assert_eq!(by_project.len(), 2);
        assert_eq!(by_project[0].project, "dataxlr8"); // highest output first
    }
}
