//! Factory pipeline — single command → multi-agent orchestration.
//!
//! `:go dataxlr8 add OAuth login` → dev → qa + security → review
//! Pipeline state derived from queue tasks via pipeline_id. No separate storage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::queue;
use crate::scanner;

// ============================================================
// Pipeline data types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineView {
    pub id: String,
    pub project: String,
    pub description: String,
    pub template: String,
    pub created_at: String,
    pub status: String,
    pub stages: Vec<StageView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageView {
    pub name: String,
    pub role: String,
    pub task_id: String,
    pub status: String,
    pub pane: Option<u8>,
    pub summary: Option<String>,
}

// ============================================================
// Pipeline templates
// ============================================================

struct StageTemplate {
    name: &'static str,
    role: &'static str,
    parallel_with: &'static [&'static str],
    prompt: &'static str,
}

struct PipelineTemplate {
    name: &'static str,
    stages: &'static [StageTemplate],
}

const PROMPT_DEV: &str = r#"You are the DEVELOPER stage of a factory pipeline.

## Task
{{task}}

## Project: {{project}} ({{project_path}})

## Instructions
1. Implement the requested feature/fix completely
2. Write clean, well-structured code
3. Run existing tests to make sure nothing breaks
4. Commit and push your changes with a clear commit message
5. At the end, summarize: what you built, key decisions, files changed

A QA agent and security auditor will verify your work after you finish."#;

const PROMPT_QA: &str = r#"You are the QA stage of a factory pipeline.

## Task
Test and verify: {{task}}

## Project: {{project}} ({{project_path}})

