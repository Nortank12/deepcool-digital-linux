//! Reads live GPU data from the Linux kernel. Supports both GPUs and iGPUs (APU).

use crate::error;
use std::{fs::read_dir, fs::read_to_string, process::exit};

pub struct Gpu {
    usage_file: String,
    hwmon_dir: String,
}

impl Gpu {
    pub fn new(pci_address: &str) -> Self {
        let path = format!("/sys/bus/pci/devices/{pci_address}");

        let usage_file = match find_card(&path) {
            Some(file) => file,
            None => {
                error!(format!("Failed access GPU (AMD) PCI_ADDR={pci_address}"));
                exit(1);
            }
        };

        let hwmon_dir = match find_hwmon_dir(&path) {
            Some(dir) => dir,
            None => {
                error!("Failed to locate GPU temperature sensor (AMD)");
                exit(1);
            }
        };

        Gpu { usage_file, hwmon_dir }
    }

    /// Reads the value of the GPU temperature sensor and calculates it to be `˚C` or `˚F`.
    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        // Read sensor data
        let data = read_to_string(format!("{}/temp1_input", &self.hwmon_dir)).unwrap_or_else(|_| {
            error!("Failed to get GPU temperature (AMD)");
            exit(1);
        });

        // Calculate temperature
        let mut temp = data.trim_end().parse::<u32>().unwrap();
        if fahrenheit {
            temp = temp * 9 / 5 + 32000
        }

        (temp as f32 / 1000.0).round() as u8
    }

    /// Reads the value of the GPU usage in percentage.
    pub fn get_usage(&self) -> u8 {
        let data = read_to_string(&self.usage_file).unwrap_or_else(|_| {
            error!("Failed to get GPU usage (AMD)");
            exit(1);
        });

        data.trim_end().parse::<u8>().unwrap()
    }

    /// Reads the value of the GPU power consumption in Watts.
    pub fn get_power(&self) -> u16 {
        let data = read_to_string(format!("{}/power1_average", &self.hwmon_dir)).unwrap_or_else(|_| {
            error!("Failed to get GPU power (AMD)");
            exit(1);
        });
        let power = data.trim_end().parse::<u64>().unwrap();

        (power / 1_000_000) as u16
    }

    /// Reads the value of the GPU core frequency in MHz.
    pub fn get_frequency(&self) -> u16 {
        let data = read_to_string(format!("{}/freq1_input", &self.hwmon_dir)).unwrap_or_else(|_| {
            error!("Failed to get GPU core frequency (AMD)");
            exit(1);
        });
        let frequency = data.trim_end().parse::<u64>().unwrap();

        (frequency / 1_000_000) as u16
    }
}

/// Confirms that the specified path belongs to an AMD GPU and returns the path of the "GPU Usage" file.
fn find_card(path: &str) -> Option<String> {
    if let Ok(data) = read_to_string(format!("{path}/uevent")) {
        let driver = data.lines().next()?;
        if driver.ends_with("amdgpu") {
            return Some(format!("{path}/gpu_busy_percent"));
        }
    }

    None
}

/// Looks for the hwmon directory of the specified AMD GPU.
fn find_hwmon_dir(path: &str) -> Option<String> {
    let hwmon_path = read_dir(format!("{path}/hwmon")).ok()?.next()?.ok()?.path();
    if let Ok(name) = read_to_string(hwmon_path.join("name")) {
        if name.starts_with("amdgpu") {
            return Some(hwmon_path.to_str()?.to_owned());
        }
    }

    None
}
