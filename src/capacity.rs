use chrono::Local;
use serde_json::{json, Value};

use crate::config;
use crate::state::persistence::{read_json, write_json};

// === CONSTANTS ===

const TYPE_BASE_ACU: &[(&str, f64)] = &[
    ("bug", 0.5), ("task", 1.0), ("feature", 2.0), ("improvement", 1.5), ("epic", 8.0),
];
const COMPLEXITY_MULT: &[(&str, f64)] = &[
    ("low", 0.5), ("medium", 1.0), ("high", 2.0), ("very_high", 4.0),
];

fn now_iso() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn ensure_dirs() {
    let _ = std::fs::create_dir_all(config::capacity_root());
    let _ = std::fs::create_dir_all(config::capacity_root().join("sprints"));
}

fn load_config() -> Value {
    ensure_dirs();
    let path = config::capacity_root().join("config.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<Value>(&content) {
                return v;
            }
        }
    }
    let default = json!({
        "pane_count": 9, "hours_per_day": 8, "availability_factor": 0.8,
        "review_bandwidth": 12, "build_slots": 2,
        "roles": {
            "pm":        {"name": "Product Manager",  "typical_acu": 0.5, "review_pct": 90, "parallelizable": false},
            "architect": {"name": "System Architect",  "typical_acu": 1.0, "review_pct": 80, "parallelizable": false},
            "developer": {"name": "Developer",         "typical_acu": 2.0, "review_pct": 50, "parallelizable": true},
            "qa":        {"name": "QA Engineer",        "typical_acu": 1.0, "review_pct": 30, "parallelizable": true},
            "devops":    {"name": "DevOps Engineer",    "typical_acu": 0.5, "review_pct": 60, "parallelizable": false},
        },
    });
    let _ = std::fs::write(&path, serde_json::to_string_pretty(&default).unwrap_or_default());
    default
}

fn save_config(cfg: &Value) {
    ensure_dirs();
    let path = config::capacity_root().join("config.json");
    let _ = std::fs::write(&path, serde_json::to_string_pretty(cfg).unwrap_or_default());
}

fn load_work_log() -> Value {
    ensure_dirs();
    let path = config::capacity_root().join("work_log.json");
    read_json(&path)
}

fn save_work_log(log: &Value) {
    ensure_dirs();
    let path = config::capacity_root().join("work_log.json");
    let _ = write_json(&path, log);
}

fn daily_acu(cfg: &Value) -> f64 {
    let panes = cfg["pane_count"].as_f64().unwrap_or(9.0);
    let hours = cfg["hours_per_day"].as_f64().unwrap_or(8.0);
    let factor = cfg["availability_factor"].as_f64().unwrap_or(0.8);
    panes * hours * factor
}

fn load_sprint(sprint_id: &str) -> Option<Value> {
    let path = config::capacity_root().join("sprints").join(format!("{}.json", sprint_id));
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            return serde_json::from_str(&content).ok();
        }
    }
    None
}

fn save_sprint(sprint: &Value) {
    ensure_dirs();
    if let Some(id) = sprint["id"].as_str() {
        let path = config::capacity_root().join("sprints").join(format!("{}.json", id));
        let _ = std::fs::write(&path, serde_json::to_string_pretty(sprint).unwrap_or_default());
    }
}

fn list_sprints() -> Vec<Value> {
    let dir = config::capacity_root().join("sprints");
    let mut sprints = vec![];
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(v) = serde_json::from_str::<Value>(&content) {
                        sprints.push(v);
                    }
                }
            }
        }
    }
    sprints.sort_by(|a, b| {
        let sa = a["start_date"].as_str().unwrap_or("");
        let sb = b["start_date"].as_str().unwrap_or("");
        sa.cmp(sb)
    });
    sprints
}

// === EXISTING PUBLIC API (used by tools.rs) ===

pub struct CapacityData {
    pub acu_used: f64,
    pub acu_total: f64,
    pub reviews_used: usize,
    pub reviews_total: usize,
}

