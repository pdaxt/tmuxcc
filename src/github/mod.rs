//! GitHub integration — show PR status, commits, branch info per agent pane.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::Result;
// serde::Deserialize available if needed for JSON parsing

/// Git info for a single repo/pane
#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub branch: String,
    pub dirty_files: u32,
    pub ahead: u32,
    pub behind: u32,
    pub last_commit_msg: String,
    pub last_commit_age: String,
    /// Open PR for current branch (if any)
    pub pr: Option<PrInfo>,
    pub last_fetched: Option<Instant>,
}

/// Pull request info
#[derive(Debug, Clone, Default)]
pub struct PrInfo {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub checks_passing: Option<bool>,
    pub review_status: String,
    pub url: String,
}

/// GitHub tracker — caches git/PR info per project path
#[derive(Debug)]
pub struct GitHubTracker {
    cache: HashMap<String, GitInfo>,
    cache_ttl: Duration,
}

impl GitHubTracker {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(30),
        }
    }

    /// Get git info for a path (cached)
    pub fn get_info(&mut self, project_path: &str) -> &GitInfo {
        let needs_refresh = self
            .cache
            .get(project_path)
            .map(|info| {
                info.last_fetched
                    .map(|t| t.elapsed() > self.cache_ttl)
                    .unwrap_or(true)
            })
            .unwrap_or(true);

        if needs_refresh {
            let info = fetch_git_info(project_path);
            self.cache.insert(project_path.to_string(), info);
        }

        self.cache.get(project_path).unwrap()
    }

    /// Force refresh for a path
    pub fn refresh(&mut self, project_path: &str) {
        let info = fetch_git_info(project_path);
        self.cache.insert(project_path.to_string(), info);
    }

    /// Get cached info without refreshing
    pub fn cached(&self, project_path: &str) -> Option<&GitInfo> {
        self.cache.get(project_path)
    }

    /// Refresh all cached paths
    pub fn refresh_all(&mut self) {
        let paths: Vec<String> = self.cache.keys().cloned().collect();
        for path in paths {
            let info = fetch_git_info(&path);
            self.cache.insert(path, info);
        }
    }
}

impl Default for GitHubTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Fetch git info synchronously (fast local git commands)
fn fetch_git_info(project_path: &str) -> GitInfo {
    let path = Path::new(project_path);
    if !path.join(".git").exists() && !path.join(".git").is_file() {
        return GitInfo::default();
    }

    let mut info = GitInfo {
        last_fetched: Some(Instant::now()),
        ..Default::default()
    };

    // Branch name
    if let Ok(out) = run_git(project_path, &["rev-parse", "--abbrev-ref", "HEAD"]) {
        info.branch = out.trim().to_string();
    }

    // Dirty file count
    if let Ok(out) = run_git(project_path, &["status", "--porcelain"]) {
        info.dirty_files = out.lines().filter(|l| !l.is_empty()).count() as u32;
    }

    // Ahead/behind
    if let Ok(out) = run_git(
        project_path,
        &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
    ) {
        let parts: Vec<&str> = out.trim().split('\t').collect();
        if parts.len() == 2 {
            info.ahead = parts[0].parse().unwrap_or(0);
            info.behind = parts[1].parse().unwrap_or(0);
        }
    }

    // Last commit message
    if let Ok(out) = run_git(project_path, &["log", "-1", "--pretty=%s"]) {
        info.last_commit_msg = out.trim().to_string();
    }

    // Last commit age
    if let Ok(out) = run_git(project_path, &["log", "-1", "--pretty=%ar"]) {
        info.last_commit_age = out.trim().to_string();
    }

    // PR info via gh CLI (if available)
    if !info.branch.is_empty() && info.branch != "main" && info.branch != "master" {
        info.pr = fetch_pr_info(project_path, &info.branch);
    }

    info
}

/// Fetch PR info using gh CLI
fn fetch_pr_info(project_path: &str, branch: &str) -> Option<PrInfo> {
    let output = Command::new("gh")
        .args([
            "pr", "view", branch,
            "--json", "number,title,state,url,reviewDecision,statusCheckRollup",
        ])
        .current_dir(project_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;

    let checks_passing = json.get("statusCheckRollup")
        .and_then(|v| v.as_array())
        .map(|checks| {
            checks.iter().all(|c| {
                c.get("conclusion")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "SUCCESS")
                    .unwrap_or(false)
            })
        });

    Some(PrInfo {
        number: json.get("number")?.as_u64()?,
        title: json.get("title")?.as_str()?.to_string(),
        state: json.get("state")?.as_str()?.to_string(),
        url: json.get("url")?.as_str()?.to_string(),
        checks_passing,
        review_status: json
            .get("reviewDecision")
            .and_then(|v| v.as_str())
            .unwrap_or("PENDING")
            .to_string(),
    })
}

fn run_git(cwd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_info_default() {
        let info = GitInfo::default();
        assert!(info.branch.is_empty());
        assert_eq!(info.dirty_files, 0);
    }

    #[test]
    fn test_nonexistent_path() {
        let info = fetch_git_info("/nonexistent/path");
        assert!(info.branch.is_empty());
    }
}
