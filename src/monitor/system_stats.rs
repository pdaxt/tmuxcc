use std::time::Instant;
use sysinfo::System;

/// Number of CPU history samples to keep for sparkline
const CPU_HISTORY_LEN: usize = 30;

/// System resource statistics
#[derive(Debug, Clone)]
pub struct SystemStats {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// Used memory in bytes
    pub memory_used: u64,
    /// Total memory in bytes
    pub memory_total: u64,
    /// CPU usage history for sparkline (oldest → newest)
    pub cpu_history: Vec<f32>,
    /// Last update time
    last_update: Instant,
}

impl Default for SystemStats {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            cpu_history: Vec::with_capacity(CPU_HISTORY_LEN),
            last_update: Instant::now(),
        }
    }
}

impl SystemStats {
    /// Creates a new SystemStats with initial values
    pub fn new() -> Self {
        Self::default()
    }

    /// Memory usage percentage (0-100)
    pub fn memory_percent(&self) -> f32 {
        if self.memory_total == 0 {
            0.0
        } else {
            (self.memory_used as f64 / self.memory_total as f64 * 100.0) as f32
        }
    }

    /// Format memory as human-readable string (e.g., "8.2G/16.0G")
    pub fn memory_display(&self) -> String {
        format!(
            "{}/{}",
            Self::format_bytes(self.memory_used),
            Self::format_bytes(self.memory_total)
        )
    }

    /// Render CPU history as a sparkline string
    pub fn cpu_sparkline(&self) -> String {
        const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        self.cpu_history
            .iter()
            .map(|&v| {
                let idx = ((v / 100.0) * (BLOCKS.len() - 1) as f32).round() as usize;
                BLOCKS[idx.min(BLOCKS.len() - 1)]
            })
            .collect()
    }

    /// Format bytes as human-readable string
    fn format_bytes(bytes: u64) -> String {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;

        if bytes >= GB {
            format!("{:.1}G", bytes as f64 / GB as f64)
        } else {
            format!("{:.0}M", bytes as f64 / MB as f64)
        }
    }
}

/// Manager for collecting system statistics
pub struct SystemStatsCollector {
    system: System,
    stats: SystemStats,
}

impl SystemStatsCollector {
    /// Creates a new collector
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let cpu = system.global_cpu_usage();
        let stats = SystemStats {
            cpu_usage: cpu,
            memory_used: system.used_memory(),
            memory_total: system.total_memory(),
            cpu_history: vec![cpu],
            last_update: Instant::now(),
        };

        Self { system, stats }
    }

    /// Refresh statistics (throttled to avoid excessive updates)
    pub fn refresh(&mut self) {
        const UPDATE_INTERVAL_MS: u128 = 1000; // Update every 1 second

        if self.stats.last_update.elapsed().as_millis() >= UPDATE_INTERVAL_MS {
            self.system.refresh_cpu_usage();
            self.system.refresh_memory();

            self.stats.cpu_usage = self.system.global_cpu_usage();
            self.stats.memory_used = self.system.used_memory();
            self.stats.memory_total = self.system.total_memory();
            // Push to history, keep bounded
            self.stats.cpu_history.push(self.stats.cpu_usage);
            if self.stats.cpu_history.len() > CPU_HISTORY_LEN {
                self.stats.cpu_history.remove(0);
            }
            self.stats.last_update = Instant::now();
        }
    }

    /// Get current stats snapshot
    pub fn stats(&self) -> &SystemStats {
        &self.stats
    }
}

impl Default for SystemStatsCollector {
    fn default() -> Self {
        Self::new()
    }
}
