//! PTY Manager — orchestrates all PTY sessions.
//!
//! This is the replacement for tmux. Instead of shelling out to `tmux send-keys`,
//! we own the PTY file descriptors directly.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use parking_lot::Mutex;
use tokio::sync::mpsc;

use super::session::{PtySession, PtySessionHandle, ScrollbackBuffer, SessionEvent};

/// Configuration for spawning an agent
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub project: String,
    pub project_path: PathBuf,
    pub role: String,
    pub pane_num: u8,
    pub theme: String,
    pub task: Option<String>,
    pub autonomous: bool,
    /// Custom command (default: claude-start)
    pub command: Option<String>,
}

/// A managed pane with its PTY session
pub struct ManagedPane {
    pub config: SpawnConfig,
    pub handle: PtySessionHandle,
    pub scrollback: Arc<Mutex<ScrollbackBuffer>>,
    pub reader_handle: tokio::task::JoinHandle<()>,
    /// Accumulated token stats
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read: u64,
    /// Number of tool calls detected
    pub tool_calls: u64,
    /// Cost estimate in USD (based on Claude pricing)
    pub estimated_cost_usd: f64,
}

impl ManagedPane {
    /// Get scrollback content (last N lines)
    pub fn content(&self, lines: usize) -> String {
        self.scrollback.lock().tail(lines)
    }

    /// Update token stats and recalculate cost
    pub fn add_tokens(&mut self, input: u64, output: u64, cache: u64) {
        self.total_input_tokens += input;
        self.total_output_tokens += output;
        self.total_cache_read += cache;
        // Claude pricing estimate (approximate):
        // Input: $3/MTok, Output: $15/MTok, Cache read: $0.30/MTok
        self.estimated_cost_usd = (self.total_input_tokens as f64 * 3.0
            + self.total_output_tokens as f64 * 15.0
            + self.total_cache_read as f64 * 0.30)
            / 1_000_000.0;
    }
}

/// The PTY Manager — owns all terminal sessions
pub struct PtyManager {
    /// Active panes indexed by pane number (1-9)
    panes: HashMap<u8, ManagedPane>,
    /// Channel for session events
    event_tx: mpsc::Sender<SessionEvent>,
    event_rx: mpsc::Receiver<SessionEvent>,
    /// Default terminal size
    default_rows: u16,
    default_cols: u16,
}

