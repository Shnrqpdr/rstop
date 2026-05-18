use std::time::Duration;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "crabtop", version, about = "Linux TUI system monitor")]
pub struct CliArgs {
    /// Polling interval in milliseconds (htop's default is 1500)
    #[arg(short, long, default_value_t = 1500)]
    pub interval: u64,

    /// Number of historical samples kept for the line charts
    #[arg(long, default_value_t = 120)]
    pub history: usize,

    /// Show the detailed breakdown for every panel
    #[arg(short, long)]
    pub detailed: bool,

    /// Show per-core CPU gauges instead of the aggregate line chart
    #[arg(long)]
    pub cpu_cores: bool,

    /// Show RAM/swap gauges instead of the aggregate line chart
    #[arg(long)]
    pub mem_detailed: bool,

    /// Show per-GPU blocks instead of the aggregate line chart
    #[arg(long)]
    pub gpu_detailed: bool,

    /// Show the per-sensor list instead of the aggregate line chart
    #[arg(long)]
    pub temp_detailed: bool,
}

/// Which panels render their detailed view instead of the aggregate line chart.
#[derive(Debug, Clone, Copy)]
pub struct Detail {
    pub cpu: bool,
    pub mem: bool,
    pub gpu: bool,
    pub temp: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub interval: Duration,
    pub history: usize,
    pub detail: Detail,
}

impl From<CliArgs> for Config {
    fn from(a: CliArgs) -> Self {
        let all = a.detailed;
        Self {
            interval: Duration::from_millis(a.interval.max(50)),
            history: a.history.max(8),
            detail: Detail {
                cpu: all || a.cpu_cores,
                mem: all || a.mem_detailed,
                gpu: all || a.gpu_detailed,
                temp: all || a.temp_detailed,
            },
        }
    }
}
