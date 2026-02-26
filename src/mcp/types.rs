use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SpawnRequest {
    #[schemars(description = "Pane reference (1-9, theme name like 'cyan', or shortcut like 'c')")]
    pub pane: String,
    #[schemars(description = "Project name or path (fuzzy matched against ~/Projects)")]
    pub project: String,
    #[schemars(description = "Agent role: pm/architect/frontend/backend/qa/security/devops/developer")]
    pub role: Option<String>,
    #[schemars(description = "Task description for the agent")]
    pub task: Option<String>,
    #[schemars(description = "Optional initial prompt to send after launch")]
    pub prompt: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KillRequest {
    #[schemars(description = "Pane reference (1-9, theme name, or shortcut)")]
    pub pane: String,
    #[schemars(description = "Optional reason for killing")]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RestartRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReassignRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
    #[schemars(description = "New project (optional)")]
    pub project: Option<String>,
    #[schemars(description = "New role (optional)")]
    pub role: Option<String>,
    #[schemars(description = "New task description (optional)")]
    pub task: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AssignRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
    #[schemars(description = "Issue ID from tracker (e.g. 'MAIL-5')")]
    pub issue_id: String,
    #[schemars(description = "Tracker space name (e.g. 'mailforge')")]
    pub space: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AssignAdhocRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
    #[schemars(description = "Task description")]
    pub task: String,
    #[schemars(description = "Agent role (default: developer)")]
    pub role: Option<String>,
    #[schemars(description = "Project name or path")]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CollectRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompleteRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
    #[schemars(description = "Completion summary")]
    pub summary: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetMcpsRequest {
    #[schemars(description = "Project name")]
    pub project: String,
    #[schemars(description = "List of MCP names to enable")]
    pub mcps: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetPreambleRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
    #[schemars(description = "Preamble markdown content")]
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConfigShowRequest {
    #[schemars(description = "Pane reference (optional, shows all if empty)")]
    pub pane: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DashboardRequest {
    #[schemars(description = "Output format: 'text' or 'json'")]
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LogsRequest {
    #[schemars(description = "Pane reference (optional)")]
    pub pane: Option<String>,
    #[schemars(description = "Number of entries (default 20)")]
    pub lines: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct McpListRequest {
    #[schemars(description = "Filter by category (e.g. 'data', 'infrastructure', 'monitoring')")]
    pub category: Option<String>,
    #[schemars(description = "Filter by project name")]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct McpRouteRequest {
    #[schemars(description = "Project name")]
    pub project: String,
    #[schemars(description = "Task description to route MCPs for")]
    pub task: String,
    #[schemars(description = "Agent role (helps refine MCP selection)")]
    pub role: Option<String>,
    #[schemars(description = "If true, auto-apply the routed MCPs to the project config")]
    pub apply: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct McpSearchRequest {
    #[schemars(description = "Search query (matches name, description, capabilities, keywords)")]
    pub query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitSyncRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitStatusRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
    #[schemars(description = "Include full diff output")]
    pub verbose: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitPushRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
    #[schemars(description = "Commit message (default: auto-generated)")]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitPrRequest {
    #[schemars(description = "Pane reference")]
    pub pane: String,
    #[schemars(description = "PR title (default: task description)")]
    pub title: Option<String>,
    #[schemars(description = "PR body/description")]
    pub body: Option<String>,
}

// === QUEUE / AUTO-CYCLE ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueueAddRequest {
    #[schemars(description = "Project name or path")]
    pub project: String,
    #[schemars(description = "Task description")]
    pub task: String,
    #[schemars(description = "Full prompt to send to the agent")]
    pub prompt: Option<String>,
    #[schemars(description = "Agent role (default: developer)")]
    pub role: Option<String>,
    #[schemars(description = "Priority 1-5 (1=highest, default=3)")]
    pub priority: Option<u8>,
    #[schemars(description = "Task IDs this depends on (must complete first)")]
    pub depends_on: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueueListRequest {
    #[schemars(description = "Filter by status: pending, running, done, failed")]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueueDoneRequest {
    #[schemars(description = "Task ID to mark done")]
    pub task_id: String,
    #[schemars(description = "Result summary")]
    pub result: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AutoConfigRequest {
    #[schemars(description = "Max parallel panes (1-9)")]
    pub max_parallel: Option<u8>,
    #[schemars(description = "Reserved panes (never auto-assigned)")]
    pub reserved_panes: Option<Vec<u8>>,
    #[schemars(description = "Auto-complete when agent finishes")]
    pub auto_complete: Option<bool>,
    #[schemars(description = "Auto-assign next task when pane frees")]
    pub auto_assign: Option<bool>,
    #[schemars(description = "Background auto-cycle interval in seconds (0 = disabled, default 30)")]
    pub cycle_interval_secs: Option<u64>,
}

// === MULTI-AGENT COORDINATION ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PortAllocateRequest {
    #[schemars(description = "Service name (e.g. 'dataxlr8-web')")]
    pub service: String,
    #[schemars(description = "Pane ID (e.g. 'claude6:1.0')")]
    pub pane_id: String,
    #[schemars(description = "Preferred port number")]
    pub preferred: Option<u16>,
    #[schemars(description = "Description of what the port is for")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PortReleaseRequest {
    #[schemars(description = "Port number to release")]
    pub port: u16,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PortGetRequest {
    #[schemars(description = "Service name to look up")]
    pub service: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AgentRegisterRequest {
    #[schemars(description = "Pane ID (e.g. 'claude6:1.0')")]
    pub pane_id: String,
    #[schemars(description = "Project name")]
    pub project: String,
    #[schemars(description = "Current task description")]
    pub task: String,
    #[schemars(description = "Files being worked on")]
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AgentUpdateRequest {
    #[schemars(description = "Pane ID")]
    pub pane_id: String,
    #[schemars(description = "Updated task description")]
    pub task: String,
    #[schemars(description = "Updated file list")]
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AgentListRequest {
    #[schemars(description = "Filter by project name")]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AgentDeregisterRequest {
    #[schemars(description = "Pane ID to deregister")]
    pub pane_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LockAcquireRequest {
    #[schemars(description = "Pane ID requesting lock")]
    pub pane_id: String,
    #[schemars(description = "File paths to lock")]
    pub files: Vec<String>,
    #[schemars(description = "Reason for locking")]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LockReleaseRequest {
    #[schemars(description = "Pane ID releasing locks")]
    pub pane_id: String,
    #[schemars(description = "Specific files to release (empty = release all)")]
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LockCheckRequest {
    #[schemars(description = "File paths to check")]
    pub files: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitClaimBranchRequest {
    #[schemars(description = "Pane ID claiming the branch")]
    pub pane_id: String,
    #[schemars(description = "Branch name")]
    pub branch: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "Purpose of the branch")]
    pub purpose: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitReleaseBranchRequest {
    #[schemars(description = "Pane ID releasing the branch")]
    pub pane_id: String,
    #[schemars(description = "Branch name")]
    pub branch: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitListBranchesRequest {
    #[schemars(description = "Filter by repository name")]
    pub repo: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GitPreCommitCheckRequest {
    #[schemars(description = "Pane ID doing the commit")]
    pub pane_id: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "Files being committed")]
    pub files: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuildClaimRequest {
    #[schemars(description = "Pane ID claiming the build")]
    pub pane_id: String,
    #[schemars(description = "Project name")]
    pub project: String,
    #[schemars(description = "Build type (e.g. 'cargo', 'npm', 'docker')")]
    pub build_type: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuildReleaseRequest {
    #[schemars(description = "Pane ID releasing the build")]
    pub pane_id: String,
    #[schemars(description = "Project name")]
    pub project: String,
    #[schemars(description = "Whether the build succeeded")]
    pub success: bool,
    #[schemars(description = "Build output or summary")]
    pub output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuildStatusRequest {
    #[schemars(description = "Project name to check")]
    pub project: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BuildGetLastRequest {
    #[schemars(description = "Project name")]
    pub project: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MaTaskAddRequest {
    #[schemars(description = "Project name")]
    pub project: String,
    #[schemars(description = "Task title")]
    pub title: String,
    #[schemars(description = "Task description")]
    pub description: Option<String>,
    #[schemars(description = "Priority: urgent, high, medium, low")]
    pub priority: Option<String>,
    #[schemars(description = "Pane ID adding the task")]
    pub added_by: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MaTaskClaimRequest {
    #[schemars(description = "Pane ID claiming a task")]
    pub pane_id: String,
    #[schemars(description = "Filter by project")]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MaTaskCompleteRequest {
    #[schemars(description = "Task ID to complete")]
    pub task_id: String,
    #[schemars(description = "Pane ID completing the task")]
    pub pane_id: String,
    #[schemars(description = "Result summary")]
    pub result: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MaTaskListRequest {
    #[schemars(description = "Filter by status: pending, claimed, completed, all")]
    pub status: Option<String>,
    #[schemars(description = "Filter by project")]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KbAddRequest {
    #[schemars(description = "Pane ID adding knowledge")]
    pub pane_id: String,
    #[schemars(description = "Project name")]
    pub project: String,
    #[schemars(description = "Category (e.g. 'code_location', 'bug', 'architecture')")]
    pub category: String,
    #[schemars(description = "Title/summary")]
    pub title: String,
    #[schemars(description = "Full content")]
    pub content: String,
    #[schemars(description = "Related file paths")]
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KbSearchRequest {
    #[schemars(description = "Search query")]
    pub query: String,
    #[schemars(description = "Filter by project")]
    pub project: Option<String>,
    #[schemars(description = "Filter by category")]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KbListRequest {
    #[schemars(description = "Filter by project")]
    pub project: Option<String>,
    #[schemars(description = "Max entries to return (default 20)")]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MsgBroadcastRequest {
    #[schemars(description = "Sender pane ID")]
    pub from_pane: String,
    #[schemars(description = "Message content")]
    pub message: String,
    #[schemars(description = "Priority: info, warning, urgent")]
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MsgSendRequest {
    #[schemars(description = "Sender pane ID")]
    pub from_pane: String,
    #[schemars(description = "Recipient pane ID")]
    pub to_pane: String,
    #[schemars(description = "Message content")]
    pub message: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MsgGetRequest {
    #[schemars(description = "Pane ID to get messages for")]
    pub pane_id: String,
    #[schemars(description = "Mark messages as read")]
    pub mark_read: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StatusOverviewRequest {
    #[schemars(description = "Filter by project")]
    pub project: Option<String>,
}
