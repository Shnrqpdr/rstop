use sysinfo::System;

#[derive(Debug, Default, Clone)]
pub struct MemMetrics {
    pub ram_total: u64,
    pub ram_used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

impl MemMetrics {
    pub fn ram_used_pct(&self) -> f64 {
        if self.ram_total == 0 {
            0.0
        } else {
            (self.ram_used as f64 / self.ram_total as f64) * 100.0
        }
    }

    pub fn swap_used_pct(&self) -> f64 {
        if self.swap_total == 0 {
            0.0
        } else {
            (self.swap_used as f64 / self.swap_total as f64) * 100.0
        }
    }
}

pub fn collect(sys: &System) -> MemMetrics {
    MemMetrics {
        ram_total: sys.total_memory(),
        ram_used: sys.used_memory(),
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
    }
}
