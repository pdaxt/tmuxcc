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
}
