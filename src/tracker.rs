use std::path::PathBuf;
use anyhow::Result;
use chrono::Local;
use serde_json::{json, Value};

use crate::config;

// === HELPERS ===

fn now_iso() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn get_prefix(space: &str) -> String {
    match space {
        "mailforge" => "MAIL".into(),
        "dataxlr8" => "DX".into(),
        "bskiller" => "BSK".into(),
        _ => space.chars().take(4).collect::<String>().to_uppercase(),
    }
}

fn issues_dir(space: &str) -> PathBuf {
    let dir = config::collab_root().join("spaces").join(space).join("issues");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn milestones_dir(space: &str) -> PathBuf {
    let dir = config::collab_root().join("spaces").join(space).join("milestones");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn processes_dir(space: &str) -> PathBuf {
    let dir = config::collab_root().join("spaces").join(space).join("processes");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn templates_dir() -> PathBuf {
    let dir = config::collab_root().join("templates");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn next_id(space: &str) -> u32 {
    let dir = issues_dir(space);
    let mut max_num: u32 = 0;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(v) = serde_json::from_str::<Value>(&content) {
                        if let Some(n) = v.get("number").and_then(|v| v.as_u64()) {
                            max_num = max_num.max(n as u32);
                        }
                    }
                }
            }
        }
    }
    max_num + 1
}

fn load_issue_by_id(space: &str, issue_id: &str) -> Option<Value> {
    let prefix = get_prefix(space);
    let num_str = if issue_id.contains('-') {
        issue_id.rsplit('-').next().unwrap_or(issue_id)
    } else {
        issue_id
    };
    let path = issues_dir(space).join(format!("{}-{}.json", prefix, num_str));
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            return serde_json::from_str(&content).ok();
        }
    }
    find_issue(space, issue_id)
}

fn save_issue_file(space: &str, issue: &Value) -> Result<()> {
    let prefix = get_prefix(space);
    let number = issue.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
    let path = issues_dir(space).join(format!("{}-{}.json", prefix, number));
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(issue)?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

// === EXISTING PUBLIC FUNCTIONS (used by tools.rs) ===

pub fn load_issues(space: &str) -> Vec<Value> {
    let dir = config::collab_root().join("spaces").join(space).join("issues");
    if !dir.exists() { return Vec::new(); }
    let mut issues = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |e| e == "json") {
                if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                    if let Ok(v) = serde_json::from_str::<Value>(&contents) {
                        issues.push(v);
                    }
                }
            }
        }
    }
    issues.sort_by(|a, b| {
        let a_id = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let b_id = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        a_id.cmp(b_id)
    });
    issues
}

pub fn find_issue(space: &str, issue_id: &str) -> Option<Value> {
    load_issues(space).into_iter().find(|issue| {
        issue.get("id").and_then(|v| v.as_str()) == Some(issue_id)
    })
}

