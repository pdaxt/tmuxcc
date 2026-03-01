//! Reads local AgentOS state files (JSON configs) for dashboard display.

use chrono::{Local, NaiveDate};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn read_json(path: &std::path::Path) -> Option<Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

// =============================================================================
// Capacity
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct CapacityData {
    pub acu_used: f64,
    pub acu_total: f64,
    pub reviews_used: u32,
    pub reviews_total: u32,
}

impl CapacityData {
    pub fn acu_pct(&self) -> f64 {
        if self.acu_total > 0.0 {
            self.acu_used / self.acu_total * 100.0
        } else {
            0.0
        }
    }

    pub fn bottleneck(&self) -> &'static str {
        let rev_pct = if self.reviews_total > 0 {
            self.reviews_used as f64 / self.reviews_total as f64 * 100.0
        } else {
            0.0
        };
        if rev_pct > 80.0 {
            "REVIEW"
        } else if self.acu_pct() > 90.0 {
            "COMPUTE"
        } else {
            "BALANCED"
        }
    }
}

pub fn load_capacity() -> CapacityData {
    let cap_root = home_dir().join(".config").join("capacity");
    let cfg = read_json(&cap_root.join("config.json")).unwrap_or_default();

    let pane_count = cfg.get("pane_count").and_then(|v| v.as_f64()).unwrap_or(9.0);
    let hours = cfg.get("hours_per_day").and_then(|v| v.as_f64()).unwrap_or(8.0);
    let factor = cfg.get("availability_factor").and_then(|v| v.as_f64()).unwrap_or(0.8);
    let rev_bw = cfg.get("review_bandwidth").and_then(|v| v.as_u64()).unwrap_or(12) as u32;
    let daily = pane_count * hours * factor;

    let today = Local::now().format("%Y-%m-%d").to_string();
    let log = read_json(&cap_root.join("work_log.json")).unwrap_or_default();
    let entries = log.get("entries").and_then(|v| v.as_array());

    let (acu_used, reviews) = entries
        .map(|entries| {
            let mut acu = 0.0;
            let mut rev = 0u32;
            for e in entries {
                let logged = e.get("logged_at").and_then(|v| v.as_str()).unwrap_or("");
                if logged.starts_with(&today) {
                    acu += e.get("acu_spent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    if e.get("review_needed").and_then(|v| v.as_bool()).unwrap_or(false) {
                        rev += 1;
                    }
                }
            }
            (acu, rev)
        })
        .unwrap_or((0.0, 0));

    CapacityData {
        acu_used: (acu_used * 10.0).round() / 10.0,
        acu_total: (daily * 10.0).round() / 10.0,
        reviews_used: reviews,
        reviews_total: rev_bw,
    }
}

// =============================================================================
// Sprint
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct SprintData {
    pub name: String,
    pub space: String,
    pub total_issues: usize,
    pub done_issues: usize,
    pub total_acu: f64,
    pub used_acu: f64,
    pub days_left: i64,
    pub ended: bool,
}

impl SprintData {
    pub fn pct(&self) -> f64 {
        if self.total_issues > 0 {
            self.done_issues as f64 / self.total_issues as f64 * 100.0
        } else {
            0.0
        }
    }
}