## Predecessor Results
(Auto-injected by the queue system from the developer agent's output)

## Instructions
1. Review the developer's changes (check git log, git diff)
2. Run the full test suite — report pass/fail counts
3. Write new tests for any untested new code
4. Test edge cases and error handling paths
5. Verify the feature works end-to-end
6. Commit any new tests you write
7. Final verdict: PASS or FAIL with details

Use quality tools: log_test, log_build to record results."#;

const PROMPT_SECURITY: &str = r#"You are the SECURITY AUDITOR stage of a factory pipeline.

## Task
Security audit: {{task}}

## Project: {{project}} ({{project_path}})

## Predecessor Results
(Auto-injected by the queue system from the developer agent's output)

## Instructions
1. Review all changed files for security vulnerabilities
2. Check OWASP Top 10: injection, XSS, broken auth, CSRF, SSRF
3. Verify input validation and output sanitization
4. Check for hardcoded secrets, API keys, credentials
5. Review new dependencies for known CVEs
6. Check authentication/authorization patterns
7. Review error handling — no sensitive info leaked in errors
8. Report findings by severity: CRITICAL / HIGH / MEDIUM / LOW
9. If clean: explicitly state "No security issues found"

Create tracker issues for any findings with priority=critical or priority=high."#;

const PROMPT_PENTEST: &str = r#"You are the PENETRATION TESTER stage of a factory pipeline.

## Task
Pentest: {{task}}

## Project: {{project}} ({{project_path}})

## Predecessor Results
(Auto-injected by the queue system from the developer agent's output)

## Instructions
1. Attempt injection attacks (SQL, XSS, command injection) on any new endpoints
2. Test for path traversal and unauthorized file access
3. Check for broken authentication/session management
4. Test rate limiting and resource exhaustion vectors
5. Verify authorization boundaries (horizontal and vertical privilege escalation)
6. Report exploits found with severity and remediation steps"#;

const PROMPT_REVIEW: &str = r#"You are the REVIEWER — the final gate of a factory pipeline.

## Task
Final review: {{task}}

## Project: {{project}} ({{project_path}})

## All Stage Results
(Auto-injected by the queue system from all predecessor stages)

## Instructions
1. Review the developer's code for quality, correctness, architecture
2. Verify QA tests passed — flag if they didn't
3. Review security findings — verify critical/high issues were addressed
4. Check code style, naming conventions, documentation
5. Create a pull request if not already done
6. Merge if everything passes
7. Final verdict: APPROVE or REQUEST_CHANGES with specific items"#;

// Static template definitions
const TMPL_FULL: PipelineTemplate = PipelineTemplate {
    name: "full",
    stages: &[
        StageTemplate { name: "dev", role: "developer", parallel_with: &[], prompt: PROMPT_DEV },
        StageTemplate { name: "qa", role: "qa", parallel_with: &["security"], prompt: PROMPT_QA },
        StageTemplate { name: "security", role: "security", parallel_with: &["qa"], prompt: PROMPT_SECURITY },
        StageTemplate { name: "review", role: "reviewer", parallel_with: &[], prompt: PROMPT_REVIEW },
    ],
};

const TMPL_QUICK: PipelineTemplate = PipelineTemplate {
    name: "quick",
    stages: &[
        StageTemplate { name: "dev", role: "developer", parallel_with: &[], prompt: PROMPT_DEV },
        StageTemplate { name: "qa", role: "qa", parallel_with: &[], prompt: PROMPT_QA },
    ],
};

const TMPL_SECURE: PipelineTemplate = PipelineTemplate {
    name: "secure",
    stages: &[
        StageTemplate { name: "dev", role: "developer", parallel_with: &[], prompt: PROMPT_DEV },
        StageTemplate { name: "qa", role: "qa", parallel_with: &["security", "pentest"], prompt: PROMPT_QA },
        StageTemplate { name: "security", role: "security", parallel_with: &["qa", "pentest"], prompt: PROMPT_SECURITY },
        StageTemplate { name: "pentest", role: "security", parallel_with: &["qa", "security"], prompt: PROMPT_PENTEST },
        StageTemplate { name: "review", role: "reviewer", parallel_with: &[], prompt: PROMPT_REVIEW },
    ],
};

const ALL_TEMPLATES: &[&PipelineTemplate] = &[&TMPL_FULL, &TMPL_QUICK, &TMPL_SECURE];

pub fn template_names() -> Vec<&'static str> {
    ALL_TEMPLATES.iter().map(|t| t.name).collect()
}

pub fn template_info() -> Vec<(&'static str, Vec<&'static str>)> {
    ALL_TEMPLATES.iter().map(|t| {
        let stages: Vec<&str> = t.stages.iter().map(|s| s.name).collect();
        (t.name, stages)
    }).collect()
}

// ============================================================
// Pipeline creation
// ============================================================

/// Create a multi-stage pipeline. Returns (pipeline_id, task_ids).
pub fn create_pipeline(
    project: &str,
    description: &str,
    template_name: &str,
    priority: u8,
) -> Result<(String, Vec<String>)> {
    let template = ALL_TEMPLATES.iter()
        .find(|t| t.name == template_name)
        .ok_or_else(|| anyhow::anyhow!(
            "Unknown template '{}'. Available: {}",
            template_name,
            template_names().join(", ")
        ))?;

    // Resolve project path for prompt enrichment
    let project_path = scanner::project_by_name(project)
        .map(|p| p.path.clone())
        .unwrap_or_default();

    let pipeline_id = gen_pipeline_id();
    let groups = build_stage_groups(template.stages);
    let mut task_ids: Vec<String> = Vec::new();
    let mut prev_group_ids: Vec<String> = Vec::new();

    for group in &groups {
        let mut group_ids: Vec<String> = Vec::new();

        for stage in group {
            let prompt = stage.prompt
                .replace("{{task}}", description)
                .replace("{{project}}", project)
                .replace("{{project_path}}", &project_path);

            let task_label = format!("[{}] {}", stage.name, description);

            let task = queue::add_task(
                project,
                stage.role,
                &task_label,
                &prompt,
                priority,
                prev_group_ids.clone(),
            )?;

            // Set pipeline_id on the created task
            let mut q = queue::load_queue();
            if let Some(t) = q.tasks.iter_mut().find(|t| t.id == task.id) {
                t.pipeline_id = Some(pipeline_id.clone());
            }
            queue::save_queue(&q)?;

            group_ids.push(task.id.clone());
            task_ids.push(task.id);
        }

        prev_group_ids = group_ids;
    }

    Ok((pipeline_id, task_ids))
}

/// Group stages: stages with parallel_with links form a group.
/// Groups execute sequentially; stages within a group run in parallel.
fn build_stage_groups<'a>(stages: &'a [StageTemplate]) -> Vec<Vec<&'a StageTemplate>> {
    let mut groups: Vec<Vec<&'a StageTemplate>> = Vec::new();
    let mut assigned: HashSet<&str> = HashSet::new();

    for stage in stages {
        if assigned.contains(stage.name) {
            continue;
        }

        let mut group = vec![stage];
        assigned.insert(stage.name);

        for parallel_name in stage.parallel_with {
            if assigned.contains(parallel_name) {
                continue;
            }
            if let Some(ps) = stages.iter().find(|s| s.name == *parallel_name) {
                group.push(ps);
                assigned.insert(ps.name);
            }
        }

        groups.push(group);
    }

    groups
}

fn gen_pipeline_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    let hash: u32 = (ts as u32).wrapping_mul(2654435761);
    format!("pipe_{}_{:04x}", ts % 10_000_000, hash % 0xFFFF)
}

// ============================================================
// Pipeline status (derived from queue)
// ============================================================

