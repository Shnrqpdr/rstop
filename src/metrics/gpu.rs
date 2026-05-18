#[derive(Debug, Default, Clone)]
pub struct GpuMetrics {
    pub vendor: GpuVendor,
    pub name: String,
    pub utilization_pct: Option<f32>,
    pub vram_used: Option<u64>,
    pub vram_total: Option<u64>,
    pub temperature_c: Option<f32>,
    pub power_w: Option<f32>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    #[default]
    Unknown,
    Nvidia,
}

impl GpuVendor {
    pub fn label(&self) -> &'static str {
        match self {
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Unknown => "GPU",
        }
    }
}

impl GpuMetrics {
    pub fn vram_used_pct(&self) -> Option<f64> {
        match (self.vram_used, self.vram_total) {
            (Some(u), Some(t)) if t > 0 => Some((u as f64 / t as f64) * 100.0),
            _ => None,
        }
    }
}

#[cfg(feature = "nvidia")]
mod nv {
    use std::ffi::OsStr;

    use super::{GpuMetrics, GpuVendor};
    use nvml_wrapper::Nvml;

    /// `Nvml::init()` dlopens the unversioned `libnvidia-ml.so`, which is
    /// missing on WSL (only `libnvidia-ml.so.1` ships, under /usr/lib/wsl/lib)
    /// and on hosts without the `-dev` package. Fall back to the versioned
    /// soname and the WSL path before giving up.
    fn init_nvml() -> Option<Nvml> {
        if let Ok(n) = Nvml::init() {
            return Some(n);
        }
        for path in [
            "libnvidia-ml.so.1",
            "/usr/lib/wsl/lib/libnvidia-ml.so.1",
        ] {
            if let Ok(n) = Nvml::builder().lib_path(OsStr::new(path)).init() {
                return Some(n);
            }
        }
        None
    }

    pub struct NvmlBackend {
        nvml: Option<Nvml>,
    }

    impl NvmlBackend {
        pub fn new() -> Self {
            Self { nvml: init_nvml() }
        }

        pub fn collect(&self) -> Vec<GpuMetrics> {
            let nvml = match &self.nvml {
                Some(n) => n,
                None => return Vec::new(),
            };

            let count = match nvml.device_count() {
                Ok(c) => c,
                Err(_) => return Vec::new(),
            };

            (0..count)
                .filter_map(|i| {
                    let dev = nvml.device_by_index(i).ok()?;
                    let name = dev.name().unwrap_or_else(|_| format!("NVIDIA #{}", i));
                    let util = dev.utilization_rates().ok().map(|u| u.gpu as f32);
                    let mem = dev.memory_info().ok();
                    let temp = dev
                        .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                        .ok()
                        .map(|t| t as f32);
                    let power = dev.power_usage().ok().map(|p| p as f32 / 1000.0);
                    Some(GpuMetrics {
                        vendor: GpuVendor::Nvidia,
                        name,
                        utilization_pct: util,
                        vram_used: mem.as_ref().map(|m| m.used),
                        vram_total: mem.as_ref().map(|m| m.total),
                        temperature_c: temp,
                        power_w: power,
                    })
                })
                .collect()
        }
    }
}

#[cfg(not(feature = "nvidia"))]
mod nv {
    use super::GpuMetrics;
    pub struct NvmlBackend;
    impl NvmlBackend {
        pub fn new() -> Self {
            Self
        }
        pub fn collect(&self) -> Vec<GpuMetrics> {
            Vec::new()
        }
    }
}

pub struct GpuCollector {
    nvml: nv::NvmlBackend,
}

impl GpuCollector {
    pub fn new() -> Self {
        Self {
            nvml: nv::NvmlBackend::new(),
        }
    }

    pub fn collect(&mut self) -> Vec<GpuMetrics> {
        self.nvml.collect()
    }
}

#[cfg(test)]
mod tests {
    /// Not an assertion — prints what NVML detects on this machine.
    /// Run with: cargo test gpu_probe -- --nocapture
    #[test]
    fn gpu_probe() {
        let mut c = super::GpuCollector::new();
        eprintln!("detected GPUs: {:#?}", c.collect());
    }
}