pub fn update_issue(space: &str, issue_id: &str, updates: &Value) -> Result<bool> {
    let dir = config::collab_root().join("spaces").join(space).join("issues");
    if !dir.exists() { return Ok(false); }
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(mut data) = serde_json::from_str::<Value>(&contents) {
                        if data.get("id").and_then(|v| v.as_str()) == Some(issue_id) {
                            if let (Some(obj), Some(upd)) = (data.as_object_mut(), updates.as_object()) {
                                for (k, v) in upd { obj.insert(k.clone(), v.clone()); }
                            }
                            std::fs::write(&path, serde_json::to_string_pretty(&data)?)?;
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

pub fn load_board_summary() -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    let spaces_dir = config::collab_root().join("spaces");
    if !spaces_dir.exists() { return counts; }
    if let Ok(entries) = std::fs::read_dir(&spaces_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let space = entry.file_name().to_string_lossy().to_string();
                for issue in load_issues(&space) {
                    let status = issue.get("status").and_then(|v| v.as_str()).unwrap_or("backlog").to_string();
                    *counts.entry(status).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

// === NEW MCP TOOL FUNCTIONS ===

pub fn issue_create(
    space: &str, title: &str, issue_type: &str, priority: &str,
    description: &str, assignee: &str, milestone: &str, labels: &[String],
    estimated_acu: f64, role: &str, sprint: &str,
) -> Value {
    let valid_types = ["bug", "feature", "task", "improvement", "epic"];
    let valid_priorities = ["critical", "high", "medium", "low"];
    let itype = if issue_type.is_empty() { "task" } else { issue_type };
    let ipriority = if priority.is_empty() { "medium" } else { priority };

    if !valid_types.contains(&itype) {
        return json!({"error": format!("Invalid type: {}. Use: {:?}", itype, valid_types)});
    }
    if !valid_priorities.contains(&ipriority) {
        return json!({"error": format!("Invalid priority: {}. Use: {:?}", ipriority, valid_priorities)});
    }

    let number = next_id(space);
    let prefix = get_prefix(space);
    let issue_id = format!("{}-{}", prefix, number);

    let issue = json!({
        "id": issue_id, "number": number, "space": space,
        "title": title, "type": itype, "status": "todo", "priority": ipriority,
        "description": description, "assignee": assignee, "milestone": milestone,
        "labels": labels, "blocked_by": [], "linked_docs": [], "linked_commits": [],
        "linked_prs": [], "comments": [], "estimated_acu": estimated_acu,
        "actual_acu": 0.0, "role": role, "parallelizable": true, "sprint": sprint,
        "created_at": now_iso(), "updated_at": now_iso(), "closed_at": null,
    });

    match save_issue_file(space, &issue) {
        Ok(()) => json!({"created": issue_id, "title": title, "status": "todo"}),
        Err(e) => json!({"error": e.to_string()}),
    }
}

pub fn issue_update_full(
    space: &str, issue_id: &str, status: &str, priority: &str,
    assignee: &str, title: &str, description: &str, milestone: &str,
    add_label: &str, remove_label: &str, estimated_acu: f64,
    actual_acu: f64, role: &str, sprint: &str,
) -> Value {
    let mut issue = match load_issue_by_id(space, issue_id) {
        Some(i) => i,
        None => return json!({"error": format!("Issue not found: {}", issue_id)}),
    };

    let valid_statuses = ["backlog", "todo", "in_progress", "review", "done", "closed", "blocked"];
    if !status.is_empty() {
        if !valid_statuses.contains(&status) {
            return json!({"error": format!("Invalid status: {}. Use: {:?}", status, valid_statuses)});
        }
        issue["status"] = json!(status);
        if status == "done" || status == "closed" {
            issue["closed_at"] = json!(now_iso());
        }
    }
    if !priority.is_empty() { issue["priority"] = json!(priority); }
    if !assignee.is_empty() { issue["assignee"] = json!(assignee); }
    if !title.is_empty() { issue["title"] = json!(title); }
    if !description.is_empty() { issue["description"] = json!(description); }
    if !milestone.is_empty() { issue["milestone"] = json!(milestone); }
    if !add_label.is_empty() {
        if let Some(labels) = issue["labels"].as_array_mut() {
            if !labels.iter().any(|l| l.as_str() == Some(add_label)) {
                labels.push(json!(add_label));
            }
        }
    }
    if !remove_label.is_empty() {
        if let Some(labels) = issue["labels"].as_array_mut() {
            labels.retain(|l| l.as_str() != Some(remove_label));
        }
    }
    if estimated_acu > 0.0 { issue["estimated_acu"] = json!(estimated_acu); }
    if actual_acu > 0.0 { issue["actual_acu"] = json!(actual_acu); }
    if !role.is_empty() { issue["role"] = json!(role); }
    if !sprint.is_empty() { issue["sprint"] = json!(sprint); }
    issue["updated_at"] = json!(now_iso());

    match save_issue_file(space, &issue) {
        Ok(()) => json!({"updated": issue["id"], "status": issue["status"], "priority": issue["priority"]}),
        Err(e) => json!({"error": e.to_string()}),
    }
}

pub fn issue_list_filtered(
    space: &str, status: &str, issue_type: &str, priority: &str,
    assignee: &str, milestone: &str, label: &str, sprint: &str, role: &str,
) -> Value {
    let all = load_issues(space);
    let mut results: Vec<Value> = all.into_iter().filter(|i| {
        if !status.is_empty() && i["status"].as_str() != Some(status) { return false; }
        if !issue_type.is_empty() && i["type"].as_str() != Some(issue_type) { return false; }
        if !priority.is_empty() && i["priority"].as_str() != Some(priority) { return false; }
        if !assignee.is_empty() && i["assignee"].as_str() != Some(assignee) { return false; }
        if !milestone.is_empty() && i["milestone"].as_str() != Some(milestone) { return false; }
        if !label.is_empty() {
            let has = i["labels"].as_array().map_or(false, |l| l.iter().any(|v| v.as_str() == Some(label)));
            if !has { return false; }
        }
        if !sprint.is_empty() && i["sprint"].as_str() != Some(sprint) { return false; }
        if !role.is_empty() && i["role"].as_str() != Some(role) { return false; }
        true
    }).map(|i| json!({
        "id": i["id"], "title": i["title"], "type": i["type"], "status": i["status"],
        "priority": i["priority"], "assignee": i["assignee"], "milestone": i["milestone"],
        "estimated_acu": i["estimated_acu"], "actual_acu": i["actual_acu"],
        "role": i["role"], "sprint": i["sprint"],
    })).collect();

    // Sort: by status order then priority
    let status_ord = |s: &str| -> u8 {
        match s { "blocked"=>0, "in_progress"=>1, "review"=>2, "todo"=>3, "backlog"=>4, "done"=>5, "closed"=>6, _=>9 }
    };
    let prio_ord = |p: &str| -> u8 {
        match p { "critical"=>0, "high"=>1, "medium"=>2, "low"=>3, _=>9 }
    };
    results.sort_by(|a, b| {
        let sa = status_ord(a["status"].as_str().unwrap_or(""));
        let sb = status_ord(b["status"].as_str().unwrap_or(""));
        let pa = prio_ord(a["priority"].as_str().unwrap_or(""));
        let pb = prio_ord(b["priority"].as_str().unwrap_or(""));
        (sa, pa).cmp(&(sb, pb))
    });

    let count = results.len();
    json!({"issues": results, "count": count, "space": space})
}

pub fn issue_view(space: &str, issue_id: &str) -> Value {
    match load_issue_by_id(space, issue_id) {
        Some(i) => i,
        None => json!({"error": format!("Issue not found: {}", issue_id)}),
    }
}

pub fn issue_comment(space: &str, issue_id: &str, text: &str, author: &str) -> Value {
    let mut issue = match load_issue_by_id(space, issue_id) {
        Some(i) => i,
        None => return json!({"error": format!("Issue not found: {}", issue_id)}),
    };
    let comment_id = issue["comments"].as_array().map_or(1, |c| c.len() + 1);
    let comment = json!({"id": comment_id, "author": author, "text": text, "timestamp": now_iso()});
    if let Some(comments) = issue["comments"].as_array_mut() {
        comments.push(comment.clone());
    }
    issue["updated_at"] = json!(now_iso());
    let _ = save_issue_file(space, &issue);
    json!({"comment_added": comment})
}

pub fn issue_link(space: &str, issue_id: &str, link_type: &str, reference: &str) -> Value {
    let mut issue = match load_issue_by_id(space, issue_id) {
        Some(i) => i,
        None => return json!({"error": format!("Issue not found: {}", issue_id)}),
    };
    let key = format!("linked_{}s", link_type);
    if issue.get(&key).is_none() {
        return json!({"error": format!("Invalid link type: {}. Use: doc, commit, pr", link_type)});
    }
    if let Some(arr) = issue[&key].as_array_mut() {
        if !arr.iter().any(|v| v.as_str() == Some(reference)) {
            arr.push(json!(reference));
        }
    }
    issue["updated_at"] = json!(now_iso());
    let _ = save_issue_file(space, &issue);
    json!({"linked": format!("{}:{}", link_type, reference), "issue": issue["id"]})
}

pub fn issue_close(space: &str, issue_id: &str, resolution: &str) -> Value {
    let mut issue = match load_issue_by_id(space, issue_id) {
        Some(i) => i,
        None => return json!({"error": format!("Issue not found: {}", issue_id)}),
    };
    issue["status"] = json!("closed");
    issue["closed_at"] = json!(now_iso());
    issue["updated_at"] = json!(now_iso());
    let comment_id = issue["comments"].as_array().map_or(1, |c| c.len() + 1);
    if let Some(comments) = issue["comments"].as_array_mut() {
        comments.push(json!({"id": comment_id, "author": "system", "text": format!("Closed: {}", resolution), "timestamp": now_iso()}));
    }
    let _ = save_issue_file(space, &issue);
    json!({"closed": issue["id"], "resolution": resolution})
}

// === MILESTONES ===

pub fn milestone_create(space: &str, name: &str, description: &str, due_date: &str) -> Value {
    let slug = name.to_lowercase().replace(' ', "-");
    let path = milestones_dir(space).join(format!("{}.json", slug));
    if path.exists() {
        return json!({"error": format!("Milestone already exists: {}", slug)});
    }
    let ms = json!({"name": name, "slug": slug, "description": description, "due_date": due_date, "created_at": now_iso(), "status": "open"});
    let _ = std::fs::write(&path, serde_json::to_string_pretty(&ms).unwrap_or_default());
    json!({"created": slug, "due_date": due_date})
}

pub fn milestone_list(space: &str) -> Value {
    let dir = milestones_dir(space);
    let issues = load_issues(space);
    let mut results = vec![];

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(ms) = serde_json::from_str::<Value>(&content) {
                        let ms_name = ms["name"].as_str().unwrap_or("");
                        let ms_slug = ms["slug"].as_str().unwrap_or("");
                        let total = issues.iter().filter(|i| {
                            let m = i["milestone"].as_str().unwrap_or("");
                            m == ms_name || m == ms_slug
                        }).count();
                        let done = issues.iter().filter(|i| {
                            let m = i["milestone"].as_str().unwrap_or("");
                            let s = i["status"].as_str().unwrap_or("");
                            (m == ms_name || m == ms_slug) && (s == "done" || s == "closed")
                        }).count();
                        let pct = if total > 0 { done * 100 / total } else { 0 };
                        results.push(json!({
                            "name": ms_name, "slug": ms_slug,
                            "due_date": ms["due_date"], "status": ms["status"],
                            "total_issues": total, "done_issues": done,
                            "progress": format!("{}%", pct),
                        }));
                    }
                }
            }
        }
    }
    json!({"milestones": results, "count": results.len()})
}

// === TIMELINE ===

pub fn timeline_generate(space: &str, milestone_filter: &str) -> Value {
    let issues = load_issues(space);
    let mut sections: std::collections::HashMap<String, Vec<Value>> = std::collections::HashMap::new();

    for issue in &issues {
        if !milestone_filter.is_empty() && issue["milestone"].as_str() != Some(milestone_filter) { continue; }
        if issue["status"].as_str() == Some("closed") { continue; }

        let ms = issue.get("milestone").and_then(|v| v.as_str()).unwrap_or("Unassigned").to_string();
        let gantt_status = match issue["status"].as_str().unwrap_or("") {
            "done" => "done,",
            "in_progress" | "review" => "active,",
            "blocked" => "crit,",
            _ => "",
        };
        let id_str = issue["id"].as_str().unwrap_or("x").to_lowercase().replace('-', "");
        let title: String = issue["title"].as_str().unwrap_or("").chars().take(40).collect();
        let est = issue.get("estimate").and_then(|v| v.as_str()).unwrap_or("1d");
        let duration = if est.ends_with("pts") { format!("{}d", est.trim_end_matches("pts")) }
            else if est.ends_with('h') { "1d".into() }
            else if est.ends_with('d') { est.into() }
            else { "1d".into() };

        sections.entry(ms).or_default().push(json!({
            "id": id_str, "title": title, "status": gantt_status, "duration": duration,
        }));
    }

    let mut lines = vec![
        "gantt".into(),
        format!("    title {} Timeline", space),
        "    dateFormat YYYY-MM-DD".into(),
    ];
    for (section, items) in &sections {
        lines.push(format!("    section {}", section));
        for item in items {
            lines.push(format!("    {}    :{} {}, {}",
                item["title"].as_str().unwrap_or(""),
                item["status"].as_str().unwrap_or(""),
                item["id"].as_str().unwrap_or(""),
                item["duration"].as_str().unwrap_or("1d"),
            ));
        }
    }
    let mermaid = lines.join("\n");
    let total: usize = sections.values().map(|v| v.len()).sum();
    json!({"mermaid": mermaid, "embed": format!("```mermaid\n{}\n```", mermaid), "total_issues": total})
}

// === PROCESSES ===

pub fn process_start(space: &str, template_name: &str, context: &Value) -> Value {
    let template_path = templates_dir().join(format!("{}.md", template_name));
    if !template_path.exists() {
        let available: Vec<String> = std::fs::read_dir(templates_dir()).ok()
            .map(|entries| entries.flatten()
                .filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().to_string()))
                .collect())
            .unwrap_or_default();
        return json!({"error": format!("Template not found: {}", template_name), "available": available});
    }

    let mut content = std::fs::read_to_string(&template_path).unwrap_or_default();
    if let Some(ctx) = context.as_object() {
        for (k, v) in ctx {
            content = content.replace(&format!("{{{{{}}}}}", k), v.as_str().unwrap_or(""));
        }
    }

    let ts = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let process_id = format!("{}-{}", template_name, ts);
    let mut steps = vec![];
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            let done = trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]");
            let text = trimmed.trim_start_matches("- [ ]").trim_start_matches("- [x]").trim_start_matches("- [X]").trim();
            steps.push(json!({"index": steps.len(), "text": text, "done": done, "completed_at": null}));
        }
    }
    let total = steps.len();
    let completed = steps.iter().filter(|s| s["done"].as_bool().unwrap_or(false)).count();
    let process = json!({
        "id": process_id, "template": template_name, "space": space, "context": context,
        "status": "active", "started_at": now_iso(), "steps": steps,
        "total_steps": total, "completed_steps": completed,
    });
    let path = processes_dir(space).join(format!("{}.json", process_id));
    let _ = std::fs::write(&path, serde_json::to_string_pretty(&process).unwrap_or_default());
    json!({"process_id": process_id, "template": template_name, "total_steps": total})
}

