//! MCP server implementation using rmcp.
//!
//! Exposes terminal orchestration as MCP tools that any Claude Code
//! instance can call to spawn, monitor, and control agents.

use serde_json::{json, Value};
use tokio::sync::mpsc;

/// Commands the MCP server sends to the main app
#[derive(Debug)]
pub enum McpCommand {
    ListSessions,
    SpawnAgent {
        pane_num: u8,
        project: String,
        role: String,
        task: String,
        autonomous: bool,
    },
    KillAgent {
        pane_num: u8,
    },
    SendInput {
        pane_num: u8,
        input: String,
    },
    GetContent {
        pane_num: u8,
        lines: usize,
    },
    GetAnalytics,
    GetGitInfo {
        pane_num: u8,
    },
}

/// Response from the main app back to MCP
#[derive(Debug, Clone)]
pub struct McpResponse {
    pub data: Value,
}

/// Handle for the MCP server
pub struct McpServerHandle {
    command_tx: mpsc::Sender<(McpCommand, tokio::sync::oneshot::Sender<McpResponse>)>,
}

impl McpServerHandle {
    /// Create a new MCP server handle with a command channel
    pub fn new() -> (
        Self,
        mpsc::Receiver<(McpCommand, tokio::sync::oneshot::Sender<McpResponse>)>,
    ) {
        let (tx, rx) = mpsc::channel(32);
        (Self { command_tx: tx }, rx)
    }

    /// Send a command and wait for response
    pub async fn send(&self, cmd: McpCommand) -> Option<McpResponse> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.command_tx.send((cmd, resp_tx)).await.ok()?;
        resp_rx.await.ok()
    }
}

/// Tool definitions that will be registered with the MCP server.
/// These define what other agents can call.
pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "list_sessions",
            "description": "List all active agent sessions with their status, project, role, and token usage.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        json!({
            "name": "spawn_agent",
            "description": "Spawn a new Claude Code agent in a pane with full auto-config.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pane": { "type": "integer", "description": "Pane number (1-9)" },
                    "project": { "type": "string", "description": "Project name or path" },
                    "role": { "type": "string", "description": "Agent role (developer, qa, architect, etc.)", "default": "developer" },
                    "task": { "type": "string", "description": "Task description for the agent" },
                    "autonomous": { "type": "boolean", "description": "Run without permission prompts", "default": false }
                },
                "required": ["pane", "project", "task"]
            }
        }),
        json!({
            "name": "kill_agent",
            "description": "Stop an agent running in a pane.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pane": { "type": "integer", "description": "Pane number (1-9)" }
                },
                "required": ["pane"]
            }
        }),
        json!({
            "name": "send_input",
            "description": "Send text input to an agent in a pane.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pane": { "type": "integer", "description": "Pane number (1-9)" },
                    "input": { "type": "string", "description": "Text to send" }
                },
                "required": ["pane", "input"]
            }
        }),
        json!({
            "name": "get_content",
            "description": "Get the terminal output from an agent's pane.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pane": { "type": "integer", "description": "Pane number (1-9)" },
                    "lines": { "type": "integer", "description": "Number of lines to capture", "default": 50 }
                },
                "required": ["pane"]
            }
        }),
        json!({
            "name": "get_analytics",
            "description": "Get token usage, cost, and performance analytics for all sessions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "period": { "type": "string", "description": "Time period: 'session', 'today', 'week'", "default": "session" }
                },
                "required": []
            }
        }),
        json!({
            "name": "get_git_info",
            "description": "Get git branch, PR status, and commit info for an agent's project.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pane": { "type": "integer", "description": "Pane number (1-9)" }
                },
                "required": ["pane"]
            }
        }),
    ]
}
