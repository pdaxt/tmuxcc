use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use agentos_types::{MCPCallResult, MCPDescriptor, MCPStatus};
use rmcp::model::{CallToolRequestParams, Tool};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use serde_json::Value;
use tokio::process::Command;
use tokio::sync::Mutex;

/// A running micro MCP child process
struct RunningMCP {
    service: RunningService<RoleClient, ()>,
    tools: Vec<Tool>,
    started_at: Instant,
    last_used: Mutex<Instant>,
    _pid: Option<u32>,
}

/// MCP Gateway: spawn, route, cache, and garbage-collect micro MCPs
pub struct MCPRegistry {
    descriptors: HashMap<String, MCPDescriptor>,
    running: HashMap<String, RunningMCP>,
    descriptors_dir: PathBuf,
}

impl MCPRegistry {
    /// Create a new registry, loading descriptors from the given directory
    pub fn new(descriptors_dir: PathBuf) -> Self {
        let mut registry = Self {
            descriptors: HashMap::new(),
            running: HashMap::new(),
            descriptors_dir,
        };
        registry.load_descriptors();
        registry
    }

    /// Load/reload MCP descriptors from TOML files in the descriptors directory
    pub fn load_descriptors(&mut self) {
        if !self.descriptors_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&self.descriptors_dir) {
                tracing::warn!("Failed to create MCP descriptors dir: {}", e);
                return;
            }
        }

        let entries = match std::fs::read_dir(&self.descriptors_dir) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::warn!("Failed to read MCP descriptors dir: {}", e);
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "toml") {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match toml::from_str::<MCPDescriptor>(&content) {
                        Ok(desc) => {
                            tracing::info!("Loaded MCP descriptor: {} ({} capabilities)",
                                desc.name, desc.capabilities.len());
                            self.descriptors.insert(desc.name.clone(), desc);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse MCP descriptor {:?}: {}", path, e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to read MCP descriptor {:?}: {}", path, e);
                    }
                }
            }
        }
    }

    /// Register a descriptor programmatically (without a TOML file)
    pub fn register(&mut self, descriptor: MCPDescriptor) {
        self.descriptors.insert(descriptor.name.clone(), descriptor);
    }

    /// Find MCPs matching a capability keyword
    pub fn discover(&self, capability: &str) -> Vec<&MCPDescriptor> {
        let cap_lower = capability.to_lowercase();
        self.descriptors
            .values()
            .filter(|d| {
                d.capabilities.iter().any(|c| c.to_lowercase().contains(&cap_lower))
                    || d.name.to_lowercase().contains(&cap_lower)
                    || d.description.to_lowercase().contains(&cap_lower)
            })
            .collect()
    }

    /// Ensure a micro MCP is running, spawning it if necessary
    pub async fn ensure_running(&mut self, name: &str) -> anyhow::Result<()> {
        if self.running.contains_key(name) {
            return Ok(());
        }

        let descriptor = self
            .descriptors
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("No MCP descriptor found for '{}'", name))?
            .clone();

        tracing::info!("Spawning micro MCP: {} (command: {:?})", name, descriptor.command);

        if descriptor.command.is_empty() {
            return Err(anyhow::anyhow!("MCP '{}' has empty command", name));
        }

        let mut cmd = Command::new(&descriptor.command[0]);
        if descriptor.command.len() > 1 {
            cmd.args(&descriptor.command[1..]);
        }
        for (k, v) in &descriptor.env {
            cmd.env(k, v);
        }

        let transport = TokioChildProcess::new(cmd)?;
        let pid = transport.id();
        let service: RunningService<RoleClient, ()> = ().serve(transport).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to MCP '{}': {}", name, e))?;

        // List available tools
        let tools_result = service.peer().list_tools(None).await
            .map_err(|e| anyhow::anyhow!("Failed to list tools from '{}': {}", name, e))?;

        let tool_count = tools_result.tools.len();
        let tool_names: Vec<String> = tools_result.tools.iter().map(|t| t.name.to_string()).collect();
        tracing::info!("MCP '{}' connected: {} tools ({:?})", name, tool_count, tool_names);

        let now = Instant::now();
        self.running.insert(name.to_string(), RunningMCP {
            service,
            tools: tools_result.tools,
            started_at: now,
            last_used: Mutex::new(now),
            _pid: pid,
        });

        Ok(())
    }

    /// Call a tool on a running micro MCP
    pub async fn call(
        &self,
        mcp_name: &str,
        tool_name: &str,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> anyhow::Result<MCPCallResult> {
        let mcp = self
            .running
            .get(mcp_name)
            .ok_or_else(|| anyhow::anyhow!("MCP '{}' is not running", mcp_name))?;

        // Update last_used
        *mcp.last_used.lock().await = Instant::now();

        let result = mcp
            .service
            .peer()
            .call_tool(CallToolRequestParams {
                name: tool_name.to_string().into(),
                arguments,
                meta: None,
                task: None,
            })
            .await;

        match result {
            Ok(call_result) => {
                let content_text = call_result
                    .content
                    .iter()
                    .filter_map(|c| {
                        if let Some(text) = c.raw.as_text() {
                            Some(text.text.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                let is_error = call_result.is_error.unwrap_or(false);

                Ok(MCPCallResult {
                    mcp: mcp_name.to_string(),
                    tool: tool_name.to_string(),
                    success: !is_error,
                    content: serde_json::Value::String(content_text),
                    error: if is_error {
                        Some("Tool returned error".to_string())
                    } else {
                        None
                    },
                })
            }
            Err(e) => Ok(MCPCallResult {
                mcp: mcp_name.to_string(),
                tool: tool_name.to_string(),
                success: false,
                content: Value::Null,
                error: Some(format!("{}", e)),
            }),
        }
    }

    /// List status of all running MCPs
    pub async fn list_running(&self) -> Vec<MCPStatus> {
        let mut statuses = Vec::new();
        let now = Instant::now();

        for (name, mcp) in &self.running {
            let last_used = *mcp.last_used.lock().await;
            statuses.push(MCPStatus {
                name: name.clone(),
                running: true,
                tool_count: mcp.tools.len(),
                tools: mcp.tools.iter().map(|t| t.name.to_string()).collect(),
                uptime_secs: now.duration_since(mcp.started_at).as_secs(),
                last_used_secs_ago: now.duration_since(last_used).as_secs(),
            });
        }

        statuses
    }

    /// List all registered descriptors (running or not)
    pub fn list_all(&self) -> Vec<(&str, bool)> {
        self.descriptors
            .keys()
            .map(|name| (name.as_str(), self.running.contains_key(name)))
            .collect()
    }

    /// Shutdown MCPs idle for longer than max_idle
    pub async fn gc_idle(&mut self, max_idle: Duration) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for (name, mcp) in &self.running {
            let last_used = *mcp.last_used.lock().await;
            if now.duration_since(last_used) > max_idle {
                tracing::info!("GC: shutting down idle MCP '{}' (idle {}s)",
                    name, now.duration_since(last_used).as_secs());
                to_remove.push(name.clone());
            }
        }

        for name in to_remove {
            self.running.remove(&name);
        }
    }

    /// Shutdown a specific MCP
    pub fn shutdown(&mut self, name: &str) -> bool {
        self.running.remove(name).is_some()
    }

    /// Shutdown all running MCPs
    pub fn shutdown_all(&mut self) {
        self.running.clear();
    }

    /// Get tool list for a specific running MCP
    pub fn get_tools(&self, name: &str) -> Option<&[Tool]> {
        self.running.get(name).map(|mcp| mcp.tools.as_slice())
    }

    /// Number of running MCPs
    pub fn running_count(&self) -> usize {
        self.running.len()
    }

    /// Number of registered descriptors
    pub fn descriptor_count(&self) -> usize {
        self.descriptors.len()
    }
}

/// Save an MCP descriptor as a TOML file
pub fn save_descriptor(dir: &std::path::Path, desc: &MCPDescriptor) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{}.toml", desc.name));
    let content = toml::to_string_pretty(desc)?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptor_loading() {
        let dir = std::env::temp_dir().join("agentos_test_mcps");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let desc = MCPDescriptor {
            name: "test_mcp".to_string(),
            command: vec!["echo".to_string(), "hello".to_string()],
            capabilities: vec!["testing".to_string(), "echo".to_string()],
            auto_start: false,
            env: HashMap::new(),
            description: "Test MCP for unit tests".to_string(),
        };
        save_descriptor(&dir, &desc).unwrap();

        let registry = MCPRegistry::new(dir.clone());
        assert_eq!(registry.descriptor_count(), 1);

        let found = registry.discover("testing");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "test_mcp");

        let found_by_name = registry.discover("test");
        assert_eq!(found_by_name.len(), 1);

        let not_found = registry.discover("nonexistent");
        assert!(not_found.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_register_programmatic() {
        let dir = std::env::temp_dir().join("agentos_test_mcps_reg");
        let _ = std::fs::remove_dir_all(&dir);

        let mut registry = MCPRegistry::new(dir.clone());
        assert_eq!(registry.descriptor_count(), 0);

        registry.register(MCPDescriptor {
            name: "custom".to_string(),
            command: vec!["my_mcp".to_string()],
            capabilities: vec!["custom_cap".to_string()],
            auto_start: false,
            env: HashMap::new(),
            description: String::new(),
        });

        assert_eq!(registry.descriptor_count(), 1);
        assert!(!registry.discover("custom_cap").is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
