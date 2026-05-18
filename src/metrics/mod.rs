pub mod cpu;
pub mod gpu;
pub mod mem;
pub mod temp;

use sysinfo::System;

use crate::history::History;
use cpu::CpuMetrics;
use gpu::{GpuCollector, GpuMetrics};
use mem::MemMetrics;
use temp::TempMetrics;

pub struct Collectors {
    system: System,
    gpu: GpuCollector,
}

impl Collectors {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();
        system.refresh_memory();
        Self {
            system,
            gpu: GpuCollector::new(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
    }

    pub fn cpu(&self) -> CpuMetrics {
        cpu::collect(&self.system)
    }

    pub fn mem(&self) -> MemMetrics {
        mem::collect(&self.system)
    }

    pub fn temp(&self) -> TempMetrics {
        temp::collect()
    }

    pub fn gpu(&mut self) -> Vec<GpuMetrics> {
        self.gpu.collect()
    }
}

#[derive(Debug, Default)]
pub struct Snapshot {
    pub cpu: CpuMetrics,
    pub mem: MemMetrics,
    pub temp: TempMetrics,
    pub gpus: Vec<GpuMetrics>,
}

#[derive(Debug)]
pub struct Histories {
    pub cpu_total: History,
    pub cpu_per_core: Vec<History>,
    pub mem_used_pct: History,
    pub swap_used_pct: History,
    pub gpu_util: Vec<History>,
    pub gpu_vram_pct: Vec<History>,
    pub cpu_temp_max: History,
    capacity: usize,
}

impl Histories {
    pub fn new(capacity: usize) -> Self {
        Self {
            cpu_total: History::new(capacity),
            cpu_per_core: Vec::new(),
            mem_used_pct: History::new(capacity),
            swap_used_pct: History::new(capacity),
            gpu_util: Vec::new(),
            gpu_vram_pct: Vec::new(),
            cpu_temp_max: History::new(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, snap: &Snapshot) {
        self.cpu_total.push(snap.cpu.total_usage as f64);

        if self.cpu_per_core.len() != snap.cpu.per_core.len() {
            self.cpu_per_core = (0..snap.cpu.per_core.len())
                .map(|_| History::new(self.capacity))
                .collect();
        }
        for (h, c) in self.cpu_per_core.iter_mut().zip(snap.cpu.per_core.iter()) {
            h.push(c.usage as f64);
        }

        self.mem_used_pct.push(snap.mem.ram_used_pct());
        self.swap_used_pct.push(snap.mem.swap_used_pct());

        if self.gpu_util.len() != snap.gpus.len() {
            self.gpu_util = (0..snap.gpus.len())
                .map(|_| History::new(self.capacity))
                .collect();
            self.gpu_vram_pct = (0..snap.gpus.len())
                .map(|_| History::new(self.capacity))
                .collect();
        }
        for (h, g) in self.gpu_util.iter_mut().zip(snap.gpus.iter()) {
            h.push(g.utilization_pct.unwrap_or(0.0) as f64);
        }
        for (h, g) in self.gpu_vram_pct.iter_mut().zip(snap.gpus.iter()) {
            h.push(g.vram_used_pct().unwrap_or(0.0));
        }

        let temp_max = snap
            .temp
            .cpu_sensors
            .iter()
            .map(|s| s.temp_c)
            .fold(f32::MIN, f32::max);
        if temp_max.is_finite() {
            self.cpu_temp_max.push(temp_max as f64);
        }
    }
}
