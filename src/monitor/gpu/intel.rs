use crate::error;
use std::{fs::read_dir, fs::read_to_string, process::exit};

pub struct Gpu {
    hwmon_dir: String,
    drm_dir: String,
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            hwmon_dir: find_hwmon_dir(),
            drm_dir: find_drm_dir(),
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

    /// Estimate GPU usage based on frequency scaling, using the current and maximum frequency.
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

/// Looks for the hwmon directory corresponding to Intel GPUs in /sys/class/hwmon.
fn find_hwmon_dir() -> String {
    match read_dir("/sys/class/hwmon") {
        Ok(sensors) => {
            for sensor in sensors {
                let path = sensor.unwrap().path().to_str().unwrap().to_owned();
                if let Ok(name) = read_to_string(format!("{}/name", path)) {
                    // This is a generic check for Intel GPUs
                    if name.contains("i915") || name.contains("intel") {
                        return path;
                    }
                }
            }
        }
        Err(_) => (),
    }
    error!("Failed to locate GPU temperature sensor (Intel)");
    exit(1);
}

/// Looks for the DRM device directory corresponding to Intel GPUs in /sys/class/drm.
fn find_drm_dir() -> String {
    match read_dir("/sys/class/drm") {
        Ok(devices) => {
            for device in devices {
                let path = device.unwrap().path().to_str().unwrap().to_owned();
                // Check for Intel GPU directories
                if path.contains("card") {
                    return path;
                }
            }
        }
        Err(_) => (),
    }
    error!("Failed to locate DRM device (Intel)");
    exit(1);
}
