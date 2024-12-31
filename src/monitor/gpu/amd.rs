//! Reads live GPU data from the Linux kernel.

use crate::error;
use std::{fs::read_dir, fs::read_to_string, process::exit};

pub struct Gpu {
    hwmon_dir: String,
    usage_file: String,
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            hwmon_dir: find_hwmon_dir(),
            usage_file: find_card(),
        }
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

/// Looks for the hwmon folder of the AMD GPU.
fn find_hwmon_dir() -> String {
    match read_dir("/sys/class/hwmon") {
        Ok(sensors) => {
            for sensor in sensors {
                let path = sensor.unwrap().path().to_str().unwrap().to_owned();
                match read_to_string(format!("{path}/name")) {
                    Ok(name) => {
                        if name.starts_with("amdgpu") {
                            return path;
                        }
                    }
                    Err(_) => (),
                }
            }
        }
        Err(_) => (),
    }
    error!("Failed to locate GPU temperature sensor (AMD)");
    exit(1);
}

/// Looks for the PCI device folder of the AMD GPU.
fn find_card() -> String {
    match read_dir("/sys/bus/pci/devices") {
        Ok(devices) => {
            for device in devices {
                let path = device.unwrap().path().to_str().unwrap().to_owned();
                match read_to_string(format!("{path}/uevent")) {
                    Ok(data) => {
                        let driver = data.lines().next().unwrap();
                        if driver.ends_with("amdgpu") {
                            return format!("{path}/gpu_busy_percent");
                        }
                    }
                    Err(_) => (),
                }
            }
        }
        Err(_) => (),
    }
    error!("PCI data was not found (AMD)");
    exit(1);
}
