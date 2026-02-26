pub mod types;
pub mod tools;

use std::sync::Arc;
use rmcp::{
    ServerHandler, ServiceExt,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use crate::app::App;
use self::types::*;

#[derive(Clone)]
pub struct AgentOSService {
    app: Arc<App>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl AgentOSService {
    pub fn new(app: Arc<App>) -> Self {
        Self {
            app,
            tool_router: Self::tool_router(),
        }
    }

    // === AGENT LIFECYCLE ===

    #[tool(description = "Spawn a Claude agent in a pane with full auto-config. Resolves project path, sets MCPs, generates role preamble.")]
    async fn os_spawn(
        &self,
        Parameters(req): Parameters<SpawnRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::spawn(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Gracefully stop an agent and clean up state.")]
    async fn os_kill(
        &self,
        Parameters(req): Parameters<KillRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::kill(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Kill and re-spawn an agent with same config.")]
    async fn os_restart(
        &self,
        Parameters(req): Parameters<RestartRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::restart(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Update an agent's task/project/role without full restart.")]
    async fn os_reassign(
        &self,
        Parameters(req): Parameters<ReassignRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::reassign(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // === TASK ASSIGNMENT ===

    #[tool(description = "Pull a tracker issue and assign it to a pane with auto-config.")]
    async fn os_assign(
        &self,
        Parameters(req): Parameters<AssignRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::assign(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Assign an ad-hoc task (not from tracker) to a pane.")]
    async fn os_assign_adhoc(
        &self,
        Parameters(req): Parameters<AssignAdhocRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::assign_adhoc(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Capture agent output and detect completion/errors.")]
    async fn os_collect(
        &self,
        Parameters(req): Parameters<CollectRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::collect(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Mark task complete, log ACU, update tracker, free pane.")]
    async fn os_complete(
        &self,
        Parameters(req): Parameters<CompleteRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::complete(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // === CONFIGURATION ===

    #[tool(description = "Set project-level MCPs in claude config.")]
    async fn os_set_mcps(
        &self,
        Parameters(req): Parameters<SetMcpsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::set_mcps(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Write a custom preamble for an agent.")]
    async fn os_set_preamble(
        &self,
        Parameters(req): Parameters<SetPreambleRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::set_preamble(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Show configuration for a pane or all panes.")]
    async fn os_config_show(
        &self,
        Parameters(req): Parameters<ConfigShowRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::config_show(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // === MONITORING ===

    #[tool(description = "Full status of all panes: project, role, task, ACU, status.")]
    async fn os_status(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::status(&self.app).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Rich dashboard with capacity gauges, agent list, kanban summary.")]
    async fn os_dashboard(
        &self,
        Parameters(req): Parameters<DashboardRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::dashboard(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get recent activity log, optionally filtered by pane.")]
    async fn os_logs(
        &self,
        Parameters(req): Parameters<LogsRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::logs(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Health check: stuck agents, idle panes, error detection.")]
    async fn os_health(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::health(&self.app).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // === MCP ROUTING ===

    #[tool(description = "List all available MCPs with descriptions, capabilities, and categories. Filter by category or project.")]
    async fn os_mcp_list(
        &self,
        Parameters(req): Parameters<McpListRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::mcp_list(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Smart MCP routing: given a project, task, and role, suggests the best MCPs to enable. Set apply=true to auto-configure.")]
    async fn os_mcp_route(
        &self,
        Parameters(req): Parameters<McpRouteRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::mcp_route(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Search MCPs by name, description, capability, or keyword.")]
    async fn os_mcp_search(
        &self,
        Parameters(req): Parameters<McpSearchRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::mcp_search(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // === GIT ISOLATION ===

    #[tool(description = "Sync agent's worktree with latest from base branch (fetch + rebase).")]
    async fn os_git_sync(
        &self,
        Parameters(req): Parameters<GitSyncRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::git_sync(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Show git status and diff for an agent's isolated worktree.")]
    async fn os_git_status(
        &self,
        Parameters(req): Parameters<GitStatusRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::git_status_tool(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Commit and push an agent's current work to its branch.")]
    async fn os_git_push(
        &self,
        Parameters(req): Parameters<GitPushRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::git_push(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Create a pull request from an agent's branch. Commits and pushes first.")]
    async fn os_git_pr(
        &self,
        Parameters(req): Parameters<GitPrRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::git_pr(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // === QUEUE / AUTO-CYCLE ===

    #[tool(description = "Add a task to the queue. Tasks are auto-assigned to free panes when os_auto is called.")]
    async fn os_queue_add(
        &self,
        Parameters(req): Parameters<QueueAddRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::queue_add(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "List all queued tasks with status. Filter by: pending, running, done, failed.")]
    async fn os_queue_list(
        &self,
        Parameters(req): Parameters<QueueListRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::queue_list(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Mark a queued task as done. Unblocks dependent tasks.")]
    async fn os_queue_done(
        &self,
        Parameters(req): Parameters<QueueDoneRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::queue_done(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Run one auto-cycle: complete finished agents, spawn next queued tasks on free panes. Call repeatedly (every 30-60s) for continuous operation.")]
    async fn os_auto(
        &self,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::auto_cycle(&self.app).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Configure auto-cycle behavior: max parallel panes, reserved panes, auto-complete, auto-assign.")]
    async fn os_auto_config(
        &self,
        Parameters(req): Parameters<AutoConfigRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = tools::auto_config(&self.app, req).await;
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // === MULTI-AGENT COORDINATION (31 tools) ===

    #[tool(description = "Allocate a port for a service. Finds free port in 3001-3099 range, checks for conflicts.")]
    async fn port_allocate(
        &self,
        Parameters(req): Parameters<PortAllocateRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::port_allocate(
            &req.service, &req.pane_id, req.preferred, &req.description.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Release an allocated port back to the pool.")]
    async fn port_release(
        &self,
        Parameters(req): Parameters<PortReleaseRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::port_release(req.port);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "List all allocated ports with active/inactive status.")]
    async fn port_list(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::port_list();
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Get the port allocated for a service by name.")]
    async fn port_get(
        &self,
        Parameters(req): Parameters<PortGetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::port_get(&req.service);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Register an agent in a pane. Returns other agents on same project for coordination.")]
    async fn agent_register(
        &self,
        Parameters(req): Parameters<AgentRegisterRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let files = req.files.unwrap_or_default();
        let result = crate::multi_agent::agent_register(&req.pane_id, &req.project, &req.task, &files);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Update an agent's current task and file list.")]
    async fn agent_update(
        &self,
        Parameters(req): Parameters<AgentUpdateRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::agent_update(&req.pane_id, &req.task, req.files.as_deref());
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "List all registered agents, optionally filtered by project.")]
    async fn agent_list(
        &self,
        Parameters(req): Parameters<AgentListRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::agent_list(req.project.as_deref());
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Deregister an agent and release its locks.")]
    async fn agent_deregister(
        &self,
        Parameters(req): Parameters<AgentDeregisterRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::agent_deregister(&req.pane_id);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Acquire file locks to prevent concurrent edits. Returns blocked status if files locked by others.")]
    async fn lock_acquire(
        &self,
        Parameters(req): Parameters<LockAcquireRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::lock_acquire(
            &req.pane_id, &req.files, &req.reason.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Release file locks. Empty files list releases all locks for this pane.")]
    async fn lock_release(
        &self,
        Parameters(req): Parameters<LockReleaseRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let files = req.files.unwrap_or_default();
        let result = crate::multi_agent::lock_release(&req.pane_id, &files);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Check if files are locked and by whom.")]
    async fn lock_check(
        &self,
        Parameters(req): Parameters<LockCheckRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::lock_check(&req.files);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Claim a git branch for exclusive use. Prevents other agents from using the same branch.")]
    async fn git_claim_branch(
        &self,
        Parameters(req): Parameters<GitClaimBranchRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::git_claim_branch(
            &req.pane_id, &req.branch, &req.repo, &req.purpose.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Release a claimed git branch.")]
    async fn git_release_branch(
        &self,
        Parameters(req): Parameters<GitReleaseBranchRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::git_release_branch(&req.pane_id, &req.branch, &req.repo);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "List all claimed git branches, optionally filtered by repo.")]
    async fn git_list_branches(
        &self,
        Parameters(req): Parameters<GitListBranchesRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::git_list_branches(req.repo.as_deref());
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Check for conflicts before committing: file locks and concurrent edits.")]
    async fn git_pre_commit_check(
        &self,
        Parameters(req): Parameters<GitPreCommitCheckRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::git_pre_commit_check(&req.pane_id, &req.repo, &req.files);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Claim exclusive build access for a project. Prevents concurrent builds.")]
    async fn build_claim(
        &self,
        Parameters(req): Parameters<BuildClaimRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::build_claim(
            &req.pane_id, &req.project, &req.build_type.unwrap_or_else(|| "default".into()),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Release build claim and record result in history.")]
    async fn build_release(
        &self,
        Parameters(req): Parameters<BuildReleaseRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::build_release(
            &req.pane_id, &req.project, req.success, &req.output.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Check if a project is currently being built.")]
    async fn build_status(
        &self,
        Parameters(req): Parameters<BuildStatusRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::build_status(&req.project);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Get the last build result for a project.")]
    async fn build_get_last(
        &self,
        Parameters(req): Parameters<BuildGetLastRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::build_get_last(&req.project);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Add an inter-agent task to the shared queue (not the AgentOS auto-cycle queue).")]
    async fn ma_task_add(
        &self,
        Parameters(req): Parameters<MaTaskAddRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::task_add(
            &req.project, &req.title, &req.description.unwrap_or_default(),
            &req.priority.unwrap_or_else(|| "medium".into()), &req.added_by,
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Claim the next pending inter-agent task by priority.")]
    async fn ma_task_claim(
        &self,
        Parameters(req): Parameters<MaTaskClaimRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::task_claim(&req.pane_id, req.project.as_deref());
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Mark an inter-agent task as completed.")]
    async fn ma_task_complete(
        &self,
        Parameters(req): Parameters<MaTaskCompleteRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::task_complete(
            &req.task_id, &req.pane_id, &req.result.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "List inter-agent tasks, optionally filtered by status and project.")]
    async fn ma_task_list(
        &self,
        Parameters(req): Parameters<MaTaskListRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::task_list(req.status.as_deref(), req.project.as_deref());
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Add a knowledge base entry for cross-agent learning.")]
    async fn kb_add(
        &self,
        Parameters(req): Parameters<KbAddRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let files = req.files.unwrap_or_default();
        let result = crate::multi_agent::kb_add(
            &req.pane_id, &req.project, &req.category, &req.title, &req.content, &files,
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Search the knowledge base by query, optionally filtered by project and category.")]
    async fn kb_search(
        &self,
        Parameters(req): Parameters<KbSearchRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::kb_search(&req.query, req.project.as_deref(), req.category.as_deref());
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "List recent knowledge base entries.")]
    async fn kb_list(
        &self,
        Parameters(req): Parameters<KbListRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::kb_list(req.project.as_deref(), req.limit.unwrap_or(20));
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Broadcast a message to all agents.")]
    async fn msg_broadcast(
        &self,
        Parameters(req): Parameters<MsgBroadcastRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::msg_broadcast(
            &req.from_pane, &req.message, &req.priority.unwrap_or_else(|| "info".into()),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Send a direct message to a specific agent.")]
    async fn msg_send(
        &self,
        Parameters(req): Parameters<MsgSendRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::msg_send(&req.from_pane, &req.to_pane, &req.message);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Get unread messages for this agent. Marks as read by default.")]
    async fn msg_get(
        &self,
        Parameters(req): Parameters<MsgGetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::msg_get(&req.pane_id, req.mark_read.unwrap_or(true));
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Clean up stale entries: ports, agents, locks, branches, builds from inactive panes.")]
    async fn cleanup_all(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::cleanup_all();
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Full status overview: ports, agents, locks, builds, pending tasks.")]
    async fn status_overview(
        &self,
        Parameters(req): Parameters<StatusOverviewRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::multi_agent::status_overview(req.project.as_deref());
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    // === TRACKER TOOLS (15 tools) ===

    #[tool(description = "Create a new issue in a tracker space. Returns issue ID.")]
    async fn issue_create(
        &self,
        Parameters(req): Parameters<IssueCreateRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::issue_create(
            &req.space, &req.title,
            &req.issue_type.unwrap_or_default(), &req.priority.unwrap_or_default(),
            &req.description.unwrap_or_default(), &req.assignee.unwrap_or_default(),
            &req.milestone.unwrap_or_default(), &req.labels.unwrap_or_default(),
            req.estimated_acu.unwrap_or(0.0), &req.role.unwrap_or_default(),
            &req.sprint.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Update an issue's fields: status, priority, assignee, labels, ACU, etc.")]
    async fn issue_update_full(
        &self,
        Parameters(req): Parameters<IssueUpdateFullRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::issue_update_full(
            &req.space, &req.issue_id,
            &req.status.unwrap_or_default(), &req.priority.unwrap_or_default(),
            &req.assignee.unwrap_or_default(), &req.title.unwrap_or_default(),
            &req.description.unwrap_or_default(), &req.milestone.unwrap_or_default(),
            &req.add_label.unwrap_or_default(), &req.remove_label.unwrap_or_default(),
            req.estimated_acu.unwrap_or(0.0), req.actual_acu.unwrap_or(0.0),
            &req.role.unwrap_or_default(), &req.sprint.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "List issues with filters: status, type, priority, assignee, milestone, label, sprint, role.")]
    async fn issue_list_filtered(
        &self,
        Parameters(req): Parameters<IssueListFilteredRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::issue_list_filtered(
            &req.space, &req.status.unwrap_or_default(), &req.issue_type.unwrap_or_default(),
            &req.priority.unwrap_or_default(), &req.assignee.unwrap_or_default(),
            &req.milestone.unwrap_or_default(), &req.label.unwrap_or_default(),
            &req.sprint.unwrap_or_default(), &req.role.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "View full details of a single issue including comments and links.")]
    async fn issue_view(
        &self,
        Parameters(req): Parameters<IssueViewRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::issue_view(&req.space, &req.issue_id);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Add a comment to an issue.")]
    async fn issue_comment(
        &self,
        Parameters(req): Parameters<IssueCommentRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::issue_comment(
            &req.space, &req.issue_id, &req.text, &req.author.unwrap_or_else(|| "agent".into()),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Link a doc, commit, or PR to an issue.")]
    async fn issue_link(
        &self,
        Parameters(req): Parameters<IssueLinkRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::issue_link(&req.space, &req.issue_id, &req.link_type, &req.reference);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Close an issue with a resolution note.")]
    async fn issue_close(
        &self,
        Parameters(req): Parameters<IssueCloseRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::issue_close(
            &req.space, &req.issue_id, &req.resolution.unwrap_or_default().as_str(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Create a milestone for a space with optional due date.")]
    async fn milestone_create(
        &self,
        Parameters(req): Parameters<MilestoneCreateRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::milestone_create(
            &req.space, &req.name, &req.description.unwrap_or_default(), &req.due_date.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "List milestones with progress for a space.")]
    async fn milestone_list(
        &self,
        Parameters(req): Parameters<MilestoneListRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::milestone_list(&req.space);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Generate a Mermaid Gantt timeline from open issues.")]
    async fn timeline_generate(
        &self,
        Parameters(req): Parameters<TimelineGenerateRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::timeline_generate(&req.space, &req.milestone.unwrap_or_default());
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Start a process from a checklist template. Context vars substitute {{var}} placeholders.")]
    async fn process_start(
        &self,
        Parameters(req): Parameters<ProcessStartRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let ctx = req.context.unwrap_or(serde_json::json!({}));
        let result = crate::tracker::process_start(&req.space, &req.template_name, &ctx);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Update a process step as done or undone.")]
    async fn process_update(
        &self,
        Parameters(req): Parameters<ProcessUpdateRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::process_update(
            &req.space, &req.process_id, req.step_index, req.done.unwrap_or(true),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "List all processes in a space with progress.")]
    async fn process_list(
        &self,
        Parameters(req): Parameters<ProcessListRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::process_list(&req.space);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Create a checklist template from markdown with - [ ] items.")]
    async fn process_template_create(
        &self,
        Parameters(req): Parameters<ProcessTemplateCreateRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::process_template_create(&req.name, &req.content);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Kanban board view of all issues in a space grouped by status.")]
    async fn board_view(
        &self,
        Parameters(req): Parameters<BoardViewRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::tracker::board_view(&req.space);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    // === CAPACITY TOOLS (8 tools) ===

    #[tool(description = "Configure capacity: pane count, hours, availability factor, review bandwidth, build slots.")]
    async fn cap_configure(
        &self,
        Parameters(req): Parameters<CapConfigureRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::capacity::cap_configure(
            req.pane_count, req.hours_per_day, req.availability_factor,
            req.review_bandwidth, req.build_slots,
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Estimate ACU for a task based on type, complexity, and role.")]
    async fn cap_estimate(
        &self,
        Parameters(req): Parameters<CapEstimateRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::capacity::cap_estimate(
            &req.description, &req.complexity.unwrap_or_default(),
            &req.task_type.unwrap_or_default(), &req.role.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Log work done: ACU spent on an issue with role and review tracking.")]
    async fn cap_log_work(
        &self,
        Parameters(req): Parameters<CapLogWorkRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::capacity::cap_log_work_full(
            &req.issue_id, &req.space, &req.role, &req.pane_id.unwrap_or_default(),
            req.acu_spent, req.review_needed.unwrap_or(false), &req.notes.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Plan a sprint: assign issues, calculate capacity vs load, detect bottlenecks.")]
    async fn cap_plan_sprint(
        &self,
        Parameters(req): Parameters<CapPlanSprintRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::capacity::cap_plan_sprint(
            &req.space, &req.name.unwrap_or_default(), &req.start_date.unwrap_or_default(),
            req.days.unwrap_or(5), &req.issue_ids.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Capacity dashboard: today's ACU usage, review load, active sprint progress.")]
    async fn cap_dashboard(
        &self,
        Parameters(req): Parameters<CapDashboardRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::capacity::cap_dashboard(
            &req.space.unwrap_or_default(), &req.sprint_id.unwrap_or_default(),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Sprint burndown chart: ideal vs actual progress with projection.")]
    async fn cap_burndown(
        &self,
        Parameters(req): Parameters<CapBurndownRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::capacity::cap_burndown(&req.sprint_id.unwrap_or_default());
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Sprint velocity: historical throughput across sprints with accuracy tracking.")]
    async fn cap_velocity(
        &self,
        Parameters(req): Parameters<CapVelocityRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::capacity::cap_velocity(
            &req.space.unwrap_or_default(), req.count.unwrap_or(5),
        );
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "List all roles with definitions and today's utilization per role.")]
    async fn cap_roles(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let result = crate::capacity::cap_roles();
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }
}

#[tool_handler]
impl ServerHandler for AgentOSService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "AgentOS: Terminal orchestrator for AI agent teams. \
                 Spawns, assigns, monitors Claude agents across configurable panes \
                 from a single control plane. Fully autonomous with auto-cycle.".into()
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}

pub async fn run_mcp_server(app: Arc<App>) -> anyhow::Result<()> {
    tracing::info!("Starting AgentOS MCP server");

    let server = AgentOSService::new(app);
    let service = server.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("MCP serve error: {:?}", e);
    })?;

    service.waiting().await?;
    tracing::info!("AgentOS MCP server stopped");
    Ok(())
}