pub fn process_update(space: &str, process_id: &str, step_index: usize, done: bool) -> Value {
    let path = processes_dir(space).join(format!("{}.json", process_id));
    if !path.exists() {
        return json!({"error": format!("Process not found: {}", process_id)});
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut process: Value = serde_json::from_str(&content).unwrap_or(json!({}));

    let steps_len = process["steps"].as_array().map_or(0, |s| s.len());
    if step_index >= steps_len {
        return json!({"error": format!("Step index out of range. Max: {}", steps_len.saturating_sub(1))});
    }

    process["steps"][step_index]["done"] = json!(done);
    process["steps"][step_index]["completed_at"] = if done { json!(now_iso()) } else { json!(null) };
    let completed = process["steps"].as_array()
        .map_or(0, |s| s.iter().filter(|st| st["done"].as_bool().unwrap_or(false)).count());
    process["completed_steps"] = json!(completed);
    if completed == steps_len { process["status"] = json!("completed"); }

    let _ = std::fs::write(&path, serde_json::to_string_pretty(&process).unwrap_or_default());
    let step_text = process["steps"][step_index]["text"].as_str().unwrap_or("").to_string();
    json!({"process_id": process_id, "step": step_text, "done": done, "progress": format!("{}/{}", completed, steps_len)})
}

pub fn process_list(space: &str) -> Value {
    let dir = processes_dir(space);
    let mut results = vec![];
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(p) = serde_json::from_str::<Value>(&content) {
                        results.push(json!({
                            "id": p["id"], "template": p["template"], "status": p["status"],
                            "progress": format!("{}/{}", p["completed_steps"], p["total_steps"]),
                            "started": p["started_at"],
                        }));
                    }
                }
            }
        }
    }
    let count = results.len();
    json!({"processes": results, "count": count})
}

