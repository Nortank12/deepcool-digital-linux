//! Reads live GPU data from the Linux kernel.

use crate::error;
use std::{
    fs::{read_dir, read_to_string},
    process::exit,
};

pub struct Gpu {
    drm_dir: String,
    hwmon_dir: String,
}

impl Gpu {
    pub fn new(pci_address: &str) -> Self {
        let path = format!("/sys/bus/pci/devices/{pci_address}");

        let drm_dir = match find_drm_dir(&path) {
            Some(dir) => dir,
            None => {
                error!(format!("Failed access GPU (Intel) PCI_ADDR={pci_address}"));
                exit(1);
            }
        };

        let hwmon_dir = match find_hwmon_dir(&path) {
            Some(dir) => dir,
            None => {
                error!("Failed to locate GPU temperature sensor (Intel)");
                exit(1);
            }
        };

        Gpu { drm_dir, hwmon_dir }
    }

    /// Reads GPU temperature (A-series + B-series)
    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        // ===== A-series (unchanged) =====
        if let Ok(data) = read_to_string(format!("{}/temp1_input", &self.hwmon_dir)) {
            let mut temp = data.trim().parse::<u32>().unwrap_or(0);
            if fahrenheit {
                temp = temp * 9 / 5 + 32000;
            }
            return (temp as f32 / 1000.0).round() as u8;
        }

        // ===== B-series (pkg temp) =====
        for idx in [2, 3] {
            let label = read_to_string(format!("{}/temp{}_label", &self.hwmon_dir, idx));
            let data  = read_to_string(format!("{}/temp{}_input", &self.hwmon_dir, idx));

            if let (Ok(label), Ok(data)) = (label, data) {
                if label.trim() == "pkg" {
                    let mut temp = data.trim().parse::<u32>().unwrap_or(0);
                    if fahrenheit {
                        temp = temp * 9 / 5 + 32000;
                    }
                    return (temp as f32 / 1000.0).round() as u8;
                }
            }
        }

        error!("Failed to get GPU temperature");
        exit(1);
    }

    /// Estimates GPU usage (A-series + B-series)
    pub fn get_usage(&self) -> u8 {
        // ===== A-series (unchanged) =====
        let cur = read_to_string(format!("{}/device/gt_cur_freq_mhz", &self.drm_dir))
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok());

        let max = read_to_string(format!("{}/device/gt_max_freq_mhz", &self.drm_dir))
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok());

        if let (Some(cur), Some(max)) = (cur, max) {
            if max > 0 {
                return ((cur as f32 / max as f32) * 100.0).round() as u8;
            }
        }

        // ===== B-series (xe) fallback =====
        let base = format!("{}/device/tile0/gt0/freq0", &self.drm_dir);

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
        .or_else(|_| read_to_string(format!("{}/power/average", &self.hwmon_dir)))
        .unwrap_or_else(|_| {
            error!("Failed to get GPU power");
            exit(1);
        });

        (data.trim().parse::<u64>().unwrap_or(0) / 1_000_000) as u16
    }

    /// Reads GPU frequency (A-series only)
    pub fn get_frequency(&self) -> u16 {
        let data = read_to_string(format!("{}/freq1_input", &self.hwmon_dir))
        .unwrap_or_else(|_| {
            error!("Failed to get GPU core frequency");
            exit(1);
        });

        (data.trim().parse::<u64>().unwrap_or(0) / 1_000_000) as u16
    }
}

/// Finds DRM directory
fn find_drm_dir(path: &str) -> Option<String> {
    let data = read_to_string(format!("{path}/uevent")).ok()?;

    if data.lines().any(|l| l == "DRIVER=i915" || l == "DRIVER=xe") {
        for dir in read_dir(format!("{path}/drm")).ok()? {
            let name = dir.ok()?.file_name().into_string().ok()?;
            if name.starts_with("card") {
                return Some(format!("{path}/drm/{name}"));
            }
        }
    }

    None
}

/// Finds hwmon directory
fn find_hwmon_dir(path: &str) -> Option<String> {
    for entry in read_dir(format!("{path}/hwmon")).ok()? {
        let hwmon = entry.ok()?.path();
        let name = read_to_string(hwmon.join("name")).ok()?;

        if name.trim() == "i915" || name.trim() == "xe" {
            return Some(hwmon.to_string_lossy().into_owned());
        }
    }

    None
}
