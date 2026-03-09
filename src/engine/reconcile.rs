//! State reconciliation: ensure persisted state matches actual tmux/process reality.
//!
//! On startup: any pane marked "active" that has no running tmux window gets reset to "idle".
//! Periodically: check active panes for completion/error and update state automatically.

use crate::config;
use crate::state::StateManager;
use crate::tmux;
use std::sync::Arc;

/// Run once at startup: clear stale "active" states that have no backing process.
pub async fn reconcile_on_startup(state: &Arc<StateManager>) {
    let snap = state.get_state_snapshot().await;
    let mut reconciled = 0;

    for (key, pane) in &snap.panes {
        if pane.status != "active" {
            continue;
        }

        let is_alive = if let Some(ref target) = pane.tmux_target {
            // Check if tmux pane still exists by trying to capture output
            let output = tmux::capture_output(target);
            !output.is_empty() || !tmux::check_done(target)
        } else {
            // No tmux target — definitely stale
            false
        };

        if !is_alive {
            if let Ok(pane_num) = key.parse::<u8>() {
                tracing::info!(
                    "Reconcile: pane {} ({}) was 'active' but has no running agent — resetting to idle",
                    pane_num, pane.project
                );
                let mut reset = pane.clone();
                reset.status = "idle".into();
                reset.tmux_target = None;
                state.set_pane(pane_num, reset).await;
                reconciled += 1;
            }
        }
    }

    if reconciled > 0 {
        tracing::info!("Reconciled {} stale panes on startup", reconciled);
    }
}

/// Periodic reconciler: check all active tmux agents for done/error.
/// Called every 10s by the engine. Auto-marks panes as "done" when their agent finishes.
pub async fn reconcile_active_panes(state: &Arc<StateManager>) {
    for i in 1..=config::pane_count() {
        let pd = state.get_pane(i).await;
        if pd.status != "active" {
            continue;
        }

        let target = match &pd.tmux_target {
            Some(t) => t.clone(),
            None => continue,
        };

        // Check if agent is done
        if tmux::check_done(&target) {
            tracing::info!("Reconciler: pane {} agent finished (shell prompt detected)", i);
            state.update_pane_status(i, "done").await;
            state.log_activity(i, "auto_done", &format!("Agent finished: {}", &pd.task)).await;
        }

        // Check for errors
        if let Some(error) = tmux::check_error(&target) {
            tracing::warn!("Reconciler: pane {} has error: {}", i, error);
            state.update_pane_status(i, "error").await;
            state.log_activity(i, "auto_error", &format!("Error detected: {}", error)).await;
        }
    }
}
