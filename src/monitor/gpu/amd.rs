//! Reads live GPU data from the Linux kernel.

use crate::error;
use std::{fs::read_dir, fs::read_to_string, process::exit};

pub struct Gpu {
    temp_sensor: String,
    usage_file: String,
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            temp_sensor: find_temp_sensor(),
            usage_file: find_card(),
        }
    }

    /// Reads the value of the GPU temperature sensor and calculates it to be `˚C` or `˚F`.
    pub fn get_temp(&self, fahrenheit: bool) -> u8 {
        // Read sensor data
        let data = read_to_string(&self.temp_sensor).unwrap_or_else(|_| {
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
}

/// Looks for the appropriate CPU temperature sensor datastream in the hwmon folder.
fn find_temp_sensor() -> String {
    match read_dir("/sys/class/hwmon") {
        Ok(sensors) => {
            for sensor in sensors {
                let path = sensor.unwrap().path().to_str().unwrap().to_owned();
                match read_to_string(format!("{path}/name")) {
                    Ok(name) => {
                        if name.starts_with("amdgpu") {
                            return format!("{path}/temp1_input");
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
