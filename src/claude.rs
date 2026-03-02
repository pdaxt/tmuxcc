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

    // Parse prompt sections: split on known headers
    let mut regular_prompt = String::new();
    let mut handoff_section = String::new();
    let mut gate_section = String::new();
    let mut coord_section = String::new();

    let mut current_target = &mut regular_prompt;
    for line in prompt.lines() {
        if line.starts_with("## Predecessor Results") {
            current_target = &mut handoff_section;
            continue;
        } else if line.starts_with("## Quality Gate Results") {
            current_target = &mut gate_section;
            continue;
        } else if line.starts_with("## Pipeline Coordination") {
            current_target = &mut coord_section;
            continue;
        }
        current_target.push_str(line);
        current_target.push('\n');
    }

    let extra = {
        let trimmed = regular_prompt.trim();
        if trimmed.is_empty() { String::new() }
        else { format!("{}\n\n", trimmed) }
    };

    let handoff = {
        let trimmed = handoff_section.trim();
        if trimmed.is_empty() { String::new() }
        else { format!("## Predecessor Results\nThese tasks completed before yours. Use their output as context:\n{}\n\n", trimmed) }
    };

    let gate = {
        let trimmed = gate_section.trim();
        if trimmed.is_empty() { String::new() }
        else { format!("## Quality Gate Results\n{}\n\n", trimmed) }
    };

    let coord = {
        let trimmed = coord_section.trim();
        if trimmed.is_empty() {
            "## Coordination\n\
             - Use multi_agent MCP to register and coordinate with other agents\n\
             - Lock files before editing shared code\n\
             - When done: summarize what you accomplished\n".to_string()
        } else {
            format!("{}\n", trimmed)
        }
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
         {gate}\
         {coord}\n",
    )
}

/// Get the account config dir for a pane — distributes panes across all available accounts.
/// Check if preamble exists
pub fn preamble_exists(pane: u8) -> bool {
    let path = config::preamble_dir().join(format!("pane_{}.md", pane));
    path.exists()
}

