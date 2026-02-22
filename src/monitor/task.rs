use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tracing::{debug, error, warn};

use crate::agents::{AgentStatus, MonitoredAgent};
use crate::agentos::{AgentOSClient, AgentOSQueueTask};
use crate::app::AgentTree;
use crate::parsers::ParserRegistry;
use crate::tmux::{refresh_process_cache, TmuxClient};

/// Hysteresis duration - keep "Processing" status for this long after last active detection
const STATUS_HYSTERESIS_MS: u64 = 2000;

/// Update message sent from monitor to UI
#[derive(Debug, Clone)]
pub struct MonitorUpdate {
    pub agents: AgentTree,
    pub queue_tasks: Vec<AgentOSQueueTask>,
    pub agentos_connected: bool,
}

/// Background task that monitors tmux panes and AgentOS for AI agents
pub struct MonitorTask {
    tmux_client: Arc<TmuxClient>,
    parser_registry: Arc<ParserRegistry>,
    agentos_client: Option<AgentOSClient>,
    tx: mpsc::Sender<MonitorUpdate>,
    poll_interval: Duration,
    /// Track when each agent was last seen as "active" (Processing/AwaitingApproval)
    /// Key: agent target string
    last_active: HashMap<String, Instant>,
}

impl MonitorTask {
    pub fn new(
        tmux_client: Arc<TmuxClient>,
        parser_registry: Arc<ParserRegistry>,
        agentos_client: Option<AgentOSClient>,
        tx: mpsc::Sender<MonitorUpdate>,
        poll_interval: Duration,
    ) -> Self {
        Self {
            tmux_client,
            parser_registry,
            agentos_client,
            tx,
            poll_interval,
            last_active: HashMap::new(),
        }
    }

    /// Runs the monitoring loop
    pub async fn run(mut self) {
        loop {
            let (tree, queue_tasks, connected) = match self.poll_all().await {
                Ok(result) => result,
                Err(e) => {
                    warn!("Monitor poll error: {}", e);
                    (AgentTree::new(), Vec::new(), false)
                }
            };

            let update = MonitorUpdate {
                agents: tree,
                queue_tasks,
                agentos_connected: connected,
            };
            if self.tx.send(update).await.is_err() {
                debug!("Monitor channel closed, stopping");
                break;
            }

            tokio::time::sleep(self.poll_interval).await;
        }
    }

    async fn poll_all(&mut self) -> anyhow::Result<(AgentTree, Vec<AgentOSQueueTask>, bool)> {
        // Poll tmux agents
        let mut tree = self.poll_tmux_agents().await?;

        // Poll AgentOS (if configured)
        let mut queue_tasks = Vec::new();
        let mut connected = false;

        if let Some(ref client) = self.agentos_client {
            // Fetch panes from AgentOS
            match client.fetch_panes().await {
                Ok(panes) => {
                    connected = true;
                    for pane in &panes {
                        if pane.pty_running || pane.status == "active" {
                            let agent = AgentOSClient::pane_to_agent(pane);
                            // Only add if not already detected via tmux
                            let already_exists = tree.root_agents.iter().any(|a| {
                                a.path == agent.path && a.agent_type == agent.agent_type
                            });
                            if !already_exists {
                                tree.root_agents.push(agent);
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("AgentOS API unavailable: {}", e);
                }
            }

            // Fetch queue
            match client.fetch_queue().await {
                Ok(tasks) => {
                    queue_tasks = tasks;
                }
                Err(e) => {
                    debug!("AgentOS queue unavailable: {}", e);
                }
            }
        }

        // Sort agents by target for consistent ordering
        tree.root_agents.sort_by(|a, b| a.target.cmp(&b.target));

        Ok((tree, queue_tasks, connected))
    }

    async fn poll_tmux_agents(&mut self) -> anyhow::Result<AgentTree> {
        // Refresh process cache once per poll cycle (much faster than per-pane)
        refresh_process_cache();

        let panes = self.tmux_client.list_panes()?;
        let mut tree = AgentTree::new();

        for pane in panes {
            // Try to find a matching parser for the pane (checks command, title, cmdline)
            if let Some(parser) = self.parser_registry.find_parser_for_pane(&pane) {
                let target = pane.target();

                // Capture pane content
                let content = match self.tmux_client.capture_pane(&target) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to capture pane {}: {}", target, e);
                        continue;
                    }
                };

                // Parse status from content
                let mut status = parser.parse_status(&content);

                // Check pane title for spinner (Claude Code specific)
                let title_has_spinner = pane.title.chars().any(|c| {
                    matches!(
                        c,
                        '⠿' | '⠇'
                            | '⠋'
                            | '⠙'
                            | '⠸'
                            | '⠴'
                            | '⠦'
                            | '⠧'
                            | '⠖'
                            | '⠏'
                            | '⠹'
                            | '⠼'
                            | '⠷'
                            | '⠾'
                            | '⠽'
                            | '⠻'
                            | '⠐'
                            | '⠑'
                            | '⠒'
                            | '⠓'
                    )
                });

                // If title has spinner, override to Processing
                if title_has_spinner && matches!(status, AgentStatus::Idle | AgentStatus::Unknown) {
                    status = AgentStatus::Processing {
                        activity: "Working...".to_string(),
                    };
                }

                // Apply hysteresis
                let now = Instant::now();
                let is_active = matches!(
                    status,
                    AgentStatus::Processing { .. } | AgentStatus::AwaitingApproval { .. }
                );

                if is_active {
                    self.last_active.insert(target.clone(), now);
                } else if matches!(status, AgentStatus::Idle) {
                    if let Some(last) = self.last_active.get(&target) {
                        if now.duration_since(*last) < Duration::from_millis(STATUS_HYSTERESIS_MS) {
                            status = AgentStatus::Processing {
                                activity: "Working...".to_string(),
                            };
                        }
                    }
                }

                // Parse subagents
                let subagents = parser.parse_subagents(&content);

                // Parse context remaining
                let context_remaining = parser.parse_context_remaining(&content);

                // Create monitored agent
                let mut agent = MonitoredAgent::new(
                    format!("{}-{}", target, pane.pid),
                    target,
                    pane.session.clone(),
                    pane.window,
                    pane.window_name.clone(),
                    pane.pane,
                    pane.path.clone(),
                    parser.agent_type(),
                    pane.pid,
                );
                agent.status = status;
                agent.subagents = subagents;
                agent.last_content = content;
                agent.context_remaining = context_remaining;
                agent.touch();

                tree.root_agents.push(agent);
            }
        }

        Ok(tree)
    }
}