pub fn load_sprint() -> Option<SprintData> {
    let sprint_dir = home_dir().join(".config").join("capacity").join("sprints");
    if !sprint_dir.exists() {
        return None;
    }

    let mut sprints: Vec<_> = std::fs::read_dir(&sprint_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|e| e == "json").unwrap_or(false))
        .collect();
    sprints.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    let data = read_json(&sprints.first()?.path())?;

    let name = data
        .get("name")
        .or(data.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("?")
        .to_string();
    let space = data
        .get("space")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let planned = data.get("planned").and_then(|v| v.as_object())?;
    let issues = planned.get("issues").and_then(|v| v.as_array())?;

    let total_issues = issues.len();
    let done_issues = issues
        .iter()
        .filter(|i| {
            i.get("status")
                .and_then(|v| v.as_str())
                .map(|s| s == "done")
                .unwrap_or(false)
                || i.get("actual_acu").is_some()
        })
        .count();

    let total_acu: f64 = planned
        .get("total_acu")
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| {
            issues
                .iter()
                .map(|i| i.get("estimated_acu").and_then(|v| v.as_f64()).unwrap_or(0.0))
                .sum()
        });

    let used_acu: f64 = issues
        .iter()
        .filter_map(|i| i.get("actual_acu").and_then(|v| v.as_f64()))
        .sum();

    let end_date = data.get("end_date").and_then(|v| v.as_str()).unwrap_or("");
    let (days_left, ended) = if let Ok(end) = NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
        let today = Local::now().date_naive();
        let days = (end - today).num_days() + 1;
        (days.max(0), days < 0)
    } else {
        (0, false)
    };

    Some(SprintData {
        name,
        space,
        total_issues,
        done_issues,
        total_acu: (total_acu * 10.0).round() / 10.0,
        used_acu: (used_acu * 10.0).round() / 10.0,
        days_left,
        ended,
    })
}

// =============================================================================
// Board (issues by space)
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct BoardData {
    pub spaces: Vec<(String, HashMap<String, usize>)>,
}

impl BoardData {
    pub fn total_issues(&self) -> usize {
        self.spaces
            .iter()
            .map(|(_, counts)| counts.values().sum::<usize>())
            .sum()
    }
}

pub fn load_board() -> BoardData {
    let spaces_dir = home_dir().join(".config").join("collab").join("spaces");
    let mut spaces = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&spaces_dir) {
        let mut dirs: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        dirs.sort_by_key(|e| e.file_name());

        for entry in dirs {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let issues_dir = entry.path().join("issues");
            if !issues_dir.exists() {
                continue;
            }
            let mut counts: HashMap<String, usize> = HashMap::new();
            if let Ok(files) = std::fs::read_dir(&issues_dir) {
                for f in files.filter_map(|e| e.ok()) {
                    if f.path().extension().map(|e| e == "json").unwrap_or(false) {
                        if let Some(data) = read_json(&f.path()) {
                            let status = data
                                .get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("backlog")
                                .to_string();
                            *counts.entry(status).or_insert(0) += 1;
                        }
                    }
                }
            }
            if !counts.is_empty() {
                spaces.push((entry.file_name().to_string_lossy().to_string(), counts));
            }
        }
    }

    BoardData { spaces }
}

// =============================================================================
// MCP Servers
// =============================================================================

#[derive(Debug, Clone)]
pub struct McpServer {
    pub name: String,
    pub tools: String,
    pub is_rust: bool,
}

pub fn load_mcps() -> Vec<McpServer> {
    let claude_json = home_dir().join(".claude.json");
    let data = match read_json(&claude_json) {
        Some(d) => d,
        None => return Vec::new(),
    };

    let servers = match data.get("mcpServers").and_then(|v| v.as_object()) {
        Some(s) => s,
        None => return Vec::new(),
    };

    let mut result = Vec::new();
    for (name, cfg) in servers {
        let cmd = cfg
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let (display_name, tools, is_rust) = if cmd.contains("mcp-mega") || name == "mcp-mega" {
            ("mcp-mega".to_string(), "887".to_string(), true)
        } else if cmd.contains("agentos") || name == "agentos" {
            ("agentos".to_string(), "135".to_string(), true)
        } else if name == "forge" {
            ("forge".to_string(), "---".to_string(), true)
        } else {
            (name.chars().take(14).collect(), "?".to_string(), false)
        };
        result.push(McpServer {
            name: display_name,
            tools,
            is_rust,
        });
    }
    result
}

// =============================================================================
// Activity Log
// =============================================================================

#[derive(Debug, Clone)]
pub struct ActivityEntry {
    pub ts: String,
    pub pane: u8,
    pub event: String,
    pub summary: String,
}

