use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct TempSensor {
    pub label: String,
    pub temp_c: f32,
}

#[derive(Debug, Default, Clone)]
pub struct TempMetrics {
    pub cpu_sensors: Vec<TempSensor>,
    pub other_sensors: Vec<TempSensor>,
}

const HWMON_ROOT: &str = "/sys/class/hwmon";

pub fn collect() -> TempMetrics {
    let mut cpu_sensors = Vec::new();
    let mut other_sensors = Vec::new();

    let entries = match fs::read_dir(HWMON_ROOT) {
        Ok(e) => e,
        Err(_) => return TempMetrics::default(),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let chip_name = read_trim(&path.join("name")).unwrap_or_default();
        let is_cpu = matches!(
            chip_name.as_str(),
            "coretemp" | "k10temp" | "zenpower" | "k8temp"
        );

        let inputs = match collect_temp_inputs(&path) {
            Ok(v) => v,
            Err(_) => continue,
        };

        for (label, temp) in inputs {
            let sensor = TempSensor {
                label: format!("{}: {}", chip_name, label),
                temp_c: temp,
            };
            if is_cpu {
                cpu_sensors.push(sensor);
            } else {
                other_sensors.push(sensor);
            }
        }
    }

    cpu_sensors.sort_by(|a, b| a.label.cmp(&b.label));
    other_sensors.sort_by(|a, b| a.label.cmp(&b.label));

    TempMetrics {
        cpu_sensors,
        other_sensors,
    }
}

fn collect_temp_inputs(chip_dir: &Path) -> std::io::Result<Vec<(String, f32)>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(chip_dir)? {
        let entry = entry?;
        let fname = entry.file_name();
        let name = match fname.to_str() {
            Some(n) => n,
            None => continue,
        };

        if !name.starts_with("temp") || !name.ends_with("_input") {
            continue;
        }
        let idx = name
            .trim_start_matches("temp")
            .trim_end_matches("_input");
        let label_path: PathBuf = chip_dir.join(format!("temp{}_label", idx));
        let label = read_trim(&label_path).unwrap_or_else(|| format!("temp{}", idx));

        if let Some(raw) = read_trim(&entry.path()) {
            if let Ok(milli) = raw.parse::<i64>() {
                out.push((label, milli as f32 / 1000.0));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn read_trim(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}