pub fn load_capacity() -> CapacityData {
    let cfg = load_config();
    let daily = daily_acu(&cfg);
    let review_bw = cfg["review_bandwidth"].as_u64().unwrap_or(12) as usize;
    let today = Local::now().format("%Y-%m-%d").to_string();

    let log = load_work_log();
    let entries = log["entries"].as_array().cloned().unwrap_or_default();
    let today_entries: Vec<_> = entries.iter().filter(|e| {
        e["logged_at"].as_str().map_or(false, |s| s.starts_with(&today))
    }).collect();

    let acu_used: f64 = today_entries.iter()
        .filter_map(|e| e["acu_spent"].as_f64())
        .sum();
    let reviews_used = today_entries.iter()
        .filter(|e| e["review_needed"].as_bool().unwrap_or(false))
        .count();

    CapacityData {
        acu_used: (acu_used * 10.0).round() / 10.0,
        acu_total: (daily * 10.0).round() / 10.0,
        reviews_used,
        reviews_total: review_bw,
    }
}

pub fn log_work_entry(entry: Value) -> anyhow::Result<()> {
    let path = config::capacity_root().join("work_log.json");
    let mut log = read_json(&path);
    let root = log.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("work_log.json is not an object"))?;
    let entries = root
        .entry("entries")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| anyhow::anyhow!("work_log entries is not an array"))?;
    entries.push(entry);
    // Keep last 500
    if entries.len() > 500 {
        let drain = entries.len() - 500;
        entries.drain(..drain);
    }
    write_json(&path, &log)?;
    Ok(())
}

// === NEW MCP TOOL FUNCTIONS ===

pub fn cap_configure(
    pane_count: Option<u32>, hours_per_day: Option<f64>,
    availability_factor: Option<f64>, review_bandwidth: Option<u32>, build_slots: Option<u32>,
) -> Value {
    let mut cfg = load_config();
    if let Some(v) = pane_count { if v > 0 { cfg["pane_count"] = json!(v); } }
    if let Some(v) = hours_per_day { if v > 0.0 { cfg["hours_per_day"] = json!(v); } }
    if let Some(v) = availability_factor { if v > 0.0 && v <= 1.0 { cfg["availability_factor"] = json!(v); } }
    if let Some(v) = review_bandwidth { if v > 0 { cfg["review_bandwidth"] = json!(v); } }
    if let Some(v) = build_slots { if v > 0 { cfg["build_slots"] = json!(v); } }
    cfg["updated_at"] = json!(now_iso());
    save_config(&cfg);

    let daily = daily_acu(&cfg);
    let rb = cfg["review_bandwidth"].as_f64().unwrap_or(12.0);
    json!({
        "config": {"pane_count": cfg["pane_count"], "hours_per_day": cfg["hours_per_day"],
            "availability_factor": cfg["availability_factor"], "review_bandwidth": cfg["review_bandwidth"],
            "build_slots": cfg["build_slots"]},
        "derived": {"daily_acu": (daily * 10.0).round() / 10.0, "weekly_acu": (daily * 50.0).round() / 10.0,
            "bottleneck": if rb < daily { "review" } else { "compute" }},
    })
}

pub fn cap_estimate(description: &str, complexity: &str, task_type: &str, role: &str) -> Value {
    let cfg = load_config();
    let base = TYPE_BASE_ACU.iter().find(|(t, _)| *t == task_type).map(|(_, v)| *v).unwrap_or(1.0);
    let mult = COMPLEXITY_MULT.iter().find(|(c, _)| *c == complexity).map(|(_, v)| *v).unwrap_or(1.0);
    let estimated = (base * mult * 100.0).round() / 100.0;

    let role_info = &cfg["roles"][role];
    let review_pct = role_info["review_pct"].as_u64().unwrap_or(50);
    let needs_review = review_pct > 50;
    let mut review_gates: u32 = if needs_review { 1 } else { 0 };
    if task_type == "feature" || task_type == "epic" { review_gates += 1; }
    let parallelizable = role_info["parallelizable"].as_bool().unwrap_or(true);
    let panes = cfg["pane_count"].as_f64().unwrap_or(9.0);
    let wall_mins = (estimated * 60.0 / if parallelizable { panes } else { 1.0 }) as u32;

    json!({
        "estimated_acu": estimated, "review_gates": review_gates,
        "parallelizable": parallelizable, "role": role,
        "wall_clock_estimate": format!("{}min", wall_mins),
        "description": description,
        "breakdown": {"type_base": base, "complexity_multiplier": mult},
    })
}