pub fn load_activity(limit: usize) -> Vec<ActivityEntry> {
    let state_file = home_dir().join(".config").join("agentos").join("state.json");
    let data = match read_json(&state_file) {
        Some(d) => d,
        None => return Vec::new(),
    };

    data.get("activity_log")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .take(limit)
                .filter_map(|e| {
                    Some(ActivityEntry {
                        ts: e.get("ts").and_then(|v| v.as_str())?.to_string(),
                        pane: e.get("pane").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                        event: e
                            .get("event")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        summary: e
                            .get("summary")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

// =============================================================================
// Auto-Cycle Config
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct AutoCycleConfig {
    pub max_parallel: u8,
    pub reserved_panes: Vec<u8>,
    pub auto_assign: bool,
    pub cycle_interval: u32,
}

pub fn load_auto_config() -> AutoCycleConfig {
    let path = home_dir()
        .join(".config")
        .join("agentos")
        .join("auto_config.json");
    let data = match read_json(&path) {
        Some(d) => d,
        None => return AutoCycleConfig::default(),
    };

    AutoCycleConfig {
        max_parallel: data
            .get("max_parallel")
            .and_then(|v| v.as_u64())
            .unwrap_or(4) as u8,
        reserved_panes: data
            .get("reserved_panes")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect()
            })
            .unwrap_or_default(),
        auto_assign: data
            .get("auto_assign")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        cycle_interval: data
            .get("cycle_interval_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(30) as u32,
    }
}

// =============================================================================
// Session State
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct SessionData {
    pub current_task: String,
    pub completed: Vec<String>,
    pub next_steps: Vec<String>,
    pub blocked_on: Option<String>,
}

pub fn load_session() -> SessionData {
    let path = home_dir()
        .join(".config")
        .join("agentos")
        .join("session_state.json");
    let data = match read_json(&path) {
        Some(d) => d,
        None => return SessionData::default(),
    };

    SessionData {
        current_task: data
            .get("current_task")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        completed: data
            .get("completed")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        next_steps: data
            .get("next_steps")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        blocked_on: data
            .get("blocked_on")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    }
}

// =============================================================================
// Multi-Agent Coordination
// =============================================================================

#[derive(Debug, Clone)]
pub struct MultiAgentEntry {
    pub pane_id: String,
    pub project: String,
    pub task: String,
    pub last_update: String,
}

pub fn load_multi_agent() -> Vec<MultiAgentEntry> {
    let path = home_dir()
        .join(".claude")
        .join("multi_agent")
        .join("agents.json");
    let data = match read_json(&path) {
        Some(d) => d,
        None => return Vec::new(),
    };

    let obj = match data.as_object() {
        Some(o) => o,
        None => return Vec::new(),
    };

    obj.iter()
        .map(|(pane_id, info)| MultiAgentEntry {
            pane_id: pane_id.clone(),
            project: info
                .get("project")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            task: info
                .get("task")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            last_update: info
                .get("last_update")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect()
}

// =============================================================================
// Combined Dashboard Data
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct DashboardData {
    pub capacity: CapacityData,
    pub sprint: Option<SprintData>,
    pub board: BoardData,
    pub mcps: Vec<McpServer>,
    pub activity: Vec<ActivityEntry>,
    pub auto_config: AutoCycleConfig,
    pub session: SessionData,
    pub multi_agent: Vec<MultiAgentEntry>,
}

impl DashboardData {
    pub fn total_mcp_tools(&self) -> u32 {
        self.mcps
            .iter()
            .filter_map(|m| m.tools.parse::<u32>().ok())
            .sum()
    }
}

pub fn load_dashboard() -> DashboardData {
    DashboardData {
        capacity: load_capacity(),
        sprint: load_sprint(),
        board: load_board(),
        mcps: load_mcps(),
        activity: load_activity(8),
        auto_config: load_auto_config(),
        session: load_session(),
        multi_agent: load_multi_agent(),
    }
}
