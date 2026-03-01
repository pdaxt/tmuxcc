use anyhow::Result;
use crate::config;
use crate::state::persistence::{read_json, write_json};

/// Set project-level MCPs in ~/.claude.json
pub fn set_project_mcps(project_path: &str, mcp_names: &[String]) -> Result<()> {
    let claude_json = config::claude_json_path();
    let mut config = read_json(&claude_json);

    let all_servers = config.get("mcpServers")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let mut proj_servers = serde_json::Map::new();
    for name in mcp_names {
        if let Some(server) = all_servers.get(name) {
            proj_servers.insert(name.clone(), server.clone());
        }
    }

    let root = match config.as_object_mut() {
        Some(obj) => obj,
        None => anyhow::bail!("claude.json is not a JSON object"),
    };

    let projects = root
        .entry("projects")
        .or_insert_with(|| serde_json::json!({}));

    let project_entry = match projects.as_object_mut() {
        Some(obj) => obj.entry(project_path).or_insert_with(|| serde_json::json!({})),
        None => anyhow::bail!("claude.json 'projects' is not an object"),
    };

    match project_entry.as_object_mut() {
        Some(obj) => { obj.insert("mcpServers".to_string(), serde_json::Value::Object(proj_servers)); }
        None => anyhow::bail!("claude.json project entry is not an object"),
    };

    write_json(&claude_json, &config)?;
    Ok(())
}

/// Read the claude.json config
pub fn read_claude_config() -> serde_json::Value {
    read_json(&config::claude_json_path())
}

/// Write preamble file for a pane
pub fn write_preamble(pane: u8, content: &str) -> Result<String> {
    let dir = config::preamble_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("pane_{}.md", pane));
    std::fs::write(&path, content)?;
    Ok(path.to_string_lossy().to_string())
}

/// Generate a preamble for an agent
pub fn generate_preamble(
    pane: u8,
    theme: &str,
    project: &str,
    role: &str,
    task: &str,
    prompt: &str,
) -> String {
    let role_short = config::role_short(role);

    // Split prompt into handoff context and regular context
    let (regular_prompt, handoff_section) = if prompt.contains("## Predecessor Results") {
        let parts: Vec<&str> = prompt.splitn(2, "## Predecessor Results").collect();
        (parts[0].trim().to_string(), Some(parts.get(1).unwrap_or(&"").trim().to_string()))
    } else {
        (prompt.to_string(), None)
    };

    let extra = if regular_prompt.is_empty() {
        String::new()
    } else {
        format!("Additional context: {}\n\n", regular_prompt)
    };

    let handoff = match handoff_section {
        Some(ref ctx) if !ctx.is_empty() => format!(
            "## Predecessor Results\nThese tasks completed before yours. Use their output as context:\n{}\n\n",
            ctx
        ),
        _ => String::new(),
    };

    format!(
        "# TASK: {task}\n\
         **Role:** {role_short} | **Project:** {project} | **Pane:** {pane} ({theme})\n\
         \n\
         ## Role Instructions\n\
         You are the {role} agent. Focus on your assigned task.\n\
         \n\
         ## Task Details\n\
         {task}\n\
         {extra}\
         {handoff}\
         ## Coordination\n\
         - Use multi_agent MCP to register and coordinate with other agents\n\
         - Lock files before editing shared code\n\
         - When done: summarize what you accomplished\n",
    )
}

/// Get the account config dir for a pane — distributes panes across all available accounts.
/// Discovers ~/.claude-acc1, ~/.claude-acc2, ... ~/.claude-accN automatically.
/// Panes are assigned round-robin: pane 1 → acc1, pane 2 → acc2, ..., pane N+1 → acc1.
/// Falls back to ~/.claude if no account dirs exist.
pub fn account_config_dir(pane: u8) -> String {
    let home = config::home_dir();
    let accounts = discover_account_dirs(&home);
    if accounts.is_empty() {
        return home.join(".claude").to_string_lossy().to_string();
    }
    let idx = ((pane as usize).wrapping_sub(1)) % accounts.len();
    accounts[idx].to_string_lossy().to_string()
}

/// Discover all ~/.claude-accN directories, sorted by N
fn discover_account_dirs(home: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut dirs: Vec<(u32, std::path::PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(home) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(num_str) = name.strip_prefix(".claude-acc") {
                if let Ok(n) = num_str.parse::<u32>() {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs.push((n, path));
                    }
                }
            }
        }
    }
    dirs.sort_by_key(|(n, _)| *n);
    dirs.into_iter().map(|(_, p)| p).collect()
}

/// Check if preamble exists
pub fn preamble_exists(pane: u8) -> bool {
    let path = config::preamble_dir().join(format!("pane_{}.md", pane));
    path.exists()
}

