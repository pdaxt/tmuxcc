//! AgentOS integration — reads state from AgentOS web API and converts to MonitoredAgent

use crate::agents::{AgentStatus, AgentType, MonitoredAgent};
use serde::Deserialize;

const DEFAULT_API_URL: &str = "http://localhost:3100";

/// AgentOS pane state from the /api/status endpoint
#[derive(Debug, Deserialize)]
pub struct AgentOSPane {
    pub pane: u8,
    pub theme: String,
    pub theme_color: String,
    pub status: String,
    pub project: String,
    pub task: String,
    pub role_full: String,
    pub role: String,
    pub branch: Option<String>,
    pub workspace: Option<String>,
    pub pty_active: bool,
    pub pty_running: bool,
    pub line_count: usize,
    pub started_at: Option<String>,
    pub acu: f64,
    pub space: Option<String>,
    pub issue_id: Option<String>,
}

/// AgentOS queue task from /api/queue endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct AgentOSQueueTask {
    pub id: String,
    pub project: String,
    pub role: String,
    pub task: String,
    pub priority: u8,
    pub status: String,
    pub pane: Option<u8>,
    pub depends_on: Vec<String>,
    pub added_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// 24h daily digest from /api/analytics/digest
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AnalyticsDigest {
    #[serde(default)]
    pub tool_calls: i64,
    #[serde(default)]
    pub errors: i64,
    #[serde(default)]
    pub error_rate: String,
    #[serde(default)]
    pub commits: i64,
    #[serde(default)]
    pub files_touched: i64,
    #[serde(default)]
    pub agents_active: i64,
    #[serde(default)]
    pub tasks_completed: i64,
}

/// Single alert from /api/analytics/alerts
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Alert {
    #[serde(default)]
    pub level: String,
    #[serde(rename = "type", default)]
    pub alert_type: String,
    // Optional extra fields depending on alert type
    #[serde(default)]
    pub pane_id: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub error_rate: Option<String>,
}

/// Alerts response from /api/analytics/alerts
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AlertsResponse {
    #[serde(default)]
    pub alerts: Vec<Alert>,
    #[serde(default)]
    pub count: i64,
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    panes: Vec<AgentOSPane>,
}

#[derive(Debug, Deserialize)]
struct QueueResponse {
    tasks: Vec<AgentOSQueueTask>,
}

pub struct AgentOSClient {
    api_url: String,
    client: reqwest::Client,
}

impl AgentOSClient {
    pub fn new(api_url: Option<String>) -> Self {
        Self {
            api_url: api_url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Fetch pane states from AgentOS API
    pub async fn fetch_panes(&self) -> anyhow::Result<Vec<AgentOSPane>> {
        let url = format!("{}/api/status", self.api_url);
        let resp: StatusResponse = self.client.get(&url).send().await?.json().await?;
        Ok(resp.panes)
    }

    /// Fetch queue tasks from AgentOS API
    pub async fn fetch_queue(&self) -> anyhow::Result<Vec<AgentOSQueueTask>> {
        let url = format!("{}/api/queue", self.api_url);
        let resp: QueueResponse = self.client.get(&url).send().await?.json().await?;
        Ok(resp.tasks)
    }

    /// Fetch 24h analytics digest
    pub async fn fetch_digest(&self) -> anyhow::Result<AnalyticsDigest> {
        let url = format!("{}/api/analytics/digest", self.api_url);
        let resp: AnalyticsDigest = self.client.get(&url).send().await?.json().await?;
        Ok(resp)
    }

    /// Fetch active alerts
    pub async fn fetch_alerts(&self) -> anyhow::Result<AlertsResponse> {
        let url = format!("{}/api/analytics/alerts", self.api_url);
        let resp: AlertsResponse = self.client.get(&url).send().await?.json().await?;
        Ok(resp)
    }

    /// Fetch pane output (last N lines)
    pub async fn fetch_output(&self, pane_id: u8) -> anyhow::Result<String> {
        let url = format!("{}/api/pane/{}/output", self.api_url, pane_id);
        let resp = self.client.get(&url).send().await?.text().await?;
        Ok(resp)
    }

    /// Convert AgentOS pane to MonitoredAgent
    pub fn pane_to_agent(pane: &AgentOSPane) -> MonitoredAgent {
        let status = match pane.status.as_str() {
            "active" if pane.pty_running => AgentStatus::Processing {
                activity: pane.task.clone(),
            },
            "active" => AgentStatus::Idle,
            "idle" => AgentStatus::Idle,
            _ => AgentStatus::Unknown,
        };

        let session_name = format!("agentos-{}", pane.theme.to_lowercase());
        let window_name = if pane.project != "--" {
            pane.project.clone()
        } else {
            format!("pane-{}", pane.pane)
        };

        let mut agent = MonitoredAgent::new(
            format!("agentos-{}", pane.pane),
            format!("agentos:{}:{}", pane.pane, pane.theme.to_lowercase()),
            session_name,
            0,
            window_name,
            pane.pane as u32,
            pane.workspace
                .clone()
                .unwrap_or_else(|| pane.project.clone()),
            AgentType::ClaudeCode,
            0, // PID unknown from API
        );
        agent.status = status;
        agent.touch();

        agent
    }
}