impl PtyManager {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel(256);
        Self {
            panes: HashMap::new(),
            event_tx,
            event_rx,
            default_rows: 56,
            default_cols: 80,
        }
    }

    /// Set default terminal size (called on window resize)
    pub fn set_default_size(&mut self, rows: u16, cols: u16) {
        self.default_rows = rows;
        self.default_cols = cols;
    }

    /// Spawn a new agent in a pane
    pub fn spawn(&mut self, config: SpawnConfig) -> Result<u8> {
        let pane_num = config.pane_num;

        // Kill existing session in this pane
        if self.panes.contains_key(&pane_num) {
            self.kill(pane_num)?;
        }

        // Build environment
        let mut env = vec![
            ("P".to_string(), pane_num.to_string()),
            ("TERM".to_string(), "xterm-256color".to_string()),
        ];
        if config.autonomous {
            env.push(("CLAUDE_AUTONOMOUS".to_string(), "1".to_string()));
        }

        // Default command is zsh (we'll send claude-start after)
        let command = config
            .command
            .as_deref()
            .unwrap_or("zsh");

        let session_id = format!("pane-{}-{}", pane_num, uuid::Uuid::new_v4().as_simple());

        let session = PtySession::spawn(
            session_id.clone(),
            config.project.clone(),
            config.project_path.clone(),
            config.role.clone(),
            pane_num,
            config.theme.clone(),
            command,
            &[],
            env,
            self.default_rows,
            self.default_cols,
            self.event_tx.clone(),
        )
        .context(format!("Failed to spawn PTY for pane {}", pane_num))?;

        let scrollback = session.scrollback.clone();
        let (handle, reader_handle) = session.start_reader()?;

        // If command is zsh, send the actual claude-start command
        if command == "zsh" {
            let start_cmd = format!(
                "cd {} && P={} ~/bin/claude-start",
                config.project_path.display(),
                pane_num
            );
            handle.write_line(&start_cmd)?;
        }

        self.panes.insert(
            pane_num,
            ManagedPane {
                config,
                handle,
                scrollback,
                reader_handle,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_cache_read: 0,
                tool_calls: 0,
                estimated_cost_usd: 0.0,
            },
        );

        Ok(pane_num)
    }

    /// Kill a session in a pane
    pub fn kill(&mut self, pane_num: u8) -> Result<()> {
        if let Some(pane) = self.panes.remove(&pane_num) {
            // Try graceful exit first
            let _ = pane.handle.write_line("/exit");
            // Then abort the reader
            pane.reader_handle.abort();
        }
        Ok(())
    }

    /// Send input to a pane
    pub fn send_input(&self, pane_num: u8, input: &str) -> Result<()> {
        let pane = self
            .panes
            .get(&pane_num)
            .context(format!("No session in pane {}", pane_num))?;
        pane.handle.write_line(input)
    }

    /// Send raw bytes to a pane (for keyboard passthrough)
    pub fn send_bytes(&self, pane_num: u8, data: &[u8]) -> Result<()> {
        let pane = self
            .panes
            .get(&pane_num)
            .context(format!("No session in pane {}", pane_num))?;
        pane.handle.write_bytes(data)
    }

    /// Paste text into a pane (using bracketed paste)
    pub fn paste(&self, pane_num: u8, text: &str) -> Result<()> {
        let pane = self
            .panes
            .get(&pane_num)
            .context(format!("No session in pane {}", pane_num))?;
        pane.handle.paste(text)
    }

    /// Get scrollback content for a pane
    pub fn content(&self, pane_num: u8, lines: usize) -> Option<String> {
        self.panes.get(&pane_num).map(|p| p.content(lines))
    }

    /// Get a reference to a managed pane
    pub fn pane(&self, pane_num: u8) -> Option<&ManagedPane> {
        self.panes.get(&pane_num)
    }

    /// Get a mutable reference to a managed pane
    pub fn pane_mut(&mut self, pane_num: u8) -> Option<&mut ManagedPane> {
        self.panes.get_mut(&pane_num)
    }

    /// Get all active pane numbers
    pub fn active_panes(&self) -> Vec<u8> {
        let mut panes: Vec<u8> = self.panes.keys().copied().collect();
        panes.sort();
        panes
    }

    /// Process pending events (call from main loop)
    pub async fn process_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                SessionEvent::TokenUsage {
                    session_id,
                    input_tokens,
                    output_tokens,
                    cache_read,
                } => {
                    // Find the pane by session_id and update tokens
                    for pane in self.panes.values_mut() {
                        if pane.handle.session_id == session_id {
                            pane.add_tokens(input_tokens, output_tokens, cache_read);
                            break;
                        }
                    }
                }
                SessionEvent::Exited { session_id, .. } => {
                    tracing::info!("Session {} exited", session_id);
                }
                SessionEvent::Output { .. } => {
                    // Output is already handled by the reader thread writing to scrollback
                }
            }
        }
    }

    /// Resize all panes
    pub fn resize_all(&mut self, rows: u16, cols: u16) {
        self.default_rows = rows;
        self.default_cols = cols;
        // Note: individual resize needs master_pty reference, which is moved into PtySession
        // For now, new sessions will use the updated size
    }

    /// Get aggregate analytics across all panes
    pub fn aggregate_analytics(&self) -> AggregateAnalytics {
        let mut analytics = AggregateAnalytics::default();
        for pane in self.panes.values() {
            analytics.total_input_tokens += pane.total_input_tokens;
            analytics.total_output_tokens += pane.total_output_tokens;
            analytics.total_cache_read += pane.total_cache_read;
            analytics.total_cost_usd += pane.estimated_cost_usd;
            analytics.active_sessions += 1;
        }
        analytics
    }
}

/// Aggregate analytics across all sessions
#[derive(Debug, Default, Clone)]
pub struct AggregateAnalytics {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read: u64,
    pub total_cost_usd: f64,
    pub active_sessions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_manager_new() {
        let manager = PtyManager::new();
        assert!(manager.active_panes().is_empty());
    }

    #[test]
    fn test_aggregate_analytics_default() {
        let analytics = AggregateAnalytics::default();
        assert_eq!(analytics.total_input_tokens, 0);
        assert_eq!(analytics.active_sessions, 0);
    }
}
