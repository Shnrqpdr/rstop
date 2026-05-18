use sysinfo::System;

#[derive(Debug, Default, Clone)]
pub struct CoreUsage {
    pub usage: f32,
    pub frequency_mhz: u64,
}

#[derive(Debug, Default, Clone)]
pub struct CpuMetrics {
    pub total_usage: f32,
    pub per_core: Vec<CoreUsage>,
    pub load_avg: (f64, f64, f64),
}

pub fn collect(sys: &System) -> CpuMetrics {
    let per_core = sys
        .cpus()
        .iter()
        .map(|c| CoreUsage {
            usage: c.cpu_usage(),
            frequency_mhz: c.frequency(),
        })
        .collect::<Vec<_>>();

    let total_usage = if per_core.is_empty() {
        0.0
    } else {
        per_core.iter().map(|c| c.usage).sum::<f32>() / per_core.len() as f32
    };

    let load = sysinfo::System::load_average();
    CpuMetrics {
        total_usage,
        per_core,
        load_avg: (load.one, load.five, load.fifteen),
    }
}
