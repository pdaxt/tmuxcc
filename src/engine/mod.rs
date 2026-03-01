pub mod health;
pub mod reaper;
pub mod retention;

/// Spawn background maintenance tasks alongside the MCP server.
/// Uses coordination_db() per-call (no shared Db struct needed).
pub async fn start_background_tasks() {
    // Reaper: detect dead agents every 120s
    tokio::spawn(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(120)).await;
            if let Err(e) = reaper::reap_dead_agents() {
                tracing::warn!("Reaper error: {e}");
            }
        }
    });

    // Lock expiry: clean expired locks every 60s
    tokio::spawn(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            if let Err(e) = reaper::expire_locks() {
                tracing::warn!("Lock expiry error: {e}");
            }
        }
    });

    // Retention: prune old data every 6 hours
    tokio::spawn(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
            if let Err(e) = retention::prune() {
                tracing::warn!("Retention error: {e}");
            }
        }
    });

    // Project scanner: discover repos every 5 minutes
    tokio::spawn(async {
        // Initial scan at startup
        let reg = crate::scanner::scan_all();
        tracing::info!("Project scanner: discovered {} repos", reg.projects.len());
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(300)).await;
            let reg = crate::scanner::scan_all();
            tracing::info!("Project scanner: {} repos", reg.projects.len());
        }
    });

    // Health monitor: run tests/builds every 15 minutes for changed projects
    tokio::spawn(async {
        // Delay to let scanner populate registry first
        tokio::time::sleep(std::time::Duration::from_secs(120)).await;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15 * 60)).await;
            health::health_cycle().await;
        }
    });
}
