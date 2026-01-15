//! Reads live GPU data from the Linux kernel.

use std::fs::{read_dir, read_to_string};

pub struct Gpu {
    drm_dir: Option<String>,
    hwmon_dir: String,
    #[allow(dead_code)]
    name: String,
}

impl Gpu {
    pub fn new(pci_address: &str) -> Option<Self> {
        let pci_path = format!("/sys/bus/pci/devices/{pci_address}");

        // Attempt to find DRM directory (Standard)
        let mut drm_dir = find_drm_dir(&pci_path);

        // Scan for HWMON directory
        let (hwmon_dir, name) = match find_hwmon_dir() {
            Some(res) => res,
            None => {
                // If no HWMON found, we cannot report anything useful.
                // We return None so main.rs can fallback to CPU-only.
                return None;
            }
        };

        // If using fallback "coretemp", disable usage stats (force 0%)
        if name == "Intel Xe (Shared)" {
            drm_dir = None;
        }

        println!("Intel GPU Sensor found at: {}", hwmon_dir);

        Some(Gpu {
            drm_dir,
            hwmon_dir,
            name,
        })
    }

    /// Reads GPU temperature
    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        // Try reading standard temp1_input (common for xe, i915, and coretemp)
        if let Ok(data) = read_to_string(format!("{}/temp1_input", &self.hwmon_dir)) {
            let mut temp = data.trim().parse::<u32>().unwrap_or(0);
            if fahrenheit {
                temp = temp * 9 / 5 + 32000;
            }
            return (temp as f32 / 1000.0).round() as u8;
        }

        // Fallback: Check for package temperature (B-series/other drivers)
        for idx in 2..=5 {
            let label = read_to_string(format!("{}/temp{}_label", &self.hwmon_dir, idx));
            let data = read_to_string(format!("{}/temp{}_input", &self.hwmon_dir, idx));

            if let (Ok(label), Ok(data)) = (label, data) {
                if label.trim().eq_ignore_ascii_case("pkg") || label.trim().eq_ignore_ascii_case("package id 0") {
                    let mut temp = data.trim().parse::<u32>().unwrap_or(0);
                    if fahrenheit {
                        temp = temp * 9 / 5 + 32000;
                    }
                    return (temp as f32 / 1000.0).round() as u8;
                }
            }
        }

        // If we are in fallback mode (coretemp), we expect to have found it above.
        // If not, return 0 instead of crashing.
        0
    }

    /// Estimates GPU usage
    pub fn get_usage(&self) -> u8 {
        let drm_dir = match &self.drm_dir {
            Some(dir) => dir,
            None => return 0, // No DRM means no usage stats
        };

        // ===== A-series / Standard i915 =====
        let cur = read_to_string(format!("{}/device/gt_cur_freq_mhz", drm_dir))
            .ok()
            .and_then(|v| v.trim().parse::<u32>().ok());

        let max = read_to_string(format!("{}/device/gt_max_freq_mhz", drm_dir))
            .ok()
            .and_then(|v| v.trim().parse::<u32>().ok());

        if let (Some(cur), Some(max)) = (cur, max) {
            if max > 0 {
                return ((cur as f32 / max as f32) * 100.0).round() as u8;
            }
        }

        // ===== B-series (xe) fallback =====
        let base = format!("{}/device/tile0/gt0/freq0", drm_dir);

        let cur = read_to_string(format!("{}/cur_freq", base))
            .ok()
            .and_then(|v| v.trim().parse::<u32>().ok());

        let max = read_to_string(format!("{}/max_freq", base))
            .ok()
            .and_then(|v| v.trim().parse::<u32>().ok());

        if let (Some(cur), Some(max)) = (cur, max) {
            if max > 0 {
                return ((cur as f32 / max as f32) * 100.0).round() as u8;
            }
        }

        0
    }

    /// Reads GPU power in Watts
    pub fn get_power(&self) -> u16 {
        let data = read_to_string(format!("{}/power1_average", &self.hwmon_dir))
            .or_else(|_| read_to_string(format!("{}/power/average", &self.hwmon_dir)));

        match data {
            Ok(d) => (d.trim().parse::<u64>().unwrap_or(0) / 1_000_000) as u16,
            Err(_) => 0, // Return 0 if power file missing (common in fallback)
        }
    }

    /// Reads GPU frequency
    pub fn get_frequency(&self) -> u16 {
        let data = read_to_string(format!("{}/freq1_input", &self.hwmon_dir));
        
        match data {
            Ok(d) => (d.trim().parse::<u64>().unwrap_or(0) / 1_000_000) as u16,
            Err(_) => 0,
        }
    }
}

/// Finds DRM directory (Standard PCI-based)
fn find_drm_dir(path: &str) -> Option<String> {
    let data = read_to_string(format!("{path}/uevent")).ok()?;

    if data.lines().any(|l| l.contains("DRIVER=i915") || l.contains("DRIVER=xe")) {
        for dir in read_dir(format!("{path}/drm")).ok()? {
            let name = dir.ok()?.file_name().into_string().ok()?;
            if name.starts_with("card") {
                return Some(format!("{path}/drm/{name}"));
            }
        }
    }
    None
}

/// Finds hwmon directory (Global Scan)
/// Phase 1: Search for 'xe', 'i915', 'intel_arc', 'drm'
/// Phase 2: Fallback to 'coretemp'
fn find_hwmon_dir() -> Option<(String, String)> {
    let hwmon_root = "/sys/class/hwmon";
    let mut fallback_path = None;

    let entries = read_dir(hwmon_root).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name_path = path.join("name");
        
        if let Ok(name) = read_to_string(&name_path) {
            let name = name.trim();

            // Phase 1: Dedicated GPU drivers
            if name.contains("xe") || name.contains("i915") || name.contains("intel_arc") || name.contains("drm") {
                 return Some((path.to_string_lossy().to_string(), "Intel Xe".to_string()));
            }

            // Phase 2 Candidate
            if name == "coretemp" {
                fallback_path = Some(path.to_string_lossy().to_string());
            }
        }
    }

    // Return fallback if found and no dedicated GPU was found
    if let Some(path) = fallback_path {
        return Some((path, "Intel Xe (Shared)".to_string()));
    }

    None
}