/// List all pipelines, derived from queue tasks grouped by pipeline_id.
pub fn list_pipelines() -> Vec<PipelineView> {
    let q = queue::load_queue();
    let mut map: HashMap<String, Vec<&queue::QueueTask>> = HashMap::new();

    for task in &q.tasks {
        if let Some(ref pid) = task.pipeline_id {
            map.entry(pid.clone()).or_default().push(task);
        }
    }

    let mut pipelines: Vec<PipelineView> = map.into_iter().map(|(pid, tasks)| {
        let stages: Vec<StageView> = tasks.iter().map(|t| {
            let name = t.task.strip_prefix('[')
                .and_then(|s| s.split(']').next())
                .unwrap_or("?")
                .to_string();

            let summary = t.result.as_ref().map(|r| r.chars().take(60).collect());

            StageView {
                name,
                role: t.role.clone(),
                task_id: t.id.clone(),
                status: format!("{:?}", t.status).to_lowercase(),
                pane: t.pane,
                summary,
            }
        }).collect();

        let project = tasks.first().map(|t| t.project.clone()).unwrap_or_default();
        let description = tasks.first()
            .map(|t| t.task.split(']').last().unwrap_or(&t.task).trim().to_string())
            .unwrap_or_default();
        let created_at = tasks.first().map(|t| t.added_at.clone()).unwrap_or_default();

        let status = if tasks.iter().any(|t| t.status == queue::QueueStatus::Failed) {
            "failed"
        } else if tasks.iter().all(|t| t.status == queue::QueueStatus::Done) {
            "done"
        } else if tasks.iter().any(|t| t.status == queue::QueueStatus::Running) {
            "running"
        } else {
            "pending"
        }.to_string();

        let stage_names: Vec<&str> = stages.iter().map(|s| s.name.as_str()).collect();
        let template = if stage_names.iter().any(|n| *n == "pentest") { "secure" }
            else if stage_names.iter().any(|n| *n == "review") { "full" }
            else if stage_names.len() <= 2 { "quick" }
            else { "custom" }.to_string();

        PipelineView { id: pid, project, description, template, created_at, status, stages }
    }).collect();

    pipelines.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    pipelines
}

/// Get a single pipeline by ID.
pub fn get_pipeline(pipeline_id: &str) -> Option<PipelineView> {
    list_pipelines().into_iter().find(|p| p.id == pipeline_id)
}

// ============================================================
// Classification (rule-based project detection)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IntentType {
    NewFeature, BugFix, Refactor, Test, Docs, Security, Performance, Unknown,
}

/// Try to detect which project a request is about from the description text.
/// Returns (project_name, confidence) or None.
pub fn detect_project(description: &str) -> Option<(String, f32)> {
    let registry = scanner::load_registry();
    let lower = description.to_lowercase();
    let mut scores: Vec<(String, u32)> = Vec::new();

    for project in &registry.projects {
        let mut score: u32 = 0;
        let name_lower = project.name.to_lowercase();

        if lower.contains(&name_lower) { score += 100; }
        for word in lower.split_whitespace() {
            if word.len() >= 3 && name_lower.contains(word) { score += 50; }
        }
        for tech in &project.tech {
            if lower.contains(&tech.to_lowercase()) { score += 30; }
        }

        if score > 0 {
            scores.push((project.name.clone(), score));
        }
    }

    scores.sort_by(|a, b| b.1.cmp(&a.1));
    scores.first().map(|(name, score)| (name.clone(), (*score as f32 / 150.0).min(1.0)))
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_grouping_full() {
        let groups = build_stage_groups(TMPL_FULL.stages);
        assert_eq!(groups.len(), 3); // dev | qa+security | review
        assert_eq!(groups[0].len(), 1); // dev alone
        assert_eq!(groups[1].len(), 2); // qa + security parallel
        assert_eq!(groups[2].len(), 1); // review alone
    }

    #[test]
    fn test_stage_grouping_quick() {
        let groups = build_stage_groups(TMPL_QUICK.stages);
        assert_eq!(groups.len(), 2); // dev | qa
        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[1].len(), 1);
    }

    #[test]
    fn test_stage_grouping_secure() {
        let groups = build_stage_groups(TMPL_SECURE.stages);
        assert_eq!(groups.len(), 3); // dev | qa+security+pentest | review
        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[1].len(), 3); // qa + security + pentest parallel
        assert_eq!(groups[2].len(), 1);
    }

    #[test]
    fn test_template_names() {
        let names = template_names();
        assert!(names.contains(&"full"));
        assert!(names.contains(&"quick"));
        assert!(names.contains(&"secure"));
    }

    #[test]
    fn test_pipeline_id_format() {
        let id = gen_pipeline_id();
        assert!(id.starts_with("pipe_"));
        assert!(id.len() > 10);
    }
}
