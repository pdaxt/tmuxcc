//! Analytics engine — tracks tokens, costs, and session metrics.
//!
//! Persists to SQLite so you can see historical data across sessions.
//! Answers: "How many tokens did I burn today?" "Which project costs the most?"

mod db;
mod tracker;

pub use db::AnalyticsDb;
pub use tracker::{SessionMetrics, TokenEvent, UsageTracker};
