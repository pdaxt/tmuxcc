//! Factory tools: factory_run, factory_status, factory_list.
//!
//! Thin MCP wrappers over crate::factory pipeline API.

use crate::app::App;
use crate::factory;
use super::super::types::*;
use super::helpers::*;

/// Start a factory pipeline: detect project → create pipeline → trigger auto_cycle.
pub async fn factory_run(app: &App, req: FactoryRequest) -> String {
    // Step 1: Resolve project
    let project_name: String = if let Some(ref explicit) = req.project {
        explicit.clone()
    } else {
        match factory::detect_project(&req.request) {
            Some((name, confidence)) if confidence >= 0.2 => name,
            Some((name, confidence)) => {
                return json_err(&format!(
                    "Low confidence match ({:.0}%) to '{}'. Use project= to override.",
                    confidence * 100.0, name
                ));
            }
            None => {
                return json_err("Could not identify a project. Be more specific or add project= parameter.");
            }
        }
    };

    // Step 2: Create pipeline (default: "full" template = dev → qa+security → review)
    let template = req.template.as_deref().unwrap_or("full");
    let (pipeline_id, task_ids) = match factory::create_pipeline(
        &project_name,
        &req.request,
        template,
        req.priority.unwrap_or(1),
    ) {
        Ok(result) => result,
        Err(e) => return json_err(&format!("Pipeline creation failed: {}", e)),
    };

    // Step 3: Trigger auto_cycle to spawn the dev task immediately
    let _cycle = super::queue_tools::auto_cycle(app).await;

    // Step 4: Return pipeline info
    let pipeline = factory::get_pipeline(&pipeline_id);
    let stages: Vec<serde_json::Value> = pipeline.as_ref()
        .map(|p| p.stages.iter().map(|s| serde_json::json!({
            "name": s.name,
            "role": s.role,
            "task_id": s.task_id,
            "status": s.status,
            "pane": s.pane,
        })).collect())
        .unwrap_or_default();

    serde_json::json!({
        "status": "pipeline_started",
        "pipeline_id": pipeline_id,
        "project": project_name,
        "template": template,
        "task_ids": task_ids,
        "stages": stages,
        "message": "Pipeline started. Dev agent spawns on next free pane. QA + Security auto-trigger when dev completes.",
    }).to_string()
}

/// Get status of a specific pipeline (or list all if pipeline_id omitted).
pub fn factory_status(req: &FactoryStatusRequest) -> String {
    match &req.pipeline_id {
        Some(pid) => match factory::get_pipeline(pid) {
            Some(pipeline) => serde_json::to_string(&pipeline).unwrap_or_else(|e| json_err(&e.to_string())),
            None => json_err(&format!("Pipeline '{}' not found", pid)),
        },
        None => factory_list(),
    }
}

/// List all pipelines.
pub fn factory_list() -> String {
    let pipelines = factory::list_pipelines();
    let active: Vec<&factory::PipelineView> = pipelines.iter()
        .filter(|p| p.status != "done" && p.status != "failed")
        .collect();
    let done = pipelines.iter().filter(|p| p.status == "done").count();
    let failed = pipelines.iter().filter(|p| p.status == "failed").count();

    let summaries: Vec<serde_json::Value> = pipelines.iter().map(|p| {
        serde_json::json!({
            "id": p.id,
            "project": p.project,
            "description": truncate(&p.description, 50),
            "template": p.template,
            "status": p.status,
            "stages": p.stages.len(),
            "created_at": p.created_at,
        })
    }).collect();

    serde_json::json!({
        "pipelines": summaries,
        "active": active.len(),
        "done": done,
        "failed": failed,
        "total": pipelines.len(),
        "templates": factory::template_info().iter().map(|(name, stages)| {
            serde_json::json!({"name": name, "stages": stages})
        }).collect::<Vec<_>>(),
    }).to_string()
}