pub fn cap_log_work_full(
    issue_id: &str, space: &str, role: &str, pane_id: &str,
    acu_spent: f64, review_needed: bool, notes: &str,
) -> Value {
    let entry = json!({
        "id": format!("wl_{}", Local::now().format("%Y%m%d_%H%M%S")),
        "issue_id": issue_id, "space": space, "role": role, "pane_id": pane_id,
        "acu_spent": acu_spent, "review_needed": review_needed,
        "review_completed": false, "notes": notes, "logged_at": now_iso(),
    });

    let mut log = load_work_log();
    if let Some(entries) = log["entries"].as_array_mut() {
        entries.push(entry.clone());
        if entries.len() > 500 { let drain = entries.len() - 500; entries.drain(..drain); }
    } else {
        log["entries"] = json!([entry]);
    }
    save_work_log(&log);

    json!({"logged": entry["id"], "issue": issue_id, "acu_spent": acu_spent})
}

pub fn cap_plan_sprint(space: &str, name: &str, start_date: &str, days: u32, issue_ids: &str) -> Value {
    let cfg = load_config();
    let daily = daily_acu(&cfg);
    let start = if start_date.is_empty() { Local::now().format("%Y-%m-%d").to_string() } else { start_date.into() };
    let sprint_name = if name.is_empty() {
        let week = Local::now().format("%W").to_string();
        format!("Sprint W{}", week)
    } else { name.into() };
    let sprint_id = sprint_name.to_lowercase().replace(' ', "-");

    let ids: Vec<&str> = issue_ids.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    let mut planned = vec![];
    let mut total_acu = 0.0;
    let mut total_reviews: u32 = 0;

    for iid in &ids {
        if let Some(issue) = crate::tracker::find_issue(space, iid) {
            let est = issue["estimated_acu"].as_f64().unwrap_or_else(|| {
                TYPE_BASE_ACU.iter().find(|(t, _)| issue["type"].as_str() == Some(t)).map(|(_, v)| *v).unwrap_or(1.0)
            });
            let rg = issue["review_gates"].as_u64().unwrap_or(1) as u32;
            planned.push(json!({
                "issue_id": issue["id"], "title": issue["title"],
                "estimated_acu": est, "role": issue["role"],
                "review_gates": rg, "priority": issue["priority"],
            }));
            total_acu += est;
            total_reviews += rg;
        }
    }

    let sprint_compute = (daily * days as f64 * 10.0).round() / 10.0;
    let sprint_reviews = cfg["review_bandwidth"].as_u64().unwrap_or(12) as u32 * days;
    let compute_util = if sprint_compute > 0.0 { total_acu / sprint_compute } else { 0.0 };
    let review_util = if sprint_reviews > 0 { total_reviews as f64 / sprint_reviews as f64 } else { 0.0 };
    let bottleneck = if review_util > compute_util { "review" } else if compute_util > 0.9 { "compute" } else { "balanced" };
    let capacity_status = if total_acu > sprint_compute { "over_capacity" }
        else if total_acu > sprint_compute * 0.8 { "near_capacity" }
        else { "within_capacity" };

    let sprint = json!({
        "id": sprint_id, "name": sprint_name, "space": space,
        "start_date": start, "days": days,
        "capacity": {"total_acu": sprint_compute, "review_slots": sprint_reviews, "daily_acu": (daily * 10.0).round() / 10.0},
        "planned": {"issues": planned, "total_acu": (total_acu * 100.0).round() / 100.0, "total_reviews": total_reviews},
        "actual": {"acu_spent": 0.0, "reviews_consumed": 0, "issues_completed": 0},
        "analysis": {"bottleneck": bottleneck, "capacity_status": capacity_status,
            "compute_utilization": format!("{:.0}%", compute_util * 100.0),
            "review_utilization": format!("{:.0}%", review_util * 100.0)},
        "status": "active", "created_at": now_iso(),
    });
    save_sprint(&sprint);
    sprint
}

