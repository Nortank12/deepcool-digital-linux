//! Reads live GPU data from the Linux kernel.

use crate::error;
use std::{fs::read_dir, fs::read_to_string, process::exit};

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

        Gpu {
            drm_dir,
            hwmon_dir,
        }
    }

    /// Reads the value of the GPU temperature sensor and calculates it to be `˚C` or `˚F`.
    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        // Read temperature data from hwmon
        let data = read_to_string(format!("{}/temp1_input", &self.hwmon_dir)).unwrap_or_else(|_| {
            error!("Failed to get GPU temperature");
            exit(1);
        });

        // Calculate temperature
        let mut temp = data.trim_end().parse::<u32>().unwrap();
        if fahrenheit {
            temp = temp * 9 / 5 + 32000;
        }

        (temp as f32 / 1000.0).round() as u8
    }

    /// Estimates GPU usage based on frequency scaling, using the current and maximum frequency.
    pub fn get_usage(&self) -> u8 {
        // Read current frequency and max frequency from DRM
        let current_freq = read_to_string(format!("{}/device/gt_cur_freq_mhz", &self.drm_dir))
            .unwrap_or_else(|_| {
                error!("Failed to get GPU current frequency");
                exit(1);
            })
            .trim_end()
            .parse::<u32>()
            .unwrap_or(0);

        let max_freq = read_to_string(format!("{}/device/gt_max_freq_mhz", &self.drm_dir))
            .unwrap_or_else(|_| {
                error!("Failed to get GPU max frequency");
                exit(1);
            })
            .trim_end()
            .parse::<u32>()
            .unwrap_or(0);

        // Estimate usage as a percentage
        if max_freq > 0 {
            ((current_freq as f32 / max_freq as f32) * 100.0).round() as u8
        } else {
            0
        }
    }

    /// Reads the value of the GPU power consumption in Watts from hwmon.
    pub fn get_power(&self) -> u16 {
        let data = read_to_string(format!("{}/power1_average", &self.hwmon_dir)).unwrap_or_else(|_| {
            error!("Failed to get GPU power");
            exit(1);
        });
        let power = data.trim_end().parse::<u64>().unwrap_or(0);

        (power / 1_000_000) as u16
    }

    /// Reads the GPU core frequency in MHz from hwmon.
    pub fn get_frequency(&self) -> u16 {
        let data = read_to_string(format!("{}/freq1_input", &self.hwmon_dir)).unwrap_or_else(|_| {
            error!("Failed to get GPU core frequency");
            exit(1);
        });
        let frequency = data.trim_end().parse::<u64>().unwrap_or(0);

        (frequency / 1_000_000) as u16
    }
}

/// Confirms that the specified path belongs to an Intel GPU and looks for the DRM device directory.
fn find_drm_dir(path: &str) -> Option<String> {
    if let Ok(data) = read_to_string(format!("{path}/uevent")) {
        let driver = data.lines().next()?;
        if driver.ends_with("i915") {
            for dir in read_dir(format!("{path}/drm")).ok()? {
                let dir_name = dir.ok()?.file_name().into_string().ok()?;
                if dir_name.starts_with("card") {
                    return Some(format!("{path}/drm/{dir_name}"));
                }
            }
        }
    }

    None
}

/// Looks for the hwmon directory of the specified Intel GPU.
fn find_hwmon_dir(path: &str) -> Option<String> {
    let hwmon_path = read_dir(format!("{path}/hwmon")).ok()?.next()?.ok()?.path();
    if let Ok(name) = read_to_string(hwmon_path.join("name")) {
        if name.starts_with("i915") {
            return Some(hwmon_path.to_str()?.to_owned());
        }
    }

    None
}