pub fn process_template_create(name: &str, content: &str) -> Value {
    let path = templates_dir().join(format!("{}.md", name));
    match std::fs::write(&path, content) {
        Ok(()) => json!({"created": name, "path": path.to_string_lossy()}),
        Err(e) => json!({"error": e.to_string()}),
    }
}

// === BOARD ===

pub fn board_view(space: &str) -> Value {
    let valid_statuses = ["backlog", "todo", "in_progress", "review", "done", "closed", "blocked"];
    let issues = load_issues(space);
    let mut columns: std::collections::HashMap<&str, Vec<Value>> = std::collections::HashMap::new();
    for s in &valid_statuses { columns.insert(s, vec![]); }

    for issue in &issues {
        let status = issue["status"].as_str().unwrap_or("backlog");
        if let Some(col) = columns.get_mut(status) {
            let title: String = issue["title"].as_str().unwrap_or("").chars().take(50).collect();
            col.push(json!({
                "id": issue["id"], "title": title, "type": issue["type"],
                "priority": issue["priority"], "assignee": issue["assignee"],
                "estimated_acu": issue["estimated_acu"], "actual_acu": issue["actual_acu"],
                "role": issue["role"], "sprint": issue["sprint"],
            }));
        }
    }

    // Only non-empty columns
    let board: serde_json::Map<String, Value> = columns.into_iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect();
    let total: usize = board.values().map(|v| v.as_array().map_or(0, |a| a.len())).sum();
    json!({"board": board, "total": total, "space": space})
}
