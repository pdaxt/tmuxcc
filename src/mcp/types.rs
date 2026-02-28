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

// === TRACKER TOOLS ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueCreateRequest {
    #[schemars(description = "Collab space name (e.g. 'mailforge')")]
    pub space: String,
    #[schemars(description = "Issue title")]
    pub title: String,
    #[schemars(description = "Type: bug, feature, task, improvement, epic")]
    pub issue_type: Option<String>,
    #[schemars(description = "Priority: critical, high, medium, low")]
    pub priority: Option<String>,
    #[schemars(description = "Markdown description")]
    pub description: Option<String>,
    #[schemars(description = "Assignee")]
    pub assignee: Option<String>,
    #[schemars(description = "Milestone name")]
    pub milestone: Option<String>,
    #[schemars(description = "Labels for categorization")]
    pub labels: Option<Vec<String>>,
    #[schemars(description = "Agent Capacity Units estimate")]
    pub estimated_acu: Option<f64>,
    #[schemars(description = "Agent role (pm/architect/developer/qa/devops)")]
    pub role: Option<String>,
    #[schemars(description = "Sprint assignment")]
    pub sprint: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueUpdateFullRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Issue ID (e.g. 'MAIL-1')")]
    pub issue_id: String,
    #[schemars(description = "New status")]
    pub status: Option<String>,
    #[schemars(description = "New priority")]
    pub priority: Option<String>,
    #[schemars(description = "New assignee")]
    pub assignee: Option<String>,
    #[schemars(description = "New title")]
    pub title: Option<String>,
    #[schemars(description = "New description")]
    pub description: Option<String>,
    #[schemars(description = "New milestone")]
    pub milestone: Option<String>,
    #[schemars(description = "Add a label")]
    pub add_label: Option<String>,
    #[schemars(description = "Remove a label")]
    pub remove_label: Option<String>,
    #[schemars(description = "Estimated ACU")]
    pub estimated_acu: Option<f64>,
    #[schemars(description = "Actual ACU consumed")]
    pub actual_acu: Option<f64>,
    #[schemars(description = "Agent role")]
    pub role: Option<String>,
    #[schemars(description = "Sprint")]
    pub sprint: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueListFilteredRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Filter by status")]
    pub status: Option<String>,
    #[schemars(description = "Filter by type")]
    pub issue_type: Option<String>,
    #[schemars(description = "Filter by priority")]
    pub priority: Option<String>,
    #[schemars(description = "Filter by assignee")]
    pub assignee: Option<String>,
    #[schemars(description = "Filter by milestone")]
    pub milestone: Option<String>,
    #[schemars(description = "Filter by label")]
    pub label: Option<String>,
    #[schemars(description = "Filter by sprint")]
    pub sprint: Option<String>,
    #[schemars(description = "Filter by role")]
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueViewRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Issue ID")]
    pub issue_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueCommentRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Issue ID")]
    pub issue_id: String,
    #[schemars(description = "Comment text (markdown)")]
    pub text: String,
    #[schemars(description = "Author name")]
    pub author: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueLinkRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Issue ID")]
    pub issue_id: String,
    #[schemars(description = "Link type: doc, commit, or pr")]
    pub link_type: String,
    #[schemars(description = "Reference (doc name, commit hash, PR number)")]
    pub reference: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueCloseRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Issue ID")]
    pub issue_id: String,
    #[schemars(description = "Resolution note")]
    pub resolution: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MilestoneCreateRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Milestone name")]
    pub name: String,
    #[schemars(description = "Description")]
    pub description: Option<String>,
    #[schemars(description = "Due date (YYYY-MM-DD)")]
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MilestoneListRequest {
    #[schemars(description = "Space name")]
    pub space: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TimelineGenerateRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Filter by milestone (empty = all)")]
    pub milestone: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProcessStartRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Template name")]
    pub template_name: String,
    #[schemars(description = "Context variables as JSON object")]
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProcessUpdateRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Process ID")]
    pub process_id: String,
    #[schemars(description = "Step index (0-based)")]
    pub step_index: usize,
    #[schemars(description = "Mark done (true) or undone (false)")]
    pub done: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProcessListRequest {
    #[schemars(description = "Space name")]
    pub space: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProcessTemplateCreateRequest {
    #[schemars(description = "Template name")]
    pub name: String,
    #[schemars(description = "Markdown checklist with - [ ] items")]
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BoardViewRequest {
    #[schemars(description = "Space name")]
    pub space: String,
}

// === CAPACITY TOOLS ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CapConfigureRequest {
    #[schemars(description = "Number of parallel panes")]
    pub pane_count: Option<u32>,
    #[schemars(description = "Working hours per day")]
    pub hours_per_day: Option<f64>,
    #[schemars(description = "Productivity factor 0.0-1.0")]
    pub availability_factor: Option<f64>,
    #[schemars(description = "Max human reviews per day")]
    pub review_bandwidth: Option<u32>,
    #[schemars(description = "Max concurrent builds")]
    pub build_slots: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CapEstimateRequest {
    #[schemars(description = "Task description")]
    pub description: String,
    #[schemars(description = "Complexity: low, medium, high, very_high")]
    pub complexity: Option<String>,
    #[schemars(description = "Type: bug, task, feature, improvement, epic")]
    pub task_type: Option<String>,
    #[schemars(description = "Role: pm, architect, developer, qa, devops")]
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CapLogWorkRequest {
    #[schemars(description = "Issue ID")]
    pub issue_id: String,
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Agent role")]
    pub role: String,
    #[schemars(description = "Pane ID")]
    pub pane_id: Option<String>,
    #[schemars(description = "ACU consumed")]
    pub acu_spent: f64,
    #[schemars(description = "Whether human review needed")]
    pub review_needed: Option<bool>,
    #[schemars(description = "Notes")]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CapPlanSprintRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Sprint name (auto if empty)")]
    pub name: Option<String>,
    #[schemars(description = "Start date YYYY-MM-DD (default today)")]
    pub start_date: Option<String>,
    #[schemars(description = "Working days (default 5)")]
    pub days: Option<u32>,
    #[schemars(description = "Comma-separated issue IDs")]
    pub issue_ids: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CapDashboardRequest {
    #[schemars(description = "Filter by space")]
    pub space: Option<String>,
    #[schemars(description = "Specific sprint ID")]
    pub sprint_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CapBurndownRequest {
    #[schemars(description = "Sprint ID (empty = latest active)")]
    pub sprint_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CapVelocityRequest {
    #[schemars(description = "Filter by space")]
    pub space: Option<String>,
    #[schemars(description = "Number of sprints to analyze")]
    pub count: Option<usize>,
}

// === COLLAB TOOLS ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SpaceCreateRequest {
    #[schemars(description = "Space name (lowercase, hyphens ok)")]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocListRequest {
    #[schemars(description = "Filter by space (empty = all)")]
    pub space: Option<String>,
    #[schemars(description = "Filter by status: draft, review, approved, locked")]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocReadRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name (without .md)")]
    pub name: String,
    #[schemars(description = "Include metadata, comments, directives (default true)")]
    pub include_meta: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocCreateRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name (no .md extension)")]
    pub name: String,
    #[schemars(description = "Initial markdown content")]
    pub content: Option<String>,
    #[schemars(description = "Initial status: draft, review, approved")]
    pub status: Option<String>,
    #[schemars(description = "Tags for categorization")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocEditRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
    #[schemars(description = "New full markdown content")]
    pub content: String,
    #[schemars(description = "Your agent/pane ID for lock checking")]
    pub agent_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocProposeRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
    #[schemars(description = "Full proposed markdown content")]
    pub content: String,
    #[schemars(description = "Brief description of changes")]
    pub summary: Option<String>,
    #[schemars(description = "Your agent/pane ID")]
    pub agent_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocApproveRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
    #[schemars(description = "Proposal ID or 'latest'")]
    pub proposal_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocRejectRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
    #[schemars(description = "Proposal ID to reject")]
    pub proposal_id: String,
    #[schemars(description = "Rejection reason")]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocLockRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
    #[schemars(description = "Who is locking (default: 'human')")]
    pub locked_by: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocUnlockRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocCommentRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
    #[schemars(description = "Comment text")]
    pub text: String,
    #[schemars(description = "Author (default: 'claude')")]
    pub author: Option<String>,
    #[schemars(description = "Line number reference (0 = general)")]
    pub line: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocCommentsRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocStatusRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
    #[schemars(description = "New status: draft, review, approved, archived")]
    pub status: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocSearchRequest {
    #[schemars(description = "Search text (case-insensitive)")]
    pub query: String,
    #[schemars(description = "Limit to specific space (empty = all)")]
    pub space: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocDirectivesRequest {
    #[schemars(description = "Limit to specific space (empty = all)")]
    pub space: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocHistoryRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
    #[schemars(description = "Max history entries (default 10)")]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocDeleteRequest {
    #[schemars(description = "Space name")]
    pub space: String,
    #[schemars(description = "Document name")]
    pub name: String,
    #[schemars(description = "Must be true to delete")]
    pub confirm: Option<bool>,
}
