use std::path::PathBuf;
use std::process::Command;
use anyhow::Result;
use chrono::Local;
use serde_json::{json, Value};

use crate::config;

const DEFAULT_PORT_MIN: u16 = 3001;
const DEFAULT_PORT_MAX: u16 = 3099;
const MAX_KB_ENTRIES: usize = 500;
const MAX_MESSAGES: usize = 200;
const MAX_BUILD_HISTORY: usize = 50;

fn registry_dir() -> PathBuf {
    config::home_dir().join(".claude").join("multi_agent")
}

fn ensure_dir() {
    let dir = registry_dir();
    let _ = std::fs::create_dir_all(&dir);
}

fn now_iso() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn load_json(name: &str) -> Value {
    let path = registry_dir().join(name);
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str(&content) {
                return v;
            }
        }
    }
    json!({})
}

fn save_json(name: &str, data: &Value) -> Result<()> {
    ensure_dir();
    let path = registry_dir().join(name);
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(data)?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

fn is_port_in_use(port: u16) -> (bool, Option<String>) {
    if let Ok(output) = Command::new("lsof")
        .args(["-i", &format!(":{}", port), "-t"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if output.status.success() && !stdout.is_empty() {
            let pid = stdout.lines().next().unwrap_or("").to_string();
            return (true, Some(pid));
        }
    }
    (false, None)
}

fn is_pane_active(pane_id: &str) -> bool {
    if let Ok(output) = Command::new("tmux")
        .args(["list-panes", "-a", "-F", "#{session_name}:#{window_index}.#{pane_index}"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.lines().any(|l| l.trim() == pane_id);
    }
    false
}

fn gen_short_id(seed: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    now_iso().hash(&mut h);
    format!("{:08x}", h.finish() as u32)
}

// ============================================================================
// PORT REGISTRY
// ============================================================================

pub fn port_allocate(service: &str, pane_id: &str, preferred: Option<u16>, description: &str) -> Value {
    let mut ports = load_json("ports.json");
    let allocs = ports.as_object_mut().unwrap();
    if !allocs.contains_key("allocations") { allocs.insert("allocations".into(), json!({})); }
    if !allocs.contains_key("services") { allocs.insert("services".into(), json!({})); }

    // Check if service already allocated
    if let Some(existing_port) = ports["services"].get(service).and_then(|v| v.as_u64()) {
        let (in_use, pid) = is_port_in_use(existing_port as u16);
        if in_use {
            return json!({"status": "exists", "port": existing_port, "pid": pid});
        }
    }

    // Find free port
    let mut port: Option<u16> = None;
    if let Some(pref) = preferred {
        let (in_use, _) = is_port_in_use(pref);
        if !in_use && !ports["allocations"].as_object().unwrap().contains_key(&pref.to_string()) {
            port = Some(pref);
        }
    }
    if port.is_none() {
        for p in DEFAULT_PORT_MIN..=DEFAULT_PORT_MAX {
            if !ports["allocations"].as_object().unwrap().contains_key(&p.to_string()) {
                let (in_use, _) = is_port_in_use(p);
                if !in_use {
                    port = Some(p);
                    break;
                }
            }
        }
    }

    let Some(port) = port else {
        return json!({"error": "No free ports available"});
    };

    ports["allocations"][port.to_string()] = json!({
        "service": service, "pane_id": pane_id,
        "description": description, "allocated_at": now_iso()
    });
    ports["services"][service] = json!(port);
    let _ = save_json("ports.json", &ports);
    json!({"status": "allocated", "port": port, "service": service})
}

pub fn port_release(port: u16) -> Value {
    let mut ports = load_json("ports.json");
    let key = port.to_string();
    if let Some(alloc) = ports["allocations"].get(&key).cloned() {
        if let Some(service) = alloc["service"].as_str() {
            if let Some(svcs) = ports["services"].as_object_mut() { svcs.remove(service); }
        }
        if let Some(a) = ports["allocations"].as_object_mut() { a.remove(&key); }
        let _ = save_json("ports.json", &ports);
        return json!({"status": "released", "port": port});
    }
    json!({"status": "not_found"})
}

pub fn port_list() -> Value {
    let ports = load_json("ports.json");
    let mut result = vec![];
    if let Some(allocs) = ports["allocations"].as_object() {
        for (port_str, info) in allocs {
            let port: u16 = port_str.parse().unwrap_or(0);
            let (active, pid) = is_port_in_use(port);
            result.push(json!({
                "port": port, "service": info["service"],
                "pane_id": info["pane_id"], "active": active, "pid": pid
            }));
        }
    }
    json!({"ports": result})
}

pub fn port_get(service: &str) -> Value {
    let ports = load_json("ports.json");
    if let Some(port) = ports["services"].get(service).and_then(|v| v.as_u64()) {
        let (active, pid) = is_port_in_use(port as u16);
        return json!({"found": true, "port": port, "active": active, "pid": pid});
    }
    json!({"found": false})
}

// ============================================================================
// AGENT COORDINATION
// ============================================================================

pub fn agent_register(pane_id: &str, project: &str, task: &str, files: &[String]) -> Value {
    let mut agents = load_json("agents.json");
    if agents.get("agents").is_none() { agents["agents"] = json!({}); }
    if agents.get("locks").is_none() { agents["locks"] = json!({}); }

    agents["agents"][pane_id] = json!({
        "project": project, "task": task, "files": files,
        "registered_at": now_iso(), "last_update": now_iso()
    });
    let _ = save_json("agents.json", &agents);

    let mut others = vec![];
    if let Some(all) = agents["agents"].as_object() {
        for (p, info) in all {
            if p != pane_id && info["project"].as_str() == Some(project) {
                others.push(json!({"pane": p, "task": info["task"]}));
            }
        }
    }
    json!({"status": "registered", "other_agents": others})
}

pub fn agent_update(pane_id: &str, task: &str, files: Option<&[String]>) -> Value {
    let mut agents = load_json("agents.json");
    if agents["agents"].get(pane_id).is_some() {
        agents["agents"][pane_id]["task"] = json!(task);
        agents["agents"][pane_id]["last_update"] = json!(now_iso());
        if let Some(f) = files { agents["agents"][pane_id]["files"] = json!(f); }
        let _ = save_json("agents.json", &agents);
        return json!({"status": "updated"});
    }
    json!({"status": "not_found"})
}

pub fn agent_list(project: Option<&str>) -> Value {
    let agents = load_json("agents.json");
    let mut result = vec![];
    if let Some(all) = agents["agents"].as_object() {
        for (pane_id, info) in all {
            if let Some(p) = project {
                if info["project"].as_str() != Some(p) { continue; }
            }
            result.push(json!({
                "pane_id": pane_id, "project": info["project"],
                "task": info["task"], "files": info["files"],
                "active": is_pane_active(pane_id),
                "last_update": info["last_update"]
            }));
        }
    }
    json!({"agents": result})
}

pub fn agent_deregister(pane_id: &str) -> Value {
    let mut agents = load_json("agents.json");
    if let Some(a) = agents["agents"].as_object_mut() {
        if a.remove(pane_id).is_some() {
            // Release locks held by this pane
            if let Some(locks) = agents["locks"].as_object_mut() {
                locks.retain(|_, v| v["pane_id"].as_str() != Some(pane_id));
            }
            let _ = save_json("agents.json", &agents);
            return json!({"status": "deregistered"});
        }
    }
    json!({"status": "not_found"})
}

// ============================================================================
// FILE LOCKS
// ============================================================================

pub fn lock_acquire(pane_id: &str, files: &[String], reason: &str) -> Value {
    let mut agents = load_json("agents.json");
    if agents.get("locks").is_none() { agents["locks"] = json!({}); }

    let mut blocked = vec![];
    for f in files {
        if let Some(lock) = agents["locks"].get(f.as_str()) {
            if lock["pane_id"].as_str() != Some(pane_id) {
                blocked.push(json!({
                    "file": f, "locked_by": lock["pane_id"], "reason": lock["reason"]
                }));
            }
        }
    }
    if !blocked.is_empty() {
        return json!({"status": "blocked", "blocked": blocked});
    }

    for f in files {
        agents["locks"][f.as_str()] = json!({
            "pane_id": pane_id, "reason": reason, "acquired_at": now_iso()
        });
    }
    let _ = save_json("agents.json", &agents);
    json!({"status": "acquired", "files": files})
}

pub fn lock_release(pane_id: &str, files: &[String]) -> Value {
    let mut agents = load_json("agents.json");
    let mut released = vec![];
    if let Some(locks) = agents["locks"].as_object_mut() {
        let keys: Vec<String> = locks.keys().cloned().collect();
        for f in keys {
            if locks[&f]["pane_id"].as_str() == Some(pane_id) {
                if files.is_empty() || files.iter().any(|x| x == &f) {
                    released.push(f.clone());
                    locks.remove(&f);
                }
            }
        }
    }
    let _ = save_json("agents.json", &agents);
    json!({"status": "released", "files": released})
}

pub fn lock_check(files: &[String]) -> Value {
    let agents = load_json("agents.json");
    let mut locked = vec![];
    for f in files {
        if let Some(lock) = agents["locks"].get(f.as_str()) {
            locked.push(json!({
                "file": f, "locked_by": lock["pane_id"], "reason": lock["reason"]
            }));
        }
    }
    json!({"locked": locked, "clear": locked.is_empty()})
}

// ============================================================================
// GIT COORDINATION
// ============================================================================

pub fn git_claim_branch(pane_id: &str, branch: &str, repo: &str, purpose: &str) -> Value {
    let mut git = load_json("git.json");
    if git.get("branches").is_none() { git["branches"] = json!({}); }

    let key = format!("{}:{}", repo, branch);
    if let Some(existing) = git["branches"].get(&key) {
        if existing["pane_id"].as_str() != Some(pane_id) {
            if is_pane_active(existing["pane_id"].as_str().unwrap_or("")) {
                return json!({
                    "status": "claimed_by_other",
                    "owner": existing["pane_id"],
                    "purpose": existing["purpose"]
                });
            }
        }
    }

    git["branches"][&key] = json!({
        "pane_id": pane_id, "purpose": purpose, "claimed_at": now_iso()
    });
    let _ = save_json("git.json", &git);
    json!({"status": "claimed", "branch": branch})
}

pub fn git_release_branch(pane_id: &str, branch: &str, repo: &str) -> Value {
    let mut git = load_json("git.json");
    let key = format!("{}:{}", repo, branch);
    if let Some(entry) = git["branches"].get(&key) {
        if entry["pane_id"].as_str() == Some(pane_id) {
            if let Some(b) = git["branches"].as_object_mut() { b.remove(&key); }
            let _ = save_json("git.json", &git);
            return json!({"status": "released"});
        }
        return json!({"status": "not_owner"});
    }
    json!({"status": "not_found"})
}

pub fn git_list_branches(repo: Option<&str>) -> Value {
    let git = load_json("git.json");
    let mut result = vec![];
    if let Some(branches) = git["branches"].as_object() {
        for (key, info) in branches {
            if let Some((r, b)) = key.rsplit_once(':') {
                if let Some(filter) = repo {
                    if r != filter { continue; }
                }
                result.push(json!({
                    "repo": r, "branch": b,
                    "pane_id": info["pane_id"], "purpose": info["purpose"],
                    "active": is_pane_active(info["pane_id"].as_str().unwrap_or(""))
                }));
            }
        }
    }
    json!({"branches": result})
}

pub fn git_pre_commit_check(pane_id: &str, _repo: &str, files: &[String]) -> Value {
    let agents = load_json("agents.json");
    let mut conflicts = vec![];

    // Check file locks
    if let Some(locks) = agents["locks"].as_object() {
        for f in files {
            if let Some(lock) = locks.get(f.as_str()) {
                if lock["pane_id"].as_str() != Some(pane_id) {
                    conflicts.push(json!({"type": "file_lock", "file": f, "owner": lock["pane_id"]}));
                }
            }
        }
    }

    // Check concurrent edits
    if let Some(all) = agents["agents"].as_object() {
        for (p, info) in all {
            if p != pane_id {
                if let Some(agent_files) = info["files"].as_array() {
                    let overlap: Vec<&String> = files.iter()
                        .filter(|f| agent_files.iter().any(|af| af.as_str() == Some(f.as_str())))
                        .collect();
                    if !overlap.is_empty() {
                        conflicts.push(json!({"type": "concurrent_edit", "pane": p, "files": overlap}));
                    }
                }
            }
        }
    }

    json!({"safe": conflicts.is_empty(), "conflicts": conflicts})
}

// ============================================================================
// BUILD COORDINATION
// ============================================================================

pub fn build_claim(pane_id: &str, project: &str, build_type: &str) -> Value {
    let mut builds = load_json("builds.json");
    if builds.get("active").is_none() { builds["active"] = json!({}); }
    if builds.get("history").is_none() { builds["history"] = json!([]); }

    if let Some(existing) = builds["active"].get(project) {
        if is_pane_active(existing["pane_id"].as_str().unwrap_or("")) {
            return json!({"status": "busy", "owner": existing["pane_id"], "started": existing["started_at"]});
        }
    }

    builds["active"][project] = json!({
        "pane_id": pane_id, "build_type": build_type, "started_at": now_iso()
    });
    let _ = save_json("builds.json", &builds);
    json!({"status": "claimed"})
}

pub fn build_release(pane_id: &str, project: &str, success: bool, output: &str) -> Value {
    let mut builds = load_json("builds.json");
    if let Some(active) = builds["active"].get(project).cloned() {
        if active["pane_id"].as_str() == Some(pane_id) {
            let mut entry = active.clone();
            entry["completed_at"] = json!(now_iso());
            entry["success"] = json!(success);
            entry["output"] = json!(output);
            entry["project"] = json!(project);

            if let Some(history) = builds["history"].as_array_mut() {
                history.push(entry);
                if history.len() > MAX_BUILD_HISTORY {
                    let drain = history.len() - MAX_BUILD_HISTORY;
                    history.drain(..drain);
                }
            }

            if let Some(a) = builds["active"].as_object_mut() { a.remove(project); }
            let _ = save_json("builds.json", &builds);
            return json!({"status": "released"});
        }
        return json!({"status": "not_owner"});
    }
    json!({"status": "not_found"})
}

pub fn build_status(project: &str) -> Value {
    let builds = load_json("builds.json");
    if let Some(info) = builds["active"].get(project) {
        return json!({"building": true, "owner": info["pane_id"], "started": info["started_at"]});
    }
    json!({"building": false})
}

pub fn build_get_last(project: &str) -> Value {
    let builds = load_json("builds.json");
    if let Some(history) = builds["history"].as_array() {
        for entry in history.iter().rev() {
            if entry["project"].as_str() == Some(project) {
                return json!({"found": true, "build": entry});
            }
        }
    }
    json!({"found": false})
}

// ============================================================================
// TASK QUEUE (inter-agent, not os_queue)
// ============================================================================

pub fn task_add(project: &str, title: &str, description: &str, priority: &str, added_by: &str) -> Value {
    let mut tasks = load_json("tasks.json");
    if tasks.get("queue").is_none() { tasks["queue"] = json!([]); }

    let task_id = gen_short_id(title);
    if let Some(queue) = tasks["queue"].as_array_mut() {
        queue.push(json!({
            "id": task_id, "project": project, "title": title,
            "description": description, "priority": priority,
            "status": "pending", "added_by": added_by, "added_at": now_iso()
        }));
    }
    let _ = save_json("tasks.json", &tasks);
    json!({"status": "added", "task_id": task_id})
}

pub fn task_claim(pane_id: &str, project: Option<&str>) -> Value {
    let mut tasks = load_json("tasks.json");
    let priority_order = |p: &str| -> u8 {
        match p { "urgent" => 0, "high" => 1, "medium" => 2, "low" => 3, _ => 2 }
    };

    if let Some(queue) = tasks["queue"].as_array_mut() {
        // Find first pending task matching filter, sorted by priority
        let mut candidates: Vec<usize> = queue.iter().enumerate()
            .filter(|(_, t)| {
                t["status"].as_str() == Some("pending")
                    && project.map_or(true, |p| t["project"].as_str() == Some(p))
            })
            .map(|(i, _)| i)
            .collect();

        candidates.sort_by_key(|&i| {
            let p = queue[i]["priority"].as_str().unwrap_or("medium");
            (priority_order(p), queue[i]["added_at"].as_str().unwrap_or("").to_string())
        });

        if let Some(&idx) = candidates.first() {
            queue[idx]["status"] = json!("claimed");
            queue[idx]["claimed_by"] = json!(pane_id);
            queue[idx]["claimed_at"] = json!(now_iso());
            let task = queue[idx].clone();
            let _ = save_json("tasks.json", &tasks);
            return json!({"status": "claimed", "task": task});
        }
    }
    json!({"status": "empty"})
}

pub fn task_complete(task_id: &str, pane_id: &str, result: &str) -> Value {
    let mut tasks = load_json("tasks.json");
    if let Some(queue) = tasks["queue"].as_array_mut() {
        for task in queue.iter_mut() {
            if task["id"].as_str() == Some(task_id) {
                if task["claimed_by"].as_str() == Some(pane_id) {
                    task["status"] = json!("completed");
                    task["completed_at"] = json!(now_iso());
                    task["result"] = json!(result);
                    let _ = save_json("tasks.json", &tasks);
                    return json!({"status": "completed"});
                }
                return json!({"status": "not_owner"});
            }
        }
    }
    json!({"status": "not_found"})
}

pub fn task_list(status: Option<&str>, project: Option<&str>) -> Value {
    let tasks = load_json("tasks.json");
    let mut result = vec![];
    if let Some(queue) = tasks["queue"].as_array() {
        for t in queue {
            let matches_status = status.map_or(true, |s| s == "all" || t["status"].as_str() == Some(s));
            let matches_project = project.map_or(true, |p| t["project"].as_str() == Some(p));
            if matches_status && matches_project {
                result.push(t.clone());
            }
        }
    }
    json!({"tasks": result})
}

// ============================================================================
// KNOWLEDGE BASE
// ============================================================================

pub fn kb_add(pane_id: &str, project: &str, category: &str, title: &str, content: &str, files: &[String]) -> Value {
    let mut kb = load_json("knowledge.json");
    if kb.get("entries").is_none() { kb["entries"] = json!([]); }

    let entry_id = gen_short_id(title);
    if let Some(entries) = kb["entries"].as_array_mut() {
        entries.push(json!({
            "id": entry_id, "pane_id": pane_id, "project": project,
            "category": category, "title": title, "content": content,
            "files": files, "added_at": now_iso()
        }));
        if entries.len() > MAX_KB_ENTRIES {
            let drain = entries.len() - MAX_KB_ENTRIES;
            entries.drain(..drain);
        }
    }
    let _ = save_json("knowledge.json", &kb);
    json!({"status": "added", "entry_id": entry_id})
}

pub fn kb_search(query: &str, project: Option<&str>, category: Option<&str>) -> Value {
    let kb = load_json("knowledge.json");
    let query_lower = query.to_lowercase();
    let mut results = vec![];
    if let Some(entries) = kb["entries"].as_array() {
        for entry in entries {
            if let Some(p) = project {
                if entry["project"].as_str() != Some(p) { continue; }
            }
            if let Some(c) = category {
                if entry["category"].as_str() != Some(c) { continue; }
            }
            let title = entry["title"].as_str().unwrap_or("").to_lowercase();
            let content = entry["content"].as_str().unwrap_or("").to_lowercase();
            if title.contains(&query_lower) || content.contains(&query_lower) {
                results.push(entry.clone());
            }
        }
    }
    let len = results.len();
    let start = if len > 20 { len - 20 } else { 0 };
    json!({"results": &results[start..]})
}

pub fn kb_list(project: Option<&str>, limit: usize) -> Value {
    let kb = load_json("knowledge.json");
    let mut entries = vec![];
    if let Some(all) = kb["entries"].as_array() {
        for e in all {
            if project.map_or(true, |p| e["project"].as_str() == Some(p)) {
                entries.push(e.clone());
            }
        }
    }
    let len = entries.len();
    let start = if len > limit { len - limit } else { 0 };
    json!({"entries": &entries[start..]})
}

// ============================================================================
// MESSAGING
// ============================================================================

pub fn msg_broadcast(from_pane: &str, message: &str, priority: &str) -> Value {
    let mut msgs = load_json("messages.json");
    if msgs.get("messages").is_none() { msgs["messages"] = json!([]); }

    if let Some(arr) = msgs["messages"].as_array_mut() {
        arr.push(json!({
            "from": from_pane, "to": "all", "message": message,
            "priority": priority, "timestamp": now_iso(), "read_by": []
        }));
        if arr.len() > MAX_MESSAGES {
            let drain = arr.len() - MAX_MESSAGES;
            arr.drain(..drain);
        }
    }
    let _ = save_json("messages.json", &msgs);
    json!({"status": "sent"})
}

pub fn msg_send(from_pane: &str, to_pane: &str, message: &str) -> Value {
    let mut msgs = load_json("messages.json");
    if msgs.get("messages").is_none() { msgs["messages"] = json!([]); }

    if let Some(arr) = msgs["messages"].as_array_mut() {
        arr.push(json!({
            "from": from_pane, "to": to_pane, "message": message,
            "priority": "info", "timestamp": now_iso(), "read_by": []
        }));
        if arr.len() > MAX_MESSAGES {
            let drain = arr.len() - MAX_MESSAGES;
            arr.drain(..drain);
        }
    }
    let _ = save_json("messages.json", &msgs);
    json!({"status": "sent"})
}

pub fn msg_get(pane_id: &str, mark_read: bool) -> Value {
    let mut msgs = load_json("messages.json");
    let mut unread = vec![];

    if let Some(arr) = msgs["messages"].as_array_mut() {
        for m in arr.iter_mut() {
            let from = m["from"].as_str().unwrap_or("");
            let to = m["to"].as_str().unwrap_or("");
            if from == pane_id { continue; }
            if to != "all" && to != pane_id { continue; }

            let already_read = m["read_by"].as_array()
                .map_or(false, |rb| rb.iter().any(|r| r.as_str() == Some(pane_id)));
            if !already_read {
                unread.push(m.clone());
                if mark_read {
                    if let Some(rb) = m["read_by"].as_array_mut() {
                        rb.push(json!(pane_id));
                    }
                }
            }
        }
    }

    if mark_read { let _ = save_json("messages.json", &msgs); }
    json!({"messages": unread})
}

// ============================================================================
// CLEANUP
// ============================================================================

pub fn cleanup_all() -> Value {
    let mut cleaned = json!({"ports": 0, "agents": 0, "locks": 0, "branches": 0, "builds": 0});

    // Clean ports — collect removals first to avoid double mutable borrow
    let mut ports = load_json("ports.json");
    let mut port_keys_to_remove = vec![];
    let mut services_to_remove = vec![];
    if let Some(allocs) = ports["allocations"].as_object() {
        for (key, info) in allocs {
            let port: u16 = key.parse().unwrap_or(0);
            let (in_use, _) = is_port_in_use(port);
            let pane_id = info["pane_id"].as_str().unwrap_or("");
            if !in_use && !is_pane_active(pane_id) {
                port_keys_to_remove.push(key.clone());
                if let Some(service) = info["service"].as_str() {
                    services_to_remove.push(service.to_string());
                }
            }
        }
    }
    for key in &port_keys_to_remove {
        if let Some(allocs) = ports["allocations"].as_object_mut() { allocs.remove(key); }
    }
    for service in &services_to_remove {
        if let Some(svcs) = ports["services"].as_object_mut() { svcs.remove(service); }
    }
    cleaned["ports"] = json!(port_keys_to_remove.len());
    let _ = save_json("ports.json", &ports);

    // Clean agents + locks
    let mut agents = load_json("agents.json");
    if let Some(all) = agents["agents"].as_object_mut() {
        let keys: Vec<String> = all.keys().cloned().collect();
        for key in keys {
            if !is_pane_active(&key) {
                all.remove(&key);
                cleaned["agents"] = json!(cleaned["agents"].as_u64().unwrap_or(0) + 1);
            }
        }
    }
    if let Some(locks) = agents["locks"].as_object_mut() {
        let keys: Vec<String> = locks.keys().cloned().collect();
        for key in keys {
            let pane_id = locks[&key]["pane_id"].as_str().unwrap_or("");
            if !is_pane_active(pane_id) {
                locks.remove(&key);
                cleaned["locks"] = json!(cleaned["locks"].as_u64().unwrap_or(0) + 1);
            }
        }
    }
    let _ = save_json("agents.json", &agents);

    // Clean git branches
    let mut git = load_json("git.json");
    if let Some(branches) = git["branches"].as_object_mut() {
        let keys: Vec<String> = branches.keys().cloned().collect();
        for key in keys {
            let pane_id = branches[&key]["pane_id"].as_str().unwrap_or("");
            if !is_pane_active(pane_id) {
                branches.remove(&key);
                cleaned["branches"] = json!(cleaned["branches"].as_u64().unwrap_or(0) + 1);
            }
        }
    }
    let _ = save_json("git.json", &git);

    // Clean builds
    let mut builds = load_json("builds.json");
    if let Some(active) = builds["active"].as_object_mut() {
        let keys: Vec<String> = active.keys().cloned().collect();
        for key in keys {
            let pane_id = active[&key]["pane_id"].as_str().unwrap_or("");
            if !is_pane_active(pane_id) {
                active.remove(&key);
                cleaned["builds"] = json!(cleaned["builds"].as_u64().unwrap_or(0) + 1);
            }
        }
    }
    let _ = save_json("builds.json", &builds);

    json!({"cleaned": cleaned})
}

// ============================================================================
// STATUS OVERVIEW
// ============================================================================

pub fn status_overview(project: Option<&str>) -> Value {
    let ports = load_json("ports.json");
    let agents = load_json("agents.json");
    let builds = load_json("builds.json");
    let tasks = load_json("tasks.json");

    let mut port_list = vec![];
    if let Some(allocs) = ports["allocations"].as_object() {
        for (port_str, info) in allocs {
            let port: u16 = port_str.parse().unwrap_or(0);
            let (active, _) = is_port_in_use(port);
            port_list.push(json!({"port": port, "service": info["service"], "active": active}));
        }
    }

    let mut agent_list = vec![];
    if let Some(all) = agents["agents"].as_object() {
        for (pane_id, info) in all {
            if let Some(p) = project {
                if info["project"].as_str() != Some(p) { continue; }
            }
            let task_str = info["task"].as_str().unwrap_or("");
            let short_task: String = task_str.chars().take(50).collect();
            agent_list.push(json!({
                "pane": pane_id, "project": info["project"],
                "task": short_task, "active": is_pane_active(pane_id)
            }));
        }
    }

    let lock_count = agents["locks"].as_object().map_or(0, |l| l.len());
    let active_builds: Vec<&str> = builds["active"].as_object()
        .map_or(vec![], |a| a.keys().map(|s| s.as_str()).collect());
    let pending_tasks = tasks["queue"].as_array()
        .map_or(0, |q| q.iter().filter(|t| t["status"].as_str() == Some("pending")).count());

    json!({
        "ports": port_list, "agents": agent_list,
        "locks": lock_count, "active_builds": active_builds,
        "pending_tasks": pending_tasks
    })
}