pub fn cap_dashboard(space: &str, sprint_id: &str) -> Value {
    let cfg = load_config();
    let daily = daily_acu(&cfg);
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log = load_work_log();
    let entries = log["entries"].as_array().cloned().unwrap_or_default();

    let today_entries: Vec<&Value> = entries.iter().filter(|e| {
        e["logged_at"].as_str().map_or(false, |s| s.starts_with(&today))
            && (space.is_empty() || e["space"].as_str() == Some(space))
    }).collect();

    let today_acu: f64 = today_entries.iter().filter_map(|e| e["acu_spent"].as_f64()).sum();
    let today_reviews = today_entries.iter().filter(|e| e["review_needed"].as_bool().unwrap_or(false)).count();
    let review_cap = cfg["review_bandwidth"].as_u64().unwrap_or(12) as usize;
    let bottleneck = if today_reviews >= (review_cap * 8 / 10) { "review" }
        else if today_acu >= daily * 0.9 { "compute" } else { "balanced" };

    let mut result = json!({
        "today": {
            "acu_used": (today_acu * 100.0).round() / 100.0,
            "acu_available": (daily * 10.0).round() / 10.0,
            "acu_pct": if daily > 0.0 { (today_acu / daily * 100.0) as u32 } else { 0 },
            "reviews_pending": today_reviews,
            "reviews_capacity": review_cap,
            "bottleneck": bottleneck,
        },
    });

    // Sprint info
    let sprint = if !sprint_id.is_empty() {
        load_sprint(sprint_id)
    } else {
        list_sprints().into_iter().rev().find(|s| s["status"].as_str() == Some("active"))
    };
    if let Some(sp) = sprint {
        let planned_acu = sp["planned"]["total_acu"].as_f64().unwrap_or(0.0);
        let sprint_entries: Vec<&Value> = entries.iter().filter(|e| {
            e["logged_at"].as_str().map_or(false, |s| s >= sp["start_date"].as_str().unwrap_or(""))
        }).collect();
        let sprint_acu: f64 = sprint_entries.iter().filter_map(|e| e["acu_spent"].as_f64()).sum();
        result["sprint"] = json!({
            "id": sp["id"], "name": sp["name"],
            "acu_planned": (planned_acu * 100.0).round() / 100.0,
            "acu_spent": (sprint_acu * 100.0).round() / 100.0,
            "acu_remaining": ((planned_acu - sprint_acu).max(0.0) * 100.0).round() / 100.0,
            "progress_pct": if planned_acu > 0.0 { (sprint_acu / planned_acu * 100.0) as u32 } else { 0 },
            "issues_count": sp["planned"]["issues"].as_array().map_or(0, |a| a.len()),
        });
    }
    result
}

pub fn cap_burndown(sprint_id: &str) -> Value {
    let sprint = if !sprint_id.is_empty() {
        load_sprint(sprint_id)
    } else {
        list_sprints().into_iter().rev().find(|s| s["status"].as_str() == Some("active"))
    };
    let sprint = match sprint {
        Some(s) => s,
        None => return json!({"error": "No active sprint found"}),
    };

    let planned_acu = sprint["planned"]["total_acu"].as_f64().unwrap_or(0.0);
    let days = sprint["days"].as_u64().unwrap_or(5) as usize;
    let daily_burn = if days > 0 { planned_acu / days as f64 } else { 0.0 };

    let ideal: Vec<Value> = (0..=days).map(|d| {
        json!({"day": d, "remaining": ((planned_acu - daily_burn * d as f64) * 100.0).round() / 100.0})
    }).collect();

    let log = load_work_log();
    let entries = log["entries"].as_array().cloned().unwrap_or_default();
    let start_date = sprint["start_date"].as_str().unwrap_or("");
    let space = sprint["space"].as_str().unwrap_or("");

    let mut actual = vec![];
    let mut cumulative = 0.0;
    let _today = Local::now().format("%Y-%m-%d").to_string();
    for d in 0..=days {
        // Calculate date for day d (approximate — just offset start_date string)
        let day_entries: Vec<&Value> = entries.iter().filter(|e| {
            let logged = e["logged_at"].as_str().unwrap_or("");
            logged >= start_date && (space.is_empty() || e["space"].as_str() == Some(space))
        }).collect();
        // Simplified: just use cumulative from all entries up to now
        if d == 0 { continue; }
        let day_acu: f64 = day_entries.iter().filter_map(|e| e["acu_spent"].as_f64()).sum();
        cumulative = day_acu; // Use total for now
        actual.push(json!({"day": d, "remaining": ((planned_acu - cumulative).max(0.0) * 100.0).round() / 100.0}));
        break; // Only one data point for current day
    }

    let burn_rate = if !actual.is_empty() { cumulative } else { 0.0 };
    json!({
        "sprint": sprint["id"], "planned_acu": planned_acu,
        "ideal": ideal, "actual": actual,
        "projection": {"burn_rate_per_day": (burn_rate * 100.0).round() / 100.0},
    })
}

pub fn cap_velocity(space: &str, count: usize) -> Value {
    let all_sprints = list_sprints();
    let filtered: Vec<&Value> = all_sprints.iter()
        .filter(|s| space.is_empty() || s["space"].as_str() == Some(space))
        .collect();
    let recent = if filtered.len() > count { &filtered[filtered.len() - count..] } else { &filtered };

    if recent.is_empty() {
        return json!({"error": "No sprints found", "sprints_analyzed": 0});
    }

    let log = load_work_log();
    let entries = log["entries"].as_array().cloned().unwrap_or_default();
    let mut velocity_data = vec![];

    for sp in recent {
        let start = sp["start_date"].as_str().unwrap_or("");
        let planned_acu = sp["planned"]["total_acu"].as_f64().unwrap_or(0.0);
        let sprint_acu: f64 = entries.iter()
            .filter(|e| e["logged_at"].as_str().map_or(false, |s| s >= start)
                && (space.is_empty() || e["space"].as_str() == Some(space)))
            .filter_map(|e| e["acu_spent"].as_f64())
            .sum();
        let accuracy = if planned_acu > 0.0 { (sprint_acu / planned_acu * 100.0).round() / 100.0 } else { 0.0 };
        velocity_data.push(json!({
            "sprint": sp["id"], "planned_acu": (planned_acu * 100.0).round() / 100.0,
            "actual_acu": (sprint_acu * 100.0).round() / 100.0, "accuracy": accuracy,
        }));
    }

    let avg_acu = velocity_data.iter().filter_map(|v| v["actual_acu"].as_f64()).sum::<f64>()
        / velocity_data.len().max(1) as f64;
    json!({
        "sprints_analyzed": velocity_data.len(), "velocity": velocity_data,
        "summary": {"avg_acu_per_sprint": (avg_acu * 100.0).round() / 100.0},
    })
}

pub fn cap_roles() -> Value {
    let cfg = load_config();
    let log = load_work_log();
    let today = Local::now().format("%Y-%m-%d").to_string();
    let entries = log["entries"].as_array().cloned().unwrap_or_default();

    let mut roles = serde_json::Map::new();
    if let Some(role_defs) = cfg["roles"].as_object() {
        for (key, info) in role_defs {
            let today_entries: Vec<&Value> = entries.iter().filter(|e| {
                e["role"].as_str() == Some(key) && e["logged_at"].as_str().map_or(false, |s| s.starts_with(&today))
            }).collect();
            let today_acu: f64 = today_entries.iter().filter_map(|e| e["acu_spent"].as_f64()).sum();
            roles.insert(key.clone(), json!({
                "name": info["name"], "typical_acu_per_task": info["typical_acu"],
                "review_pct": info["review_pct"], "parallelizable": info["parallelizable"],
                "today_acu": (today_acu * 100.0).round() / 100.0, "today_tasks": today_entries.len(),
            }));
        }
    }
    json!({"roles": roles})
}
